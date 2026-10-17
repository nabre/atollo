use super::*;

#[test]
fn finds_itself() {
    let me = std::env::current_exe().unwrap();
    let dir = me.parent().unwrap();
    assert!(running_under(dir)
        .iter()
        .any(|p| p.pid == std::process::id()));
    assert!(running_under(Path::new(r"Z:\no\such\folder")).is_empty());
}

#[test]
fn parent_from_proc_stat() {
    assert_eq!(stat_parent("42 (bash) S 7 42 42 0"), Some(7));
    // A name with spaces and parentheses.
    assert_eq!(stat_parent("9 (a) b (c)) R 1 9 9"), Some(1));
    assert_eq!(stat_parent("garbage"), None);
}

#[test]
fn trailing_separator_does_not_matter() {
    let p = Proc {
        pid: 1,
        parent: 0,
        name: "x".into(),
        path: Path::new("kit").join("apps").join("x"),
    };
    let dir = format!(
        "{}{MAIN_SEPARATOR}",
        Path::new("KIT").join("apps").display()
    );
    assert_eq!(running_from(std::slice::from_ref(&p), Path::new(&dir)), [1]);
    assert_eq!(running_from(&[p], &Path::new("KIT").join("apps")), [1]);
}
