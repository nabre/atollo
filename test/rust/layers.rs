//! The layers of the engine (D-189, docs/app/ARCHITECTURE.md): each folder
//! of `tauri/src` uses only folders of its own layer or of a lower one.
//! Inside a layer the edges allowed are listed one by one ([`SAME`]); an
//! edge to a higher layer is debt, listed with its reason ([`DEBT`]). A new
//! edge fails, and so does one of the lists naming an edge that is gone:
//! they only shrink. Comments, strings and the test modules are left out.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;

/// The layers, lowest first: 0 the infrastructure, 1 the tools, 2 the core
/// (the sandbox, the terminal and editor, the levels that place every
/// program), 3 the domains, 4 the interface and the app.
const LAYERS: [&[&str]; 5] = [
    &["common", "kit", "os", "platform", "env", "net"],
    &["setup", "install", "versions", "router"],
    &["isolation", "editor", "levels"],
    &[
        "projects",
        "detect",
        "db",
        "servers",
        "agents",
        "llm",
        "github",
        "diagnostics",
        "ports",
    ],
    &["ipc", "app"],
];

/// The edges allowed inside a layer.
const SAME: &[(&str, &str)] = &[
    // 0: infrastructure
    ("common", "env"),
    ("common", "kit"),
    ("common", "net"),
    ("common", "platform"),
    ("env", "common"),
    ("env", "kit"),
    ("env", "net"),
    ("env", "platform"),
    ("kit", "common"),
    ("kit", "platform"),
    ("net", "common"),
    ("net", "kit"),
    ("net", "platform"),
    ("os", "common"),
    ("os", "env"),
    ("os", "kit"),
    ("os", "platform"),
    ("platform", "common"),
    ("platform", "env"),
    ("platform", "kit"),
    ("platform", "os"),
    // 1: tools
    ("install", "router"),
    ("install", "setup"),
    ("install", "versions"),
    ("router", "install"),
    ("router", "setup"),
    ("router", "versions"),
    ("setup", "install"),
    ("setup", "router"),
    ("setup", "versions"),
    ("versions", "install"),
    ("versions", "router"),
    ("versions", "setup"),
    // 2: core
    ("editor", "isolation"),
    ("editor", "levels"),
    ("isolation", "editor"),
    ("isolation", "levels"),
    ("levels", "editor"),
    ("levels", "isolation"),
    // 3: domains
    ("agents", "github"),
    ("agents", "llm"),
    ("agents", "projects"),
    ("db", "agents"),
    ("db", "ports"),
    ("db", "projects"),
    ("detect", "projects"),
    ("diagnostics", "agents"),
    ("diagnostics", "db"),
    ("diagnostics", "github"),
    ("diagnostics", "ports"),
    ("github", "projects"),
    ("llm", "agents"),
    ("llm", "db"),
    ("llm", "ports"),
    ("ports", "db"),
    ("ports", "llm"),
    ("ports", "projects"),
    ("ports", "servers"),
    ("projects", "agents"),
    ("projects", "db"),
    ("projects", "detect"),
    ("projects", "ports"),
    ("projects", "servers"),
    ("servers", "db"),
    ("servers", "detect"),
    ("servers", "ports"),
    ("servers", "projects"),
    // 4: interface
    ("app", "ipc"),
    ("ipc", "app"),
];

/// The edges to a higher layer still there, each with why and where it
/// starts (the line may move).
const DEBT: &[(&str, &str, &str)] = &[
    // 0 -> 1
    ("env", "setup", "the kit's environment reads the tools' folders (kit_path, the catalog): env/langs.rs, env/mod.rs"),
    ("env", "versions", "the kit's environment applies the chosen lines: env/langs.rs, env/mod.rs"),
    ("kit", "install", "a moved kit relocates the components (install::relocate): kit/mod.rs"),
    // 1 -> 2, 3
    ("install", "isolation", "the tools sandbox runs installers for the VM's Linux: install/guest.rs"),
    ("install", "projects", "Apache's PHP blocks live with the projects' Apache: install/web/apache.rs"),
    ("setup", "detect", "what an action needs reads detect::Action: setup/needs.rs"),
    ("setup", "github", "the downloads use the kit's GitHub token: setup/task.rs, setup/updates.rs"),
    ("setup", "isolation", "a failed Docker Sandboxes install is noted for the retry: setup/runner.rs"),
    ("versions", "db", "the databases' data and lines follow the versions: versions/disk.rs, versions/relations/projects.rs"),
    ("versions", "detect", "the projects' pins are read from their files: versions/relations/"),
    ("versions", "projects", "a project's chosen lines are its settings: versions/choice.rs, versions/mod.rs"),
    // 2 -> 3, 4
    ("editor", "agents", "a terminal's agent profiles: editor/terminals.rs"),
    ("editor", "detect", "the editor's extensions for a project's languages: editor/ext.rs"),
    ("editor", "ipc", "the terminal's sessions emit the interface's events: editor/terminal.rs"),
    ("editor", "projects", "the editor opens a project's folder and settings: editor/ext.rs, editor/mod.rs"),
    ("isolation", "agents", "the Gemini key and the agents' builds for a sandbox: isolation/mod.rs"),
    ("isolation", "db", "a sandbox reaches its project's databases: isolation/mod.rs"),
    ("isolation", "github", "«GitHub nella VM» gives the kit's token to a sandbox: isolation/mod.rs"),
    ("isolation", "llm", "Pi's sandbox reaches the local models: isolation/mod.rs"),
    ("isolation", "projects", "a sandbox follows its project's settings and local environments: isolation/mod.rs"),
    ("levels", "agents", "a session gets its agent's account: levels/prepare.rs"),
    ("levels", "db", "a session gets its project's databases: levels/env.rs, levels/prepare.rs"),
    ("levels", "detect", "an action is started from its detected command: levels/start.rs"),
    ("levels", "projects", "a session gets its project's lines and runner: levels/env.rs, levels/start.rs"),
    // 3 -> 4
    ("diagnostics", "app", "the diagnostics zip names the app's version and log: diagnostics/mod.rs"),
];

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

/// `text` without comments (line and block) and with every string literal
/// emptied, raw strings too.
fn code_only(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < b.len() {
        let rest = &text[i..];
        if rest.starts_with("//") {
            i += rest.find('\n').unwrap_or(rest.len());
        } else if rest.starts_with("/*") {
            i += rest.find("*/").map_or(rest.len(), |e| e + 2);
        } else if let Some(hashes) = rest
            .strip_prefix('r')
            .map(|r| r.len() - r.trim_start_matches('#').len())
            .filter(|h| rest[1 + h..].starts_with('"'))
        {
            let end = format!("\"{}", "#".repeat(hashes));
            let body = 2 + hashes;
            i += rest[body..]
                .find(&end)
                .map_or(rest.len(), |e| body + e + end.len());
            out.push_str("\"\"");
        } else if rest.starts_with('"') {
            let mut j = 1;
            while j < rest.len() && rest.as_bytes()[j] != b'"' {
                j += if rest.as_bytes()[j] == b'\\' { 2 } else { 1 };
            }
            i += (j + 1).min(rest.len());
            out.push_str("\"\"");
        } else if rest.starts_with("'\"'") || rest.starts_with("'\\\"'") {
            i += if rest.starts_with("'\"'") { 3 } else { 4 };
            out.push_str("' '");
        } else {
            let c = rest.chars().next().unwrap();
            out.push(c);
            i += c.len_utf8();
        }
    }
    out
}

fn layer_of(module: &str) -> Option<usize> {
    LAYERS.iter().position(|l| l.contains(&module))
}

/// Every `crate::<folder>` edge between two folders of `tauri/src`, with
/// the first place it is written.
fn edges() -> BTreeMap<(String, String), String> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let folders: BTreeSet<String> = fs::read_dir(&src)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    let single = Regex::new(r"\bcrate::(\w+)").unwrap();
    let group = Regex::new(r"\bcrate::\{([^}]*)\}").unwrap();
    let part = Regex::new(r"(?:^|,)\s*(\w+)").unwrap();
    // A test module's declaration (`#[cfg(test)] #[path = …] mod tests;`):
    // its file is the test's, never the engine's.
    let tests = Regex::new(r"#\[cfg\(test\)\]\s*(?:#\[path[^\]]*\]\s*)?mod \w+;").unwrap();
    let mut out = BTreeMap::new();
    for folder in &folders {
        let mut files = Vec::new();
        sources(&src.join(folder), &mut files);
        for file in files {
            let code = code_only(&fs::read_to_string(&file).unwrap());
            let code = tests.replace_all(&code, "");
            let code = code.as_ref();
            let mut found: Vec<(String, usize)> = single
                .captures_iter(code)
                .map(|c| (c[1].to_owned(), c.get(0).unwrap().start()))
                .collect();
            for c in group.captures_iter(code) {
                let at = c.get(0).unwrap().start();
                found.extend(part.captures_iter(&c[1]).map(|p| (p[1].to_owned(), at)));
            }
            for (to, at) in found {
                if to != *folder && folders.contains(&to) {
                    let line = code[..at].matches('\n').count() + 1;
                    let rel = file.strip_prefix(&src).unwrap().display().to_string();
                    out.entry((folder.clone(), to))
                        .or_insert_with(|| format!("{rel}:{line}"));
                }
            }
        }
    }
    out
}

#[test]
fn every_folder_has_a_layer() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for e in fs::read_dir(src).unwrap().flatten() {
        if e.path().is_dir() {
            let name = e.file_name().to_string_lossy().into_owned();
            assert!(
                layer_of(&name).is_some(),
                "{name}: no layer in test/rust/layers.rs (D-189)"
            );
        }
    }
}

#[test]
fn the_layers_go_one_way() {
    let found = edges();
    let mut wrong = Vec::new();
    for ((from, to), at) in &found {
        let (lf, lt) = (layer_of(from).unwrap(), layer_of(to).unwrap());
        let listed_same = SAME.contains(&(from.as_str(), to.as_str()));
        let listed_debt = DEBT.iter().any(|(f, t, _)| f == from && t == to);
        let ok = lt < lf || (lt == lf && listed_same) || (lt > lf && listed_debt);
        if !ok {
            wrong.push(format!("{from} -> {to} ({at}): layer {lf} -> {lt}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "edges against the layers (D-189): move the item down, pass it in, or list the edge:\n{}",
        wrong.join("\n")
    );
    let gone: Vec<String> = SAME
        .iter()
        .map(|(f, t)| (*f, *t))
        .chain(DEBT.iter().map(|(f, t, _)| (*f, *t)))
        .filter(|(f, t)| !found.contains_key(&((*f).to_owned(), (*t).to_owned())))
        .map(|(f, t)| format!("{f} -> {t}"))
        .collect();
    assert!(
        gone.is_empty(),
        "listed edges that are gone, take them off the lists:\n{}",
        gone.join("\n")
    );
    for (from, to) in SAME {
        assert_eq!(
            layer_of(from),
            layer_of(to),
            "SAME across layers: {from} -> {to}"
        );
    }
    for (from, to, why) in DEBT {
        assert!(
            layer_of(to) > layer_of(from),
            "DEBT not upward: {from} -> {to}"
        );
        assert!(!why.is_empty());
    }
}
