use super::*;

#[test]
fn macos_14_or_later_is_needed() {
    assert_eq!(
        gaps_of(Some("13.6.1")),
        vec![Gap::OldMacos("13.6.1".into())]
    );
    assert!(gaps_of(Some("14.0")).is_empty());
    assert!(gaps_of(Some("26.1")).is_empty());
    assert!(gaps_of(None).is_empty());
}
