use super::*;
use crate::test_support::prepared_kit;

/// The login an earlier Atollo saved is read as kit data: one object or a
/// list, nothing from a missing or broken file.
#[test]
fn the_saved_login_is_read() {
    let (_tmp, kit) = prepared_kit("keyring-read");
    let dir = kit.data().join("antigravity");
    assert!(!has_login(&kit));
    assert!(saved_secrets(&dir).is_empty());
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("keyring.json"), "{ broken").unwrap();
    assert!(!has_login(&kit));
    std::fs::write(
        dir.join("keyring.json"),
        r#"{"Target":"gemini:antigravity","Blob":"c2VjcmV0"}"#,
    )
    .unwrap();
    assert!(has_login(&kit));
    assert_eq!(saved_secrets(&dir), ["c2VjcmV0"]);
    std::fs::write(
        dir.join("keyring.json"),
        r#"[{"Target":"a","Blob":"eA=="},{"Target":"b"}]"#,
    )
    .unwrap();
    assert_eq!(saved_secrets(&dir), ["eA=="]);
}
