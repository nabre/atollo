//! Texts of the app in Italian (source), English, French and German.
//!
//! Source strings are Italian; `i18n/catalog.json` maps
//! each one to its translations and is embedded in the exe. The test at the
//! bottom applies these rules: every text
//! in every language, same `{n}` placeholders, no empty translation.

use std::collections::BTreeMap;
use std::sync::OnceLock;

pub const LANGS: [&str; 4] = ["it", "en", "fr", "de"];

type Catalog = BTreeMap<String, BTreeMap<String, String>>;

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        // A broken catalog fails the tests; at run time it degrades to Italian.
        serde_json::from_str(include_str!("../../i18n/catalog.json")).unwrap_or_default()
    })
}

/// The two-letter language to use: the setting, then `ATOLLO_LANG`, then the
/// language of Windows; anything other than it/fr/de becomes en.
pub fn resolve(
    setting: Option<&str>,
    atollo_lang: Option<&str>,
    windows: Option<&str>,
) -> &'static str {
    let pick = setting
        .filter(|s| !s.is_empty())
        .or(atollo_lang.filter(|s| !s.is_empty()))
        .or(windows)
        .unwrap_or("en");
    let two = pick.get(..2).unwrap_or("").to_ascii_lowercase();
    LANGS.into_iter().find(|l| *l == two).unwrap_or("en")
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
#[path = "../../../test/rust/common/i18n.rs"]
mod tests;
