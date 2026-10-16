use super::*;
use crate::kit::{tests::TempDir, Arch};

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
    fs::create_dir_all(projects.join(r"portale\storage\uploads")).unwrap();
    fs::create_dir_all(projects.join(r"common\assets")).unwrap();
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
    };
    let vhost = apache_vhost_conf(
        8095,
        &projects.join("portale"),
        &projects,
        &projects.join(r"portale\public"),
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
    fs::create_dir_all(projects.join(r"alpha\public")).unwrap();
    fs::create_dir_all(projects.join(r"beta\public")).unwrap();

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
            },
            AttachedSite {
                project: "beta".into(),
                action: "site".into(),
                folder: "public".into(),
                server_name: "beta.localhost".into(),
                domains: Vec::new(),
                aliases: BTreeMap::new(),
                url: "http://beta.localhost:8088".into(),
            },
        ],
        conf_dir: r"data\servers\nginx-default".into(),
        orphan: false,
        profile: None,
    };

    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let main_conf = fs::read_to_string(&conf_path).unwrap();
    assert!(main_conf.contains("listen 127.0.0.1:8088 default_server;"));
    assert!(main_conf.contains("server_name localhost;"));

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
        conf_dir: format!(r"data\servers\{id}"),
        orphan: false,
        profile: None,
    }
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

    fs::create_dir_all(projects.join(r"good\public")).unwrap();
    // `missing` has no public\ folder: only its secrets at the root.
    fs::create_dir_all(projects.join("missing")).unwrap();
    fs::write(projects.join(r"missing\.env"), "DB_PASSWORD=x").unwrap();
    // `linked` has public\ as a junction to a folder outside the projects.
    fs::create_dir_all(projects.join("linked")).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    crate::os::safe_path::tests::junction(&projects.join(r"linked\public"), &outside);

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
    let child = jobs::spawn_in(
        std::process::Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
        &job,
    )
    .unwrap();
    servers.running.lock().unwrap().insert(
        "gone".into(),
        RunningPool {
            child,
            port: 8094,
            fcgi_port: None,
            php_line: None,
            php: None,
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

/// `id` installed at `version`, the default folder of its line.
fn fake_server(kit: &Kit, id: &str, version: &str) {
    let c = Catalog::get().find(id).unwrap();
    let dir = crate::versions::install_dir(kit, c, version);
    let folder = c.folder.as_deref().unwrap();
    for k in std::iter::once(&c.key).chain(&c.keys) {
        if let Some(rel) = k.strip_prefix(folder) {
            let file = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
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

fn system32(exe: &str) -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    format!(r"{root}\System32\{exe}")
}

/// BUG-05r: a pool whose php-cgi ended is taken out of the running ones
/// (and its server ended), never left "running" answering 502.
#[test]
fn pool_with_dead_php_cgi_is_not_running() {
    let job = Job::new().unwrap();
    let server = jobs::spawn_in(
        std::process::Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
        &job,
    )
    .unwrap();
    let mut php = jobs::spawn_in(
        std::process::Command::new(system32("cmd.exe")).args(["/d", "/c", "exit 0"]),
        &job,
    )
    .unwrap();
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
            fcgi_port: Some(9000),
            php_line: Some("8.4".into()),
            php: Some(php),
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
        fcgi_port: fcgi,
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

    let vhosts = kit.data().join(r"servers\legacy\vhosts");
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
    let child = jobs::spawn_in(
        std::process::Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
        &job,
    )
    .unwrap();
    servers.running.lock().unwrap().insert(
        started.id.clone(),
        RunningPool {
            child,
            port: started.port,
            fcgi_port: None,
            php_line: started.php_line.clone(),
            php: None,
            started,
            by_user: true,
            attached: BTreeSet::new(),
            _job: job,
        },
    );
}

/// Defect 5: a running pool the scan no longer has (its automatic id
/// changed with a newer line) stays on the page, marked, until stopped;
/// it serves no new run and is not reloaded.
#[test]
fn orphan_running_pool_is_listed_until_stopped() {
    let tmp = TempDir::new("servers-orphan");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());
    fake_running(
        &servers,
        nginx_pool("nginx-1.25", 8083, vec![site("shop", "public", 8083)]),
    );
    let view = servers.list("it", true);
    assert_eq!(view.pools.len(), 1);
    let orphan = &view.pools[0];
    assert!(orphan.orphan && orphan.running && orphan.pid.is_some());
    assert_eq!(orphan.sites[0].project, "shop");
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);
    let err = servers
        .reload("nginx-1.25", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.contains("fermalo"), "{err}");
    assert!(servers.is_running("nginx-1.25"));
    servers.stop("nginx-1.25");
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
    fs::create_dir_all(kit.projects().join(r"shop\public")).unwrap();
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
    let mut alive = jobs::spawn_in(
        std::process::Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
        &job,
    )
    .unwrap();
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
    let mut ended = jobs::spawn_in(
        std::process::Command::new(system32("cmd.exe")).args(["/d", "/c", "exit 1"]),
        &job,
    )
    .unwrap();
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
    fs::create_dir_all(projects.join(r"shop\public")).unwrap();
    fs::create_dir_all(projects.join(r"blog\public")).unwrap();
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
