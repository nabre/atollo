//! What a project is and how to start it (port of nabre/dev-kit
//! `core/detectors`): platform, languages, the kit's programs it needs and
//! the actions of the Projects page. Every action has:
//! - `command`: run by `cmd.exe /d /s /c` in the project folder, with the
//!   kit's environment (as in nabre/dev-kit: `npm.ps1` in PowerShell would
//!   swallow the `--` of `npm run dev -- --port`);
//! - `type`: `web` (a server on its own port: `{port}` in the command, `PORT`
//!   in the environment, preview in the browser), `desktop` (an app with its
//!   own window: Electron, Tauri), `command` (ends by itself: build, test);
//! - `folder`: a subfolder to run it in (composite projects);
//! - `env`, `url`: optional, `{port}` replaced at start;
//! - `static`: a folder of the project served by the exe's file server
//!   (`Dev-Kit.exe serve`), instead of a command;
//! - `apache`: a folder of the project served by an Apache of its own
//!   (`apache.rs`), instead of a command.
//!
//! `panel.json` in the project (old name `pannello.json`) corrects or
//! replaces what was detected (`overrides.rs`). One file per platform.

mod go;
mod node;
mod overrides;
mod php;
mod python;
mod rust;
mod site;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub use overrides::{
    remove_action, save_action, set_default_action, set_site, ActionEdit, SiteKind, SITE_ACTION,
};

/// A folder inside the project (relative, no `..`, no link leading out).
pub fn inside_dir(project_dir: &Path, folder: &str) -> anyhow::Result<std::path::PathBuf> {
    overrides::inside(project_dir, folder).map(|(full, _)| full)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Command,
    Web,
    Desktop,
}

impl Kind {
    fn parse(text: &str) -> Option<Kind> {
        match text {
            "web" => Some(Kind::Web),
            "desktop" => Some(Kind::Desktop),
            // Old Italian value.
            "command" | "comando" => Some(Kind::Command),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    pub id: String,
    pub label: String,
    /// `cmd.exe` command line, empty for `static`.
    pub command: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub folder: Option<String>,
    pub url: Option<String>,
    pub env: BTreeMap<String, String>,
    #[serde(rename = "static")]
    pub static_dir: Option<String>,
    /// A folder served by the project's own Apache.
    #[serde(rename = "apache")]
    pub apache_dir: Option<String>,
    /// From panel.json.
    pub custom: bool,
}

impl Action {
    fn new(id: &str, label: &str, command: &str, kind: Kind) -> Action {
        Action {
            id: id.to_owned(),
            label: label.to_owned(),
            command: command.to_owned(),
            kind,
            folder: None,
            url: None,
            env: BTreeMap::new(),
            static_dir: None,
            apache_dir: None,
            custom: false,
        }
    }

    #[cfg(test)]
    pub fn for_test(id: &str, command: &str, kind: Kind) -> Action {
        Action::new(id, id, command, kind)
    }

    /// A folder of the project served by the exe's file server.
    fn serve(id: &str, label: &str, folder: &str) -> Action {
        Action {
            static_dir: Some(folder.to_owned()),
            ..Action::new(id, label, "", Kind::Web)
        }
    }

    /// A folder of the project served by its own Apache.
    fn apache(id: &str, label: &str, folder: &str) -> Action {
        Action {
            apache_dir: Some(folder.to_owned()),
            ..Action::new(id, label, "", Kind::Web)
        }
    }
}

/// What one detector found in one folder.
#[derive(Debug, Clone)]
struct Found {
    platform: String,
    kind: Kind,
    reason: String,
    actions: Vec<Action>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    pub platform: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub reason: String,
    pub languages: Vec<String>,
    /// Ids of the kit's components it needs.
    pub requires: Vec<String>,
    pub actions: Vec<Action>,
    pub warnings: Vec<String>,
    pub custom: bool,
    /// Id of the action of the main button.
    pub default: Option<String>,
}

/// Folders that do not hold sources.
const IGNORED: [&str; 18] = [
    "node_modules",
    ".git",
    "target",
    "vendor",
    ".venv",
    "venv",
    "env",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    ".svelte-kit",
    "__pycache__",
    ".idea",
    ".vscode",
    "coverage",
    "storage",
];

/// Files that make a subfolder a project of its own (frontend\, backend\).
const MANIFESTS: [&str; 7] = [
    "package.json",
    "composer.json",
    "pyproject.toml",
    "requirements.txt",
    "manage.py",
    "Cargo.toml",
    "go.mod",
];

pub const OVERRIDE_FILES: [&str; 2] = ["panel.json", "pannello.json"];

fn tr(lang: &str, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang, source, args)
}

// Every look into a project goes through `safe_path`: never through a
// symbolic link or a junction (D-77).
fn exists(dir: &Path, rel: &str) -> bool {
    crate::safe_path::exists(dir, &dir.join(rel))
}

fn read(dir: &Path, rel: &str) -> String {
    crate::safe_path::read_text(dir, &dir.join(rel)).unwrap_or_default()
}

fn read_json(dir: &Path, rel: &str) -> Option<serde_json::Value> {
    serde_json::from_str(&read(dir, rel)).ok()
}

fn list_dir(dir: &Path) -> Vec<String> {
    if crate::safe_path::is_link(dir) {
        return Vec::new();
    }
    let mut names: Vec<String> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    names.sort();
    names
}

/// Quotes a path for `cmd.exe` when it holds spaces or symbols.
fn quote(s: &str) -> String {
    if s.chars()
        .any(|c| c.is_whitespace() || "&()^,;=".contains(c))
    {
        format!("\"{s}\"")
    } else {
        s.to_owned()
    }
}

/// One walk of the project: 3 levels, at most 3000 files, no IGNORED folders.
fn walk_files(dir: &Path, on_file: &mut dyn FnMut(&str)) {
    fn walk(d: &Path, depth: u32, seen: &mut u32, on_file: &mut dyn FnMut(&str)) {
        // Below the root, links are not entered (a DirEntry's type is the
        // link's own); the root itself is checked here.
        if depth > 3 || *seen > 3000 || (depth == 0 && crate::safe_path::is_link(d)) {
            return;
        }
        let Ok(entries) = fs::read_dir(d) else {
            return;
        };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() {
                if !IGNORED.contains(&name.as_str()) {
                    walk(&e.path(), depth + 1, seen, on_file);
                }
                continue;
            }
            *seen += 1;
            on_file(&name);
        }
    }
    let mut seen = 0;
    walk(dir, 0, &mut seen, on_file);
}

fn ext_of(name: &str) -> String {
    Path::new(name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy().to_ascii_lowercase()))
        .unwrap_or_default()
}

/// The languages a project can have, by file extension, in the order the
/// matrix of editor extensions shows them (`editor_ext.rs`).
pub const LANGUAGES: [(&str, &[&str]); 27] = [
    ("JavaScript", &[".js", ".mjs", ".cjs", ".jsx"]),
    ("TypeScript", &[".ts", ".tsx"]),
    ("Vue", &[".vue"]),
    ("Svelte", &[".svelte"]),
    ("Astro", &[".astro"]),
    ("HTML", &[".html"]),
    ("CSS", &[".css", ".scss"]),
    ("PHP", &[".php"]),
    ("Python", &[".py"]),
    ("Rust", &[".rs"]),
    ("Go", &[".go"]),
    ("Java", &[".java"]),
    ("Kotlin", &[".kt", ".kts"]),
    ("C#", &[".cs"]),
    ("F#", &[".fs"]),
    ("C", &[".c", ".h"]),
    ("C++", &[".cpp", ".cc", ".hpp"]),
    ("Zig", &[".zig"]),
    ("Dart", &[".dart"]),
    ("Ruby", &[".rb"]),
    ("Julia", &[".jl"]),
    ("R", &[".r"]),
    ("LaTeX", &[".tex"]),
    ("Typst", &[".typ"]),
    ("PowerShell", &[".ps1", ".psm1"]),
    ("Shell", &[".sh", ".bash"]),
    ("SQL", &[".sql"]),
];

fn language_of(ext: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .find(|(_, exts)| exts.contains(&ext))
        .map(|(name, _)| *name)
}

/// Languages by number of files, most used first.
pub fn languages(dir: &Path) -> Vec<String> {
    let mut count: BTreeMap<&'static str, usize> = BTreeMap::new();
    walk_files(dir, &mut |name| {
        if let Some(l) = language_of(&ext_of(name)) {
            *count.entry(l).or_default() += 1;
        }
    });
    let mut list: Vec<(&str, usize)> = count.into_iter().collect();
    list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    list.into_iter().map(|(l, _)| l.to_owned()).collect()
}

fn need_of_ext(ext: &str) -> Option<&'static str> {
    Some(match ext {
        ".py" => "python",
        ".php" => "php",
        ".rs" => "rust",
        ".go" => "go",
        ".java" => "java",
        ".kt" | ".kts" => "kotlin",
        ".cs" | ".fs" => "dotnet",
        ".dart" => "dart",
        ".zig" => "zig",
        ".jl" => "julia",
        ".rb" => "ruby",
        ".r" | ".rmd" => "r",
        ".c" | ".cc" | ".cpp" | ".cxx" | ".h" | ".hpp" => "cc",
        ".tex" => "tinytex",
        ".typ" => "typst",
        _ => return None,
    })
}

fn need_of_file(name: &str) -> Option<&'static str> {
    Some(match name {
        "requirements.txt" | "pyproject.toml" | "Pipfile" | "manage.py" => "python",
        "composer.json" => "php",
        "Cargo.toml" => "rust",
        "go.mod" => "go",
        "pom.xml" | "build.gradle" => "java",
        "build.gradle.kts" => "kotlin",
        "pubspec.yaml" => "dart",
        "build.zig" => "zig",
        "Project.toml" => "julia",
        "bun.lockb" | "bun.lock" | "bunfig.toml" => "bun",
        "deno.json" | "deno.jsonc" | "deno.lock" => "deno",
        "pnpm-lock.yaml" | "pnpm-workspace.yaml" | "yarn.lock" => "pnpm",
        "CMakeLists.txt" => "cmake",
        "Gemfile" | "Gemfile.lock" => "ruby",
        "renv.lock" => "r",
        _ => return None,
    })
}

/// The database servers a project uses (D-73), from its root: SQLite files,
/// `.env` (Laravel's `DB_CONNECTION`, `DATABASE_URL`) and the drivers in
/// its manifests (npm, pip, Composer). MySQL means the kit's MariaDB.
fn databases(dir: &Path) -> BTreeSet<&'static str> {
    let mut found = BTreeSet::new();
    let read = |name: &str| read(dir, name).to_ascii_lowercase();
    let env = read(".env") + &read(".env.example");
    for line in env.lines().map(str::trim) {
        let value = line
            .split_once('=')
            .map_or("", |(_, v)| v.trim().trim_matches('"'));
        if line.starts_with("db_connection=") || line.starts_with("database_url=") {
            if value.starts_with("mysql") || value.starts_with("mariadb") {
                found.insert("mariadb");
            } else if value.starts_with("mongodb") {
                found.insert("mongodb");
            } else if value.starts_with("sqlite") {
                found.insert("sqlite");
            }
        }
    }
    let manifests = read("package.json")
        + &read("requirements.txt")
        + &read("pyproject.toml")
        + &read("composer.json");
    let has = |words: &[&str]| {
        words.iter().any(|w| {
            manifests.match_indices(w).any(|(i, _)| {
                let before = manifests[..i].chars().next_back();
                let after = manifests[i + w.len()..].chars().next();
                let edge = |c: Option<char>| {
                    c.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
                };
                edge(before) && edge(after)
            })
        })
    };
    if has(&[
        "mysql",
        "mysql2",
        "mariadb",
        "pymysql",
        "mysqlclient",
        "aiomysql",
    ]) {
        found.insert("mariadb");
    }
    if has(&[
        "mongodb",
        "mongoose",
        "pymongo",
        "motor",
        "mongoengine",
        "mongodb/mongodb",
    ]) {
        found.insert("mongodb");
    }
    if has(&["sqlite3", "better-sqlite3", "aiosqlite"]) {
        found.insert("sqlite");
    }
    walk_files(dir, &mut |name| {
        if [".sqlite", ".sqlite3", ".db"].contains(&ext_of(name).as_str()) {
            found.insert("sqlite");
        }
    });
    found
}

/// Ids of the kit's components the project needs, sorted. Only ids that
/// exist in the catalog are kept (all of the above are, today).
pub fn requires(dir: &Path) -> Vec<String> {
    let mut found = BTreeSet::new();
    walk_files(dir, &mut |name| {
        let lower = name.to_ascii_lowercase();
        let id = need_of_file(name)
            .or_else(|| need_of_ext(&ext_of(name)))
            .or_else(|| {
                [".csproj", ".fsproj", ".sln"]
                    .iter()
                    .any(|s| lower.ends_with(s))
                    .then_some("dotnet")
            });
        if let Some(id) = id {
            found.insert(id);
        }
        if name == "CMakeLists.txt" {
            found.insert("cc");
        }
    });
    found.extend(databases(dir));
    let catalog = crate::catalog::Catalog::get();
    found
        .into_iter()
        .filter(|id| catalog.find(id).is_some())
        .map(str::to_owned)
        .collect()
}

/// One folder (the root or a subfolder).
fn detect_folder(dir: &Path, lang: &str) -> Vec<Found> {
    let mut found: Vec<Found> = [
        node::detect(dir, lang),
        php::detect(dir, lang),
        python::detect(dir, lang),
        rust::detect(dir, lang),
        go::detect(dir, lang),
    ]
    .into_iter()
    .flatten()
    .collect();
    // The file server only when nothing else can start the project.
    if !found.iter().any(|f| f.kind != Kind::Command) {
        found.extend(site::detect(dir, lang));
    }
    found
}

pub fn detect(dir: &Path, lang: &str) -> Project {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut results = detect_folder(dir, lang);

    // Composite projects: frontend\, backend\, api\… each with its manifest.
    for sub in list_dir(dir) {
        if sub.starts_with('.') || IGNORED.contains(&sub.as_str()) || sub == "src-tauri" {
            continue;
        }
        let sub_dir = dir.join(&sub);
        if !crate::safe_path::is_dir(dir, &sub_dir)
            || !MANIFESTS.iter().any(|m| exists(&sub_dir, m))
        {
            continue;
        }
        for mut r in detect_folder(&sub_dir, lang) {
            r.platform = format!("{} ({sub})", r.platform);
            r.reason = format!("{sub}/{}", r.reason);
            for a in &mut r.actions {
                a.id = format!("{sub}/{}", a.id);
                a.label = format!("{sub}: {}", a.label);
                a.folder = Some(sub.clone());
            }
            for w in &mut r.warnings {
                *w = format!("{sub}: {w}");
            }
            results.push(r);
        }
    }

    // Stable: desktop first, then web, then commands, in detection order.
    results.sort_by_key(|r| std::cmp::Reverse(r.kind));
    let mut platforms: Vec<String> = Vec::new();
    for r in &results {
        if !platforms.contains(&r.platform) {
            platforms.push(r.platform.clone());
        }
    }
    let mut project = Project {
        name,
        platform: if results.is_empty() {
            tr(lang, "Sconosciuto", &[])
        } else {
            platforms.join(" + ")
        },
        kind: results.first().map_or(Kind::Command, |r| r.kind),
        reason: if results.is_empty() {
            tr(
                lang,
                "Nessun file noto ({0})",
                &["package.json, composer.json, pyproject.toml, Cargo.toml, index.html..."],
            )
        } else {
            let reasons: Vec<&str> = results.iter().map(|r| r.reason.as_str()).collect();
            tr(lang, "Rilevato da: {0}", &[&reasons.join(", ")])
        },
        languages: languages(dir),
        requires: requires(dir),
        actions: results.iter().flat_map(|r| r.actions.clone()).collect(),
        warnings: results.iter().flat_map(|r| r.warnings.clone()).collect(),
        custom: false,
        default: None,
    };
    overrides::apply(&mut project, dir, lang);
    // An Apache of its own needs Apache and its PHP module.
    if project.actions.iter().any(|a| a.apache_dir.is_some()) {
        for id in ["apache", "php"] {
            if !project.requires.iter().any(|r| r == id) {
                project.requires.push(id.to_owned());
            }
        }
        project.requires.sort();
    }
    if project.default.is_none() {
        project.default = project
            .actions
            .iter()
            .find(|a| a.kind != Kind::Command)
            .or(project.actions.first())
            .map(|a| a.id.clone());
    }
    project
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::kit::tests::TempDir;

    pub fn project(name: &str, files: &[(&str, &str)]) -> TempDir {
        let tmp = TempDir::new(&format!("detect-{name}"));
        for (path, text) in files {
            let p = tmp.0.join(path);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, text).unwrap();
        }
        tmp
    }

    #[test]
    fn databases_a_project_uses() {
        let laravel = project(
            "db-laravel",
            &[
                (".env", "APP_NAME=x\nDB_CONNECTION=mysql\n"),
                ("composer.json", "{}"),
            ],
        );
        assert_eq!(databases(&laravel.0), BTreeSet::from(["mariadb"]));
        let node = project(
            "db-node",
            &[(
                "package.json",
                r#"{"dependencies":{"mongoose":"^8","better-sqlite3":"^11"}}"#,
            )],
        );
        assert_eq!(databases(&node.0), BTreeSet::from(["mongodb", "sqlite"]));
        let py = project(
            "db-py",
            &[
                ("requirements.txt", "flask\npymongo==4\n"),
                ("data/app.sqlite3", ""),
            ],
        );
        assert_eq!(databases(&py.0), BTreeSet::from(["mongodb", "sqlite"]));
        // Words inside other names are not drivers.
        let none = project(
            "db-none",
            &[(
                "package.json",
                r#"{"dependencies":{"mysqlish-ui":"1","notmongodb":"1"}}"#,
            )],
        );
        assert!(databases(&none.0).is_empty());
        assert!(requires(&laravel.0).contains(&"mariadb".to_owned()));
    }

    pub fn ids(p: &Project) -> Vec<&str> {
        p.actions.iter().map(|a| a.id.as_str()).collect()
    }

    #[test]
    fn unknown_and_static() {
        let empty = project("empty", &[]);
        let p = detect(&empty.0, "it");
        assert_eq!(p.platform, "Sconosciuto");
        assert!(p.actions.is_empty() && p.default.is_none());

        let site = project("static", &[("index.html", "<h1>x</h1>"), ("style.css", "")]);
        let p = detect(&site.0, "en");
        assert_eq!(p.kind, Kind::Web);
        assert_eq!(p.actions[0].static_dir.as_deref(), Some("."));
        assert_eq!(p.default.as_deref(), Some("static"));
        assert_eq!(p.languages, ["CSS", "HTML"]);
    }

    #[test]
    fn composite_project() {
        let tmp = project(
            "composite",
            &[
                (
                    "frontend/package.json",
                    r#"{"scripts":{"dev":"vite"},"devDependencies":{"vite":"6"}}"#,
                ),
                ("backend/requirements.txt", "flask\n"),
                (
                    "backend/app.py",
                    "from flask import Flask\napp = Flask(__name__)\n",
                ),
                ("node_modules/x/package.json", "{}"),
            ],
        );
        let p = detect(&tmp.0, "it");
        assert!(
            p.platform.contains("Vite (frontend)") && p.platform.contains("Flask (backend)"),
            "{}",
            p.platform
        );
        let dev = p
            .actions
            .iter()
            .find(|a| a.id == "frontend/npm:dev")
            .unwrap();
        assert_eq!(dev.folder.as_deref(), Some("frontend"));
        assert_eq!(
            dev.command,
            "npm run dev -- --port {port} --strictPort --host 127.0.0.1"
        );
        assert!(ids(&p).contains(&"backend/py:flask"));
        assert!(p.requires.contains(&"python".to_owned()));
    }

    #[test]
    fn quoting() {
        assert_eq!(quote("public"), "public");
        assert_eq!(quote("my site"), "\"my site\"");
        assert_eq!(quote("a&b"), "\"a&b\"");
    }
}
