use super::*;

#[test]
fn stamp_format() {
    assert_eq!(civil_from_days(0), (1970, 1, 1));
    assert_eq!(civil_from_days(20_740), (2026, 10, 14));
    assert_eq!(stamp().len(), 15);
}
