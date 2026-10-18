use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::config::{apache_pool_conf, apache_vhost_conf, nginx_pool_conf};
use super::pools::{
    auto_pool_id, canonical_auto_pool, host_label, load_store, migrate_port_ids, name_sites,
    parse_auto_pool_id, pooled_actions, save_store, store_path, StoredServers, POOL_PORT_FROM,
    POOL_PORT_TO,
};
use super::process::{
    end_dead, needs_restart, php_cgi_listening, pool_error_tail, stops_when_left, take_dead,
    wait_port_released, LivePool, RunningPool,
};
use super::*;
use crate::detect::{set_site_with_options, SiteKind, SiteOptions};
use crate::env::KitEnv;
use crate::kit::{tests::TempDir, Arch};
use crate::os::jobs::{self, Job};
use crate::projects::alias;
use crate::projects::nginx::FastCgi;
use crate::setup::catalog::Catalog;

#[test]
fn profile_crud_and_vhost_generation() {
    let tmp = TempDir::new("servers-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
    );
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

    servers.remove_profile("legacy-apache-5.6").unwrap();
    assert!(servers.list("it", false).pools.is_empty());
}

#[test]
fn nginx_pool_conf_and_vhosts_have_no_duplicate_localhost() {
    let tmp = TempDir::new("servers-nginx-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());

    let projects = kit.projects();
    fs::create_dir_all(projects.join("alpha").join("public")).unwrap();
    fs::create_dir_all(projects.join("beta").join("public")).unwrap();

    let pool = WebServerPool {
        id: "nginx-default".into(),
        name: "Nginx".into(),
        engine: WebServerEngine::Nginx,
        server_line: None,
        php_line: None,
        port: 8088,
        running: false,
        pid: None,
        manual: false,
        sites: vec![
            AttachedSite {
                project: "alpha".into(),
                action: "site".into(),
                folder: "public".into(),
                server_name: "alpha.localhost".into(),
                domains: vec!["api.alpha.localhost".into()],
                aliases: BTreeMap::new(),
                url: "http://alpha.localhost:8088".into(),
                chosen: true,
                detached: false,
            },
            AttachedSite {
                project: "beta".into(),
                action: "site".into(),
                folder: "public".into(),
                server_name: "beta.localhost".into(),
                domains: Vec::new(),
                aliases: BTreeMap::new(),
                url: "http://beta.localhost:8088".into(),
                chosen: true,
                detached: false,
            },
        ],
        conf_dir: conf_dir_text("nginx-default"),
        orphan: false,
        profile: None,
    };

    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let main_conf = fs::read_to_string(&conf_path).unwrap();
    // The pool's page for its local names; any other Host gets 421 (D-98).
    assert!(main_conf.contains(
        "        listen 127.0.0.1:8088;\r\n        server_name localhost 127.0.0.1 [::1];"
    ));
    assert!(main_conf.contains(
        "        listen 127.0.0.1:8088 default_server;\r\n        server_name _;\r\n        return 421;"
    ));
    assert_eq!(main_conf.matches("default_server").count(), 1);

    let vhosts_dir = conf_path.parent().unwrap().join("vhosts");
    let alpha_vhost = fs::read_to_string(vhosts_dir.join("alpha.localhost.conf")).unwrap();
    let beta_vhost = fs::read_to_string(vhosts_dir.join("beta.localhost.conf")).unwrap();

    assert!(alpha_vhost.contains("listen 127.0.0.1:8088;"));
    assert!(!alpha_vhost.contains("default_server"));
    assert!(alpha_vhost.contains("location ~ /\\.(?!well-known(/|$)) {"));
    assert!(alpha_vhost.contains("server_name alpha.localhost api.alpha.localhost;"));
    assert!(!alpha_vhost.contains("server_name alpha.localhost api.alpha.localhost localhost;"));

    assert!(beta_vhost.contains("listen 127.0.0.1:8088;"));
    assert!(!beta_vhost.contains("default_server"));
    assert!(beta_vhost.contains("server_name beta.localhost;"));
}

fn nginx_pool(id: &str, port: u16, sites: Vec<AttachedSite>) -> WebServerPool {
    WebServerPool {
        id: id.into(),
        name: "Nginx".into(),
        engine: WebServerEngine::Nginx,
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
fn conf_dir_text(id: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!("data{sep}servers{sep}{id}")
}

fn site(project: &str, folder: &str, port: u16) -> AttachedSite {
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

/// N-S1: a site whose document root is missing or a junction is not served
/// at all (never the project root in its place, with `.env` and `.git`).
#[test]
fn pool_skips_site_with_missing_or_linked_docroot() {
    let tmp = TempDir::new("servers-docroot");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();

    fs::create_dir_all(projects.join("good").join("public")).unwrap();
    // `missing` has no public\ folder: only its secrets at the root.
    fs::create_dir_all(projects.join("missing")).unwrap();
    fs::write(projects.join("missing").join(".env"), "DB_PASSWORD=x").unwrap();
    // `linked` has public\ as a junction to a folder outside the projects.
    fs::create_dir_all(projects.join("linked")).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    crate::os::safe_path::tests::junction(&projects.join("linked").join("public"), &outside);

    let pool = nginx_pool(
        "nginx-docroot",
        8089,
        vec![
            site("good", "public", 8089),
            site("missing", "public", 8089),
            site("linked", "public", 8089),
        ],
    );
    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let vhosts = conf_path.parent().unwrap().join("vhosts");
    let good = fs::read_to_string(vhosts.join("good.localhost.conf")).unwrap();
    assert!(good.contains("good/public"), "{good}");
    assert!(!vhosts.join("missing.localhost.conf").exists());
    assert!(!vhosts.join("linked.localhost.conf").exists());
    let all: String = fs::read_dir(&vhosts)
        .unwrap()
        .flatten()
        .map(|e| fs::read_to_string(e.path()).unwrap())
        .collect();
    assert!(
        !all.contains("missing") && !all.contains("outside"),
        "{all}"
    );
}

/// N-S4a: `list` (which saves new ports) running next to `save_profile`
/// never loses a profile: every read-modify-write holds `store_lock`.
#[test]
fn concurrent_list_and_save_profile_keep_profile() {
    let tmp = TempDir::new("servers-store-lock");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    std::thread::scope(|s| {
        s.spawn(|| {
            for i in 0..20 {
                servers
                    .save_profile(
                        WebServerProfileInput {
                            id: String::new(),
                            name: format!("p{i}"),
                            engine: WebServerEngine::Nginx,
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
    let tmp = TempDir::new("servers-remove-op");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: "gone".into(),
                name: "gone".into(),
                engine: WebServerEngine::Nginx,
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
            started: nginx_pool("gone", 8094, Vec::new()),
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

/// M2: a port released within a moment (a pool just ended) is waited for,
/// not reported as used by another program; one kept taken still is.
#[test]
fn start_waits_for_a_port_being_released() {
    let tmp = TempDir::new("servers-start-port");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    servers
        .save_profile(
            WebServerProfileInput {
                id: "busy".into(),
                name: "busy".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(port),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let busy = "La porta";

    // Kept taken: the port error, after the grace.
    let err = servers
        .start("busy", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with(busy), "{err}");

    // Released while waiting: on to the start (no Nginx in this kit).
    std::thread::scope(|s| {
        s.spawn(move || {
            std::thread::sleep(Duration::from_millis(400));
            drop(listener);
        });
        let err = servers
            .start("busy", &KitEnv::default(), "it")
            .unwrap_err()
            .to_string();
        assert!(!err.starts_with(busy), "{err}");
        assert!(err.contains("Nginx is not installed"), "{err}");
    });
}

/// BUG-07: the 3 s poll reuses the scan; `fresh` and every change made
/// through `WebServers` read `servers.json` and the projects again.
#[test]
fn list_uses_cache_until_invalidated() {
    let tmp = TempDir::new("servers-list-cache");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    assert!(servers.list("it", false).pools.is_empty());

    // Changed behind its back: the cached scan is still shown...
    let profile = |id: &str| WebServerProfileInput {
        id: id.into(),
        name: id.into(),
        engine: WebServerEngine::Nginx,
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

/// BUG-03: what httpd/nginx print on stderr (a syntax error, before their
/// error log is open) reaches the error shown to the user.
#[test]
fn pool_error_tail_reads_stderr_log() {
    let tmp = TempDir::new("servers-stderr-tail");
    let pool_dir = tmp.0.join("pool");
    fs::create_dir_all(&pool_dir).unwrap();
    assert_eq!(pool_error_tail(&pool_dir), "");
    fs::write(
        pool_dir.join("stderr.log"),
        "AH00526: Syntax error on line 12 of httpd.conf:\r\nInvalid command 'Lisen'\r\n\r\n",
    )
    .unwrap();
    assert_eq!(
        pool_error_tail(&pool_dir),
        ": AH00526: Syntax error on line 12 of httpd.conf: | Invalid command 'Lisen'"
    );
    // The server's own error log, when it has lines, comes first.
    fs::write(pool_dir.join("error.log"), "[error] bind failed\n").unwrap();
    assert_eq!(pool_error_tail(&pool_dir), ": [error] bind failed");
}

/// BUG-04: the reload waits for the port of the stopped server to be free.
#[test]
fn wait_port_released_true_after_listener_dropped() {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    assert!(!wait_port_released(port, Duration::from_millis(250)));
    let begun = std::time::Instant::now();
    std::thread::scope(|s| {
        s.spawn(move || {
            std::thread::sleep(Duration::from_millis(300));
            drop(listener);
        });
        assert!(wait_port_released(port, Duration::from_secs(5)));
    });
    assert!(begun.elapsed() < Duration::from_secs(5));
}

/// N-S4c: a fresh list drops the ports of pools that no longer exist; the
/// cached poll leaves `servers.json` as it is.
#[test]
fn fresh_list_prunes_ports_of_gone_pools() {
    let tmp = TempDir::new("servers-prune-ports");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let mut ports = BTreeMap::new();
    ports.insert("apache-2.2-php-5.6".to_owned(), 8123);
    ports.insert("keep".to_owned(), 8097);
    let store = StoredServers {
        profiles: vec![WebServerProfileInput {
            id: "keep".into(),
            name: "Keep".into(),
            engine: WebServerEngine::Nginx,
            server_line: None,
            php_line: None,
            port: Some(8097),
            keep_alive: true,
            autostart: false,
        }],
        ports,
    };
    save_store(&kit, &store).unwrap();

    servers.list("it", false);
    assert_eq!(load_store(&kit).ports.len(), 2);
    servers.list("it", true);
    let ports = load_store(&kit).ports;
    assert_eq!(ports.keys().collect::<Vec<_>>(), ["keep"]);
    assert_eq!(ports["keep"], 8097);
}

/// A port kept in `servers.json` by a pool gone since does not block a
/// profile: saving it drops the ports of the pools that no longer exist.
#[test]
fn save_profile_takes_the_port_of_a_gone_pool() {
    let tmp = TempDir::new("servers-gone-port");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let mut store = StoredServers::default();
    store.ports.insert("apache-2.2_php-5.6".to_owned(), 8124);
    save_store(&kit, &store).unwrap();
    servers
        .save_profile(
            WebServerProfileInput {
                id: "mine".into(),
                name: "Mine".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(8124),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let ports = load_store(&kit).ports;
    assert_eq!(ports.keys().collect::<Vec<_>>(), ["mine"]);

    // Opening the app drops them too.
    let mut store = load_store(&kit);
    store.ports.insert("nginx-1.20".to_owned(), 8125);
    save_store(&kit, &store).unwrap();
    servers.autostart(&KitEnv::default(), "it");
    assert!(!load_store(&kit).ports.contains_key("nginx-1.20"));
}

/// N-S5: a Host no project names (an IP from the LAN) gets Apache's own
/// page from the first VirtualHost, never the first project.
#[test]
fn apache_pool_conf_has_default_vhost_first() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n";
    let conf = apache_pool_conf(base, Path::new(r"C:\kit\data\servers\p"), 8081, true).unwrap();
    let default = conf
        .find("<VirtualHost *:8081>\r\n    ServerName localhost\r\n    DocumentRoot \"${SRVROOT}/htdocs\"\r\n</VirtualHost>")
        .expect("default vhost");
    let include = conf
        .find("IncludeOptional \"C:/kit/data/servers/p/vhosts/*.conf\"")
        .unwrap();
    assert!(default < include);
    assert_eq!(conf.matches("<VirtualHost").count(), 1);
    assert_eq!(conf.find("<VirtualHost"), Some(default));
    // A3 handoff: no PHP module, no `.php` sent as source.
    assert!(conf.contains("<IfModule !php_module>"));
    assert!(conf.contains(
        "<FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">\r\n                Require all denied"
    ));
}

/// Out of LAN mode the pool's first VirtualHost refuses a Host nobody names
/// with 403, as a project's own Apache (D-98); its local names keep
/// Apache's page, and both come before the projects' sites.
#[test]
fn apache_pool_conf_refuses_foreign_hosts() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n";
    let conf = apache_pool_conf(base, Path::new(r"C:\kit\data\servers\p"), 8081, false).unwrap();
    let refused = conf
        .find("<VirtualHost *:8081>\r\n    ServerName refused.invalid\r\n    <Location \"/\">\r\n        Require all denied")
        .expect("refusing vhost");
    let local = conf
        .find("<VirtualHost *:8081>\r\n    ServerName localhost\r\n    ServerAlias 127.0.0.1 [::1]\r\n    DocumentRoot \"${SRVROOT}/htdocs\"\r\n</VirtualHost>")
        .expect("local vhost");
    let include = conf
        .find("IncludeOptional \"C:/kit/data/servers/p/vhosts/*.conf\"")
        .unwrap();
    assert_eq!(conf.find("<VirtualHost"), Some(refused));
    assert!(refused < local && local < include);
    assert_eq!(conf.matches("<VirtualHost").count(), 2);
    assert!(!conf.contains("*.localhost"));
}

/// In LAN mode the Nginx pool keeps its own page as default server.
#[test]
fn nginx_pool_conf_lan_keeps_default_page() {
    let conf = nginx_pool_conf(
        Path::new(r"C:\kit\apps\nginx"),
        Path::new(r"C:\kit\data\servers\p"),
        8082,
        true,
    );
    assert!(conf.contains("        listen 8082 default_server;\r\n        server_name localhost;"));
    assert!(!conf.contains("return 421;"));
}

/// `id` installed at `version`, the default folder of its line, with the
/// key paths of the kit's target (`nginx.exe` on Windows, `sbin/nginx`
/// elsewhere).
fn fake_server(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    let dir = crate::versions::install_dir(kit, c, version);
    let folder = c.folder.as_deref().unwrap();
    let target = kit.target();
    for k in std::iter::once(c.key_for(target)).chain(c.keys_for(target)) {
        if let Some(rel) = k.strip_prefix(folder) {
            let file = crate::fsx::join_rel(&dir, rel);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(&file, b"").unwrap();
        }
    }
    c.save_version(kit, version).unwrap();
}

/// BUG-05r: with Nginx and PHP installed but no project, the default pool
/// starts no php-cgi; nor does a manual profile without a chosen PHP.
#[test]
fn default_nginx_pool_has_no_php_without_php_projects() {
    let tmp = TempDir::new("servers-default-php");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "nginx", "1.27.4");
    fake_server(&kit, "php", "8.4.13");
    let servers = WebServers::new(kit.clone());

    let view = servers.list("it", true);
    assert_eq!(view.php_lines, ["8.4"]);
    assert_eq!(view.pools.len(), 1);
    let pool = &view.pools[0];
    assert_eq!(pool.engine, WebServerEngine::Nginx);
    assert_eq!(pool.php_line, None);
    assert_eq!(pool.id, "nginx-1.27");

    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Static".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(8098),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let view = servers.list("it", true);
    let manual = view.pools.iter().find(|p| p.id == "static").unwrap();
    assert_eq!(manual.php_line, None);
    assert_eq!(manual.server_line.as_deref(), Some("1.27"));
}

#[cfg(windows)]
fn system32(exe: &str) -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    format!(r"{root}\System32\{exe}")
}

/// A program that runs for a minute, as a server would: `ping` from
/// System32 on Windows, `sleep` elsewhere.
fn long_running() -> std::process::Command {
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
fn exiting(code: u8) -> std::process::Command {
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

/// BUG-05r: a pool whose php-cgi ended is taken out of the running ones
/// (and its server ended), never left "running" answering 502.
#[test]
fn pool_with_dead_php_cgi_is_not_running() {
    let job = Job::new().unwrap();
    let server = jobs::spawn_in(&mut long_running(), &job).unwrap();
    let mut php = jobs::spawn_in(&mut exiting(0), &job).unwrap();
    let begun = Instant::now();
    while php.try_wait().unwrap().is_none() {
        assert!(begun.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(50));
    }
    let mut running = BTreeMap::new();
    running.insert(
        "nginx-php".to_owned(),
        RunningPool {
            child: server,
            port: 8099,
            fcgi: Some(FastCgi {
                port: 9000,
                key: "k".into(),
            }),
            php_line: Some("8.4".into()),
            php: Some(php),
            admin: None,
            started: nginx_pool("nginx-php", 8099, Vec::new()),
            by_user: true,
            attached: BTreeSet::new(),
            _job: job,
        },
    );
    // I2: `is_running` (what the runs follow) says so too.
    let servers = WebServers::new(Kit::at(std::env::temp_dir(), Arch::X64));
    std::mem::swap(&mut *servers.running.lock().unwrap(), &mut running);
    assert!(!servers.is_running("nginx-php"));
    assert!(!servers.is_running("nope"));
    std::mem::swap(&mut *servers.running.lock().unwrap(), &mut running);
    let dead = take_dead(&mut running);
    assert!(running.is_empty());
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].0, "nginx-php");
    end_dead(dead);
}

/// GAP-RUN: the pool of a project's site is the profile it was assigned to.
#[test]
fn pool_of_finds_assigned_project() {
    let tmp = TempDir::new("servers-pool-of");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Legacy".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(8096),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);

    servers
        .assign_project(
            "shop",
            "public",
            WebServerEngine::Nginx,
            Some("legacy".into()),
            &KitEnv::default(),
            "it",
        )
        .unwrap();
    let (pool, site) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.id, "legacy");
    assert_eq!(site.url, "http://shop.localhost:8096");
    assert_eq!(servers.pool_of("shop", "other", "it").map(|p| p.0.id), None);
    assert_eq!(servers.pool_of("nope", "site", "it").map(|p| p.0.id), None);

    // I3: moved to another profile behind the cache's back (no
    // invalidation): the cached scan still says "legacy", pool_of does not.
    servers
        .save_profile(
            WebServerProfileInput {
                id: "modern".into(),
                name: "Modern".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(8091),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    assert!(servers
        .list("it", false)
        .pools
        .iter()
        .any(|p| p.id == "legacy" && p.sites.iter().any(|s| s.project == "shop")));
    set_site_with_options(
        &shop,
        "public",
        SiteKind::Nginx,
        &SiteOptions {
            server: Some("modern".into()),
            ..SiteOptions::default()
        },
        "it",
    )
    .unwrap();
    let cached = servers.list("it", false);
    assert!(cached
        .pools
        .iter()
        .any(|p| p.id == "legacy" && p.sites.iter().any(|s| s.project == "shop")));
    let (pool, site) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.id, "modern");
    assert_eq!(site.url, "http://shop.localhost:8091");
    // Not running: no live port to attach a run to.
    assert_eq!(servers.running_port("modern"), None);
    assert!(!servers.is_running("modern"));
}

/// I1: a reload restarts the pool (never a graceful Nginx reload keeping
/// no php-cgi) when PHP is wanted and not running, or changed; Apache and
/// a new port always restart.
#[test]
fn reload_restarts_when_php_or_port_changes() {
    let live = |php: Option<&str>, fcgi: Option<u16>| LivePool {
        port: 8093,
        php_line: php.map(str::to_owned),
        fcgi: fcgi.map(|port| FastCgi {
            port,
            key: "k".into(),
        }),
    };
    let mut pool = nginx_pool("legacy", 8093, Vec::new());
    // Static pool, static sites: graceful.
    assert!(!needs_restart(&pool, &live(None, None)));
    // A PHP site assigned to a running static pool: restart (I1).
    pool.php_line = Some("8.4".into());
    assert!(needs_restart(&pool, &live(None, None)));
    // Started with PHP but php-cgi missing: try again.
    assert!(needs_restart(&pool, &live(Some("8.4"), None)));
    // Same PHP, php-cgi running: graceful.
    assert!(!needs_restart(&pool, &live(Some("8.4"), Some(9001))));
    // Another PHP line, or PHP gone: restart.
    assert!(needs_restart(&pool, &live(Some("8.3"), Some(9001))));
    pool.php_line = None;
    assert!(needs_restart(&pool, &live(Some("8.4"), Some(9001))));
    // Another port: restart.
    pool.port = 8092;
    assert!(needs_restart(&pool, &live(None, None)));
    // Apache: always.
    let mut apache = nginx_pool("a", 8093, Vec::new());
    apache.engine = WebServerEngine::Apache;
    assert!(needs_restart(&apache, &live(None, None)));
}

/// BUG-V3-02: two web actions of one project in the same pool are two
/// VirtualHosts with their own ServerName and file; a stale file goes.
#[test]
fn two_actions_of_one_project_get_distinct_vhosts() {
    let tmp = TempDir::new("servers-two-actions");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: "legacy".into(),
                name: "Legacy".into(),
                engine: WebServerEngine::Nginx,
                server_line: None,
                php_line: None,
                port: Some(8087),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::create_dir_all(shop.join("docs")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [
            { "id": "npm:docs", "nginx": "docs", "server": "legacy" },
            { "id": "site", "nginx": "public", "server": "legacy" } ] }"#,
    )
    .unwrap();
    let view = servers.list("it", true);
    let pool = view.pools.iter().find(|p| p.id == "legacy").unwrap();
    let names: BTreeMap<&str, &str> = pool
        .sites
        .iter()
        .map(|s| (s.action.as_str(), s.server_name.as_str()))
        .collect();
    assert_eq!(names["site"], "shop.localhost");
    assert_eq!(names["npm:docs"], "npm-docs.shop.localhost");
    assert!(names.values().all(|n| alias::valid_domain(n)));
    let docs = pool.sites.iter().find(|s| s.action == "npm:docs").unwrap();
    assert_eq!(docs.url, "http://npm-docs.shop.localhost:8087");

    let vhosts = kit.data().join("servers").join("legacy").join("vhosts");
    fs::create_dir_all(&vhosts).unwrap();
    fs::write(vhosts.join("shop.conf"), "stale").unwrap();
    servers.write_pool_configs(pool, None, false).unwrap();
    let mut files: Vec<String> = fs::read_dir(&vhosts)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort();
    assert_eq!(
        files,
        ["npm-docs.shop.localhost.conf", "shop.localhost.conf"]
    );
    let main = fs::read_to_string(vhosts.join("shop.localhost.conf")).unwrap();
    assert!(main.contains("server_name shop.localhost;"), "{main}");
    assert!(main.contains("shop/public"), "{main}");
    let other = fs::read_to_string(vhosts.join("npm-docs.shop.localhost.conf")).unwrap();
    assert!(
        other.contains("server_name npm-docs.shop.localhost;"),
        "{other}"
    );
    assert!(other.contains("shop/docs"), "{other}");
}

/// Defect 7 (PLAN-DATA §9): a project is put in the pools once per site:
/// the detected `apache` and `nginx` give way to the sites of `panel.json`,
/// and only the first of them is taken.
#[test]
fn pooled_actions_drop_detected_duplicates() {
    use crate::detect::{Action, Kind};
    let web = |id: &str, apache: bool, custom: bool| {
        let mut a = Action::for_test(id, "", Kind::Web);
        if apache {
            a.apache_dir = Some("public".into());
        } else {
            a.nginx_dir = Some("public".into());
        }
        a.custom = custom;
        a
    };
    let ids = |actions: &[Action]| -> Vec<String> {
        pooled_actions(actions)
            .into_iter()
            .map(|a| a.id.clone())
            .collect()
    };
    let cmd = Action::for_test("php:server", "php -S", Kind::Web);
    let detected = [web("apache", true, false), web("nginx", false, false), cmd];
    assert_eq!(ids(&detected), ["apache"]);
    let mut with_site = detected.to_vec();
    with_site.push(web("site", false, true));
    with_site.push(web("docs", true, true));
    assert_eq!(ids(&with_site), ["site", "docs"]);
    assert!(ids(&[Action::for_test("dev", "npm run dev", Kind::Web)]).is_empty());
}

/// Defect 6: two pools never share a port: a profile is refused the port
/// of another one, and a new pool never falls back on a pool's port.
#[test]
fn ports_are_unique_among_pools() {
    let tmp = TempDir::new("servers-unique-ports");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let profile = |id: &str, port: u16| WebServerProfileInput {
        id: id.into(),
        name: id.into(),
        engine: WebServerEngine::Nginx,
        server_line: None,
        php_line: None,
        port: Some(port),
        keep_alive: true,
        autostart: false,
    };
    servers.save_profile(profile("one", 8085), "it").unwrap();
    let err = servers
        .save_profile(profile("two", 8085), "it")
        .unwrap_err()
        .to_string();
    assert!(err.contains("8085") && err.contains("one"), "{err}");
    // The same profile saved again keeps its port.
    servers.save_profile(profile("one", 8085), "it").unwrap();
    servers.save_profile(profile("two", 8084), "it").unwrap();

    // Every port of the range taken by a pool but one: that one, even
    // when another program holds it.
    let mut store = load_store(&kit);
    for (i, p) in (POOL_PORT_FROM..=POOL_PORT_TO).enumerate() {
        if p != 8500 {
            store.ports.insert(format!("p{i}"), p);
        }
    }
    let mut dirty = false;
    assert_eq!(
        servers.assign_or_get_port(&mut store, &mut dirty, "new", None),
        8500
    );
    assert!(dirty);

    // A servers.json of before: an automatic pool stored on the port a
    // profile was given by hand gets a port of its own.
    let mut store = load_store(&kit);
    store.ports.insert("one".to_owned(), 8085);
    store.ports.insert("auto".to_owned(), 8085);
    let mut dirty = false;
    let p = servers.assign_or_get_port(&mut store, &mut dirty, "auto", None);
    assert_ne!(p, 8085);
    assert!(dirty);
    assert_eq!(
        servers.assign_or_get_port(&mut store, &mut dirty, "one", Some(8085)),
        8085
    );
}

/// Defect 4: a pool whose server or PHP line is not installed does not
/// start (never the default line in its place), with an error naming it.
#[test]
fn start_refuses_a_line_not_installed() {
    let tmp = TempDir::new("servers-missing-line");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "nginx", "1.27.4");
    fake_server(&kit, "php", "8.4.13");
    let servers = WebServers::new(kit.clone());
    let port = {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    let profile = |server_line: &str, php_line: &str| WebServerProfileInput {
        id: "old".into(),
        name: "Old".into(),
        engine: WebServerEngine::Nginx,
        server_line: Some(server_line.into()),
        php_line: Some(php_line.into()),
        port: Some(port),
        keep_alive: true,
        autostart: false,
    };
    servers.save_profile(profile("1.25", "8.4"), "it").unwrap();
    let err = servers
        .start("old", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("Nginx 1.25 non è installato"), "{err}");
    servers.save_profile(profile("1.27", "7.4"), "it").unwrap();
    let err = servers
        .start("old", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("PHP 7.4 non è installato"), "{err}");
    assert!(!servers.is_running("old"));
}

/// A pool of `id` "running" (a `ping` in a Job) as started from `started`.
fn fake_running(servers: &WebServers, started: WebServerPool) {
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

/// Defect 5: a running pool the scan does not have (an id that is neither a
/// profile's nor an automatic one) stays on the page, marked, until
/// stopped; it serves no new run and is not reloaded.
#[test]
fn orphan_running_pool_is_listed_until_stopped() {
    let tmp = TempDir::new("servers-orphan");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    fake_running(
        &servers,
        nginx_pool("gone", 8083, vec![site("shop", "public", 8083)]),
    );
    let view = servers.list("it", true);
    assert_eq!(view.pools.len(), 1);
    let orphan = &view.pools[0];
    assert!(orphan.orphan && orphan.running && orphan.pid.is_some());
    assert_eq!(orphan.sites[0].project, "shop");
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);
    let err = servers
        .reload("gone", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.contains("fermalo"), "{err}");
    assert!(servers.is_running("gone"));
    servers.stop("gone");
    assert!(servers.list("it", true).pools.is_empty());
}

/// Defect 8: assigning a project reloads only the running pool that
/// serves it (never the others) and reports its failure.
#[test]
fn assign_project_reloads_only_its_pool_and_reports_errors() {
    let tmp = TempDir::new("servers-assign-reload");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
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
                    engine: WebServerEngine::Nginx,
                    server_line: None,
                    php_line: None,
                    port: Some(port),
                    keep_alive: true,
                    autostart: false,
                },
                "it",
            )
            .unwrap();
        fake_running(&servers, nginx_pool(id, port, Vec::new()));
    }
    fs::create_dir_all(kit.projects().join("shop").join("public")).unwrap();
    // The reload of `legacy` restarts it, and there is no Nginx here.
    let err = servers
        .assign_project(
            "shop",
            "public",
            WebServerEngine::Nginx,
            Some("legacy".into()),
            &KitEnv::default(),
            "it",
        )
        .unwrap_err()
        .to_string();
    assert!(err.contains("Nginx is not installed"), "{err}");
    assert!(!servers.is_running("legacy"));
    assert!(servers.is_running("other"), "not touched");
    servers.stop("other");
}

/// Defect 9: php-cgi is waited for on its FastCGI port; one that ends at
/// once (its port taken in the meantime) is reported, to try another port.
#[test]
fn php_cgi_listening_tells_ended_from_ready() {
    let job = Job::new().unwrap();
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let mut alive = jobs::spawn_in(&mut long_running(), &job).unwrap();
    let begun = Instant::now();
    assert!(php_cgi_listening(&mut alive, port, Duration::from_secs(5)));
    assert!(begun.elapsed() < Duration::from_secs(2), "answers at once");
    drop(listener);
    // Alive but not listening yet: slow, kept.
    assert!(php_cgi_listening(
        &mut alive,
        port,
        Duration::from_millis(300)
    ));
    let _ = alive.kill();
    let _ = alive.wait();
    let mut ended = jobs::spawn_in(&mut exiting(1), &job).unwrap();
    assert!(!php_cgi_listening(&mut ended, port, Duration::from_secs(5)));
}

/// PLAN.md §13.1: a pool stops with its last project only when projects
/// started it and it is not kept alive.
#[test]
fn stops_when_left_follows_who_started_it() {
    assert!(stops_when_left(false, 0, false));
    assert!(!stops_when_left(false, 1, false), "another project left");
    assert!(!stops_when_left(false, 0, true), "kept alive");
    assert!(!stops_when_left(true, 0, false), "started by the user");
}

/// PLAN.md §13.1: an older `servers.json` (no options) keeps its profiles
/// alive and starts none with the app; the options are saved and read back.
#[test]
fn profile_options_default_and_round_trip() {
    let old: StoredServers = serde_json::from_str(
        r#"{ "profiles": [ { "id": "legacy", "name": "Legacy", "engine": "nginx",
            "port": 8090 } ], "ports": { "legacy": 8090 } }"#,
    )
    .unwrap();
    assert!(old.profiles[0].keep_alive);
    assert!(!old.profiles[0].autostart);

    let tmp = TempDir::new("servers-options");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let mut profile = old.profiles[0].clone();
    profile.keep_alive = false;
    profile.autostart = true;
    servers.save_profile(profile, "it").unwrap();
    let text = fs::read_to_string(store_path(&kit)).unwrap();
    assert!(text.contains("\"keepAlive\": false") && text.contains("\"autostart\": true"));
    let view = servers.list("it", true);
    let saved = view.pools[0].profile.as_ref().unwrap();
    assert!(!saved.keep_alive && saved.autostart);
    assert_eq!(
        saved.server_line, None,
        "the profile as saved, not resolved"
    );

    // With the app: tried, and with no Nginx in this kit only logged.
    servers.autostart(&KitEnv::default(), "it");
    assert!(!servers.is_running("legacy"));
}

/// PLAN.md §13.1: a stopped project's site leaves the pool's files; an
/// automatic pool started by projects stops with the last of them.
#[test]
fn detach_leaves_the_pool_and_stops_an_empty_automatic_one() {
    let tmp = TempDir::new("servers-detach");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();
    fs::create_dir_all(projects.join("shop").join("public")).unwrap();
    fs::create_dir_all(projects.join("blog").join("public")).unwrap();
    let pool = nginx_pool(
        "nginx-auto",
        8082,
        vec![site("shop", "public", 8082), site("blog", "public", 8082)],
    );

    // Not running: only left out of the files written for the next start.
    servers
        .detach("nginx-auto", "shop", "site", &KitEnv::default(), "it")
        .unwrap();
    let conf = servers.write_pool_configs(&pool, None, false).unwrap();
    let vhosts = conf.parent().unwrap().join("vhosts");
    assert!(!vhosts.join("shop.localhost.conf").exists());
    assert!(vhosts.join("blog.localhost.conf").exists());

    // Started by the blog only: its stop stops the automatic pool.
    fake_running(&servers, pool.clone());
    {
        let mut running = servers.running.lock().unwrap();
        let r = running.get_mut("nginx-auto").unwrap();
        r.by_user = false;
        r.attached.insert(("blog".into(), "site".into()));
    }
    servers
        .detach("nginx-auto", "blog", "site", &KitEnv::default(), "it")
        .unwrap();
    assert!(!servers.is_running("nginx-auto"));
    assert!(servers
        .detached
        .lock()
        .unwrap()
        .contains(&("blog".into(), "site".into())));
}

/// The kit given by DEVKIT_TEST_KIT with Apache installed, its environment,
/// and a free port; `None` (skipped) otherwise.
fn real_apache() -> Option<(Kit, KitEnv, u16)> {
    let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
        eprintln!("DEVKIT_TEST_KIT not set: skipped");
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
        &crate::settings::Settings::default(),
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
fn real_profile(servers: &WebServers, id: &str, port: u16) {
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

fn drop_real_profile(kit: &Kit, servers: &WebServers, id: &str) {
    servers.remove_profile(id).unwrap();
    let _ = fs::remove_dir_all(kit.data().join("servers").join(id));
}

/// BUG-03 with a real httpd (DEVKIT_TEST_KIT): a syntax error in the pool's
/// configuration reaches the error of the start, not only a log.
#[test]
fn start_reports_syntax_error() {
    let Some((kit, env, port)) = real_apache() else {
        return;
    };
    let servers = WebServers::new(kit.clone());
    let id = "devkit-test-syntax";
    real_profile(&servers, id, port);
    // conf.d\ is the user's and is kept by every start.
    let conf_d = kit.data().join(r"servers").join(id).join("conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    fs::write(conf_d.join("bad.conf"), "Lisen 1\r\n").unwrap();
    let err = servers.start(id, &env, "en").map_err(|e| e.to_string());
    drop_real_profile(&kit, &servers, id);
    let err = err.unwrap_err();
    assert!(err.contains("Lisen"), "{err}");
    assert!(!servers.is_running(id));
}

/// BUG-04 with a real httpd (DEVKIT_TEST_KIT): two reloads in a row (each a
/// stop and start, Apache has no graceful reload on Windows) find the port
/// free again and leave the pool answering.
#[test]
fn apache_pool_reload_twice() {
    let Some((kit, env, port)) = real_apache() else {
        return;
    };
    let servers = WebServers::new(kit.clone());
    let id = "devkit-test-reload";
    real_profile(&servers, id, port);
    let answers = || {
        TcpStream::connect_timeout(
            &SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_secs(1),
        )
        .is_ok()
    };
    let result = (|| -> Result<()> {
        servers.start(id, &env, "en")?;
        let first = servers.list("en", false).pools;
        let pid = first.iter().find(|p| p.id == id).and_then(|p| p.pid);
        for _ in 0..2 {
            servers.reload(id, &env, "en")?;
            anyhow::ensure!(servers.is_running(id), "running after the reload");
            anyhow::ensure!(answers(), "answers on {port} after the reload");
        }
        let now = servers.list("en", false).pools;
        let again = now.iter().find(|p| p.id == id).and_then(|p| p.pid);
        anyhow::ensure!(pid != again, "a new httpd: {pid:?} {again:?}");
        Ok(())
    })();
    servers.stop(id);
    drop_real_profile(&kit, &servers, id);
    result.unwrap();
    assert!(wait_port_released(port, Duration::from_secs(5)));
}

/// `GET path` with `Host: host` on 127.0.0.1:`port`: the status and the body.
fn http_get(port: u16, host: &str, path: &str) -> Result<(u16, String)> {
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

/// WP-3.3 acceptance with a real Caddy and PHP (DEVKIT_TEST_KIT): a pool
/// with two PHP sites and an alias to another project's folder; a third
/// site added with `caddy reload` while the first one keeps answering, the
/// same Caddy process throughout.
#[test]
fn caddy_pool_two_php_sites_alias_and_reload() {
    let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
        eprintln!("DEVKIT_TEST_KIT not set: skipped");
        return;
    };
    // The host's temporary folder holds the pool's admin socket.
    let kit = Kit::at(PathBuf::from(root), Arch::detect()).with_host(Some(&std::env::temp_dir()));
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    for id in ["caddy", "php"] {
        if !catalog.find(id).is_some_and(|c| c.is_installed(&kit)) {
            eprintln!("{id} not installed: skipped");
            return;
        }
    }
    crate::setup::kit_files::write_all(&kit).unwrap();
    let env = KitEnv::build(
        &kit,
        &crate::settings::Settings::default(),
        "en",
        &crate::env::Host::capture(),
    );
    let port = {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    let id = "p6-caddy-pool";
    let projects = kit.projects();
    let site = |name: &str, extra: &str| {
        let public = projects.join(name).join("public");
        fs::create_dir_all(&public).unwrap();
        fs::write(
            public.join("index.php"),
            format!("<?php echo 'PHP ' . PHP_VERSION . ' {name} ' . $_SERVER['REQUEST_URI'];"),
        )
        .unwrap();
        fs::write(public.join(".env"), "SECRET=1").unwrap();
        fs::write(
            projects.join(name).join("panel.json"),
            format!(
                r#"{{ "actions": [ {{ "id": "site", "caddy": "public", "server": "{id}"{extra} }} ] }}"#
            ),
        )
        .unwrap();
    };
    site(
        "p6-alpha",
        r#", "aliases": { "/shared": "@p6-beta/assets" }"#,
    );
    site("p6-beta", "");
    fs::create_dir_all(projects.join("p6-beta").join("assets")).unwrap();
    fs::write(
        projects.join("p6-beta").join("assets").join("logo.txt"),
        "beta asset",
    )
    .unwrap();
    let servers = WebServers::new(kit.clone());
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
            "en",
        )
        .unwrap();
    let result = (|| -> Result<()> {
        servers.start(id, &env, "en")?;
        let (s, body) = http_get(port, "p6-alpha.localhost", "/some/route?x=1")?;
        eprintln!("alpha: {s} {body}");
        anyhow::ensure!(
            s == 200 && body.contains("p6-alpha /some/route?x=1"),
            "alpha"
        );
        let (s, body) = http_get(port, "p6-beta.localhost", "/")?;
        eprintln!("beta: {s} {body}");
        anyhow::ensure!(s == 200 && body.contains("PHP 8.") && body.contains("p6-beta"));
        let (s, body) = http_get(port, "p6-alpha.localhost", "/shared/logo.txt")?;
        eprintln!("alias: {s} {body}");
        anyhow::ensure!(s == 200 && body.contains("beta asset"), "alias");
        let (s, _) = http_get(port, "p6-alpha.localhost", "/.env")?;
        anyhow::ensure!(s == 404, ".env: {s}");
        let (s, _) = http_get(port, "evil.example", "/")?;
        anyhow::ensure!(s == 421, "other host: {s}");
        let pid = servers
            .list("en", true)
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .and_then(|p| p.pid);

        // The first site is asked all along the reload.
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let asker = {
            let stop = std::sync::Arc::clone(&stop);
            std::thread::spawn(move || {
                let (mut ok, mut failed) = (0u32, Vec::new());
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    match http_get(port, "p6-alpha.localhost", "/") {
                        Ok((200, b)) if b.contains("p6-alpha") => ok += 1,
                        other => failed.push(format!("{other:?}")),
                    }
                }
                (ok, failed)
            })
        };
        std::thread::sleep(Duration::from_millis(300));
        site("p6-gamma", "");
        servers.invalidate();
        servers.reload(id, &env, "en")?;
        std::thread::sleep(Duration::from_millis(300));
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        let (ok, failed) = asker.join().unwrap();
        eprintln!(
            "during the reload: {ok} answers, {} failed {failed:?}",
            failed.len()
        );
        anyhow::ensure!(ok > 0 && failed.is_empty(), "interrupted: {failed:?}");
        let (s, body) = http_get(port, "p6-gamma.localhost", "/")?;
        eprintln!("gamma: {s} {body}");
        anyhow::ensure!(
            s == 200 && body.contains("p6-gamma"),
            "gamma after the reload"
        );
        let again = servers
            .list("en", true)
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .and_then(|p| p.pid);
        anyhow::ensure!(
            pid.is_some() && pid == again,
            "same Caddy: {pid:?} {again:?}"
        );
        Ok(())
    })();
    servers.stop(id);
    drop_real_profile(&kit, &servers, id);
    for p in ["p6-alpha", "p6-beta", "p6-gamma"] {
        let _ = fs::remove_dir_all(projects.join(p));
    }
    result.unwrap();
}

/// BUG-V3-02: names that give the same label in one pool stay distinct.
#[test]
fn name_sites_keeps_names_unique() {
    let mut sites = vec![
        site("My Shop", "public", 8080),
        site("my-shop", "public", 8080),
        site("___", "public", 8080),
    ];
    sites[1].action = "web".into();
    name_sites(&mut sites, 8080);
    let names: Vec<&str> = sites.iter().map(|s| s.server_name.as_str()).collect();
    assert_eq!(
        names,
        ["my-shop.localhost", "my-shop-2.localhost", "site.localhost"]
    );
    assert_eq!(sites[1].url, "http://my-shop-2.localhost:8080");
    assert_eq!(host_label("Ünï Côde_42"), "n-c-de-42");
}

/// PLAN-DATA §9 1.1: an automatic pool lists (and serves) the sites the
/// projects chose in `panel.json` and those started, not every PHP project
/// that was only offered a site (Caddy since WP-3.3).
#[test]
fn automatic_pools_list_chosen_or_started_sites() {
    let tmp = TempDir::new("servers-chosen-sites");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "caddy", "2.11.7");
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();
    // Offered Caddy next to `php -S`.
    let plain = projects.join("plain").join("public");
    fs::create_dir_all(&plain).unwrap();
    fs::write(plain.join("index.php"), "<?php").unwrap();
    // A site written in panel.json.
    let shop = projects.join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(shop.join("public").join("index.php"), "<?php").unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "caddy": "public" } ] }"#,
    )
    .unwrap();

    let projects_of = |pools: &[WebServerPool]| -> Vec<String> {
        pools
            .iter()
            .flat_map(|p| p.sites.iter().map(|s| s.project.clone()))
            .collect()
    };
    let all = servers.list_all("it", true);
    assert_eq!(projects_of(&all.pools), ["plain", "shop"]);
    let shown = servers.list("it", true);
    assert_eq!(projects_of(&shown.pools), ["shop"]);
    // Every project goes to the same pool: it is not left out with them.
    let (pool, _) = servers.pool_of("plain", "caddy", "it").unwrap();
    assert_eq!(pool.engine, WebServerEngine::Caddy);
    assert!(shown.pools.iter().any(|p| p.id == pool.id));

    let pool_dir = kit.data().join("servers").join(&pool.id);
    // The sites written in the pool's Caddyfile.
    let served = || -> Vec<&'static str> {
        let text = fs::read_to_string(pool_dir.join("Caddyfile")).unwrap();
        ["plain", "shop"]
            .into_iter()
            .filter(|p| text.contains(&format!("http://{p}.localhost:{}", pool.port)))
            .collect()
    };
    servers.write_pool_configs(&pool, None, false).unwrap();
    assert_eq!(served(), ["shop"]);

    // Started by its project: listed and served.
    servers
        .started
        .lock()
        .unwrap()
        .insert(("plain".into(), "caddy".into()));
    assert_eq!(
        projects_of(&servers.list("it", true).pools),
        ["plain", "shop"]
    );
    servers.write_pool_configs(&pool, None, false).unwrap();
    assert_eq!(served(), ["plain", "shop"]);

    // Stopped: out again (the pool is not running, nothing to reload).
    servers
        .detach(&pool.id, "plain", "caddy", &KitEnv::default(), "it")
        .unwrap();
    assert_eq!(projects_of(&servers.list("it", true).pools), ["shop"]);

    // A chosen site whose project stopped: still listed, marked detached.
    let shop_site = |servers: &WebServers| {
        servers
            .list("it", false)
            .pools
            .into_iter()
            .flat_map(|p| p.sites)
            .find(|s| s.project == "shop")
            .unwrap()
    };
    assert!(!shop_site(&servers).detached);
    servers
        .detach(&pool.id, "shop", "site", &KitEnv::default(), "it")
        .unwrap();
    let shop = shop_site(&servers);
    assert!(shop.detached);
    assert_eq!(shop.server_name, "shop.localhost");
}

/// T3.3.1: Caddy is an engine of the pools: its automatic ids read back,
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

/// T3.3.2: a Caddy pool's site block: its own name and its domains, its
/// document root, PHP with the start's key, the aliases checked on disk
/// only (no alias that was not resolved in the text, as Apache and Nginx).
#[test]
fn caddy_site_block_of_a_pool() {
    let tmp = TempDir::new("servers-caddy-site");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join("shop").join("public")).unwrap();
    fs::create_dir_all(projs.join("blog").join("uploads")).unwrap();
    let mut s = site("shop", "public", 8090);
    s.domains = vec!["api.shop.localhost".into(), "no;way.localhost".into()];
    s.aliases.insert("/blog".into(), "@blog/uploads".into());
    s.aliases.insert("/gone".into(), "nowhere".into());
    s.aliases.insert("/up".into(), "../blog/uploads".into());
    let aliases = crate::projects::caddy::resolve_aliases(&projs.join("shop"), &projs, &s.aliases);
    let fcgi = FastCgi {
        port: 9333,
        key: "k".repeat(64),
    };
    let doc = projs.join("shop").join("public");
    let block = super::config::caddy_site_conf(8090, false, &doc, &aliases, Some(&fcgi), &s)
        .unwrap()
        .join("\n");
    assert!(
        block.starts_with(
            "http://shop.localhost:8090, http://api.shop.localhost:8090 {\n\tbind 127.0.0.1\n"
        ),
        "{block}"
    );
    assert!(block.contains(&format!(
        "\t\troot * \"{}\"\n",
        crate::projects::apache::fwd(&doc)
    )));
    assert!(block.contains("\t\t\tenv DEVKIT_FCGI_KEY {env.DEVKIT_FCGI_KEY}\n"));
    assert!(!block.contains(&"k".repeat(64)), "the key is never written");
    assert!(block.contains("\t\thandle_path /blog/* {\n"));
    assert!(block.contains(&format!(
        "\t\t\troot * \"{}/blog/uploads\"\n",
        crate::projects::apache::fwd(&projs)
    )));
    for absent in ["/gone", "redir /up ", "/up/*", "nowhere", "no;way", ".."] {
        assert!(!block.contains(absent), "{absent}: {block}");
    }
    // A folder Caddy cannot take is not served at all.
    let bad = Path::new("/kit/{x}/public");
    assert!(super::config::caddy_site_conf(8090, false, bad, &[], None, &s).is_none());
}

/// A Caddy pool's files: one Caddyfile with the sites it serves, the PHP
/// key, its admin endpoint, the user's snippets only when there are some.
#[test]
fn caddy_pool_writes_one_caddyfile() {
    let tmp = TempDir::new("servers-caddy-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    for p in ["shop", "blog"] {
        fs::create_dir_all(kit.projects().join(p).join("public")).unwrap();
    }
    let mut pool = nginx_pool(
        "caddy-2.11_php-8.4",
        8091,
        vec![site("shop", "public", 8091), site("blog", "public", 8091)],
    );
    pool.engine = WebServerEngine::Caddy;
    let fcgi = FastCgi {
        port: 9444,
        key: "z".repeat(64),
    };
    let sock = tmp.0.join("host").join("caddy-1").join("a.sock");
    let conf = servers
        .write_pool_configs_with(&pool, Some(&fcgi), false, Some(&sock))
        .unwrap();
    assert_eq!(conf.file_name().unwrap(), "Caddyfile");
    let text = fs::read_to_string(&conf).unwrap();
    if crate::projects::caddy::socket_path_usable(&sock) {
        assert!(text.contains(&format!(
            "\tadmin \"{}|0600\"\n",
            crate::projects::caddy::admin_address(&sock)
        )));
    }
    assert!(!text.contains("admin 127.0.0.1") && !text.contains(&"z".repeat(64)));
    assert!(text.contains("http://shop.localhost:8091 {\n"));
    assert!(text.contains("http://blog.localhost:8091 {\n"));
    assert!(text.contains("php_fastcgi 127.0.0.1:9444 {"));
    assert!(text.contains("respond 421"));
    assert!(!text.contains("import "));
    let snippets = kit.data().join("servers").join(&pool.id).join("conf.d");
    fs::write(snippets.join("mine.caddy"), "").unwrap();
    let text = fs::read_to_string(servers.write_pool_configs(&pool, None, false).unwrap()).unwrap();
    assert!(
        text.contains("\tadmin off\n") && text.contains("import \""),
        "{text}"
    );
    assert!(text.contains("respond @php 503"));
}

/// PLAN.md §13.1: a site marked `dedicated` keeps its own server (in no
/// pool); a new profile never takes an id a site's `server` means otherwise.
#[test]
fn dedicated_sites_and_reserved_profile_ids() {
    let tmp = TempDir::new("servers-dedicated");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "apache": "public", "server": "dedicated" } ] }"#,
    )
    .unwrap();
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);

    let profile = |name: &str| WebServerProfileInput {
        id: String::new(),
        name: name.into(),
        engine: WebServerEngine::Apache,
        server_line: None,
        php_line: None,
        port: None,
        keep_alive: true,
        autostart: false,
    };
    for (name, id) in [
        ("Apache 2.4", "apache-2.4-profile"),
        ("Dedicated", "dedicated-profile"),
        ("Auto", "auto-profile"),
        ("Apache test", "apache-test"),
    ] {
        servers.save_profile(profile(name), "it").unwrap();
        assert!(
            load_store(&kit).profiles.iter().any(|p| p.id == id),
            "{name}"
        );
    }
}

/// The pools of the page: an automatic pool with no site left is not
/// shown, unless it runs or it is the only pool (the one to start).
#[test]
fn shown_pools_drop_empty_automatic_ones() {
    let mut hidden = site("plain", "public", 8081);
    hidden.chosen = false;
    let started = BTreeSet::new();
    let pools = vec![
        nginx_pool("nginx-1.27", 8081, vec![hidden.clone()]),
        nginx_pool("nginx-1.26", 8082, vec![site("shop", "public", 8082)]),
    ];
    let ids =
        |pools: &[WebServerPool]| -> Vec<String> { pools.iter().map(|p| p.id.clone()).collect() };
    assert_eq!(ids(&shown_pools(pools.clone(), &started)), ["nginx-1.26"]);
    let mut running = pools.clone();
    running[0].running = true;
    let shown = shown_pools(running, &started);
    assert_eq!(ids(&shown), ["nginx-1.27", "nginx-1.26"]);
    assert!(shown[0].sites.is_empty());
    let only = shown_pools(vec![pools[0].clone()], &started);
    assert_eq!(ids(&only), ["nginx-1.27"]);
}

/// SC-2: automatic ids name their lines (`apache-2.4_php-7.4`); the ids of
/// before are read too, and a profile's id is not one.
#[test]
fn auto_pool_ids_name_their_lines() {
    let apache = WebServerEngine::Apache;
    assert_eq!(
        auto_pool_id(apache, Some("2.4"), Some("7.4")),
        "apache-2.4_php-7.4"
    );
    assert_eq!(
        auto_pool_id(WebServerEngine::Nginx, Some("1.27"), None),
        "nginx-1.27"
    );
    assert_eq!(auto_pool_id(apache, None, None), "apache-default");
    let lines = |s: Option<&str>, p: Option<&str>| {
        Some((apache, s.map(str::to_owned), p.map(str::to_owned)))
    };
    assert_eq!(
        parse_auto_pool_id("apache-2.4_php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(
        parse_auto_pool_id("apache-2.4-php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(
        parse_auto_pool_id("apache-default_php-8.4"),
        lines(None, Some("8.4"))
    );
    assert_eq!(
        canonical_auto_pool("apache-2.4_php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(canonical_auto_pool("apache-2.4-php-7.4"), None);
    for other in [
        "legacy",
        "apache",
        "apache-test",
        "nginx-1.27-php-x",
        "iis-1.0",
    ] {
        assert_eq!(parse_auto_pool_id(other), None, "{other}");
    }
}

/// SC-2: the ports stored under the ids of before move to the new ones (a
/// port already stored there wins); a profile's id stays as it is.
#[test]
fn ports_move_to_the_new_automatic_ids() {
    let mut store = StoredServers::default();
    store.ports.insert("apache-2.4-php-7.4".into(), 8081);
    store.ports.insert("nginx-1.27-php-8.4".into(), 8082);
    store.ports.insert("nginx-1.27_php-8.4".into(), 8090);
    store.ports.insert("nginx-1.26".into(), 8083);
    store.ports.insert("apache-2.2-php-5.6".into(), 8084);
    store.profiles.push(WebServerProfileInput {
        id: "apache-2.2-php-5.6".into(),
        name: "Old".into(),
        engine: WebServerEngine::Apache,
        server_line: None,
        php_line: None,
        port: Some(8084),
        keep_alive: true,
        autostart: false,
    });
    assert!(migrate_port_ids(&mut store));
    let ports: Vec<(&str, u16)> = store.ports.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    assert_eq!(
        ports,
        [
            ("apache-2.2-php-5.6", 8084),
            ("apache-2.4_php-7.4", 8081),
            ("nginx-1.26", 8083),
            ("nginx-1.27_php-8.4", 8090),
        ]
    );
    assert!(!migrate_port_ids(&mut store));
}

/// SC-2: a running automatic pool no project goes to any more stays a pool
/// of the list (not an orphan), rebuilt from its id; a site naming an
/// automatic pool in `panel.json` (old id too) goes to the lines it names.
#[test]
fn automatic_pools_come_back_from_their_ids() {
    let tmp = TempDir::new("servers-stable-ids");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fake_server(&kit, "nginx", "1.27.4");
    let servers = WebServers::new(kit.clone());
    fake_running(&servers, nginx_pool("nginx-1.25", 8083, Vec::new()));
    let view = servers.list("it", true);
    let old = view.pools.iter().find(|p| p.id == "nginx-1.25").unwrap();
    assert!(!old.orphan && old.running);
    assert_eq!(old.server_line.as_deref(), Some("1.25"));
    assert!(load_store(&kit).ports.contains_key("nginx-1.25"));
    servers.stop("nginx-1.25");

    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "nginx": "public", "server": "nginx-1.26-php-8.3" } ] }"#,
    )
    .unwrap();
    let (pool, _) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.id, "nginx-1.26_php-8.3");
    assert_eq!(pool.server_line.as_deref(), Some("1.26"));
    assert_eq!(pool.php_line.as_deref(), Some("8.3"));
    assert!(!pool.manual);
}
