use super::*;
use crate::test_support::prepared_kit;

/// A made-up token of the length `claude setup-token` prints.
fn sample() -> String {
    let body: String = (0..93)
        .map(|i| {
            let set = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
            set[(i * 7 + 3) % set.len()] as char
        })
        .collect();
    format!("sk-ant-oat01-{body}AA")
}

#[test]
fn tokens() {
    assert!(is_token(&sample()));
    assert!(!is_token("sk-ant-oat01-short"));
    assert!(!is_token(
        "sk-ant-api03-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    ));
    assert!(!is_token(&format!("{} ", sample())));
    assert!(!is_token(&format!("{}\u{1b}", sample())));
}

/// A token an earlier Atollo saved is read as kit data, and «Esci»
/// deletes it; nothing writes one any more (D-179).
#[test]
fn a_saved_token_is_read_and_removed() {
    let (_tmp, kit) = prepared_kit("claude-token");
    assert_eq!(
        path(&kit),
        kit.data().join("agents").join("claude").join("oauth-token")
    );
    assert_eq!(read(&kit), None);
    remove(&kit).unwrap();
    std::fs::create_dir_all(path(&kit).parent().unwrap()).unwrap();
    std::fs::write(path(&kit), format!("{}\n", sample())).unwrap();
    assert_eq!(read(&kit), Some(sample()));
    remove(&kit).unwrap();
    assert!(!path(&kit).exists());
    assert_eq!(read(&kit), None);
}
