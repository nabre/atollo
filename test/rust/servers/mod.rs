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
    };

    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let main_conf = fs::read_to_string(&conf_path).unwrap();
    assert!(main_conf.contains("listen 127.0.0.1:8088 default_server;"));
    assert!(main_conf.contains("server_name localhost;"));

    let vhosts_dir = conf_path.parent().unwrap().join("vhosts");
    let alpha_vhost = fs::read_to_string(vhosts_dir.join("alpha.conf")).unwrap();
    let beta_vhost = fs::read_to_string(vhosts_dir.join("beta.conf")).unwrap();

    assert!(alpha_vhost.contains("listen 127.0.0.1:8088;"));
    assert!(!alpha_vhost.contains("default_server"));
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
    let good = fs::read_to_string(vhosts.join("good.conf")).unwrap();
    assert!(good.contains("good/public"), "{good}");
    assert!(!vhosts.join("missing.conf").exists());
    assert!(!vhosts.join("linked.conf").exists());
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
        port: Some(8097),
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
            php: Some(php),
            _job: job,
        },
    );
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
}
