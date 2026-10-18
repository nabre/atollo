use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::process::Command;
use std::time::Duration;

#[test]
#[cfg_attr(not(windows), ignore = "copies and runs PING.EXE from System32")]
fn lists_and_stops_a_kit_program() {
    let tmp = TempDir::new("sessions");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    // A "program of the kit": a copy of ping in data\apps.
    let exe = kit.apps().join(r"tools\ping-test.exe");
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    let system = std::env::var("SystemRoot").unwrap();
    std::fs::copy(Path::new(&system).join(r"System32\PING.EXE"), &exe).unwrap();
    let mut child = Command::new(&exe)
        .args(["-n", "30", "127.0.0.1"])
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let open = kit_processes(&kit);
    assert!(
        open.iter()
            .any(|p| p.pid == child.id() && p.program.ends_with("ping-test.exe")),
        "{open:?}"
    );
    // Started by this process: not "foreign".
    assert!(foreign_processes(&kit).iter().all(|p| p.pid != child.id()));
    stop(&[child.id()]);
    let start = std::time::Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(start.elapsed() < Duration::from_secs(5));
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(other_kits(&kit).iter().all(|r| r != kit.root()));
}

#[test]
fn other_kit_roots_on_every_system() {
    let no = |_: &Path| false;
    let yes = |_: &Path| true;
    for (path, root) in [
        (r"E:\kit\data\apps\win32-x64\node\node.exe", r"E:\kit"),
        (
            "/Volumes/KIT/data/apps/darwin-arm64/node/bin/node",
            "/Volumes/KIT",
        ),
        (
            "/media/u/kit/data/apps/linux-x64/php/bin/php",
            "/media/u/kit",
        ),
        (r"E:\Kit\DATA\Apps\Win32-arm64\git\git.exe", r"E:\Kit"),
    ] {
        assert_eq!(
            kit_root_of(Path::new(path), "x", no),
            Some(PathBuf::from(root)),
            "{path}"
        );
    }
    assert_eq!(
        kit_root_of(Path::new("/opt/apps/linux-x64/x"), "x", yes),
        None
    );
    // The app itself, with and without .exe, only next to `data/app`.
    let exe = Path::new("/media/u/kit/Atollo");
    assert_eq!(
        kit_root_of(exe, "Atollo", yes),
        Some(PathBuf::from("/media/u/kit"))
    );
    assert_eq!(kit_root_of(exe, "Atollo", no), None);
    #[cfg(windows)]
    assert_eq!(
        kit_root_of(Path::new(r"E:\kit\Atollo.exe"), "ATOLLO.EXE", yes),
        Some(PathBuf::from(r"E:\kit"))
    );
    assert_eq!(
        kit_root_of(Path::new("/usr/bin/atollo"), "atollo", yes),
        None
    );
}

#[test]
fn tree_of_the_stopped_processes() {
    let p = |pid, parent| crate::os::processes::Proc {
        pid,
        parent,
        name: String::new(),
        path: PathBuf::new(),
    };
    // 10 -> 11 -> 12, 10 -> 13; 1 (the app) -> 10; 20 alone.
    let all = [p(10, 1), p(11, 10), p(12, 11), p(13, 10), p(20, 1), p(1, 0)];
    let mut tree = tree_of(&[10, 10], &all, 1);
    assert_eq!(tree.first(), Some(&10), "a parent before its children");
    tree.sort_unstable();
    assert_eq!(tree, [10, 11, 12, 13]);
    // The app itself is never in it.
    assert_eq!(tree_of(&[1], &all, 1), Vec::<u32>::new());
}

/// D-105: `SIGTERM` to all, `SIGKILL` only to those still running after
/// the wait; nothing waits when all have ended.
#[test]
fn gentle_stop_waits_then_kills() {
    use std::cell::RefCell;
    let sent = RefCell::new(Vec::new());
    let stubborn = 7;
    stop_gently(
        &[5, stubborn],
        Duration::from_millis(200),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        // 5 ends on SIGTERM: running when first looked at, then gone.
        |pid| match pid {
            7 => Some(70),
            _ if sent.borrow().is_empty() => Some(50),
            _ => None,
        },
    );
    assert_eq!(
        sent.into_inner(),
        [
            (5, Signal::Term),
            (stubborn, Signal::Term),
            (stubborn, Signal::Kill)
        ]
    );
    let sent = RefCell::new(Vec::new());
    let start = std::time::Instant::now();
    stop_gently(
        &[5],
        Duration::from_secs(5),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        |_| sent.borrow().is_empty().then_some(50),
    );
    assert!(start.elapsed() < Duration::from_secs(1));
    assert_eq!(sent.into_inner(), [(5, Signal::Term)]);
}

/// A pid that ends during the wait and is taken by another program (a
/// new start time) gets no `SIGKILL`: that program is not the kit's.
#[test]
fn gentle_stop_spares_a_reused_pid() {
    use std::cell::{Cell, RefCell};
    let sent = RefCell::new(Vec::new());
    let calls = Cell::new(0);
    stop_gently(
        &[9],
        Duration::from_millis(100),
        Duration::from_millis(10),
        |pid, s| sent.borrow_mut().push((pid, s)),
        |_| {
            calls.set(calls.get() + 1);
            // The kit's program, then from the second look another one.
            Some(if calls.get() == 1 { 1000 } else { 2000 })
        },
    );
    assert_eq!(sent.into_inner(), [(9, Signal::Term)]);
}

/// A real program: `SIGTERM` ends it, no `SIGKILL` needed.
#[cfg(unix)]
#[test]
fn gentle_stop_of_a_real_program() {
    use std::os::unix::process::ExitStatusExt;
    let mut child = Command::new("/bin/sleep").arg("30").spawn().unwrap();
    let start = std::time::Instant::now();
    stop(&[child.id()]);
    assert!(start.elapsed() < Duration::from_secs(4));
    assert_eq!(child.wait().unwrap().signal(), Some(libc::SIGTERM));
}
