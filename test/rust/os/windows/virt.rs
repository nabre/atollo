use super::*;

#[test]
fn windows_10_stops_at_the_system() {
    let gaps = gaps_of(Some(19045), Arch::X64, || panic!("not asked"));
    assert_eq!(gaps, vec![Gap::OldWindows(19045)]);
}

#[test]
fn windows_on_arm_has_no_sandboxes() {
    assert_eq!(
        gaps_of(Some(26200), Arch::Arm64, || true),
        vec![Gap::Processor]
    );
}

#[test]
fn windows_11_needs_the_hypervisor_only() {
    assert_eq!(
        gaps_of(Some(26200), Arch::X64, || false),
        vec![Gap::HypervisorOff]
    );
    assert!(gaps_of(Some(26200), Arch::X64, || true).is_empty());
    // An unreadable build is not held against the PC.
    assert!(gaps_of(None, Arch::X64, || true).is_empty());
}

/// The real check answers without an administrator and without failing.
#[test]
fn the_hypervisor_is_asked_without_failing() {
    let _ = hypervisor_runs();
}
