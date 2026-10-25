use super::*;

#[test]
fn locale_of_macos_and_linux() {
    assert_eq!(posix_locale("it_IT.UTF-8").as_deref(), Some("it-IT"));
    assert_eq!(posix_locale("de_DE@euro").as_deref(), Some("de-DE"));
    assert_eq!(posix_locale("fr").as_deref(), Some("fr"));
    assert_eq!(posix_locale("C.UTF-8"), None);
    assert_eq!(posix_locale("POSIX"), None);
    assert_eq!(
        crate::common::i18n::resolve(None, None, posix_locale("it_IT.UTF-8").as_deref()),
        "it"
    );
}
