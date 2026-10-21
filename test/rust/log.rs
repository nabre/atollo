use super::*;
use crate::kit::tests::TempDir;

/// The log rotates during the session too, not only at start: past the
/// limit `app.log` becomes `app.1.log` (the older one replaced) and a new
/// `app.log` begins (BUG-V6-17).
#[test]
fn rotates_during_the_session() {
    let tmp = TempDir::new("log-rotate");
    let path = tmp.0.join("app.log");
    let old = tmp.0.join("app.1.log");
    let line = "0 INFO 0123456789\n";
    for _ in 0..3 {
        append(&path, line, 64);
    }
    assert_eq!(fs::read_to_string(&path).unwrap(), line.repeat(3));
    assert!(!old.exists());
    append(&path, line, 64);
    assert_eq!(fs::read_to_string(&old).unwrap(), line.repeat(3));
    assert_eq!(fs::read_to_string(&path).unwrap(), line);
    for _ in 0..3 {
        append(&path, "0 WARN again\n", 40);
    }
    assert_eq!(
        fs::read_to_string(&old).unwrap(),
        format!("{line}0 WARN again\n"),
        "the older file replaced"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "0 WARN again\n".repeat(2)
    );
}
