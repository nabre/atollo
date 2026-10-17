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
    let parsed: Catalog = serde_json::from_str(include_str!("../../tauri/i18n/catalog.json"))
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

/// The text of a Rust string literal: escapes resolved, `\` + newline
/// skips the line break and the indentation that follows.
fn unescape(raw: &str) -> String {
    let mut out = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\r' | '\n') => {
                while chars.peek().is_some_and(|c| c.is_whitespace()) {
                    chars.next();
                }
            }
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

fn rust_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("src is readable").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The descriptions of the components (Italian in components.json) are
/// translated like the other texts.
#[test]
fn component_descriptions_are_in_the_catalog() {
    for c in crate::setup::catalog::Catalog::get().all() {
        let tr = catalog().get(&c.desc);
        for lang in ["en", "fr", "de"] {
            assert!(
                tr.and_then(|t| t.get(lang)).is_some_and(|t| !t.is_empty()),
                "{}: {lang}",
                c.id
            );
        }
    }
    assert_eq!(t("en", "PHP e Composer", &[]), "PHP and Composer");
}

/// The descriptions and notes of the frameworks (frameworks.json) too.
#[test]
fn framework_texts_are_in_the_catalog() {
    for p in crate::projects::frameworks::official() {
        let notes = p.on.values().filter_map(|v| v.note.as_ref());
        for s in std::iter::once(&p.desc).chain(&p.note).chain(notes) {
            let tr = catalog().get(s);
            for lang in ["en", "fr", "de"] {
                assert!(
                    tr.and_then(|t| t.get(lang)).is_some_and(|t| !t.is_empty()),
                    "{s}: {lang}"
                );
            }
        }
    }
}

/// The notes of the relations matrix (relations.json) too.
#[test]
fn relation_notes_are_in_the_catalog() {
    for note in crate::versions::relations::notes() {
        let tr = catalog().get(note);
        for lang in ["en", "fr", "de"] {
            assert!(
                tr.and_then(|t| t.get(lang)).is_some_and(|t| !t.is_empty()),
                "{note}: {lang}"
            );
        }
    }
}

/// Every literal Italian source passed to `tr(lang, "…")`, `i18n::t(lang,
/// "…")` or `crate::i18n::t(lang, "…")`, and every source returned by
/// `Settings::validate`, is in the catalog. Calls whose text is not a
/// literal are skipped; `mod tests` blocks are ignored.
#[test]
fn sources_are_in_the_catalog() {
    let calls =
        regex::Regex::new(r#"\b(?:tr|i18n::t)\(\s*(?:"[^"]*"|[^,()"]+),\s*"((?:[^"\\]|\\.)*)""#)
            .unwrap();
    let validate = regex::Regex::new(r#"\bErr\(\(\s*"((?:[^"\\]|\\.)*)""#).unwrap();
    let tests = regex::Regex::new(r"(?m)^#\[cfg\(test\)\]\s*\n\s*mod ").unwrap();
    let mut files = Vec::new();
    rust_files(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut files,
    );
    let mut missing = BTreeSet::new();
    let mut found = 0;
    for path in &files {
        let code = std::fs::read_to_string(path).expect("source is readable");
        let code = &code[..tests.find(&code).map_or(code.len(), |m| m.start())];
        let mut sources: Vec<&str> = calls
            .captures_iter(code)
            .filter_map(|c| c.get(1))
            .map(|m| m.as_str())
            .collect();
        if path.ends_with("settings.rs") || path.components().any(|c| c.as_os_str() == "frameworks")
        {
            sources.extend(
                validate
                    .captures_iter(code)
                    .filter_map(|c| c.get(1))
                    .map(|m| m.as_str()),
            );
        }
        for raw in sources {
            found += 1;
            let source = unescape(raw);
            if !catalog().contains_key(&source) {
                missing.insert(format!("{}: {source:?}", path.display()));
            }
        }
    }
    assert!(found > 50, "too few texts found: {found}");
    assert!(
        missing.is_empty(),
        "missing from i18n/catalog.json:\n{missing:#?}"
    );
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

#[test]
fn locale_of_macos_and_linux() {
    assert_eq!(posix_locale("it_IT.UTF-8").as_deref(), Some("it-IT"));
    assert_eq!(posix_locale("de_DE@euro").as_deref(), Some("de-DE"));
    assert_eq!(posix_locale("fr").as_deref(), Some("fr"));
    assert_eq!(posix_locale("C.UTF-8"), None);
    assert_eq!(posix_locale("POSIX"), None);
    assert_eq!(
        resolve(None, None, posix_locale("it_IT.UTF-8").as_deref()),
        "it"
    );
}

/// T1.8.5: the output of `defaults read -g AppleLanguages`.
#[test]
fn apple_languages() {
    let out = "(\n    \"it-IT\",\n    \"en-US\"\n)\n";
    assert_eq!(first_apple_language(out).as_deref(), Some("it-IT"));
    assert_eq!(
        first_apple_language("(\n    de,\n    en\n)").as_deref(),
        Some("de")
    );
    assert_eq!(
        resolve(
            None,
            None,
            first_apple_language("(\n    \"fr-CA\"\n)").as_deref()
        ),
        "fr"
    );
    assert_eq!(first_apple_language("(\n)\n"), None);
    assert_eq!(first_apple_language(""), None);
}
