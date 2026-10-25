use super::*;

#[test]
fn parent_from_proc_stat() {
    assert_eq!(stat_parent("42 (bash) S 7 42 42 0"), Some(7));
    // A name with spaces and parentheses.
    assert_eq!(stat_parent("9 (a) b (c)) R 1 9 9"), Some(1));
    assert_eq!(stat_parent("garbage"), None);
}
