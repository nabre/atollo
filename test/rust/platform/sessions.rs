use super::*;
use crate::test_support::prepared_kit;
#[cfg(unix)]
use crate::test_support::TempDir;
use std::process::Command;
use std::time::Duration;

/// A kit reached through a link (on macOS every temporary folder:
/// `/var/folders/…` is `/private/var/folders/…`): the system names its
/// programs by the real path, and they are the kit's all the same.
#[cfg(unix)]
#[test]
fn a_kit_through_a_link_lists_its_programs() {
    let tmp = TempDir::new("sessions-link");
    let real = tmp.0.join("real");
    std::fs::create_dir_all(&real).unwrap();
    let link = tmp.0.join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let kit = Kit::at(link, crate::test_support::kit_arch());
    kit.prepare().unwrap();
    let exe = kit.apps().join("tools").join("sleep-link");
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::copy("/bin/sleep", &exe).unwrap();
    let mut child = Command::new(&exe).arg("30").spawn().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let open = kit_processes(&kit);
    let _ = child.kill();
    let _ = child.wait();
    let found = open.iter().find(|p| p.pid == child.id());
    assert!(found.is_some(), "{open:?}");
    // Relative to the kit, not the real path.
    assert!(!found.unwrap().program.starts_with('/'), "{open:?}");
    assert!(other_kits(&kit).iter().all(|r| !r.ends_with("real")));
}

#[test]
fn lists_and_stops_a_kit_program() {
    let (_tmp, kit) = prepared_kit("sessions");
    // A "program of the kit": a copy of a program of the system in
    // data/apps, that waits 30 s (ping on Windows, sleep elsewhere).
    let (source, name, args): (PathBuf, &str, &[&str]) = if cfg!(windows) {
        let system = std::env::var("SystemRoot").unwrap();
        (
            Path::new(&system).join("System32").join("PING.EXE"),
            "ping-test.exe",
            &["-n", "30", "127.0.0.1"],
        )
    } else {
        (PathBuf::from("/bin/sleep"), "sleep-test", &["30"])
    };
    let exe = kit.apps().join("tools").join(name);
    std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
    std::fs::copy(source, &exe).unwrap();
    let mut child = Command::new(&exe).args(args).spawn().unwrap();
    std::thread::sleep(Duration::from_millis(300));
    let open = kit_processes(&kit);
    assert!(
        open.iter()
            .any(|p| p.pid == child.id() && p.program.ends_with(name)),
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
        (r"E:\Kit\DATA\Apps\Win32-X64\git\git.exe", r"E:\Kit"),
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
    // A release name, as downloaded and kept by the updates.
    #[cfg(windows)]
    assert_eq!(
        kit_root_of(
            Path::new(r"E:\kit\Atollo-0.2.0-win32-x64.exe"),
            "Atollo-0.2.0-win32-x64.exe",
            yes
        ),
        Some(PathBuf::from(r"E:\kit"))
    );
    assert_eq!(
        kit_root_of(Path::new("/usr/bin/atollo"), "atollo", yes),
        None
    );
}

#[test]
fn tree_of_the_stopped_processes() {
    let p = |pid, parent| crate::platform::processes::Proc {
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
