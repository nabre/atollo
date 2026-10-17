use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn local_network_setting() {
    let cmd = "php -S 127.0.0.1:7101 -t public".to_owned();
    assert_eq!(on_network(cmd.clone(), false), cmd);
    assert_eq!(on_network(cmd, true), "php -S 0.0.0.0:7101 -t public");
}

/// The DB_PORT of a project's `.env` is read (it wins over the kit's).
#[test]
fn dotenv_values() {
    let tmp = TempDir::new("run-dotenv");
    std::fs::write(
        tmp.0.join(".env"),
        "# DB_PORT=1\nDB_HOST=127.0.0.1\nexport DB_PORT=\"3310\"\n",
    )
    .unwrap();
    assert_eq!(dotenv_value(&tmp.0, "DB_PORT").as_deref(), Some("3310"));
    assert_eq!(dotenv_value(&tmp.0, "DB_NAME"), None);
    std::fs::write(tmp.0.join(".env"), "DB_PORT='3306'\n").unwrap();
    assert_eq!(dotenv_value(&tmp.0, "DB_PORT").as_deref(), Some("3306"));
}

#[test]
fn lines_and_colours() {
    let mut rest = String::new();
    assert_eq!(split_lines(&mut rest, "a\r\nb"), ["a"]);
    assert_eq!(rest, "b");
    assert_eq!(
        split_lines(&mut rest, "c\n\x1b[32mok\x1b[0m\n"),
        ["bc", "ok"]
    );
    assert_eq!(split_lines(&mut rest, "50%\r60%\r"), ["50%", "60%"]);
    assert!(rest.is_empty());
}

#[test]
fn ports_are_kept_and_locked() {
    let tmp = TempDir::new("run-ports");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let r = Runner::new(kit.clone(), Arc::new(|_| {}));
    let a = r.assign_port("p|a", "it").unwrap();
    let b = r.assign_port("p|b", "it").unwrap();
    assert_ne!(a, b);
    assert_eq!(r.assign_port("p|a", "it").unwrap(), a);
    // Kept across restarts.
    let r2 = Runner::new(kit, Arc::new(|_| {}));
    assert_eq!(r2.ports().ports["p|b"], b);
    assert!(r2.set_port("p|c", Some(a), "it").is_err());
    assert!(r2.set_port("p|c", Some(80), "it").is_err());
    assert!(
        r2.set_port("p|c", Some(7050), "it").is_err(),
        "the editor's"
    );
    assert!(r2.set_port("nokey", Some(9000), "it").is_err());
    let p = r2.set_port("p|c", Some(PORT_TO), "it").unwrap();
    assert_eq!(p.locked, ["p|c"]);
    // A locked port in use is an error, not another port.
    let busy = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let busy_port = busy.local_addr().unwrap().port();
    r2.set_port("p|d", Some(busy_port.max(1024)), "it").unwrap();
    if busy_port >= 1024 {
        assert!(r2.assign_port("p|d", "it").is_err());
    }
    r2.set_port("p|c", None, "it").unwrap();
    assert!(!r2.ports().ports.contains_key("p|c"));
}

#[test]
fn ports_per_project_file() {
    let tmp = TempDir::new("run-ports-files");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let file = |p: &str| {
        tmp.0
            .join("data")
            .join("projects")
            .join(p)
            .join("ports.json")
    };
    // no ports.json: nothing loaded, nothing written
    std::fs::create_dir_all(kit.project_data().join("empty")).unwrap();
    let r = Runner::new(kit.clone(), Arc::new(|_| {}));
    assert!(r.ports().ports.is_empty() && !file("empty").exists());
    r.save_ports().unwrap();
    assert!(!file("empty").exists());

    // two projects never get the same port
    let a = r.assign_port("one|web", "it").unwrap();
    let b = r.assign_port("two|web", "it").unwrap();
    assert_ne!(a, b);
    let text = std::fs::read_to_string(file("one")).unwrap();
    assert!(text.contains("\"web\"") && !text.contains("two"), "{text}");
    assert!(!file("empty").exists());

    // a locked port survives save + reload; unknown keys stay
    let locked = r.set_port("one|api", Some(PORT_TO), "it").unwrap();
    assert_eq!(locked.locked, ["one|api"]);
    std::fs::write(
        file("two"),
        format!(r#"{{"ports":{{"web":{b}}},"future":{{"x":1}}}}"#),
    )
    .unwrap();
    let r2 = Runner::new(kit.clone(), Arc::new(|_| {}));
    let p = r2.ports();
    assert_eq!(p.ports["one|api"], PORT_TO);
    assert_eq!(p.ports["two|web"], b);
    assert_eq!(p.locked, ["one|api"]);
    r2.set_port("two|api", Some(PORT_TO - 1), "it").unwrap();
    let two = std::fs::read_to_string(file("two")).unwrap();
    assert!(two.contains("future") && two.contains("api"), "{two}");

    // only the projects that changed are written again
    let untouched = std::fs::read_to_string(file("one")).unwrap();
    std::fs::write(file("one"), "kept as is").unwrap();
    r2.set_port("two|api", None, "it").unwrap();
    assert_eq!(std::fs::read_to_string(file("one")).unwrap(), "kept as is");
    assert!(!std::fs::read_to_string(file("two"))
        .unwrap()
        .contains("api"));
    r2.set_port("one|api", None, "it").unwrap();
    assert_ne!(std::fs::read_to_string(file("one")).unwrap(), untouched);
}

/// Commands run through `cmd.exe` on Windows and `/bin/sh` elsewhere: each
/// test command is given in the form of its shell.
#[test]
fn runs_and_stops_commands() {
    let (echo, wait) = if cfg!(windows) {
        ("echo %PORT% hello & exit /b 3", "ping -n 60 127.0.0.1")
    } else {
        ("echo $PORT hello; exit 3", "sleep 60")
    };
    let tmp = TempDir::new("run-cmd");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let proj = kit.projects().join("demo");
    std::fs::create_dir_all(&proj).unwrap();
    let events = Arc::new(AtomicUsize::new(0));
    let ev = Arc::clone(&events);
    let kit2 = kit.clone();
    let r = Arc::new(Runner::new(
        kit,
        Arc::new(move |_| {
            ev.fetch_add(1, Ordering::Relaxed);
        }),
    ));
    let env = KitEnv::build(
        &kit2,
        &crate::settings::Settings::default(),
        "it",
        &crate::env::Host::capture(),
    );
    let action = Action::for_test("echo", echo, Kind::Command);
    let info = r.start("demo", &proj, &action, &env, "it").unwrap();
    assert_eq!(info.state, State::Running);
    let key = info.key.clone();
    let start = std::time::Instant::now();
    while r.list()[0].state == State::Running {
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(50));
    }
    let done = r.list().into_iter().find(|i| i.key == key).unwrap();
    assert_eq!(done.state, State::Error);
    assert_eq!(done.code, Some(3));
    assert!(
        r.log(&key).iter().any(|l| l.contains("hello")),
        "{:?}",
        r.log(&key)
    );
    assert!(events.load(Ordering::Relaxed) >= 3);

    // A server that never ends: stopped through its job.
    let long = Action::for_test("wait", wait, Kind::Command);
    let info = r.start("demo", &proj, &long, &env, "it").unwrap();
    assert_eq!(r.running(), 1);
    r.stop(&info.key);
    let start = std::time::Instant::now();
    while r.running() > 0 {
        assert!(start.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(50));
    }
    assert_eq!(
        r.list().iter().find(|i| i.key == info.key).unwrap().state,
        State::Stopped
    );
}

fn site_action(apache: bool, server: Option<&str>) -> Action {
    let mut a = Action::for_test("site", "", Kind::Web);
    if apache {
        a.apache_dir = Some("public".into());
    } else {
        a.nginx_dir = Some("public".into());
    }
    a.server = server.map(str::to_owned);
    a
}

/// GAP-RUN: a site assigned to a server profile goes to that pool.
#[test]
fn site_mode_pool_when_assigned() {
    assert_eq!(
        site_mode(&site_action(true, Some("legacy"))),
        SiteMode::Pool("legacy".into())
    );
    assert_eq!(
        site_mode(&site_action(false, Some(" shop "))),
        SiteMode::Pool("shop".into())
    );
}

/// PLAN.md §13.1: no profile (or `auto`, or empty) goes to the automatic
/// pool of the project's lines; `dedicated` keeps the site's own server,
/// and a `server` on another kind of action means nothing.
#[test]
fn site_mode_auto_or_dedicated_otherwise() {
    for server in [None, Some("auto"), Some(""), Some("  ")] {
        assert_eq!(site_mode(&site_action(true, server)), SiteMode::Auto);
        assert_eq!(site_mode(&site_action(false, server)), SiteMode::Auto);
    }
    for server in [DEDICATED_SERVER, " dedicated "] {
        assert_eq!(
            site_mode(&site_action(true, Some(server))),
            SiteMode::Dedicated
        );
    }
    let mut cmd = Action::for_test("dev", "npm run dev", Kind::Web);
    cmd.server = Some("legacy".into());
    assert_eq!(site_mode(&cmd), SiteMode::Dedicated);
}

/// GAP-RUN: a run served by a pool shows the site's URL; stopping it only
/// marks it stopped (the shared pool is not this action's to end).
#[test]
fn pool_run_stops_without_a_process() {
    let tmp = TempDir::new("run-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let r = Runner::new(kit, Arc::new(|_| {}));
    let a = site_action(false, Some("legacy"));
    let info = r.attach_pool(
        "shop",
        &a,
        "legacy",
        8095,
        "http://shop.localhost:8095",
        "it",
    );
    assert_eq!(info.state, State::Running);
    assert_eq!(info.url.as_deref(), Some("http://shop.localhost:8095/"));
    assert_eq!(info.port, Some(8095));
    assert_eq!(r.running(), 0, "counted with the pools");
    r.clear(&info.key);
    assert_eq!(r.list().len(), 1, "not cleared while served");
    // What run_stop needs to take the site out of its pool.
    assert_eq!(
        r.pooled(&info.key),
        Some(("legacy".into(), "shop".into(), a.id.clone()))
    );
    r.stop(&info.key);
    assert_eq!(r.list()[0].state, State::Stopped);
    assert_eq!(r.pooled(&info.key), None);
    r.clear(&info.key);
    assert!(r.list().is_empty());
}

/// I2: a run served by a pool stopped from the "Server Web" page (or dead)
/// is marked stopped, and can then be cleared; one whose pool runs is not.
#[test]
fn pool_runs_follow_their_pool() {
    let tmp = TempDir::new("run-pool-reconcile");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&events);
    let r = Runner::new(
        kit,
        Arc::new(move |e| {
            if let Event::State(i) = e {
                seen.lock().unwrap().push((i.key, i.state));
            }
        }),
    );
    let a = site_action(false, Some("legacy"));
    let shop = r.attach_pool(
        "shop",
        &a,
        "legacy",
        8095,
        "http://shop.localhost:8095",
        "it",
    );
    let blog = r.attach_pool(
        "blog",
        &a,
        "other",
        8096,
        "http://blog.localhost:8096",
        "it",
    );
    events.lock().unwrap().clear();

    r.reconcile_pools(|_| true);
    assert!(r.list().iter().all(|i| i.state == State::Running));
    assert!(events.lock().unwrap().is_empty());

    r.reconcile_pools(|id| id != "legacy");
    let state = |key: &str| r.list().into_iter().find(|i| i.key == key).unwrap().state;
    assert_eq!(state(&shop.key), State::Stopped);
    assert_eq!(state(&blog.key), State::Running);
    assert_eq!(
        *events.lock().unwrap(),
        vec![(shop.key.clone(), State::Stopped)]
    );
    r.clear(&shop.key);
    assert_eq!(r.list().len(), 1);
}

/// BUG-V3-01: when the php-cgi of a project's Nginx ends, the server is
/// stopped and the run ends in error with the reason in its log; stopped by
/// the user, nothing is added.
#[cfg(windows)]
#[test]
fn nginx_stops_when_its_php_cgi_ends() {
    let tmp = TempDir::new("run-php-cgi");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let r = Runner::new(kit, Arc::new(|_| {}));
    let cmd = |args: &str| {
        use std::os::windows::process::CommandExt;
        let mut c = std::process::Command::new(jobs::system_root().join(r"System32\cmd.exe"));
        c.raw_arg(format!("/d /c \"{args}\""))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        c
    };
    let run = |project: &str, stopped: bool| {
        let info = r.attach_pool(
            project,
            &site_action(false, None),
            "-",
            8097,
            "http://x",
            "it",
        );
        let job = Job::new().unwrap();
        let mut server = jobs::spawn_in(&mut cmd("ping -n 60 127.0.0.1"), &job).unwrap();
        let php = jobs::spawn_in(&mut cmd("exit 7"), &job).unwrap();
        {
            let mut entries = r.entries();
            let e = entries.get_mut(&info.key).unwrap();
            e.pool = None;
            e.stopped = stopped;
            e.job = (!stopped).then_some(job);
        }
        let start = std::time::Instant::now();
        let code = r.wait_with_php(&info.key, &mut server, php, "it");
        assert!(start.elapsed() < Duration::from_secs(20));
        (code, r.log(&info.key))
    };

    let (code, log) = run("shop", false);
    assert_eq!(code, None, "an error, whatever the server's code");
    assert!(
        log.iter()
            .any(|l| l.contains("php-cgi terminato (codice 7)")),
        "{log:?}"
    );

    // Stopped by the user: the job is gone, the server is only waited for.
    let (_, log) = run("blog", true);
    assert!(!log.iter().any(|l| l.contains("php-cgi")), "{log:?}");
}
