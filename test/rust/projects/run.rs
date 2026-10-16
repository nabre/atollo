use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn local_network_setting() {
    let cmd = "php -S 127.0.0.1:7101 -t public".to_owned();
    assert_eq!(on_network(cmd.clone(), false), cmd);
    assert_eq!(on_network(cmd, true), "php -S 0.0.0.0:7101 -t public");
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
    let file = |p: &str| tmp.0.join(format!(r"data\projects\{p}\ports.json"));
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

#[test]
fn runs_and_stops_commands() {
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
    let action = Action::for_test("echo", "echo %PORT% hello & exit /b 3", Kind::Command);
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
    let long = Action::for_test("wait", "ping -n 60 127.0.0.1", Kind::Command);
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
