use super::*;
use crate::kit::Arch;
use std::sync::mpsc;
use std::time::Duration;

#[cfg(windows)]
fn unique_kit(tag: &str) -> Kit {
    Kit::at(
        std::path::PathBuf::from(format!(r"Z:\atollo-test-{}-{tag}", std::process::id())),
        Arch::X64,
    )
}

#[cfg(windows)]
#[test]
fn second_instance_of_same_kit_is_secondary() {
    let kit = unique_kit("same");
    let first = acquire(&kit).unwrap();
    assert!(matches!(first, Instance::Primary(_)));
    assert!(matches!(acquire(&kit).unwrap(), Instance::Secondary));
    let other = unique_kit("other");
    assert!(matches!(acquire(&other).unwrap(), Instance::Primary(_)));
    drop(first);
    assert!(matches!(acquire(&kit).unwrap(), Instance::Primary(_)));
}

#[cfg(windows)]
#[test]
fn message_reaches_primary() {
    let kit = unique_kit("pipe");
    let (tx, rx) = mpsc::channel();
    listen(&kit, move |m| {
        let _ = tx.send(m);
    });
    notify_primary(&kit, "focus").unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "focus");
}

/// Two starts of the same kit on Linux and macOS: the second one is
/// secondary and its message reaches the first through the socket.
#[cfg(unix)]
#[test]
fn second_start_reaches_the_first_on_unix() {
    let tmp = crate::kit::tests::TempDir::new("instance-unix");
    let kit = Kit::at(tmp.0.join("Kit"), Arch::X64);
    // A TCP fallback of an earlier start: gone once the lock is taken.
    let stale = kit.app_dir().join("instance.json");
    std::fs::create_dir_all(kit.app_dir()).unwrap();
    std::fs::write(&stale, r#"{"port":1,"token":"old"}"#).unwrap();
    let first = acquire(&kit).unwrap();
    assert!(!stale.exists());
    assert!(matches!(first, Instance::Primary(_)));
    assert!(matches!(acquire(&kit).unwrap(), Instance::Secondary));
    let (tx, rx) = mpsc::channel();
    listen(&kit, move |m| {
        let _ = tx.send(m);
    });
    notify_primary(&kit, "focus").unwrap();
    assert_eq!(rx.recv_timeout(Duration::from_secs(5)).unwrap(), "focus");
    // Nothing of the socket in the kit.
    assert!(!kit.app_dir().join("instance.sock").exists());
}
