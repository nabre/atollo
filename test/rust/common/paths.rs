//! No `Path::join`/`push` with a separator written inside the string
//! (D-105): on macOS and Linux `\` is a character of a file name,
//! so `data.join(r"a\b")` names one file `a\b` instead of `a/b`. Paths go
//! through `fsx::join_rel` or chained `.join()` calls.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// Joins with a backslash that are right as they are, with the reason:
/// (file under `tauri/src`, the string literal as written, why). Those not
/// yet inside an OS check say so, and which phase gates them. The files of
/// `os/windows/`, built for Windows only, are not looked at.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "editor/cfg.rs",
        r#"r"System32\cmd.exe""#,
        "a program of Windows under %SystemRoot%, inside `if os.is_windows()`: \
         the cmd profile is written for Windows only",
    ),
    (
        "env/path.rs",
        r#"r"WindowsPowerShell\v1.0""#,
        "under System32, inside `if is_win`",
    ),
    (
        "env/path.rs",
        r#"r"WindowsPowerShell\v1.0\Modules""#,
        "under System32, inside `if is_win`",
    ),
    (
        "install/web/vcruntime.rs",
        r#"r"System32\expand.exe""#,
        "a program of Windows under %SystemRoot%; not gated yet: it extracts the \
         Visual C++ cabinet, which P6 keeps to Windows",
    ),
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

/// The literal holds a path separator: `\` in a raw string, `\\` in a
/// normal one (`\n`, `\"` are not separators).
fn has_backslash(literal: &str) -> bool {
    if literal.starts_with('r') {
        literal.contains('\\')
    } else {
        literal.contains(r"\\")
    }
}

/// (line, literal) of each `.join(`/`.push(` whose first argument is a
/// string literal with a backslash, or a `format!` of one (`&` or not),
/// and of each `Path::new(`/`PathBuf::from(` of such a literal. Comment
/// lines are skipped.
fn backslash_joins(text: &str) -> Vec<(usize, String)> {
    let re = Regex::new(
        r##"(?:\.(?:join|push)\(\s*&?(?:format!\(\s*)?|\b(?:Path::new|PathBuf::from)\(\s*)(r#*"[^"]*"#*|"(?:[^"\\]|\\.)*")"##,
    )
    .unwrap();
    // Comment lines blanked, so offsets (and line numbers) stay the same.
    let code: String = text
        .split_inclusive('\n')
        .map(|line| {
            if line.trim_start().starts_with("//") {
                " ".repeat(line.len() - 1) + "\n"
            } else {
                line.to_owned()
            }
        })
        .collect();
    re.captures_iter(&code)
        .filter_map(|c| {
            let lit = c.get(1)?;
            has_backslash(lit.as_str()).then(|| {
                let line = code[..lit.start()].matches('\n').count() + 1;
                (line, lit.as_str().to_owned())
            })
        })
        .collect()
}

#[test]
fn the_scanner_finds_what_it_should() {
    let text = concat!(
        "let a = d.join(r\"x\\y\");\n",
        "let b = d.join(\"x\\\\y\");\n",
        "let c = d\n    .join(\n        r\"x\\y\",\n    );\n",
        "out.push(r#\"x\\y\"#);\n",
        "// d.join(r\"comment\\only\")\n",
        "let ok = d.join(\"x\").join(\"y\");\n",
        "let s = v.join(\"\\n\");\n",
        "let q = v.push(\"say \\\"hi\\\"\");\n",
        "let f = d.join(format!(r\"{x}\\y\"));\n",
        "let g = d.join(&format!(\"{x}\\\\y\"));\n",
        "let h = Path::new(r\"a\\b\");\n",
        "let i = PathBuf::from(\"a\\\\b\");\n",
        "let j = d.join(format!(\"{x}/y\"));\n",
        "let k = Path::new(\"a/b\");\n",
    );
    let found = backslash_joins(text);
    let lines: Vec<usize> = found.iter().map(|(l, _)| *l).collect();
    assert_eq!(lines, [1, 2, 5, 7, 12, 13, 14, 15], "{found:?}");
}

/// The last `n` names of `path`, outermost first.
fn tail(path: &Path, n: usize) -> Vec<String> {
    let names: Vec<String> = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    names[names.len().saturating_sub(n)..].to_vec()
}

/// Paths of a macOS or Linux kit are made of separate names. On Windows
/// `\` splits names anyway, so this fails only where it matters, on Unix
/// (WSL2, CI); on Windows it checks that the paths did not change.
#[test]
fn kit_paths_are_separate_names() {
    use crate::kit::{Arch, Kit, Os, Target};
    for os in [Os::Win32, Os::Linux, Os::Darwin] {
        let kit = Kit::at_target(PathBuf::from("kit"), Target::new(os, Arch::X64));
        assert_eq!(
            tail(&crate::editor::cfg::settings_path(&kit), 4),
            ["vscodium-server", "data", "Machine", "settings.json"]
        );
        assert_eq!(
            tail(&crate::editor::ext::extensions_dir(&kit), 2),
            ["vscodium-server", "extensions"]
        );
        let p = crate::agents::sync::Paths::of(&kit);
        assert_eq!(tail(&p.state, 3), ["data", "agents", ".sync"]);
        assert_eq!(
            tail(&crate::github::hosts_file(&kit), 3),
            ["data", "gh", "hosts.yml"]
        );
    }
    // A project's git config, read for its remotes, by separate names too.
    let tmp = crate::kit::tests::TempDir::new("paths-remotes");
    let git = tmp.0.join(".git");
    fs::create_dir_all(&git).unwrap();
    fs::write(
        git.join("config"),
        "[remote \"origin\"]\n\turl = https://github.com/a/b.git\n",
    )
    .unwrap();
    assert_eq!(
        crate::github::repos::remote_urls(&tmp.0),
        ["https://github.com/a/b.git"]
    );
}

#[test]
fn no_backslash_in_joined_paths() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    sources(&src, &mut files);
    assert!(files.len() > 50, "sources not found in {}", src.display());
    let mut bad = Vec::new();
    let mut used = vec![false; ALLOWED.len()];
    for file in &files {
        let rel = file
            .strip_prefix(&src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        // Built for Windows only, where `\` is the separator.
        if rel.starts_with("os/windows/") {
            continue;
        }
        let text = fs::read_to_string(file).unwrap();
        for (line, lit) in backslash_joins(&text) {
            match ALLOWED.iter().position(|(f, l, _)| *f == rel && *l == lit) {
                Some(i) => used[i] = true,
                None => bad.push(format!("tauri/src/{rel}:{line}: {lit}")),
            }
        }
    }
    assert!(
        bad.is_empty(),
        "a separator written inside a joined path: use fsx::join_rel or chained .join() calls\n{}",
        bad.join("\n")
    );
    let stale: Vec<String> = ALLOWED
        .iter()
        .zip(&used)
        .filter(|(_, u)| !**u)
        .map(|((f, l, _), _)| format!("{f}: {l}"))
        .collect();
    assert!(
        stale.is_empty(),
        "exceptions no longer needed: remove them from ALLOWED\n{}",
        stale.join("\n")
    );
}
