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
fn trailing_separator_does_not_matter() {
    let p = Proc {
        pid: 1,
        parent: 0,
        name: "x".into(),
        path: Path::new("kit").join("apps").join("x"),
    };
    let dir = format!(
        "{}{MAIN_SEPARATOR}",
        Path::new("kit").join("apps").display()
    );
    assert_eq!(running_from(std::slice::from_ref(&p), Path::new(&dir)), [1]);
    assert_eq!(
        running_from(std::slice::from_ref(&p), &Path::new("kit").join("apps")),
        [1]
    );
}

/// Case as the usual file system of the PC (D-123): `KIT` is `kit` on
/// Windows and macOS, another folder on Linux.
#[test]
fn case_follows_the_file_system() {
    let p = Proc {
        pid: 1,
        parent: 0,
        name: "x".into(),
        path: Path::new("kit").join("apps").join("x"),
    };
    let upper = running_from(&[p], &Path::new("KIT").join("apps"));
    let expected: &[u32] = if cfg!(target_os = "linux") { &[] } else { &[1] };
    assert_eq!(upper, expected);
}
