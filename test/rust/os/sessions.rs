use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::process::Command;
use std::time::Duration;

#[test]
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
