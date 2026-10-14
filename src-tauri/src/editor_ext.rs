//! Extensions of the integrated editor (port of nabre/atollo
//! `setup/vscodium/extensions.txt`, `app-extensions.js` and
//! `_vscodium-extensions.ps1`), chosen by the languages of the projects
//! (DECISIONS.md D-71).
//!
//! - the kit's matrix (`catalog/extensions.json`): an extension with
//!   `languages` (as `detect::LANGUAGES` names them) or `tools` (components a
//!   project requires: Deno, Bun, CMake) is a default of the projects that
//!   have them; one with neither is general, always installed (`programs`:
//!   only when one of those components is in the kit); `optional` ones are
//!   only added by hand;
//! - each project adds or removes extensions (`project_settings.rs`);
//! - the user's `data\app\extensions.txt` adds general ones, format
//!   `publisher.name [@program,...]`;
//! - each extension is offered once (`data\app\extensions-offered.txt`): if
//!   the user uninstalls it, it does not come back, unless a project adds it
//!   by hand; a failure is retried at the next start of the editor;
//! - installed extensions work in the whole editor: a language's ones only
//!   start on its files. Nothing is uninstalled or disabled per project;
//! - CodeLLDB comes from its GitHub release (Open VSX only has a bootstrap
//!   that downloads at run time), checked with GitHub's SHA-256;
//! - installed in the background after the editor starts and when a project
//!   is opened or changes its extensions (D-36), with the server's own
//!   command line, into `data\vscodium-server\extensions`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::{write_atomic, Kit};
use crate::net::{github_digest, Net};
use crate::project_settings::Extensions;
use crate::verify::Expect;

const MATRIX: &str = include_str!("../catalog/extensions.json");
pub const CODELLDB: &str = "vadimcn.vscode-lldb";

/// A row of the kit's matrix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    /// Lower case `publisher.name`.
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub programs: Vec<String>,
    #[serde(default)]
    pub optional: bool,
}

impl Extension {
    /// For every project: no language nor tool.
    pub fn general(&self) -> bool {
        self.languages.is_empty() && self.tools.is_empty()
    }

    /// A default of a project with these languages and tools.
    fn fits(&self, languages: &[String], tools: &[String]) -> bool {
        !self.optional
            && (self.languages.iter().any(|l| languages.contains(l))
                || self.tools.iter().any(|t| tools.contains(t)))
    }
}

/// The kit's matrix, in the order of the file.
pub fn matrix() -> &'static [Extension] {
    static ROWS: OnceLock<Vec<Extension>> = OnceLock::new();
    ROWS.get_or_init(|| {
        #[derive(Deserialize)]
        struct File {
            extensions: Vec<Extension>,
        }
        // A broken matrix fails the tests; at run time it means no extensions.
        serde_json::from_str::<File>(MATRIX).map_or_else(|_| Vec::new(), |f| f.extensions)
    })
}

/// `publisher.name`: letters, digits, `-` and `_` on both sides of the dot.
pub fn valid_id(id: &str) -> bool {
    let part = |p: &str| {
        !p.is_empty()
            && p.chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    };
    id.split_once('.')
        .is_some_and(|(publisher, name)| part(publisher) && part(name))
}

/// A line of the user's list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Lower case `publisher.name`.
    pub id: String,
    /// Component ids; empty = always.
    pub programs: Vec<String>,
}

/// Parses the user's list; later lines win for the same id.
pub fn parse(text: &str) -> Vec<Entry> {
    let mut out: Vec<Entry> = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        let mut words = line.split_whitespace();
        let Some(id) = words.next() else { continue };
        if !valid_id(id) {
            continue;
        }
        let programs: Vec<String> = words
            .filter_map(|w| w.strip_prefix('@'))
            .flat_map(|w| w.split(','))
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| !p.is_empty())
            .collect();
        let id = id.to_ascii_lowercase();
        out.retain(|e| e.id != id);
        out.push(Entry { id, programs });
    }
    out
}

fn offered_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("extensions-offered.txt")
}

pub fn extensions_dir(kit: &Kit) -> PathBuf {
    kit.data().join(r"vscodium-server\extensions")
}

fn read_set(path: &Path) -> BTreeSet<String> {
    fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_ascii_lowercase())
        .filter(|l| !l.is_empty())
        .collect()
}

fn mark_offered(kit: &Kit, ids: &[String]) -> Result<()> {
    let path = offered_path(kit);
    let mut set = read_set(&path);
    set.extend(ids.iter().map(|i| i.to_ascii_lowercase()));
    let text: String = set.iter().map(|i| format!("{i}\r\n")).collect();
    write_atomic(&path, text.as_bytes())?;
    Ok(())
}

/// Ids of the extensions already in the server's extensions folder.
pub fn installed(kit: &Kit) -> BTreeSet<String> {
    fs::read_dir(extensions_dir(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_ascii_lowercase();
            // `<publisher>.<name>-<version>[-<platform>]`
            let at = name
                .char_indices()
                .find(|(i, c)| {
                    *c == '-' && name[i + 1..].starts_with(|d: char| d.is_ascii_digit())
                })?
                .0;
            Some(name[..at].to_owned())
        })
        .collect()
}

/// An extension of a project as its panel shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectExtension {
    pub id: String,
    /// The matrix's name, or the id for one not in the matrix.
    pub name: String,
    /// One of its languages or tools brings it.
    pub by_default: bool,
    /// Wanted: a default not removed, or added.
    pub on: bool,
    pub installed: bool,
}

/// The extensions of a project: those of its languages and tools (on unless
/// removed), then those added by hand.
pub fn of_project(
    languages: &[String],
    tools: &[String],
    choice: &Extensions,
    installed: &BTreeSet<String>,
) -> Vec<ProjectExtension> {
    let mut out: Vec<ProjectExtension> = matrix()
        .iter()
        .filter(|e| e.fits(languages, tools))
        .map(|e| ProjectExtension {
            id: e.id.clone(),
            name: e.name.clone(),
            by_default: true,
            on: !choice.removed.contains(&e.id),
            installed: installed.contains(&e.id),
        })
        .collect();
    for id in &choice.added {
        if out.iter().any(|e| &e.id == id) {
            continue;
        }
        let name = matrix()
            .iter()
            .find(|e| &e.id == id)
            .map_or_else(|| id.clone(), |e| e.name.clone());
        out.push(ProjectExtension {
            id: id.clone(),
            name,
            by_default: false,
            on: true,
            installed: installed.contains(id),
        });
    }
    out
}

/// The project's choice cleaned up: valid lower case ids, sorted, an added
/// one never removed.
pub fn clean_choice(added: &[String], removed: &[String]) -> Extensions {
    let ids = |list: &[String]| -> BTreeSet<String> {
        list.iter()
            .map(|i| i.trim().to_ascii_lowercase())
            .filter(|i| valid_id(i))
            .collect()
    };
    let added = ids(added);
    let removed = ids(removed).difference(&added).cloned().collect();
    Extensions {
        added: added.into_iter().collect(),
        removed,
    }
}

/// What the kit wants installed: `offer` once (general ones, the user's,
/// the projects' defaults), `force` whenever missing (added by hand).
#[derive(Debug, Default)]
struct Wanted {
    offer: BTreeSet<String>,
    force: BTreeSet<String>,
}

fn wanted(kit: &Kit) -> Wanted {
    let catalog = Catalog::get();
    let present = |p: &str| catalog.find(p).is_some_and(|c| c.is_installed(kit));
    let allowed = |programs: &[String]| programs.is_empty() || programs.iter().any(|p| present(p));
    let mut w = Wanted::default();
    w.offer.extend(
        matrix()
            .iter()
            .filter(|e| e.general() && !e.optional && allowed(&e.programs))
            .map(|e| e.id.clone()),
    );
    let user = fs::read_to_string(kit.app_dir().join("extensions.txt")).unwrap_or_default();
    w.offer.extend(
        parse(&user)
            .into_iter()
            .filter(|e| allowed(&e.programs))
            .map(|e| e.id),
    );
    for dir in crate::projects::folders(kit) {
        let choice = dir
            .file_name()
            .map(|n| crate::project_settings::load(kit, &n.to_string_lossy()).extensions)
            .unwrap_or_default();
        let languages = crate::detect::languages(&dir);
        let tools = crate::detect::requires(&dir);
        w.offer.extend(
            matrix()
                .iter()
                .filter(|e| e.fits(&languages, &tools) && !choice.removed.contains(&e.id))
                .map(|e| e.id.clone()),
        );
        w.force.extend(choice.added);
    }
    w
}

/// What to install now: wanted, not installed, not in `skip` (failed in
/// this run of the editor); offered ones only the first time.
pub fn pending(kit: &Kit, skip: &BTreeSet<String>) -> Vec<String> {
    let w = wanted(kit);
    let offered = read_set(&offered_path(kit));
    let have = installed(kit);
    let mut ids: BTreeSet<String> = w
        .offer
        .into_iter()
        .filter(|id| !offered.contains(id))
        .collect();
    ids.extend(w.force);
    ids.into_iter()
        .filter(|id| !have.contains(id) && !skip.contains(id))
        .collect()
}

/// The matrix as "Linguaggi e strumenti" shows it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixInfo {
    /// The columns: languages with an extension, in `detect::LANGUAGES`
    /// order, then the tools (`[id, name]`).
    pub languages: Vec<String>,
    pub tools: Vec<(String, String)>,
    pub extensions: Vec<MatrixRow>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MatrixRow {
    #[serde(flatten)]
    pub extension: Extension,
    pub installed: bool,
}

pub fn matrix_info(kit: &Kit) -> MatrixInfo {
    let rows = matrix();
    let languages = crate::detect::LANGUAGES
        .iter()
        .map(|(name, _)| *name)
        .filter(|l| rows.iter().any(|e| e.languages.iter().any(|x| x == l)))
        .map(str::to_owned)
        .collect();
    let mut tools: Vec<(String, String)> = Vec::new();
    for t in rows.iter().flat_map(|e| &e.tools) {
        if !tools.iter().any(|(id, _)| id == t) {
            let name = Catalog::get()
                .find(t)
                .map_or(t.as_str(), |c| c.name.as_str());
            tools.push((t.clone(), name.to_owned()));
        }
    }
    let have = installed(kit);
    MatrixInfo {
        languages,
        tools,
        extensions: rows
            .iter()
            .map(|e| MatrixRow {
                installed: have.contains(&e.id),
                extension: e.clone(),
            })
            .collect(),
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ExtState {
    pub running: bool,
    pub total: usize,
    pub done: usize,
    pub failed: Vec<String>,
}

/// The job of a running install, closed by `Editor::stop`.
pub type JobSlot = Arc<Mutex<Option<jobs::Job>>>;

/// Called with the ids just installed (never empty), from the install thread.
pub type OnInstalled = Arc<dyn Fn(&[String]) + Send + Sync>;

/// The matrix's name of an extension, or its id.
pub fn name_of(id: &str) -> String {
    matrix()
        .iter()
        .find(|e| e.id == id)
        .map_or_else(|| id.to_owned(), |e| e.name.clone())
}

/// Installs the pending extensions in a background thread; `state` follows.
/// `retry`: the editor (re)started, the failed ones are tried again;
/// otherwise they wait for that. False when an install is already running:
/// the caller asks again later.
pub fn sync_in_background(
    kit: Kit,
    env: KitEnv,
    net: Option<Arc<Net>>,
    state: Arc<Mutex<ExtState>>,
    job: JobSlot,
    retry: bool,
    on_installed: Option<OnInstalled>,
) -> bool {
    let skip: BTreeSet<String> = {
        let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
        if s.running {
            return false;
        }
        if retry {
            s.failed.clear();
        }
        s.running = true;
        s.failed.iter().cloned().collect()
    };
    std::thread::spawn(move || {
        // Reads every project: off the caller's thread.
        let ids = pending(&kit, &skip);
        let result = if ids.is_empty() {
            Ok(Vec::new())
        } else {
            {
                let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
                s.total = ids.len();
                s.done = 0;
            }
            install(&kit, &env, net.as_deref(), &ids, &state, &job)
        };
        state.lock().unwrap_or_else(|e| e.into_inner()).running = false;
        match result {
            Ok(done) if !done.is_empty() => {
                if let Some(f) = on_installed {
                    f(&done);
                }
            }
            Ok(_) => {}
            Err(e) => crate::log::warn(&format!("editor extensions: {e:#}")),
        }
    });
    true
}

fn server_cli(kit: &Kit, env: &KitEnv) -> std::process::Command {
    let dir = crate::editor::server_dir(kit);
    let mut cmd = jobs::kit_command(dir.join("node.exe"), env);
    cmd.arg(dir.join(r"out\server-main.js"))
        .arg("--server-data-dir")
        .arg(kit.data().join("vscodium-server"))
        .arg("--accept-server-license-terms");
    cmd
}

fn install(
    kit: &Kit,
    env: &KitEnv,
    net: Option<&Net>,
    ids: &[String],
    state: &Mutex<ExtState>,
    job: &Mutex<Option<jobs::Job>>,
) -> Result<Vec<String>> {
    let mut gallery: Vec<String> = Vec::new();
    let mut sources: Vec<(String, PathBuf)> = Vec::new();
    let work = kit.data().join(r"tmp\setup\extensions");
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work)?;
    for id in ids {
        if id == CODELLDB {
            match net.map(|n| codelldb_vsix(kit, n, &work)) {
                Some(Ok(vsix)) => sources.push((id.clone(), vsix)),
                Some(Err(e)) => crate::log::warn(&format!("CodeLLDB: {e:#}")),
                None => {}
            }
        } else {
            gallery.push(id.clone());
        }
    }
    let mut cmd = server_cli(kit, env);
    for id in &gallery {
        cmd.arg("--install-extension").arg(id);
    }
    for (_, vsix) in &sources {
        cmd.arg("--install-extension").arg(vsix);
    }
    crate::log::info(&format!("editor extensions: installing {}", ids.join(", ")));
    let child = jobs::spawn(
        cmd.stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped()),
    )
    .context("server --install-extension")?;
    let own = jobs::Job::new()?;
    own.assign(&child)?;
    *job.lock().unwrap_or_else(|e| e.into_inner()) = Some(own);
    let out = child.wait_with_output();
    drop(job.lock().unwrap_or_else(|e| e.into_inner()).take());
    let out = out.context("server --install-extension")?;
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let _ = fs::remove_dir_all(&work);
    let ok = installed_from_output(&text);
    let succeeded: Vec<String> = ids
        .iter()
        .filter(|id| {
            ok.contains(*id)
                || (sources.iter().any(|(s, _)| s == *id) && installed(kit).contains(*id))
        })
        .cloned()
        .collect();
    mark_offered(kit, &succeeded)?;
    let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
    s.done = succeeded.len();
    let failed: Vec<String> = ids
        .iter()
        .filter(|i| !succeeded.contains(i))
        .cloned()
        .collect();
    if !failed.is_empty() {
        crate::log::warn(&format!(
            "editor extensions not installed (retried next start): {}",
            failed.join(", ")
        ));
    }
    s.failed.extend(failed);
    Ok(succeeded)
}

/// Ids from lines like `Extension 'a.b' v1.0 was successfully installed.`
/// (also "is already installed").
pub fn installed_from_output(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter(|l| l.contains("successfully installed") || l.contains("already installed"))
        .filter_map(|l| {
            let start = l.find('\'')? + 1;
            let end = l[start..].find('\'')? + start;
            Some(l[start..end].to_ascii_lowercase())
        })
        .collect()
}

/// The platform package of CodeLLDB for this kit. Extensions run in the
/// editor server's process, so the package follows the server: x64 on
/// ARM64 too while the server is `x64Only` (emulated).
fn codelldb_asset(kit: &Kit) -> String {
    let server_x64 = Catalog::get()
        .find("vscodium-server")
        .is_some_and(|c| c.x64_only);
    let arch = if server_x64 {
        "x64"
    } else {
        kit.arch().as_str()
    };
    format!("codelldb-win32-{arch}.vsix")
}

/// CodeLLDB's platform package from GitHub, verified (`Install-CodeLldb`).
fn codelldb_vsix(kit: &Kit, net: &Net, work: &Path) -> Result<PathBuf> {
    let rel = net.github_latest("vadimcn/codelldb")?;
    let name = codelldb_asset(kit);
    let asset = rel
        .assets
        .iter()
        .find(|a| a.name == name)
        .ok_or_else(|| anyhow!("no {name} in {}", rel.tag_name))?;
    let file = work.join(&name);
    let cancel = std::sync::atomic::AtomicBool::new(false);
    net.download(&asset.browser_download_url, &file, &cancel, &mut |_, _| {})?;
    crate::verify::check(&file, &Expect::Sha256(github_digest(asset)?))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn user_list_format() {
        let e = parse("# c\nMs-Python.python @python\nfoo.bar  # x\nnot-an-id\na.b.c\nvadimcn.vscode-lldb @rust,zig,cc\nfoo.bar @go\n");
        assert_eq!(e.len(), 3);
        assert_eq!(
            e[0],
            Entry {
                id: "ms-python.python".into(),
                programs: vec!["python".into()]
            }
        );
        assert_eq!(e[1].programs, vec!["rust", "zig", "cc"]);
        assert_eq!(
            e[2],
            Entry {
                id: "foo.bar".into(),
                programs: vec!["go".into()]
            },
            "later line wins"
        );
    }

    #[test]
    fn matrix_is_valid() {
        let rows = matrix();
        assert!(rows.len() >= 60, "embedded matrix loaded");
        let languages: Vec<&str> = crate::detect::LANGUAGES.iter().map(|(n, _)| *n).collect();
        let catalog = Catalog::get();
        let mut seen = BTreeSet::new();
        for e in rows {
            assert!(valid_id(&e.id), "{}", e.id);
            assert_eq!(e.id, e.id.to_ascii_lowercase(), "{}: lower case", e.id);
            assert!(seen.insert(&e.id), "{}: twice", e.id);
            assert!(!e.name.is_empty(), "{}: no name", e.id);
            for l in &e.languages {
                assert!(
                    languages.contains(&l.as_str()),
                    "{}: unknown language {l}",
                    e.id
                );
            }
            for p in e.tools.iter().chain(&e.programs) {
                assert!(catalog.find(p).is_some(), "{}: unknown component {p}", e.id);
            }
            assert!(
                e.programs.is_empty() || e.general(),
                "{}: programs only for general ones",
                e.id
            );
        }
        assert!(rows.iter().any(|e| e.id == CODELLDB));
        // Every language a project can have gets at least one extension.
        for l in &languages {
            assert!(
                rows.iter().any(|e| e.languages.iter().any(|x| x == l)),
                "no extension for {l}"
            );
        }
        let info = matrix_info(&Kit::at(std::env::temp_dir(), Arch::X64));
        assert_eq!(info.languages.len(), languages.len());
        assert_eq!(
            info.tools
                .iter()
                .map(|(id, _)| id.as_str())
                .collect::<Vec<_>>(),
            ["deno", "bun", "cmake"]
        );
    }

    #[test]
    fn ids() {
        for ok in ["a.b", "Ms-Python.python", "x_1.y-2"] {
            assert!(valid_id(ok), "{ok}");
        }
        for bad in ["", "a", ".b", "a.", "a.b.c", "a b.c", "a/b.c", r"..\x.y"] {
            assert!(!valid_id(bad), "{bad}");
        }
        let c = clean_choice(
            &[" Foo.Bar ".into(), "bad".into(), "a.b".into()],
            &["foo.bar".into(), "c.d".into(), "c.d".into()],
        );
        assert_eq!(c.added, ["a.b", "foo.bar"]);
        assert_eq!(c.removed, ["c.d"], "an added one is never removed");
    }

    #[test]
    fn a_project_gets_the_extensions_of_its_languages() {
        let langs = |l: &[&str]| l.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let have: BTreeSet<String> = ["ms-python.python".to_owned()].into();
        let none = Extensions::default();
        let list = of_project(&langs(&["Python"]), &[], &none, &have);
        let ids: Vec<&str> = list.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"ms-python.python") && ids.contains(&"charliermarsh.ruff"));
        assert!(!ids.contains(&"vue.volar") && !ids.contains(&"yzhang.markdown-all-in-one"));
        assert!(list.iter().all(|e| e.by_default && e.on));
        assert!(list[0].installed && !list[1].installed);
        assert!(
            !of_project(&langs(&["Dart"]), &[], &none, &have)
                .iter()
                .any(|e| e.id == "dart-code.flutter"),
            "optional: only by hand"
        );
        let deno = of_project(&langs(&["TypeScript"]), &langs(&["deno"]), &none, &have);
        assert!(deno.iter().any(|e| e.id == "denoland.vscode-deno"));
        let choice = clean_choice(
            &["dart-code.flutter".into(), "x.unknown".into()],
            &["charliermarsh.ruff".into()],
        );
        let list = of_project(&langs(&["Python"]), &[], &choice, &have);
        let ruff = list.iter().find(|e| e.id == "charliermarsh.ruff").unwrap();
        assert!(ruff.by_default && !ruff.on, "removed");
        let flutter = list.iter().find(|e| e.id == "dart-code.flutter").unwrap();
        assert!(!flutter.by_default && flutter.on && flutter.name == "Flutter");
        let unknown = list.iter().find(|e| e.id == "x.unknown").unwrap();
        assert_eq!(unknown.name, "x.unknown", "not in the matrix: its id");
    }

    #[test]
    fn pending_follows_projects_programs_offered_and_installed() {
        let tmp = TempDir::new("ext");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let none = BTreeSet::new();
        let has = |p: &[String], id: &str| p.iter().any(|x| x == id);
        let p = pending(&kit, &none);
        assert!(has(&p, "usernamehw.errorlens"), "general: always");
        assert!(
            !has(&p, "anthropic.claude-code"),
            "Claude Code not in the kit"
        );
        assert!(!has(&p, "ms-python.python"), "no Python project");
        assert!(!has(&p, "eamodio.gitlens"), "optional");

        let claude = Catalog::get().find("claude").unwrap().key_path(&kit);
        fs::create_dir_all(claude.parent().unwrap()).unwrap();
        fs::write(&claude, b"").unwrap();
        let app = kit.projects().join("app");
        fs::create_dir_all(&app).unwrap();
        fs::write(app.join("main.py"), b"").unwrap();
        let p = pending(&kit, &none);
        assert!(has(&p, "anthropic.claude-code") && has(&p, "ms-python.python"));
        assert!(!has(&p, "vue.volar"), "not a language of a project");

        crate::project_settings::update(&kit, "app", |s| {
            s.extensions =
                clean_choice(&["eamodio.gitlens".into()], &["charliermarsh.ruff".into()]);
        });
        let p = pending(&kit, &none);
        assert!(has(&p, "eamodio.gitlens"), "added by hand");
        assert!(!has(&p, "charliermarsh.ruff"), "removed from the project");

        mark_offered(&kit, &["ms-python.python".into(), "eamodio.gitlens".into()]).unwrap();
        fs::create_dir_all(extensions_dir(&kit).join("usernamehw.errorlens-3.26.0-universal"))
            .unwrap();
        let p = pending(&kit, &none);
        assert!(!has(&p, "ms-python.python"), "offered once");
        assert!(
            has(&p, "eamodio.gitlens"),
            "added by hand: back if uninstalled"
        );
        assert!(!has(&p, "usernamehw.errorlens"), "already installed");

        let skip: BTreeSet<String> = ["eamodio.gitlens".to_owned()].into();
        assert!(
            !has(&pending(&kit, &skip), "eamodio.gitlens"),
            "failed in this run"
        );
    }

    #[test]
    fn codelldb_follows_the_server() {
        let tmp = TempDir::new("codelldb");
        let x64 = Kit::at(tmp.0.clone(), Arch::X64);
        assert_eq!(codelldb_asset(&x64), "codelldb-win32-x64.vsix");
        let arm = Kit::at(tmp.0.clone(), Arch::Arm64);
        assert!(Catalog::get().find("vscodium-server").unwrap().x64_only);
        assert_eq!(
            codelldb_asset(&arm),
            "codelldb-win32-x64.vsix",
            "the server is x64 only: its extensions too"
        );
    }

    #[test]
    fn output_parsing() {
        let text = "Installing extension 'a.b'...\nExtension 'mikestead.dotenv' v1.0.1 was successfully installed.\nExtension 'X.Y' v2 is already installed.\nFailed Installing Extensions: c.d";
        let got = installed_from_output(text);
        assert!(got.contains("mikestead.dotenv") && got.contains("x.y"));
        assert!(!got.contains("a.b") && !got.contains("c.d"));
    }
}
