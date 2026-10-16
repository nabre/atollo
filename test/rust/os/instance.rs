use super::*;
use crate::kit::Arch;
use std::path::PathBuf;
use std::sync::mpsc;

fn unique_kit(tag: &str) -> Kit {
    Kit::at(
        PathBuf::from(format!(r"Z:\devkit-test-{}-{tag}", std::process::id())),
        Arch::X64,
    )
}

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
