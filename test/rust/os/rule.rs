//! The rule of `os/` (docs/app/ARCHITECTURE.md): the engine looks at which
//! system it is built for only in `tauri/src/os/`. Everywhere else (the
//! domains, `ipc/`, `platform/`) no `#[cfg]`, `#[cfg_attr]` or `cfg!` on the
//! system, no crate of one system and no `std::os::<system>`: what differs
//! goes behind a function of `platform/`. And every system offers the
//! modules `platform/` asks `os::current` for, so a build for another
//! system does not find one missing.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// The crates of one system (or of Unix only), as a path's first segment.
const SYSTEM_CRATES: [&str; 7] = [
    "libc",
    "windows",
    "webview2_com",
    "junction",
    "portable_pty",
    "rfd",
    "dbus_secret_service",
];

/// Every `.rs` file under `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            sources(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// `text` without comments and with every string literal emptied, line
/// breaks kept (the line numbers stay right).
fn code_only(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            // A character literal of a quote is no string.
            '\'' if chars.peek() == Some(&'"') => {
                out.push(c);
                out.push(chars.next().unwrap_or_default());
            }
            '\'' if chars.peek() == Some(&'\\') => {
                out.push(c);
                out.push(chars.next().unwrap_or_default());
                out.push(chars.next().unwrap_or_default());
            }
            '"' => {
                out.push('"');
                let mut escaped = false;
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                    }
                    if escaped {
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        break;
                    }
                }
                out.push('"');
            }
            _ => out.push(c),
        }
    }
    out
}

/// The predicate of a `cfg(…)`, `cfg_attr(…, …)` or `cfg!(…)` that starts
/// at `open`, the index just after its `(`.
fn predicate(code: &str, open: usize, attr: bool) -> &str {
    let mut depth = 1;
    for (i, c) in code[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if attr && depth == 1 => return &code[open..open + i],
            _ => {}
        }
        if depth == 0 {
            return &code[open..open + i];
        }
    }
    &code[open..]
}

/// Each check of the system in `code`: (line, what).
fn system_checks(code: &str) -> Vec<(usize, String)> {
    let line_of = |at: usize| code[..at].matches('\n').count() + 1;
    let mut found = Vec::new();
    let cfg = Regex::new(r"#!?\[\s*cfg(_attr)?\s*\(|\bcfg!\s*\(").unwrap();
    let system = Regex::new(r"\b(windows|unix|target_os|target_family|target_vendor)\b").unwrap();
    for m in cfg.find_iter(code) {
        let attr = m.as_str().contains("cfg_attr");
        let p = predicate(code, m.end(), attr);
        if system.is_match(p) {
            found.push((
                line_of(m.start()),
                format!("cfg on the system: {}", p.trim()),
            ));
        }
    }
    let crates = Regex::new(&format!(r"(?:^|[^\w:])({})::", SYSTEM_CRATES.join("|"))).unwrap();
    for c in crates.captures_iter(code) {
        let m = c.get(1).unwrap();
        found.push((
            line_of(m.start()),
            format!("crate of one system: {}", m.as_str()),
        ));
    }
    let std_os = Regex::new(r"std::os::(unix|windows|fd|linux|macos)\b").unwrap();
    for m in std_os.find_iter(code) {
        found.push((line_of(m.start()), m.as_str().to_owned()));
    }
    found
}

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

#[test]
fn the_system_is_checked_only_in_os() {
    let src = src();
    let mut files = Vec::new();
    sources(&src, &mut files);
    assert!(files.len() > 100, "sources not found in {}", src.display());
    let mut bad = Vec::new();
    for file in &files {
        let rel = file
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if rel.starts_with("os/") {
            continue;
        }
        let code = code_only(&fs::read_to_string(file).unwrap());
        for (line, what) in system_checks(&code) {
            bad.push(format!("tauri/src/{rel}:{line}: {what}"));
        }
    }
    assert!(
        bad.is_empty(),
        "code of one system outside tauri/src/os (put it behind platform/):\n{}",
        bad.join("\n")
    );
}

/// The checks the rule finds, and what it leaves alone.
#[test]
fn what_counts_as_a_check_of_the_system() {
    let code = code_only(
        "#[cfg(windows)]\nfn a() {}\n#[cfg(all(test, unix))]\n\
         #[cfg_attr(not(target_os = \"macos\"), allow(dead_code))]\n\
         let x = cfg!(windows);\nuse libc::c_int;\nuse std::os::unix::fs::PermissionsExt;\n\
         #![cfg_attr(not(test), windows_subsystem = \"windows\")]\n\
         #[cfg(test)]\nlet s = \"libc::x and #[cfg(windows)]\";\n\
         // #[cfg(unix)] in a comment\nuse crate::agents::keyring::save;\n",
    );
    let lines: Vec<usize> = system_checks(&code).iter().map(|(l, _)| *l).collect();
    assert_eq!(lines, [1, 3, 4, 5, 6, 7]);
}

/// The modules `platform/` asks `crate::os::current` for (`os::current::x`
/// or `sys::x` after `use crate::os::current as sys`).
fn wanted_modules() -> Vec<String> {
    let dir = src().join("platform");
    let mut files = Vec::new();
    sources(&dir, &mut files);
    let direct = Regex::new(r"os::current::([a-z_]+)").unwrap();
    let alias = Regex::new(r"\bsys::([a-z_]+)::").unwrap();
    let mut out: Vec<String> = Vec::new();
    for file in files {
        let code = code_only(&fs::read_to_string(file).unwrap());
        for c in direct
            .captures_iter(&code)
            .chain(alias.captures_iter(&code))
        {
            let name = c[1].to_owned();
            if !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out.sort();
    out
}

/// The modules a system's `mod.rs` offers: its own (`pub mod x;`) and the
/// ones of `unix/` it re-exports (`pub use super::unix::{x, …};`).
fn offered_modules(system: &str) -> Vec<String> {
    let code =
        code_only(&fs::read_to_string(src().join("os").join(system).join("mod.rs")).unwrap());
    let own = Regex::new(r"(?m)^pub mod ([a-z_]+);").unwrap();
    let shared = Regex::new(r"pub use super::unix::\{([^}]*)\}").unwrap();
    let mut out: Vec<String> = own.captures_iter(&code).map(|c| c[1].to_owned()).collect();
    for c in shared.captures_iter(&code) {
        out.extend(
            c[1].split(',')
                .map(|m| m.trim().to_owned())
                .filter(|m| !m.is_empty()),
        );
    }
    out
}

#[test]
fn every_system_offers_what_platform_asks_for() {
    let wanted = wanted_modules();
    assert!(wanted.len() > 10, "{wanted:?}");
    for system in ["windows", "macos", "linux"] {
        let offered = offered_modules(system);
        let missing: Vec<&String> = wanted.iter().filter(|m| !offered.contains(m)).collect();
        assert!(missing.is_empty(), "os/{system} has no {missing:?}");
    }
}
