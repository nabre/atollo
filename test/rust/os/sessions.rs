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
