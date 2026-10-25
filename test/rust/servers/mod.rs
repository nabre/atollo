use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

use anyhow::Result;

use super::config::apache_vhost_conf;
use super::pools::{
    auto_pool_id, load_store, parse_auto_pool_id, pooled_actions, save_store, StoredServers,
};
use super::process::RunningPool;
use super::*;
use crate::env::KitEnv;
use crate::kit::Arch;
use crate::os::jobs::{self, Job};
#[cfg(unix)]
use crate::projects::fastcgi::FcgiAddr;
use crate::setup::Catalog;
use crate::test_support::prepared_kit;
#[cfg(unix)]
use std::path::Path;

#[test]
fn profile_crud_and_vhost_generation() {
    let (_tmp, kit) = prepared_kit("servers-pool");
    let servers = WebServers::new(kit.clone());

    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Legacy Apache 5.6".into(),
                engine: WebServerEngine::Apache,
                server_line: Some("2.4".into()),
                php_line: Some("5.6".into()),
                port: Some(8095),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();

    let view = servers.list("it", false);
    assert_eq!(view.pools.len(), 1);
    assert_eq!(view.pools[0].id, "legacy-apache-5.6");
    assert_eq!(view.pools[0].port, 8095);
    assert!(view.pools[0].manual);

    let projects = kit.projects();
    let portale = projects.join("portale");
    fs::create_dir_all(portale.join("storage").join("uploads")).unwrap();
    fs::create_dir_all(projects.join("common").join("assets")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/uploads".into());
    aliases.insert("/shared".into(), "@common/assets".into());
    aliases.insert("/up".into(), "../../Windows".into());
    aliases.insert("/q".into(), "a\"\r\nInclude x".into());
    let site = AttachedSite {
        project: "portale".into(),
        action: "site".into(),
        folder: "public".into(),
        server_name: "portale.localhost".into(),
        domains: vec!["admin.portale.localhost".into(), "x\r\nListen 80".into()],
        aliases,
        url: "http://portale.localhost:8095".into(),
        chosen: true,
        detached: false,
    };
    let vhost = apache_vhost_conf(
        8095,
        &projects.join("portale"),
        &projects,
        &portale.join("public"),
        &site,
        None,
    )
    .unwrap();
    assert!(vhost.contains("<VirtualHost *:8095>"));
    assert!(vhost.contains("ServerName portale.localhost"));
    assert!(vhost.contains("ServerAlias admin.portale.localhost\r\n"));
    assert!(vhost.contains("Alias \"/media\" \"${CP_PROJECTS_FWD}/portale/storage/uploads\""));
    assert!(vhost.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
    assert!(!vhost.contains("..") && !vhost.contains("Include") && !vhost.contains("Listen"));
    // F3: hidden files refused inside the VirtualHost.
    let deny = "    <LocationMatch \"/\\.(?!well-known(/|$))\">\r\n        Require all denied\r\n    </LocationMatch>";
    let at = vhost.find(deny).unwrap_or_else(|| panic!("{vhost}"));
    assert!(at < vhost.find("</VirtualHost>").unwrap());
    assert!(vhost.contains("    <FilesMatch \"^\\.(?!well-known$)\">"));

    // BUG-V5-05: the project's own conf.d inside its VirtualHost, as its
    // dedicated server includes it.
    let conf_d = kit
        .project_data()
        .join("portale")
        .join("apache")
        .join("conf.d");
    let with = apache_vhost_conf(
        8095,
        &projects.join("portale"),
        &projects,
        &portale.join("public"),
        &site,
        Some(&conf_d),
    )
    .unwrap();
    let include = format!(
        "    IncludeOptional \"{}/*.conf\"\r\n</VirtualHost>",
        crate::projects::apache::fwd(&conf_d)
    );
    assert!(with.contains(&include) && !include.contains('\\'), "{with}");
    assert_eq!(with.matches("Include").count(), 1, "{with}");

    // BUG-V7-09: a site folder whose name would end the quoted argument
    // (legal on macOS and Linux) is left out, never written in the vhost.
    let evil = portale.join("pub\"\r\nLoadModule x y\r\n#");
    assert!(apache_vhost_conf(8095, &portale, &projects, &evil, &site, None).is_none());
    let vars = portale.join("${SRVROOT}");
    assert!(apache_vhost_conf(8095, &portale, &projects, &vars, &site, None).is_none());

    servers.remove_profile("legacy-apache-5.6").unwrap();
    assert!(servers.list("it", false).pools.is_empty());
}

/// An automatic-looking Caddy pool `id` on `port` with `sites`, not running.
pub(crate) fn caddy_pool(id: &str, port: u16, sites: Vec<AttachedSite>) -> WebServerPool {
    WebServerPool {
        id: id.into(),
        name: "Caddy".into(),
        engine: WebServerEngine::Caddy,
        server_line: None,
        php_line: None,
        port,
        running: false,
        pid: None,
        manual: false,
        sites,
        conf_dir: conf_dir_text(id),
        orphan: false,
        profile: None,
    }
}

/// `data\servers\<id>` as the page shows it: with the system's separator.
pub(crate) fn conf_dir_text(id: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!("data{sep}servers{sep}{id}")
}

pub(crate) fn site(project: &str, folder: &str, port: u16) -> AttachedSite {
    AttachedSite {
        project: project.into(),
        action: "site".into(),
        folder: folder.into(),
        server_name: format!("{project}.localhost"),
        domains: Vec::new(),
        aliases: BTreeMap::new(),
        url: format!("http://{project}.localhost:{port}"),
        chosen: true,
        detached: false,
    }
}

/// N-S4a: `list` (which saves new ports) running next to `save_profile`
/// never loses a profile: every read-modify-write holds `store_lock`.
#[test]
fn concurrent_list_and_save_profile_keep_profile() {
    let (_tmp, kit) = prepared_kit("servers-store-lock");
    let servers = WebServers::new(kit.clone());
    std::thread::scope(|s| {
        s.spawn(|| {
            for i in 0..20 {
                servers
                    .save_profile(
                        WebServerProfileInput {
                            id: String::new(),
                            name: format!("p{i}"),
                            engine: WebServerEngine::Caddy,
                            server_line: None,
                            php_line: None,
                            port: None,
                            keep_alive: true,
                            autostart: false,
                        },
                        "it",
                    )
                    .unwrap();
            }
        });
        s.spawn(|| {
            for _ in 0..40 {
                servers.list("it", true);
            }
        });
    });
    let store = load_store(&kit);
    assert_eq!(store.profiles.len(), 20);
    assert_eq!(servers.list("it", false).pools.len(), 20);
}

/// M1: removing a profile waits for a start/stop under way (`op_lock`),
/// then stops its pool and drops it from `servers.json` in one go.
#[test]
fn remove_profile_stops_and_forgets_under_op_lock() {
    let (_tmp, kit) = prepared_kit("servers-remove-op");
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: "gone".into(),
                name: "gone".into(),
                engine: WebServerEngine::Caddy,
                server_line: None,
                php_line: None,
                port: Some(8094),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let job = Job::new().unwrap();
    let child = jobs::spawn_in(&mut long_running(), &job).unwrap();
    servers.running.lock().unwrap().insert(
        "gone".into(),
        RunningPool {
            child,
            port: 8094,
            fcgi: None,
            php_line: None,
            php: None,
            admin: None,
            started: caddy_pool("gone", 8094, Vec::new()),
            by_user: true,
            attached: BTreeSet::new(),
            _job: job,
        },
    );
    let op = servers.op_lock.lock().unwrap();
    std::thread::scope(|s| {
        let remover = s.spawn(|| servers.remove_profile("gone"));
        std::thread::sleep(Duration::from_millis(300));
        assert!(!remover.is_finished(), "waits for op_lock");
        assert_eq!(load_store(&kit).profiles.len(), 1);
        assert_eq!(servers.running_count(), 1);
        drop(op);
        remover.join().unwrap().unwrap();
    });
    assert_eq!(servers.running_count(), 0);
    assert!(load_store(&kit).profiles.is_empty());
    assert!(!load_store(&kit).ports.contains_key("gone"));
}

/// BUG-07: the 3 s poll reuses the scan; `fresh` and every change made
/// through `WebServers` read `servers.json` and the projects again.
#[test]
fn list_uses_cache_until_invalidated() {
    let (_tmp, kit) = prepared_kit("servers-list-cache");
    let servers = WebServers::new(kit.clone());
    assert!(servers.list("it", false).pools.is_empty());

    // Changed behind its back: the cached scan is still shown...
    let profile = |id: &str| WebServerProfileInput {
        id: id.into(),
        name: id.into(),
        engine: WebServerEngine::Caddy,
        server_line: None,
        php_line: None,
        port: Some(if id == "outside" { 8097 } else { 8086 }),
        keep_alive: true,
        autostart: false,
    };
    let store = StoredServers {
        profiles: vec![profile("outside")],
        ports: BTreeMap::new(),
    };
    save_store(&kit, &store).unwrap();
    assert!(servers.list("it", false).pools.is_empty());
    // ...until a fresh list (the "Aggiorna" button).
    assert_eq!(servers.list("it", true).pools.len(), 1);

    // save_profile and remove_profile invalidate it.
    servers.save_profile(profile("second"), "it").unwrap();
    assert_eq!(servers.list("it", false).pools.len(), 2);
    servers.remove_profile("outside").unwrap();
    let ids: Vec<String> = servers
        .list("it", false)
        .pools
        .into_iter()
        .map(|p| p.id)
        .collect();
    assert_eq!(ids, ["second"]);

    // Another language is never served from the cache of the first.
    save_store(&kit, &StoredServers::default()).unwrap();
    assert!(servers.list("en", false).pools.is_empty());
}

#[cfg(windows)]
pub(crate) fn system32(exe: &str) -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    format!(r"{root}\System32\{exe}")
}

/// A program that runs for a minute, as a server would: `ping` from
/// System32 on Windows, `sleep` elsewhere.
pub(crate) fn long_running() -> std::process::Command {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new(system32("ping.exe"));
        cmd.args(["-n", "60", "127.0.0.1"]);
        cmd
    }
    #[cfg(unix)]
    {
        let mut cmd = std::process::Command::new("/bin/sleep");
        cmd.arg("60");
        cmd
    }
}

/// A program that ends at once with `code`: `cmd.exe` from System32 on
/// Windows, `/bin/sh` elsewhere.
pub(crate) fn exiting(code: u8) -> std::process::Command {
    #[cfg(windows)]
    {
        let mut cmd = std::process::Command::new(system32("cmd.exe"));
        cmd.args(["/d", "/c", &format!("exit {code}")]);
        cmd
    }
    #[cfg(unix)]
    {
        let mut cmd = std::process::Command::new("/bin/sh");
        cmd.args(["-c", &format!("exit {code}")]);
        cmd
    }
}

/// A pool of `id` "running" (a `ping` in a Job) as started from `started`.
pub(crate) fn fake_running(servers: &WebServers, started: WebServerPool) {
    let job = Job::new().unwrap();
    let child = jobs::spawn_in(&mut long_running(), &job).unwrap();
    servers.running.lock().unwrap().insert(
        started.id.clone(),
        RunningPool {
            child,
            port: started.port,
            fcgi: None,
            php_line: started.php_line.clone(),
            php: None,
            admin: None,
            started,
            by_user: true,
            attached: BTreeSet::new(),
            _job: job,
        },
    );
}

/// Defect 8: assigning a project reloads only the running pool that
/// serves it (never the others) and reports its failure.
#[test]
fn assign_project_reloads_only_its_pool_and_reports_errors() {
    let (_tmp, kit) = prepared_kit("servers-assign-reload");
    let servers = WebServers::new(kit.clone());
    let free = || {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    let (legacy_port, other_port) = (free(), free());
    for (id, port) in [("legacy", legacy_port), ("other", other_port)] {
        servers
            .save_profile(
                WebServerProfileInput {
                    id: id.into(),
                    name: id.into(),
                    engine: WebServerEngine::Caddy,
                    server_line: None,
                    php_line: None,
                    port: Some(port),
                    keep_alive: true,
                    autostart: false,
                },
                "it",
            )
            .unwrap();
        fake_running(&servers, caddy_pool(id, port, Vec::new()));
    }
    fs::create_dir_all(kit.projects().join("shop").join("public")).unwrap();
    // The reload of `legacy` restarts it (a fake pool has no admin socket
    // for `caddy reload`), and there is no Caddy here.
    let err = servers
        .assign_project(
            "shop",
            "public",
            WebServerEngine::Caddy,
            Some("legacy".into()),
            &KitEnv::default(),
            "it",
        )
        .unwrap_err()
        .to_string();
    assert!(err.contains("Caddy is not installed"), "{err}");
    assert!(!servers.is_running("legacy"));
    assert!(servers.is_running("other"), "not touched");
    servers.stop("other");
}

/// D-101: a stopped project's site leaves the pool's files; an
/// automatic pool started by projects stops with the last of them.
#[test]
fn detach_leaves_the_pool_and_stops_an_empty_automatic_one() {
    let (_tmp, kit) = prepared_kit("servers-detach");
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();
    fs::create_dir_all(projects.join("shop").join("public")).unwrap();
    fs::create_dir_all(projects.join("blog").join("public")).unwrap();
    let pool = caddy_pool(
        "caddy-auto",
        8082,
        vec![site("shop", "public", 8082), site("blog", "public", 8082)],
    );

    // Not running: only left out of the files written for the next start.
    servers
        .detach("caddy-auto", "shop", "site", &KitEnv::default(), "it")
        .unwrap();
    let conf = servers.write_pool_configs(&pool, None, false).unwrap();
    let text = fs::read_to_string(conf).unwrap();
    assert!(!text.contains("http://shop.localhost:8082"), "{text}");
    assert!(text.contains("http://blog.localhost:8082 {"), "{text}");

    // Started by the blog only: its stop stops the automatic pool.
    fake_running(&servers, pool.clone());
    {
        let mut running = servers.running.lock().unwrap();
        let r = running.get_mut("caddy-auto").unwrap();
        r.by_user = false;
        r.attached.insert(("blog".into(), "site".into()));
    }
    servers
        .detach("caddy-auto", "blog", "site", &KitEnv::default(), "it")
        .unwrap();
    assert!(!servers.is_running("caddy-auto"));
    assert!(servers
        .detached
        .lock()
        .unwrap()
        .contains(&("blog".into(), "site".into())));
}

/// The kit given by ATOLLO_TEST_KIT with Apache installed, its environment,
/// and a free port; `None` (skipped) otherwise.
pub(crate) fn real_apache() -> Option<(Kit, KitEnv, u16)> {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
        return None;
    };
    let kit = Kit::at(PathBuf::from(root), Arch::detect());
    kit.prepare().unwrap();
    if !Catalog::get()
        .find("apache")
        .is_some_and(|c| c.is_installed(&kit))
    {
        eprintln!("apache not installed: skipped");
        return None;
    }
    let env = KitEnv::build(
        &kit,
        &crate::common::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let port = {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    Some((kit, env, port))
}

/// A profile `id` for Apache on `port` in the real kit; removed, with its
/// folder in `data\servers\`, by [`drop_real_profile`].
pub(crate) fn real_profile(servers: &WebServers, id: &str, port: u16) {
    servers
        .save_profile(
            WebServerProfileInput {
                id: id.into(),
                name: id.into(),
                engine: WebServerEngine::Apache,
                server_line: None,
                php_line: None,
                port: Some(port),
                keep_alive: true,
                autostart: false,
            },
            "en",
        )
        .unwrap();
}

pub(crate) fn drop_real_profile(kit: &Kit, servers: &WebServers, id: &str) {
    servers.remove_profile(id).unwrap();
    let _ = fs::remove_dir_all(kit.data().join("servers").join(id));
}

/// `GET path` with `Host: host` on 127.0.0.1:`port`: the status and the body.
pub(crate) fn http_get(port: u16, host: &str, path: &str) -> Result<(u16, String)> {
    use std::io::{Read, Write};
    let mut s = TcpStream::connect_timeout(
        &SocketAddr::from(([127, 0, 0, 1], port)),
        Duration::from_secs(2),
    )?;
    s.set_read_timeout(Some(Duration::from_secs(10)))?;
    write!(
        s,
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\n\r\n"
    )?;
    let mut raw = Vec::new();
    s.read_to_end(&mut raw)?;
    let text = String::from_utf8_lossy(&raw).into_owned();
    let status = text
        .split_whitespace()
        .nth(1)
        .and_then(|c| c.parse().ok())
        .unwrap_or(0);
    let body = text.split_once("\r\n\r\n").map(|(_, b)| b).unwrap_or("");
    Ok((status, body.to_owned()))
}

/// One FastCGI request (`FCGI_RESPONDER`) with `params` and no body on
/// `stream`: what php-fpm writes on stdout.
#[cfg(unix)]
pub(crate) fn fcgi_request(
    stream: &mut std::os::unix::net::UnixStream,
    params: &[(&str, &str)],
) -> String {
    use std::io::{Read, Write};
    fn record(kind: u8, body: &[u8]) -> Vec<u8> {
        let len = body.len() as u16;
        let mut r = vec![1, kind, 0, 1, (len >> 8) as u8, len as u8, 0, 0];
        r.extend_from_slice(body);
        r
    }
    fn length(n: usize, out: &mut Vec<u8>) {
        if n < 128 {
            out.push(n as u8);
        } else {
            out.extend_from_slice(&((n as u32) | 0x8000_0000).to_be_bytes());
        }
    }
    let mut body = Vec::new();
    for (k, v) in params {
        length(k.len(), &mut body);
        length(v.len(), &mut body);
        body.extend_from_slice(k.as_bytes());
        body.extend_from_slice(v.as_bytes());
    }
    let mut out = record(1, &[0, 1, 0, 0, 0, 0, 0, 0]);
    out.extend(record(4, &body));
    out.extend(record(4, &[]));
    out.extend(record(5, &[]));
    stream.write_all(&out).unwrap();
    let mut answer = Vec::new();
    let _ = stream.read_to_end(&mut answer);
    // The stdout records' content, headers and records apart.
    let mut text = Vec::new();
    let mut at = 0;
    while at + 8 <= answer.len() {
        let kind = answer[at + 1];
        let len = usize::from(answer[at + 4]) << 8 | usize::from(answer[at + 5]);
        let pad = usize::from(answer[at + 6]);
        let end = (at + 8 + len).min(answer.len());
        if kind == 6 {
            text.extend_from_slice(&answer[at + 8..end]);
        }
        at = end + pad;
    }
    String::from_utf8_lossy(&text).into_owned()
}

/// Review of D-111 (the real test): php-fpm of a Caddy pool is on an
/// owner-only Unix socket in an owner-only folder, `fpm.conf` has no TCP
/// listen, and a request without the key is refused by the guard. What a
/// request with `PHP_VALUE` or `PHP_ADMIN_VALUE` gets is only printed: php-fpm
/// lets those turn `auto_prepend_file` off, so the socket's modes are the
/// defence, the key a second one.
#[cfg(unix)]
pub(crate) fn probe_fpm_socket(servers: &WebServers, id: &str, script: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let addr = servers
        .running
        .lock()
        .unwrap()
        .get(id)
        .and_then(|r| r.fcgi.clone())
        .map(|f| f.addr);
    let Some(FcgiAddr::Unix(sock)) = addr else {
        anyhow::bail!("php-fpm not on a Unix socket: {addr:?}");
    };
    let conf = fs::read_to_string(servers.kit.data().join("servers").join(id).join("fpm.conf"))?;
    anyhow::ensure!(!conf.contains("127.0.0.1"), "TCP in fpm.conf: {conf}");
    let mode = fs::metadata(&sock.path)?.permissions().mode() & 0o777;
    let dir_mode = fs::metadata(sock.path.parent().unwrap())?
        .permissions()
        .mode()
        & 0o777;
    eprintln!(
        "fpm socket: {} mode {mode:o}, folder {dir_mode:o}",
        sock.path.display()
    );
    anyhow::ensure!(mode & 0o077 == 0 && dir_mode == 0o700, "socket modes");
    let script = script.display().to_string();
    let base = [
        ("SCRIPT_FILENAME", script.as_str()),
        ("REQUEST_METHOD", "GET"),
        ("SERVER_PROTOCOL", "HTTP/1.1"),
    ];
    for (name, extra) in [
        ("no key", None),
        ("PHP_VALUE", Some(("PHP_VALUE", "auto_prepend_file=none"))),
        (
            "PHP_ADMIN_VALUE",
            Some(("PHP_ADMIN_VALUE", "auto_prepend_file=none")),
        ),
    ] {
        let mut params = base.to_vec();
        params.extend(extra);
        let mut stream = std::os::unix::net::UnixStream::connect(&sock.path)?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        let out = fcgi_request(&mut stream, &params);
        let status = out.lines().next().unwrap_or_default().to_owned();
        let ran = out.contains("p6-alpha");
        eprintln!("fpm without key, {name}: {status} (script ran: {ran})");
        // Only the plain request must be refused: `PHP_VALUE` does turn the
        // guard off, which is why nobody but the user reaches the socket.
        if extra.is_none() {
            anyhow::ensure!(out.starts_with("Status: 403") && !ran, "{name}: {out}");
        }
    }
    Ok(())
}

/// D-111: Caddy is an engine of the pools: its automatic ids read back,
/// its label, its sites in `panel.json` (`caddy`) pooled like the others.
#[test]
fn caddy_engine_of_the_pools() {
    assert_eq!(WebServerEngine::Caddy.as_str(), "caddy");
    assert_eq!(WebServerEngine::Caddy.label(), "Caddy");
    let id = auto_pool_id(WebServerEngine::Caddy, Some("2.11"), Some("8.4"));
    assert_eq!(id, "caddy-2.11_php-8.4");
    assert_eq!(
        parse_auto_pool_id(&id),
        Some((
            WebServerEngine::Caddy,
            Some("2.11".into()),
            Some("8.4".into())
        ))
    );
    assert_eq!(
        serde_json::to_string(&WebServerEngine::Caddy).unwrap(),
        "\"caddy\""
    );
    let mut caddy = crate::detect::Action::for_test("site", "", crate::detect::Kind::Web);
    caddy.caddy_dir = Some("public".into());
    caddy.custom = true;
    assert_eq!(
        WebServerEngine::of_action(&caddy),
        Some((WebServerEngine::Caddy, "public".into()))
    );
    let mut other = crate::detect::Action::for_test("dev", "npm run dev", crate::detect::Kind::Web);
    other.custom = true;
    let actions = [other, caddy];
    let pooled: Vec<&str> = pooled_actions(&actions)
        .into_iter()
        .map(|a| a.id.as_str())
        .collect();
    assert_eq!(pooled, ["site"]);
}
