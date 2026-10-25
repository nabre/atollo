use super::*;

#[test]
fn an_intel_mac_has_no_sandboxes() {
    assert_eq!(gaps_of(false, Some("15.1")), vec![Gap::Processor]);
}

#[test]
fn macos_14_or_later_is_needed() {
    assert_eq!(
        gaps_of(true, Some("13.6.1")),
        vec![Gap::OldMacos("13.6.1".into())]
    );
    assert!(gaps_of(true, Some("14.0")).is_empty());
    assert!(gaps_of(true, Some("26.1")).is_empty());
    assert!(gaps_of(true, None).is_empty());
}
