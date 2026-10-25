use super::*;

/// D-105: the output of `defaults read -g AppleLanguages`.
#[test]
fn apple_languages() {
    let out = "(\n    \"it-IT\",\n    \"en-US\"\n)\n";
    assert_eq!(first_apple_language(out).as_deref(), Some("it-IT"));
    assert_eq!(
        first_apple_language("(\n    de,\n    en\n)").as_deref(),
        Some("de")
    );
    assert_eq!(
        crate::common::i18n::resolve(
            None,
            None,
            first_apple_language("(\n    \"fr-CA\"\n)").as_deref()
        ),
        "fr"
    );
    assert_eq!(first_apple_language("(\n)\n"), None);
    assert_eq!(first_apple_language(""), None);
}
