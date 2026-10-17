//! Texts of the app in Italian (source), English, French and German.
//!
//! Source strings are Italian, as in nabre/atollo; `i18n/catalog.json` maps
//! each one to its translations and is embedded in the exe. The test at the
//! bottom applies the rules of nabre/atollo `setup/lang/build.js`: every text
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

/// The user's locale on macOS and Linux, from `LC_ALL`, `LC_MESSAGES` or
/// `LANG` (`it_IT.UTF-8` gives `it-IT`); `C` and `POSIX` are none. On
/// macOS the first of the languages chosen in System Settings comes first
/// (T1.8.5): an app opened from the Finder gets no `LANG`.
#[cfg(not(windows))]
pub fn windows_locale() -> Option<String> {
    #[cfg(target_os = "macos")]
    if let Some(lang) = apple_language() {
        return Some(lang);
    }
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty())
        .and_then(|v| posix_locale(&v))
}

/// `defaults read -g AppleLanguages`, read once: the first call comes at
/// start, before the app points `HOME` into the kit.
#[cfg(target_os = "macos")]
fn apple_language() -> Option<String> {
    static LANG: OnceLock<Option<String>> = OnceLock::new();
    LANG.get_or_init(|| {
        let out = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "-g", "AppleLanguages"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        first_apple_language(&String::from_utf8_lossy(&out.stdout))
    })
    .clone()
}

/// The first language of the list `defaults` prints
/// (`(\n    "it-IT",\n    en\n)`).
#[cfg(any(target_os = "macos", test))]
fn first_apple_language(output: &str) -> Option<String> {
    output
        .lines()
        .map(|l| l.trim().trim_end_matches(',').trim().trim_matches('"'))
        .find(|l| !l.is_empty() && *l != "(" && *l != ")")
        .map(str::to_owned)
}

/// `it_IT.UTF-8@euro` → `it-IT`; `C` and `POSIX` → None.
#[cfg(any(not(windows), test))]
fn posix_locale(value: &str) -> Option<String> {
    let name = value.split(['.', '@']).next().unwrap_or_default();
    if name.is_empty() || name == "C" || name == "POSIX" {
        return None;
    }
    Some(name.replace('_', "-"))
}

/// The user's default locale name from Windows, e.g. `it-IT`.
#[cfg(windows)]
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
#[path = "../../test/rust/i18n.rs"]
mod tests;
