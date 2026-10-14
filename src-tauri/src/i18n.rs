//! Texts of the app in Italian (source), English, French and German.
//!
//! Source strings are Italian, as in nabre/dev-kit; `i18n/catalog.json` maps
//! each one to its translations and is embedded in the exe. The test at the
//! bottom applies the rules of nabre/dev-kit `setup/lang/build.js`: every text
//! in every language, same `{n}` placeholders, no empty translation.

use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const LANGS: [&str; 4] = ["it", "en", "fr", "de"];

type Catalog = BTreeMap<String, BTreeMap<String, String>>;

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        // A broken catalog fails the tests; at run time it degrades to Italian.
        serde_json::from_str(include_str!("../i18n/catalog.json")).unwrap_or_default()
    })
}

/// The two-letter language to use: the setting, then `DEVKIT_LANG`, then the
/// language of Windows; anything other than it/fr/de becomes en.
pub fn resolve(
    setting: Option<&str>,
    devkit_lang: Option<&str>,
    windows: Option<&str>,
) -> &'static str {
    let pick = setting
        .filter(|s| !s.is_empty())
        .or(devkit_lang.filter(|s| !s.is_empty()))
        .or(windows)
        .unwrap_or("en");
    let two = pick.get(..2).unwrap_or("").to_ascii_lowercase();
    LANGS.into_iter().find(|l| *l == two).unwrap_or("en")
}

/// The user's default locale name from Windows, e.g. `it-IT`.
pub fn windows_locale() -> Option<String> {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    let mut buf = [0u16; 85];
    // SAFETY: the buffer is LOCALE_NAME_MAX_LENGTH (85) wide chars, as required.
    let len = unsafe { GetUserDefaultLocaleName(&mut buf) };
    (len > 1).then(|| String::from_utf16_lossy(&buf[..(len as usize - 1)]))
}

/// The translation of `it_source` in `lang`, with `{0}`, `{1}`… replaced.
pub fn t(lang: &str, it_source: &str, args: &[&str]) -> String {
    let text = if lang == "it" {
        it_source
    } else {
        catalog()
            .get(it_source)
            .and_then(|tr| tr.get(lang))
            .map(String::as_str)
            .unwrap_or(it_source)
    };
    let mut out = text.to_owned();
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), arg);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn placeholders(s: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut rest = s;
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            match after.find('}') {
                Some(end) if after[..end].chars().all(|c| c.is_ascii_digit()) && end > 0 => {
                    out.insert(after[..end].to_owned());
                    rest = &after[end + 1..];
                }
                _ => rest = after,
            }
        }
        out
    }

    #[test]
    fn catalog_is_complete() {
        let parsed: Catalog = serde_json::from_str(include_str!("../i18n/catalog.json"))
            .expect("i18n/catalog.json is not valid JSON");
        assert!(!parsed.is_empty());
        for (source, tr) in &parsed {
            for lang in ["en", "fr", "de"] {
                let text = tr
                    .get(lang)
                    .unwrap_or_else(|| panic!("{lang} missing for {source:?}"));
                assert!(!text.trim().is_empty(), "{lang} empty for {source:?}");
                assert_eq!(
                    placeholders(text),
                    placeholders(source),
                    "{lang} placeholders differ for {source:?}"
                );
            }
            assert_eq!(tr.len(), 3, "unexpected languages for {source:?}");
        }
    }

    #[test]
    fn language_choice() {
        assert_eq!(resolve(Some("de"), Some("fr"), Some("it-IT")), "de");
        assert_eq!(resolve(None, Some("FR"), Some("it-IT")), "fr");
        assert_eq!(resolve(Some(""), None, Some("it-CH")), "it");
        assert_eq!(resolve(None, None, Some("es-ES")), "en");
        assert_eq!(resolve(None, None, None), "en");
    }

    #[test]
    fn translation_and_arguments() {
        let source = "Impossibile scrivere nella cartella {0}.";
        assert_eq!(
            t("it", source, &["X"]),
            "Impossibile scrivere nella cartella X."
        );
        assert_eq!(t("en", source, &["X"]), "Cannot write in the folder X.");
        assert_eq!(t("en", "testo assente", &[]), "testo assente");
    }
}
