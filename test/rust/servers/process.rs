use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

use anyhow::Result;

use super::*;
use crate::env::KitEnv;
use crate::kit::{tests::TempDir, Arch};
use crate::platform::jobs::{self, Job};
use crate::projects::fastcgi::{FastCgi, FcgiAddr};
use crate::servers::tests::*;
use crate::servers::*;
use crate::setup::Catalog;
use crate::test_support::{fake_server, prepared_kit};

/// M2: a port released within a moment (a pool just ended) is waited for,
/// not reported as used by another program; one kept taken still is.
#[test]
fn start_waits_for_a_port_being_released() {
    let (_tmp, kit) = prepared_kit("servers-start-port");
    let servers = WebServers::new(kit.clone());
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    servers
        .save_profile(
            WebServerProfileInput {
                id: "busy".into(),
                name: "busy".into(),
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
    let busy = "La porta";

    // Kept taken: the port error, after the grace.
    let err = servers
        .start("busy", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with(busy), "{err}");

    // Released while waiting: on to the start (no Caddy in this kit).
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
        assert!(err.contains("Caddy is not installed"), "{err}");
    });
}

/// BUG-03: what httpd/caddy print on stderr (a syntax error, before their
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

/// BUG-05r: with Caddy and PHP installed but no project, the default pool
/// starts no php-cgi; nor does a manual profile without a chosen PHP.
#[test]
fn default_caddy_pool_has_no_php_without_php_projects() {
    let (_tmp, kit) = prepared_kit("servers-default-php");
    fake_server(&kit, "caddy", "2.11.7");
    fake_server(&kit, "php", "8.4.13");
    let servers = WebServers::new(kit.clone());

    let view = servers.list("it", true);
    assert_eq!(view.php_lines, ["8.4"]);
    assert_eq!(view.pools.len(), 1);
    let pool = &view.pools[0];
    assert_eq!(pool.engine, WebServerEngine::Caddy);
    assert_eq!(pool.php_line, None);
    assert_eq!(pool.id, "caddy-2.11");

    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Static".into(),
                engine: WebServerEngine::Caddy,
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
    assert_eq!(manual.server_line.as_deref(), Some("2.11"));
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
        "caddy-php".to_owned(),
        RunningPool {
            child: server,
            port: 8099,
            fcgi: Some(FastCgi::tcp(9000, "k")),
            php_line: Some("8.4".into()),
            php: Some(php),
            admin: None,
            started: caddy_pool("caddy-php", 8099, Vec::new()),
            by_user: true,
            attached: BTreeSet::new(),
            _job: job,
        },
    );
    // I2: `is_running` (what the runs follow) says so too.
    let servers = WebServers::new(Kit::at(std::env::temp_dir(), Arch::X64));
    std::mem::swap(&mut *servers.running.lock().unwrap(), &mut running);
    assert!(!servers.is_running("caddy-php"));
    assert!(!servers.is_running("nope"));
    std::mem::swap(&mut *servers.running.lock().unwrap(), &mut running);
    let dead = take_dead(&mut running);
    assert!(running.is_empty());
    assert_eq!(dead.len(), 1);
    assert_eq!(dead[0].0, "caddy-php");
    end_dead(dead);
}

/// I1: a reload restarts the Caddy pool (never a graceful `caddy reload`
/// keeping no php-cgi) when PHP is wanted and not running, or changed;
/// Apache and a new port always restart.
#[test]
fn reload_restarts_when_php_or_port_changes() {
    let live = |php: Option<&str>, fcgi: Option<u16>| LivePool {
        port: 8093,
        php_line: php.map(str::to_owned),
        fcgi: fcgi.map(|port| FastCgi::tcp(port, "k")),
    };
    let mut pool = caddy_pool("legacy", 8093, Vec::new());
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
    let mut apache = caddy_pool("a", 8093, Vec::new());
    apache.engine = WebServerEngine::Apache;
    assert!(needs_restart(&apache, &live(None, None)));
}

/// Defect 4: a pool whose server or PHP line is not installed does not
/// start (never the default line in its place), with an error naming it.
#[test]
fn start_refuses_a_line_not_installed() {
    let (_tmp, kit) = prepared_kit("servers-missing-line");
    fake_server(&kit, "caddy", "2.11.7");
    fake_server(&kit, "php", "8.4.13");
    let servers = WebServers::new(kit.clone());
    let port = {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        l.local_addr().unwrap().port()
    };
    let profile = |server_line: &str, php_line: &str| WebServerProfileInput {
        id: "old".into(),
        name: "Old".into(),
        engine: WebServerEngine::Caddy,
        server_line: Some(server_line.into()),
        php_line: Some(php_line.into()),
        port: Some(port),
        keep_alive: true,
        autostart: false,
    };
    servers.save_profile(profile("2.10", "8.4"), "it").unwrap();
    let err = servers
        .start("old", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("Caddy 2.10 non è installato"), "{err}");
    servers.save_profile(profile("2.11", "7.4"), "it").unwrap();
    let err = servers
        .start("old", &KitEnv::default(), "it")
        .unwrap_err()
        .to_string();
    assert!(err.starts_with("PHP 7.4 non è installato"), "{err}");
    assert!(!servers.is_running("old"));
}

/// Defect 5: a running pool the scan does not have (an id that is neither a
/// profile's nor an automatic one) stays on the page, marked, until
/// stopped; it serves no new run and is not reloaded.
#[test]
fn orphan_running_pool_is_listed_until_stopped() {
    let (_tmp, kit) = prepared_kit("servers-orphan");
    let servers = WebServers::new(kit.clone());
    fake_running(
        &servers,
        caddy_pool("gone", 8083, vec![site("shop", "public", 8083)]),
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

/// Defect 9: php-cgi is waited for on its FastCGI port; one that ends at
/// once (its port taken in the meantime) is reported, to try another port.
#[test]
fn php_cgi_listening_tells_ended_from_ready() {
    let job = Job::new().unwrap();
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let port = FcgiAddr::Tcp(port);
    let mut alive = jobs::spawn_in(&mut long_running(), &job).unwrap();
    let begun = Instant::now();
    assert!(php_cgi_listening(&mut alive, &port, Duration::from_secs(5)));
    assert!(begun.elapsed() < Duration::from_secs(2), "answers at once");
    drop(listener);
    // Alive but not listening yet: slow, kept.
    assert!(php_cgi_listening(
        &mut alive,
        &port,
        Duration::from_millis(300)
    ));
    let _ = alive.kill();
    let _ = alive.wait();
    let mut ended = jobs::spawn_in(&mut exiting(1), &job).unwrap();
    assert!(!php_cgi_listening(
        &mut ended,
        &port,
        Duration::from_secs(5)
    ));
}

/// php-fpm on macOS and Linux is waited for on its Unix socket (never a
/// TCP port, review of D-111): a socket that takes connections answers,
/// a file that is not one, or none, does not.
#[cfg(unix)]
#[test]
fn fcgi_answers_on_a_unix_socket() {
    use crate::servers::process::fcgi_answers;
    // A short folder: a test folder's path may be too long for `sun_path`.
    let base = std::env::temp_dir();
    let sock = crate::projects::caddy::PrivateSocket::in_leaf(
        &base,
        &format!("fcgi-test-{}", std::process::id()),
        "fpm.sock",
    )
    .unwrap();
    let path = sock.path.clone();
    let addr = FcgiAddr::Unix(std::sync::Arc::new(sock));
    assert!(!fcgi_answers(&addr), "no socket yet");
    let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
    assert!(fcgi_answers(&addr));
    drop(listener);
    fs::remove_file(&path).unwrap();
    fs::write(&path, b"").unwrap();
    assert!(!fcgi_answers(&addr), "a plain file is not a socket");
    let dir = path.parent().unwrap().to_path_buf();
    drop(addr);
    assert!(!dir.exists(), "the folder goes with the last copy");
}

/// D-101: a pool stops with its last project only when projects
/// started it and it is not kept alive.
#[test]
fn stops_when_left_follows_who_started_it() {
    assert!(stops_when_left(false, 0, false));
    assert!(!stops_when_left(false, 1, false), "another project left");
    assert!(!stops_when_left(false, 0, true), "kept alive");
    assert!(!stops_when_left(true, 0, false), "started by the user");
}

/// BUG-03 with a real httpd (ATOLLO_TEST_KIT): a syntax error in the pool's
/// configuration reaches the error of the start, not only a log.
#[test]
fn start_reports_syntax_error() {
    let Some((kit, env, port)) = real_apache() else {
        return;
    };
    let servers = WebServers::new(kit.clone());
    let id = "atollo-test-syntax";
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

/// BUG-04 with a real httpd (ATOLLO_TEST_KIT): two reloads in a row (each a
/// stop and start, Apache has no graceful reload on Windows) find the port
/// free again and leave the pool answering.
#[test]
fn apache_pool_reload_twice() {
    let Some((kit, env, port)) = real_apache() else {
        return;
    };
    let servers = WebServers::new(kit.clone());
    let id = "atollo-test-reload";
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

/// D-116 acceptance with a real Apache and PHP (ATOLLO_TEST_KIT; php-fpm
/// off Windows, the module on Windows): a pool with two PHP sites and an
/// alias to another project's folder, hidden files and other hosts
/// refused, Drupal's `sites/default/files/.htaccess` keeping PHP off in its
/// folder, and a third site after a reload (a restart for Apache).
#[test]
fn apache_pool_two_php_sites_alias_and_reload() {
    let Some((kit, env, port)) = real_apache() else {
        return;
    };
    let kit = kit.with_host(Some(&std::env::temp_dir()));
    if !Catalog::get()
        .find("php")
        .is_some_and(|c| c.is_installed(&kit))
    {
        eprintln!("php not installed: skipped");
        return;
    }
    crate::setup::kit_files::write_all(&kit).unwrap();
    let id = "d116-apache-pool";
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
                r#"{{ "actions": [ {{ "id": "site", "apache": "public", "server": "{id}"{extra} }} ] }}"#
            ),
        )
        .unwrap();
    };
    site(
        "d116-alpha",
        r#", "aliases": { "/shared": "@d116-beta/assets" }"#,
    );
    site("d116-beta", "");
    fs::create_dir_all(projects.join("d116-beta").join("assets")).unwrap();
    fs::write(
        projects.join("d116-beta").join("assets").join("logo.txt"),
        "beta asset",
    )
    .unwrap();
    // SEC-V4-01: an uploaded script and `.htaccess` in an alias' folder.
    fs::write(
        projects
            .join("d116-beta")
            .join("assets")
            .join("shell.phtml"),
        "<?php echo 'RAN' . 'ALIAS';",
    )
    .unwrap();
    fs::write(
        projects.join("d116-beta").join("assets").join("pic.jpg"),
        "<?php echo 'RAN' . 'JPG';",
    )
    .unwrap();
    fs::write(
        projects.join("d116-beta").join("assets").join(".htaccess"),
        "AddHandler application/x-httpd-php .jpg\nAddType application/x-httpd-php .jpg\n<FilesMatch \"\\.phtml$\">\n  Require all granted\n  Allow from all\n</FilesMatch>\n",
    )
    .unwrap();
    // Drupal's protection of its uploads folder (its own `.htaccess`).
    let files = projects.join("d116-beta").join("public").join("files");
    fs::create_dir_all(&files).unwrap();
    fs::write(
        files.join(".htaccess"),
        "SetHandler Drupal_Security_Do_Not_Remove_See_SA_2006_006\nOptions None\nOptions +FollowSymLinks\n<Files *>\n  SetHandler Drupal_Security_Do_Not_Remove_See_SA_2006_006\n</Files>\n<IfModule mod_php.c>\n  php_flag engine off\n</IfModule>\n",
    )
    .unwrap();
    fs::write(files.join("up.php"), "<?php echo 'RAN' . 'PHP';").unwrap();
    let servers = WebServers::new(kit.clone());
    real_profile(&servers, id, port);
    let result = (|| -> Result<()> {
        servers.start(id, &env, "en")?;
        let (s, body) = http_get(port, "d116-alpha.localhost", "/x.php")?;
        eprintln!("missing script: {s}");
        anyhow::ensure!(s == 404, "a missing .php: {s} {body}");
        let (s, body) = http_get(port, "d116-alpha.localhost", "/index.php?x=1")?;
        eprintln!("alpha: {s} {body}");
        anyhow::ensure!(
            s == 200 && body.contains("d116-alpha /index.php?x=1"),
            "alpha"
        );
        let (s, body) = http_get(port, "d116-beta.localhost", "/")?;
        eprintln!("beta: {s} {body}");
        anyhow::ensure!(s == 200 && body.contains("PHP 8.") && body.contains("d116-beta"));
        let (s, body) = http_get(port, "d116-alpha.localhost", "/shared/logo.txt")?;
        eprintln!("alias: {s} {body}");
        anyhow::ensure!(s == 200 && body.contains("beta asset"), "alias");
        let (s, body) = http_get(port, "d116-alpha.localhost", "/shared/shell.phtml")?;
        eprintln!("alias script: {s}");
        anyhow::ensure!(
            s == 403 && !body.contains("RANALIAS"),
            "a script in an alias: {s} {body}"
        );
        let (s, body) = http_get(port, "d116-alpha.localhost", "/shared/pic.jpg")?;
        eprintln!("alias .htaccess handler: {s} {body}");
        anyhow::ensure!(
            !body.contains("RANJPG"),
            "a .htaccess handler in an alias ran PHP"
        );
        let (s, _) = http_get(port, "d116-alpha.localhost", "/.env")?;
        anyhow::ensure!(s == 403, ".env: {s}");
        let (s, _) = http_get(port, "evil.example", "/")?;
        anyhow::ensure!(s == 403, "other host: {s}");
        let (s, body) = http_get(port, "d116-beta.localhost", "/files/up.php")?;
        eprintln!("drupal files: {s} {body}");
        anyhow::ensure!(!body.contains("RANPHP"), "PHP ran in Drupal's files folder");
        site("d116-gamma", "");
        servers.invalidate();
        servers.reload(id, &env, "en")?;
        let (s, body) = http_get(port, "d116-gamma.localhost", "/")?;
        eprintln!("gamma: {s} {body}");
        anyhow::ensure!(
            s == 200 && body.contains("d116-gamma"),
            "gamma after the reload"
        );
        let (s, body) = http_get(port, "d116-alpha.localhost", "/")?;
        anyhow::ensure!(
            s == 200 && body.contains("d116-alpha"),
            "alpha after the reload: {s}"
        );
        Ok(())
    })();
    servers.stop(id);
    drop_real_profile(&kit, &servers, id);
    for p in ["d116-alpha", "d116-beta", "d116-gamma"] {
        let _ = fs::remove_dir_all(projects.join(p));
    }
    result.unwrap();
}

/// D-111 acceptance with a real Caddy and PHP (ATOLLO_TEST_KIT): a pool
/// with two PHP sites and an alias to another project's folder; a third
/// site added with `caddy reload` while the first one keeps answering, the
/// same Caddy process throughout.
#[test]
fn caddy_pool_two_php_sites_alias_and_reload() {
    let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
        eprintln!("ATOLLO_TEST_KIT not set: skipped");
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
        &crate::common::settings::Settings::default(),
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
        #[cfg(unix)]
        probe_fpm_socket(&servers, id, &projects.join("p6-alpha/public/index.php"))?;
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
