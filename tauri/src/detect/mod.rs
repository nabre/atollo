//! What a project is and how to start it (port of nabre/atollo
//! `core/detectors`): platform, languages, the kit's programs it needs and
//! the actions of the Projects page. Every action has:
//! - `command`: run by `cmd.exe /d /s /c` in the project folder, with the
//!   kit's environment (as in nabre/atollo: `npm.ps1` in PowerShell would
//!   swallow the `--` of `npm run dev -- --port`);
//! - `type`: `web` (a server on its own port: `{port}` in the command, `PORT`
//!   in the environment, preview in the browser), `desktop` (an app with its
//!   own window: Electron, Tauri), `command` (ends by itself: build, test);
//! - `folder`: a subfolder to run it in (composite projects);
//! - `env`, `url`: optional, `{port}` replaced at start;
//! - `static`: a folder of the project served by the exe's file server
//!   (`Atollo.exe serve`), instead of a command;
//! - `apache`: a folder of the project served by an Apache of its own
//!   (`projects/apache.rs`), instead of a command; `nginx` and `caddy` the
//!   same with Nginx or Caddy (`projects/caddy.rs`).
//!
//! `panel.json` in the project (old name `pannello.json`) corrects or
//! replaces what was detected (`overrides.rs`). One file per platform.

mod container;
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
    remove_action, save_action, set_default_action, set_site_with_options, ActionEdit, SiteKind,
    SiteOptions, SITE_ACTION,
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
    /// A folder served by the project's own Nginx.
    #[serde(rename = "nginx")]
    pub nginx_dir: Option<String>,
    /// A folder served by Caddy, its own or a pool's (D-111).
    #[serde(rename = "caddy")]
    pub caddy_dir: Option<String>,
    /// Web server profile id, or `auto`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    /// RFC 6761 `.localhost` domain aliases.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub domains: Vec<String>,
    /// Portable relative path aliases (`/url` -> `relative/folder` or `@other/folder`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub aliases: BTreeMap<String, String>,
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
            nginx_dir: None,
            caddy_dir: None,
            server: None,
            domains: Vec::new(),
            aliases: BTreeMap::new(),
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

    /// A folder of the project served by its own Nginx.
    fn nginx(id: &str, label: &str, folder: &str) -> Action {
        Action {
            nginx_dir: Some(folder.to_owned()),
            ..Action::new(id, label, "", Kind::Web)
        }
    }

    /// A folder of the project served by Caddy.
    fn caddy(id: &str, label: &str, folder: &str) -> Action {
        Action {
            caddy_dir: Some(folder.to_owned()),
            ..Action::new(id, label, "", Kind::Web)
        }
    }

    /// Served by a web server of the kit (Apache, Nginx, Caddy): a site
    /// that can go in a shared pool.
    pub fn web_server_dir(&self) -> Option<&str> {
        self.apache_dir
            .as_deref()
            .or(self.nginx_dir.as_deref())
            .or(self.caddy_dir.as_deref())
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
    /// Lines of some of them, from the images of Compose and Dockerfile.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub lines: BTreeMap<String, String>,
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
    crate::os::safe_path::exists(dir, &dir.join(&*native(rel)))
}

fn read(dir: &Path, rel: &str) -> String {
    crate::os::safe_path::read_text(dir, &dir.join(&*native(rel))).unwrap_or_default()
}

/// A relative path written with `\` (as the detection writes them), with
/// the system's separator: on macOS and Linux `\` is a character of a name.
fn native(rel: &str) -> std::borrow::Cow<'_, str> {
    if cfg!(windows) {
        rel.into()
    } else {
        rel.replace('\\', "/").into()
    }
}

fn read_json(dir: &Path, rel: &str) -> Option<serde_json::Value> {
    serde_json::from_str(&read(dir, rel)).ok()
}

fn list_dir(dir: &Path) -> Vec<String> {
    if crate::os::safe_path::is_link(dir) {
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
#[cfg(windows)]
fn quote(s: &str) -> String {
    if s.chars()
        .any(|c| c.is_whitespace() || "&()^,;=".contains(c))
    {
        format!("\"{s}\"")
    } else {
        s.to_owned()
    }
}

/// Quotes a path for `/bin/sh` when it holds anything but plain characters:
/// in single quotes, where `$(…)`, backticks and `;` of a folder name in a
/// cloned repository stay text.
#[cfg(not(windows))]
fn quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/+@".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

/// The kit's router of `php -S` (`runtime\php-router.php`, written to
/// `data\kit\`): 403 for hidden files and `.inc` (SEC-V3-02). The command
/// runs through `cmd.exe` on Windows and `/bin/sh` elsewhere, each with its
/// own way to read `CP_DATA`.
#[cfg(windows)]
const PHP_ROUTER: &str = r#""%CP_DATA%\kit\php-router.php""#;
#[cfg(not(windows))]
const PHP_ROUTER: &str = r#""$CP_DATA/kit/php-router.php""#;

/// `php -S` on the folder `root` of the project, always through the kit's
/// router. A project's own `router.php` (in `root`, else in the project
/// folder) runs after the kit's checks: `php -S` takes one router only, so
/// the kit's includes it, given as `-d atollo.router=<path>` relative to
/// `root` (hidden files and `.inc` stay refused, D-97).
fn php_server(dir: &Path, root: &str) -> String {
    let depth = root
        .split(['\\', '/'])
        .filter(|s| !s.is_empty() && *s != ".")
        .count();
    let in_root = format!(r"{root}\router.php");
    let own = if exists(dir, in_root.trim_start_matches(r".\")) {
        Some("router.php".to_owned())
    } else if exists(dir, "router.php") {
        Some(format!(r"{}router.php", r"..\".repeat(depth)))
    } else {
        None
    };
    let own = own
        .map(|f| format!("-d {} ", quote(&format!("atollo.router={}", native(&f)))))
        .unwrap_or_default();
    format!(
        "php {own}-S 127.0.0.1:{{port}} -t {} {PHP_ROUTER}",
        quote(&native(root))
    )
}

/// Files that ask for Nginx, in the project folder or in the site's.
const NGINX_FILES: [&str; 2] = ["nginx.conf", ".nginx"];

/// A `nginx.conf` or `.nginx` in the project folder or in `root`.
fn has_nginx(dir: &Path, root: &str) -> bool {
    NGINX_FILES
        .iter()
        .any(|f| exists(dir, f) || (root != "." && exists(dir, &format!(r"{root}\{f}"))))
}

/// One walk of the project: 3 levels, at most 3000 files, no IGNORED folders.
fn walk_files(dir: &Path, on_file: &mut dyn FnMut(&str)) {
    fn walk(d: &Path, depth: u32, seen: &mut u32, on_file: &mut dyn FnMut(&str)) {
        // Below the root, links are not entered (a DirEntry's type is the
        // link's own); the root itself is checked here.
        if depth > 3 || *seen > 3000 || (depth == 0 && crate::os::safe_path::is_link(d)) {
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
/// matrix of editor extensions shows them (`editor/ext.rs`).
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
        "AndroidManifest.xml" => "android",
        "nginx.conf" => "nginx",
        _ => return None,
    })
}

/// The database servers a project uses (D-73), from its root: SQLite files,
/// `.env` (Laravel's `DB_CONNECTION`, `DATABASE_URL`) and the drivers in
/// its manifests (npm, pip, Composer). MySQL and MariaDB mean the kit's
/// MySQL (D-117).
fn databases(dir: &Path) -> BTreeSet<&'static str> {
    let mut found = BTreeSet::new();
    if exists(dir, "wp-config.php")
        || exists(dir, "wp-settings.php")
        || exists(dir, r"wp-includes\version.php")
    {
        found.insert("mysql");
    }
    let read = |name: &str| read(dir, name).to_ascii_lowercase();
    let env = read(".env") + &read(".env.example");
    for line in env.lines().map(str::trim) {
        let value = line
            .split_once('=')
            .map_or("", |(_, v)| v.trim().trim_matches('"'));
        if line.starts_with("db_connection=") || line.starts_with("database_url=") {
            if value.starts_with("mysql") || value.starts_with("mariadb") {
                found.insert("mysql");
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
        found.insert("mysql");
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

/// `.env` (or `.env.example` without one) says `DB_CONNECTION=mariadb`.
pub(crate) fn env_wants_mariadb(dir: &Path) -> bool {
    let env = match read(dir, ".env") {
        t if t.is_empty() => read(dir, ".env.example"),
        t => t,
    };
    env.lines().map(str::trim).any(|l| {
        l.split_once('=').is_some_and(|(k, v)| {
            k.trim() == "DB_CONNECTION"
                && v.trim()
                    .trim_matches(['"', '\''])
                    .eq_ignore_ascii_case("mariadb")
        })
    })
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
    // A `.htaccess` is PHP; Apache where the catalog has it and for rules
    // of its own (Caddy serves the stock ones, detect/php.rs).
    if php::HTACCESS.iter().any(|f| exists(dir, f)) {
        found.insert("php");
        if php::apache_offered(crate::kit::Target::detect(), &php::htaccess_texts(dir, ".")) {
            found.insert("apache");
        }
    }
    // The walk skips hidden names: `.nginx` is looked for here.
    if has_nginx(dir, ".") {
        found.insert("nginx");
    }
    found.extend(container::servers(dir).into_keys());
    found.extend(databases(dir));
    let catalog = crate::setup::catalog::Catalog::get();
    found
        .into_iter()
        .filter(|id| catalog.find(id).is_some())
        .map(str::to_owned)
        .collect()
}

/// The lines of the kit's components that the project's containers name
/// (`php:8.3-apache` → `php` = `8.3`).
pub fn lines(dir: &Path) -> BTreeMap<String, String> {
    container::servers(dir)
        .into_iter()
        .filter_map(|(id, line)| Some((id.to_owned(), line?)))
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
        if !crate::os::safe_path::is_dir(dir, &sub_dir)
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
        lines: lines(dir),
        actions: results.iter().flat_map(|r| r.actions.clone()).collect(),
        warnings: results.iter().flat_map(|r| r.warnings.clone()).collect(),
        custom: false,
        default: None,
    };
    // MariaDB's driver of Laravel on the kit's MySQL (D-117): its grammar
    // writes types MySQL does not have (a native `uuid` column).
    if env_wants_mariadb(dir) {
        project.warnings.push(tr(
            lang,
            "DB_CONNECTION=mariadb nel file .env: il server SQL del kit è MySQL. Usa DB_CONNECTION=mysql: il driver mariadb di Laravel scrive tipi che MySQL non ha.",
            &[],
        ));
    }
    overrides::apply(&mut project, dir, lang);
    if project.default.is_none() {
        project.default = project
            .actions
            .iter()
            .find(|a| a.kind != Kind::Command)
            .or(project.actions.first())
            .map(|a| a.id.clone());
    }
    // An Apache of its own needs Apache and its PHP module, a Nginx needs
    // Nginx, a Caddy Caddy and PHP: when it is the main action or written
    // in panel.json, not when it is only proposed beside `php -S` or
    // `artisan serve`.
    let used = |a: &&Action| a.custom || project.default.as_deref() == Some(a.id.as_str());
    let mut needed: Vec<&str> = Vec::new();
    if project
        .actions
        .iter()
        .filter(used)
        .any(|a| a.apache_dir.is_some())
    {
        needed.extend(["apache", "php"]);
    }
    if project
        .actions
        .iter()
        .filter(used)
        .any(|a| a.nginx_dir.is_some())
    {
        needed.push("nginx");
    }
    if project
        .actions
        .iter()
        .filter(used)
        .any(|a| a.caddy_dir.is_some())
    {
        needed.extend(["caddy", "php"]);
    }
    // A CMS needs the kit's SQL server, MySQL (D-93, D-117).
    if php::CMS.iter().any(|c| project.platform.contains(c)) {
        needed.push("mysql");
    }
    for id in needed {
        if !project.requires.iter().any(|r| r == id) {
            project.requires.push(id.to_owned());
        }
    }
    project.requires.sort();
    project
}

#[cfg(test)]
#[path = "../../../test/rust/detect/mod.rs"]
pub(crate) mod tests;
