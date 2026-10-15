//! Projects: the list and its operations (this file), the kit's settings of
//! a project (`settings`, `dev-settings.json`), its MCP servers (`mcp`) and
//! editor settings (`vscode`), new projects from frameworks (`frameworks`),
//! the actions started by the kit (`run`), the static file server (`serve`)
//! and a project's own Apache (`apache`). What a project is: `detect`.
//!
//! The projects in `projects\` (port of nabre/dev-kit `core/projects.js`):
//! list with what each one is, new empty project, delete an empty one (with
//! its databases on the kit's servers, PLAN-DATA.md WP-5), clone from GitHub
//! or a Git URL.

pub mod alias;
pub mod apache;
pub mod frameworks;
pub mod mcp;
pub mod migrate;
pub mod nginx;
pub mod run;
pub mod serve;
pub mod settings;
pub mod state;
pub mod vscode;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::detect::{self, Project};
use crate::env::KitEnv;
use crate::kit::{validate_project_name, Kit};
use crate::setup::catalog::Catalog;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectInfo {
    /// Folder name: the project's id.
    pub id: String,
    /// Seconds since 1970.
    pub modified: u64,
    /// An empty folder (can be deleted from the app).
    pub empty: bool,
    /// Components it needs that are not installed.
    pub missing: Vec<String>,
    /// The kit's settings of the project (`projects/settings.rs`).
    pub settings: settings::ProjectSettings,
    /// Editor extensions of its languages and added by hand (`editor/ext.rs`).
    pub extensions: Vec<crate::editor::ext::ProjectExtension>,
    #[serde(flatten)]
    pub project: Project,
}

fn modified(path: &Path) -> u64 {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs())
}

/// Folders in `projects\`, not hidden, not links.
pub fn folders(kit: &Kit) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(kit.projects())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        // A link or junction is never a project: reading through it could
        // reach a network share (D-77).
        .filter(|e| {
            e.metadata()
                .is_ok_and(|m| m.is_dir() && !crate::os::safe_path::is_reparse(&m))
        })
        .map(|e| e.path())
        .collect();
    out.sort();
    out
}

pub fn info(kit: &Kit, dir: &Path, lang: &str) -> ProjectInfo {
    let project = detect::detect(dir, lang);
    let catalog = Catalog::get();
    // What can be added from "Linguaggi e strumenti": not R on ARM64.
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let missing = project
        .requires
        .iter()
        .filter(|id| {
            catalog
                .find(id)
                .is_some_and(|c| !c.is_installed(kit) && !(c.no_arm64 && arm64))
        })
        .cloned()
        .collect();
    let empty = fs::symlink_metadata(dir).is_ok_and(|m| m.is_dir())
        && fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none());
    let id = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let settings = settings::load(kit, &id);
    let extensions = crate::editor::ext::of_project(
        &project.languages,
        &project.requires,
        &settings.extensions,
        &crate::editor::ext::installed(kit),
    );
    ProjectInfo {
        settings,
        extensions,
        id,
        modified: modified(dir),
        empty,
        missing,
        project,
    }
}

/// Every project, most recently opened or changed first.
pub fn list(kit: &Kit, lang: &str) -> Vec<ProjectInfo> {
    let mut out: Vec<ProjectInfo> = folders(kit).iter().map(|d| info(kit, d, lang)).collect();
    let recent = |p: &ProjectInfo| p.modified.max(p.settings.last_opened.unwrap_or(0));
    out.sort_by(|a, b| recent(b).cmp(&recent(a)).then_with(|| a.id.cmp(&b.id)));
    out
}

fn tr(lang: &str, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang, source, args)
}

fn check_name(name: &str, lang: &str) -> Result<()> {
    if validate_project_name(name).is_err() {
        bail!(
            "{}",
            tr(
                lang,
                "Nome non valido: niente {0}, punto iniziale o finale, nomi riservati di Windows.",
                &["\\ / : * ? \" < > |"]
            )
        );
    }
    Ok(())
}

/// The folder of a new project: a valid name, not there yet.
pub fn new_dir(kit: &Kit, name: &str, lang: &str) -> Result<PathBuf> {
    check_name(name, lang)?;
    let dir = kit.projects().join(name);
    if dir.exists() {
        bail!(
            "{}",
            tr(lang, "Esiste già una cartella {0} in projects.", &[name])
        );
    }
    Ok(dir)
}

/// A new, empty project folder.
pub fn create(kit: &Kit, name: &str, lang: &str) -> Result<PathBuf> {
    let dir = new_dir(kit, name.trim(), lang)?;
    fs::create_dir_all(kit.projects())?;
    fs::create_dir(&dir).with_context(|| dir.display().to_string())?;
    Ok(dir)
}

/// The databases of the kit's servers, as a project's deletion sees them
/// (PLAN-DATA.md WP-5): `db::services::ProjectDrops` for the real servers,
/// a fake in the tests.
pub trait ProjectDatabases {
    /// The server of `engine` is installed (otherwise its database is left).
    fn installed(&self, engine: &str) -> bool;
    /// Drops the database and user `database` on `engine` (the project's,
    /// resolved and checked by [`delete`]), starting the server when it is
    /// stopped.
    fn drop_db(&self, engine: &str, database: &str) -> Result<()>;
}

/// A database of the project that its deletion drops.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedDb {
    /// `mariadb` or `mongodb`.
    pub engine: String,
    /// Name of the database and of its user.
    pub database: String,
    /// Not installed: skipped, the database stays in the server's data.
    pub installed: bool,
}

/// What deleting a project takes away (PLAN-DATA.md T5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeletePlan {
    /// The folder in `projects\` (not a link).
    pub folder_exists: bool,
    /// Only an empty folder is deleted from the app.
    pub folder_empty: bool,
    /// The databases with a password of the kit, dropped first.
    pub databases: Vec<PlannedDb>,
    /// The copies of its databases (`private\backups`) and their bytes.
    pub backups: usize,
    pub backups_size: u64,
}

/// The project's folder in `projects\`: a folder (not a link), and whether
/// it is empty; `None` when missing.
fn folder_state(dir: &Path) -> Option<(bool, bool)> {
    let meta = fs::symlink_metadata(dir).ok()?;
    let folder = meta.is_dir() && !crate::os::safe_path::is_reparse(&meta);
    Some((
        folder,
        folder && fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none()),
    ))
}

/// What [`delete`] would take away. Reads only: nothing is created. Fails
/// when the passwords or names of the databases cannot be read or resolved
/// (deleting would then need `force_without_db`).
pub fn delete_plan(kit: &Kit, name: &str, lang: &str) -> Result<DeletePlan> {
    check_name(name, lang)?;
    let (folder_exists, folder_empty) =
        folder_state(&kit.projects().join(name)).unwrap_or((false, false));
    let databases = databases_of(kit, name)?
        .into_iter()
        .map(|(engine, database)| PlannedDb {
            installed: Catalog::get()
                .find(&engine)
                .is_some_and(|c| c.is_installed(kit)),
            engine,
            database,
        })
        .collect();
    let (backups, backups_size) = state::ProjectState::of(kit, name)
        .map(|s| s.dir().join(r"private\backups"))
        .and_then(|dir| fs::read_dir(dir).ok())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file() && !crate::os::safe_path::is_reparse(m))
        .fold((0, 0), |(n, size), m| (n + 1, size + m.len()));
    Ok(DeletePlan {
        folder_exists,
        folder_empty,
        databases,
        backups,
        backups_size,
    })
}

/// The kit's servers where the project has a database (a stored password).
fn engines_of(kit: &Kit, name: &str) -> Result<Vec<String>> {
    let passwords = crate::db::auth::try_project_passwords(kit, name)
        .context("the passwords of the project's databases")?;
    Ok(passwords
        .into_keys()
        .filter(|e| crate::db::services::SERVERS.contains(&e.as_str()))
        .collect())
}

/// The project's databases by engine, each with its name
/// (`db::auth::project_db_name`: stored, or the rule of the time it was
/// created with).
fn databases_of(kit: &Kit, name: &str) -> Result<Vec<(String, String)>> {
    engines_of(kit, name)?
        .into_iter()
        .map(|engine| {
            let database = crate::db::auth::project_db_name(kit, name, &engine)?;
            Ok((engine, database))
        })
        .collect()
}

/// Deletes a project (PLAN-DATA.md WP-5): its databases and users on the
/// kit's servers first, then its folder, which must be empty (never a link,
/// never files: `remove_dir` fails by itself if something appeared
/// meanwhile), then everything the kit keeps for it in `data\projects`.
///
/// A drop that fails stops everything, nothing deleted; so does a name
/// that another project of the kit uses as well (checked for every engine
/// before the first drop: its database would go with this one). With
/// `force_without_db` no server is touched: the folders go anyway and the
/// log names the databases and users left (never a password). A server
/// not installed is skipped, its database named in the log as well.
pub fn delete(
    kit: &Kit,
    name: &str,
    lang: &str,
    dbs: &dyn ProjectDatabases,
    force_without_db: bool,
) -> Result<()> {
    check_name(name, lang)?;
    let dir = kit.projects().join(name);
    check_deletable(&dir, name, lang)?;
    let databases = match databases_of(kit, name) {
        Ok(databases) => databases,
        Err(e) if force_without_db => {
            crate::log::warn(&format!(
                "{name}: deleted without its databases, unknown ({e:#}): \
                 its databases and users may be left on the servers"
            ));
            Vec::new()
        }
        Err(e) => return Err(e),
    };
    if !force_without_db {
        for (engine, database) in databases.iter().filter(|(e, _)| dbs.installed(e)) {
            crate::db::auth::check_name_free(kit, name, engine, database)?;
        }
    }
    let mut left = Vec::new();
    for (engine, database) in databases {
        if force_without_db {
            left.push(format!("{database} on {engine}"));
        } else if !dbs.installed(&engine) {
            crate::log::info(&format!("{name}: {engine} not installed, skipped"));
            left.push(format!("{database} on {engine}"));
        } else {
            dbs.drop_db(&engine, &database)
                .with_context(|| format!("{engine}: dropping the database {database}"))?;
        }
    }
    if !left.is_empty() {
        crate::log::warn(&format!(
            "{name}: databases and users left: {}",
            left.join(", ")
        ));
    }
    fs::remove_dir(&dir).with_context(|| dir.display().to_string())?;
    let removed = settings::remove(kit, name);
    // The users are gone (or given up): their passwords go too, even when
    // another file of the folder stayed.
    if let Err(e) = crate::db::auth::drop_project(kit, name) {
        crate::log::warn(&format!("{name}: passwords: {e:#}"));
    }
    removed.with_context(|| format!("data\\projects\\{name}"))?;
    Ok(())
}

/// Only an empty folder, never a link, is deleted from the app.
fn check_deletable(dir: &Path, name: &str, lang: &str) -> Result<()> {
    let meta = fs::symlink_metadata(dir).with_context(|| name.to_owned())?;
    if !meta.is_dir() || crate::os::safe_path::is_reparse(&meta) {
        bail!(
            "{}",
            tr(
                lang,
                "{0} è un collegamento: non viene eliminato da qui.",
                &[name]
            )
        );
    }
    if fs::read_dir(dir)?.next().is_some() {
        bail!(
            "{}",
            tr(
                lang,
                "La cartella {0} non è vuota: il progetto non viene eliminato.",
                &[name]
            )
        );
    }
    Ok(())
}

/// What to clone: `owner/repo` (GitHub) or an `https://` Git URL.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    GitHub(String),
    Url(String),
}

impl Source {
    pub fn parse(text: &str) -> Option<Source> {
        let t = text.trim();
        let part = |s: &str| {
            !s.is_empty()
                && !s.starts_with(['-', '.'])
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
        };
        if let Some((owner, repo)) = t.split_once('/') {
            if !t.contains(':') && part(owner) && part(repo.trim_end_matches(".git")) {
                return Some(Source::GitHub(format!(
                    "{owner}/{}",
                    repo.trim_end_matches(".git")
                )));
            }
        }
        let ok_url = t.starts_with("https://")
            && t.len() > 8
            && !t
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || "\"'`<>|^%".contains(c));
        ok_url.then(|| Source::Url(t.to_owned()))
    }

    /// The folder name the clone gets by default.
    pub fn default_name(&self) -> String {
        let s = match self {
            Source::GitHub(r) | Source::Url(r) => r.trim_end_matches('/'),
        };
        s.rsplit('/')
            .next()
            .unwrap_or_default()
            .trim_end_matches(".git")
            .to_owned()
    }

    fn url(&self) -> String {
        match self {
            Source::GitHub(r) => format!("https://github.com/{r}.git"),
            Source::Url(u) => u.clone(),
        }
    }
}

/// Clones `source` into `projects\<name>` with the kit's Git (and its
/// credential helper: the kit's GitHub account, `github/`). Blocking.
/// `token`: the GitHub account's token, sent to github.com only as an HTTP
/// header in the environment (`GIT_CONFIG_*`), so it is neither on the
/// command line nor saved in `.git\config`.
pub fn clone(
    kit: &Kit,
    env: &KitEnv,
    source: &str,
    name: Option<&str>,
    lang: &str,
    token: Option<&str>,
) -> Result<PathBuf> {
    let Some(src) = Source::parse(source) else {
        bail!(
            "{}",
            tr(
                lang,
                "Scrivi owner/repository di GitHub o un indirizzo https:// di Git.",
                &[]
            )
        );
    };
    let name = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map_or_else(|| src.default_name(), str::to_owned);
    check_name(&name, lang)?;
    let dir = kit.projects().join(&name);
    if dir.exists() {
        bail!(
            "{}",
            tr(lang, "Esiste già una cartella {0} in projects.", &[&name])
        );
    }
    let git = kit.apps().join(r"git\cmd\git.exe");
    if !git.is_file() {
        bail!("{}", tr(lang, "Git del kit non è installato.", &[]));
    }
    fs::create_dir_all(kit.projects())?;
    let mut cmd = crate::os::jobs::kit_command(&git, env);
    cmd.args(["clone", "--"])
        .arg(src.url())
        .arg(&dir)
        .env("GIT_TERMINAL_PROMPT", "0");
    if let Some(token) = token.filter(|_| matches!(src, Source::GitHub(_))) {
        use base64::Engine;
        let basic =
            base64::engine::general_purpose::STANDARD.encode(format!("x-access-token:{token}"));
        cmd.env("GIT_CONFIG_COUNT", "1")
            .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
            .env(
                "GIT_CONFIG_VALUE_0",
                format!("AUTHORIZATION: basic {basic}"),
            );
    }
    let out = cmd.output().context("git clone")?;
    if !out.status.success() {
        let _ = fs::remove_dir_all(&dir);
        let err = String::from_utf8_lossy(&out.stderr);
        let last = err
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("git clone");
        bail!("{}", last.trim());
    }
    Ok(dir)
}

/// Git state of a project for its page, read from the files of `.git`
/// without running Git: a cloned repository's own `.git/config` can make
/// `git status` run commands (`core.fsmonitor`, `clean` filters), and the
/// page is shown without asking (security review, 14/10/2026). Commits,
/// pushes and the list of changes stay in the editor and the agents.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    /// None: detached HEAD.
    pub branch: Option<String>,
    /// `origin/main` when the branch follows a remote one.
    pub upstream: Option<String>,
    /// The branch and its remote point to the same commit (as of the last
    /// fetch); None without an upstream.
    pub synced: Option<bool>,
    pub remote: Option<String>,
}

use crate::os::safe_path::{lexical, no_links};

/// A file of the repository, read only when it is inside `root` and no link
/// leads to it (`safe_path`).
fn read_plain(root: &Path, path: &Path) -> Option<String> {
    crate::os::safe_path::read_text(root, path)
}

/// `.git` of `dir`: the folder, or the one a `.git` file points to
/// (worktrees, submodules), only inside `root` (the kit's projects). A
/// cloned repository chooses that text: a `\\server\share` there would make
/// Windows send the user's NTLM credentials to that server as soon as the
/// path is opened (security review, 14/10/2026), so it is checked as text
/// before anything is read, and no link may lead there.
fn git_dir(dir: &Path, root: &Path) -> Option<PathBuf> {
    let dot = lexical(&dir.join(".git"))?;
    if !dot.starts_with(root) || !no_links(root, &dot) {
        return None;
    }
    if crate::os::safe_path::is_dir(root, &dot) {
        return Some(dot);
    }
    let text = read_plain(root, &dot)?;
    let target = text.trim().strip_prefix("gitdir:")?.trim();
    // Relative, or `X:\...`: no UNC, device (`\\?\`, `\\.\`) or stream path.
    let drive = target.len() > 2
        && target.as_bytes()[0].is_ascii_alphabetic()
        && target.as_bytes()[1] == b':'
        && matches!(target.as_bytes()[2], b'\\' | b'/');
    let colons = target.matches(':').count();
    if target.starts_with(['\\', '/']) || colons > usize::from(drive) || (colons == 1 && !drive) {
        return None;
    }
    let path = lexical(&dir.join(target))?;
    if !path.starts_with(root)
        || path.to_string_lossy().starts_with(r"\\")
        || !no_links(root, &path)
    {
        return None;
    }
    crate::os::safe_path::is_dir(root, &path).then_some(path)
}

/// `key` of section `[name "sub"]` in a Git config file (simple form only).
fn config_value(config: &str, section: &str, key: &str) -> Option<String> {
    let mut inside = false;
    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line.eq_ignore_ascii_case(section);
            continue;
        }
        if inside {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(v.trim().trim_matches('"').to_owned());
                }
            }
        }
    }
    None
}

/// The commit of `reference` (`refs/heads/main`), loose or packed.
fn ref_commit(root: &Path, git: &Path, reference: &str) -> Option<String> {
    if let Some(text) = read_plain(root, &git.join(reference)) {
        return Some(text.trim().to_owned());
    }
    read_plain(root, &git.join("packed-refs"))?
        .lines()
        .find_map(|l| {
            let (sha, name) = l.split_once(' ')?;
            (name.trim() == reference).then(|| sha.to_owned())
        })
}

pub fn git_info(dir: &Path, root: &Path) -> Option<GitInfo> {
    let root = &lexical(root)?;
    let git = git_dir(dir, root)?;
    let head = read_plain(root, &git.join("HEAD"))?;
    let branch = head
        .trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned);
    let config = read_plain(root, &git.join("config")).unwrap_or_default();
    let remote = config_value(&config, "[remote \"origin\"]", "url");
    let (upstream, synced) = match &branch {
        Some(b) => {
            let section = format!("[branch \"{b}\"]");
            let r = config_value(&config, &section, "remote");
            let merge = config_value(&config, &section, "merge");
            match (
                r,
                merge.as_deref().and_then(|m| m.strip_prefix("refs/heads/")),
            ) {
                (Some(r), Some(m)) => {
                    let local = ref_commit(root, &git, &format!("refs/heads/{b}"));
                    let theirs = ref_commit(root, &git, &format!("refs/remotes/{r}/{m}"));
                    let synced = match (local, theirs) {
                        (Some(a), Some(b)) => Some(a == b),
                        _ => None,
                    };
                    (Some(format!("{r}/{m}")), synced)
                }
                _ => (None, None),
            }
        }
        None => (None, None),
    };
    Some(GitInfo {
        branch,
        upstream,
        synced,
        remote,
    })
}

/// What a project's page shows besides its detection.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatus {
    /// None: not a Git repository.
    pub git: Option<GitInfo>,
    /// Instruction files for the agents found at its root.
    pub instructions: Vec<String>,
}

pub fn status(dir: &Path, root: &Path) -> ProjectStatus {
    let instructions = [
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        ".github/copilot-instructions.md",
    ]
    .iter()
    .filter(|f| crate::os::safe_path::is_file(dir, &dir.join(f)))
    .map(|f| (*f).to_owned())
    .collect();
    ProjectStatus {
        git: git_info(dir, root),
        instructions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    /// A junction in projects\ is not a project, and a junction inside a
    /// project is not read by the detection (D-77).
    #[test]
    fn junctions_are_not_projects_nor_read() {
        let tmp = TempDir::new("proj-links");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let real = kit.projects().join("real");
        fs::create_dir_all(&real).unwrap();
        fs::write(real.join("index.html"), "<h1>x</h1>").unwrap();
        let outside = tmp.0.join("outside");
        fs::create_dir_all(&outside).unwrap();
        fs::write(
            outside.join("package.json"),
            r#"{"dependencies":{"mongoose":"8"}}"#,
        )
        .unwrap();
        fs::write(outside.join(".env"), "DB_CONNECTION=mysql\n").unwrap();
        crate::os::safe_path::tests::junction(&kit.projects().join("linked"), &outside);
        let names: Vec<String> = folders(&kit)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["real"]);
        // Inside a project: a junction named like a manifest folder.
        crate::os::safe_path::tests::junction(&real.join("server"), &outside);
        let req = crate::detect::requires(&real);
        assert!(
            !req.iter().any(|r| r == "mongodb" || r == "mariadb"),
            "{req:?}"
        );
        let p = crate::detect::detect(&real, "it");
        assert!(!p.reason.contains("server"), "{}", p.reason);
        // The detection of the linked folder itself finds nothing.
        assert!(crate::detect::requires(&kit.projects().join("linked")).is_empty());
    }

    #[test]
    fn git_state_from_files() {
        let tmp = TempDir::new("gitinfo");
        let d = tmp.0.join("p");
        let g = d.join(".git");
        let heads = g.join("refs").join("heads");
        let remotes = g.join("refs").join("remotes").join("origin");
        fs::create_dir_all(&heads).unwrap();
        fs::create_dir_all(&remotes).unwrap();
        fs::write(g.join("HEAD"), "ref: refs/heads/main\n").unwrap();
        // A command in the repository's config must never run: it is only read.
        fs::write(
            g.join("config"),
            "[core]\n\tfsmonitor = calc.exe\n[remote \"origin\"]\n\turl = https://github.com/a/b.git\n\
             [branch \"main\"]\n\tremote = origin\n\tmerge = refs/heads/main\n",
        )
        .unwrap();
        fs::write(heads.join("main"), "aaa\n").unwrap();
        fs::write(
            g.join("packed-refs"),
            "# pack-refs\nbbb refs/remotes/origin/main\n",
        )
        .unwrap();
        let info = git_info(&d, &tmp.0).unwrap();
        assert_eq!(info.branch.as_deref(), Some("main"));
        assert_eq!(info.upstream.as_deref(), Some("origin/main"));
        assert_eq!(info.remote.as_deref(), Some("https://github.com/a/b.git"));
        assert_eq!(info.synced, Some(false));
        fs::write(remotes.join("main"), "aaa\n").unwrap();
        assert_eq!(git_info(&d, &tmp.0).unwrap().synced, Some(true));
        fs::write(g.join("HEAD"), "0123abcd\n").unwrap();
        assert_eq!(git_info(&d, &tmp.0).unwrap().branch, None);
        assert!(git_info(&tmp.0, &tmp.0).is_none(), "not a repository");
        // A .git file may point only inside the projects folder, never to a share.
        let wt = tmp.0.join("wt");
        fs::create_dir_all(&wt).unwrap();
        fs::write(
            wt.join(".git"),
            "gitdir: ../p/.git
",
        )
        .unwrap();
        assert!(git_info(&wt, &tmp.0).is_some(), "inside: followed");
        for bad in [
            r"gitdir: \\attacker\share\x",
            "gitdir: //attacker/share/x",
            r"gitdir: \\?\C:\Windows",
            "gitdir: ../../../Windows",
            "gitdir: C:x",
            "gitdir: ../p/.git:stream",
        ] {
            fs::write(wt.join(".git"), bad).unwrap();
            assert!(git_info(&wt, &tmp.0).is_none(), "{bad}");
        }
        // A junction (no admin rights needed) is never followed, whatever
        // it points to: here the real repository of `p`.
        let linked = tmp.0.join("linked");
        fs::create_dir_all(&linked).unwrap();
        let status = std::process::Command::new("cmd")
            .args(["/d", "/c", "mklink", "/J"])
            .arg(linked.join(".git"))
            .arg(&g)
            .stdout(std::process::Stdio::null())
            .status()
            .unwrap();
        assert!(status.success(), "mklink /J");
        assert!(git_dir(&linked, &tmp.0).is_none(), "junction .git refused");
        assert!(git_info(&linked, &tmp.0).is_none());
    }

    #[test]
    fn sources() {
        assert_eq!(
            Source::parse("nabre/dev-kit"),
            Some(Source::GitHub("nabre/dev-kit".into()))
        );
        assert_eq!(
            Source::parse(" a/b.git "),
            Some(Source::GitHub("a/b".into()))
        );
        assert_eq!(
            Source::parse("https://gitlab.com/x/y.git"),
            Some(Source::Url("https://gitlab.com/x/y.git".into()))
        );
        for bad in [
            "-o/x",
            "a/-b",
            "../x",
            "http://x/y",
            "https://x/y z",
            "ssh://x",
            "a/b/c",
            "file:///c/x",
            "--upload-pack=x",
        ] {
            assert_eq!(Source::parse(bad), None, "{bad}");
        }
        assert_eq!(
            Source::parse("https://gitlab.com/x/y.git")
                .unwrap()
                .default_name(),
            "y"
        );
        assert_eq!(
            Source::parse("nabre/dev-kit").unwrap().default_name(),
            "dev-kit"
        );
    }

    #[test]
    fn create_list_delete() {
        let tmp = TempDir::new("projects");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        create(&kit, "site", "it").unwrap();
        assert!(create(&kit, "site", "it").is_err());
        assert!(create(&kit, "..", "it").is_err());
        assert!(create(&kit, "con", "it").is_err());
        fs::write(kit.projects().join("site").join("index.html"), "x").unwrap();
        create(&kit, "empty", "it").unwrap();
        fs::create_dir_all(kit.projects().join(".hidden")).unwrap();
        let l = list(&kit, "it");
        assert_eq!(l.len(), 2);
        let site = l.iter().find(|p| p.id == "site").unwrap();
        assert!(!site.empty);
        assert_eq!(site.project.default.as_deref(), Some("static"));
        let none = FakeDbs::new(&[]);
        assert!(delete(&kit, "site", "it", &none, false).is_err());
        delete(&kit, "empty", "it", &none, false).unwrap();
        assert!(!kit.projects().join("empty").exists());
        assert!(delete(&kit, r"..\data", "it", &none, false).is_err());
        assert!(none.calls().is_empty());
    }

    /// The kit's servers, faked: which are installed, which drop fails, the
    /// drops asked (and the servers "started" for them).
    struct FakeDbs {
        installed: Vec<&'static str>,
        failing: Vec<&'static str>,
        calls: std::sync::Mutex<Vec<String>>,
    }

    impl FakeDbs {
        fn new(installed: &[&'static str]) -> FakeDbs {
            FakeDbs {
                installed: installed.to_vec(),
                failing: Vec::new(),
                calls: std::sync::Mutex::new(Vec::new()),
            }
        }
        fn calls(&self) -> Vec<String> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl ProjectDatabases for FakeDbs {
        fn installed(&self, engine: &str) -> bool {
            self.installed.contains(&engine)
        }
        fn drop_db(&self, engine: &str, database: &str) -> Result<()> {
            // as `Services::db_drop`: the server starts first
            self.calls.lock().unwrap().push(format!("start {engine}"));
            if self.failing.contains(&engine) {
                bail!("{engine} did not start");
            }
            self.calls
                .lock()
                .unwrap()
                .push(format!("drop {engine} {database}"));
            Ok(())
        }
    }

    /// Two projects whose databases have the same name (made before the
    /// names were stored: `my-app` and `my_app` both `my_app`): deleting one
    /// would drop the other's, so it is refused before any server is
    /// touched; forced, no server is touched at all.
    #[test]
    fn delete_refused_when_another_project_has_the_same_database() {
        let tmp = TempDir::new("projects-delete-same");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        for name in ["my-app", "my_app"] {
            create(&kit, name, "it").unwrap();
            crate::db::auth::set_project_password(&kit, name, "mariadb", "pw").unwrap();
        }
        crate::db::auth::set_project_password(&kit, "my-app", "mongodb", "pw").unwrap();
        assert!(delete_plan(&kit, "my_app", "it")
            .unwrap()
            .databases
            .iter()
            .all(|d| d.database == "my_app"));
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        let e = delete(&kit, "my-app", "it", &dbs, false).unwrap_err();
        assert!(format!("{e:#}").contains("my_app"), "{e:#}");
        assert!(dbs.calls().is_empty(), "{:?}", dbs.calls());
        assert!(kit.projects().join("my-app").is_dir());
        assert!(crate::db::auth::project_password(&kit, "my-app", "mariadb").is_some());

        // a server not installed is not touched: no reason to refuse
        let only_mongo = FakeDbs::new(&["mongodb"]);
        delete(&kit, "my_app", "it", &only_mongo, false).unwrap();
        assert!(only_mongo.calls().is_empty());
        // now alone with its name
        delete(&kit, "my-app", "it", &dbs, false).unwrap();
        assert_eq!(
            dbs.calls(),
            [
                "start mariadb",
                "drop mariadb my_app",
                "start mongodb",
                "drop mongodb my_app"
            ]
        );
    }

    /// Data given to a project renamed outside the app (WP-7) drops the
    /// databases it was created with, named after the old name; names of
    /// before kept by the rule of their time (`config` on MariaDB), and one
    /// of the server's own never dropped.
    #[test]
    fn delete_drops_the_names_in_use() {
        let tmp = TempDir::new("projects-delete-names");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        // an orphan of before the names, adopted by "Nuovo"
        let old = kit.project_data().join(r"Vecchio-Nome\private\db.json");
        fs::create_dir_all(old.parent().unwrap()).unwrap();
        fs::write(&old, r#"{"mariadb":"pw","mongodb":"pw2"}"#).unwrap();
        create(&kit, "Nuovo", "it").unwrap();
        state::adopt(&kit, "Vecchio-Nome", "Nuovo").unwrap();
        let plan = delete_plan(&kit, "Nuovo", "it").unwrap();
        assert!(plan.databases.iter().all(|d| d.database == "vecchio_nome"));
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        delete(&kit, "Nuovo", "it", &dbs, false).unwrap();
        assert_eq!(
            dbs.calls(),
            [
                "start mariadb",
                "drop mariadb vecchio_nome",
                "start mongodb",
                "drop mongodb vecchio_nome"
            ]
        );

        // `config` of before: its own database on MariaDB; on MongoDB the
        // server's, so nothing is dropped and the user is told
        create(&kit, "config", "it").unwrap();
        crate::db::auth::set_project_password(&kit, "config", "mariadb", "pw").unwrap();
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        assert_eq!(
            delete_plan(&kit, "config", "it").unwrap().databases[0].database,
            "config"
        );
        crate::db::auth::set_project_password(&kit, "config", "mongodb", "pw").unwrap();
        assert!(delete_plan(&kit, "config", "it").is_err());
        let e = delete(&kit, "config", "it", &dbs, false).unwrap_err();
        assert!(format!("{e:#}").contains("by hand"), "{e:#}");
        assert!(dbs.calls().is_empty());
        assert!(kit.projects().join("config").is_dir());
        delete(&kit, "config", "it", &dbs, true).unwrap();
        assert!(dbs.calls().is_empty());
        assert!(!kit.projects().join("config").exists());
    }

    /// A project with an empty folder, kit files, two databases and a copy.
    fn project_with_dbs(kit: &Kit, name: &str) -> state::ProjectState {
        create(kit, name, "it").unwrap();
        settings::update(kit, name, |s| s.agent = Some("claude".into()));
        crate::db::auth::set_project_password(kit, name, "mariadb", "pw-maria").unwrap();
        crate::db::auth::set_project_password(kit, name, "mongodb", "pw-mongo").unwrap();
        let state = state::ProjectState::of(kit, name).unwrap();
        fs::write(state.backups_dir().unwrap().join("x-1.sql"), "12345").unwrap();
        fs::write(state.backups_dir().unwrap().join("x-2.archive.gz"), "123").unwrap();
        state
    }

    #[test]
    fn delete_plan_with_and_without_databases() {
        let tmp = TempDir::new("projects-plan");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        create(&kit, "plain", "it").unwrap();
        let plan = delete_plan(&kit, "plain", "it").unwrap();
        assert_eq!(
            plan,
            DeletePlan {
                folder_exists: true,
                folder_empty: true,
                databases: Vec::new(),
                backups: 0,
                backups_size: 0,
            }
        );
        assert!(
            !kit.project_data().join("plain").exists(),
            "the plan creates nothing"
        );
        assert!(delete_plan(&kit, "..", "it").is_err());
        let missing = delete_plan(&kit, "missing", "it").unwrap();
        assert!(!missing.folder_exists && !missing.folder_empty);

        let state = project_with_dbs(&kit, "Shop Online");
        fs::write(kit.projects().join("Shop Online").join("a.txt"), "x").unwrap();
        let plan = delete_plan(&kit, "Shop Online", "it").unwrap();
        assert!(plan.folder_exists && !plan.folder_empty);
        let engines: Vec<_> = plan.databases.iter().map(|d| d.engine.as_str()).collect();
        assert_eq!(engines, ["mariadb", "mongodb"]);
        assert!(plan
            .databases
            .iter()
            .all(|d| d.database == "shop_online" && !d.installed));
        assert_eq!((plan.backups, plan.backups_size), (2, 8));
        let json = serde_json::to_string(&plan).unwrap();
        assert!(
            json.contains("\"folderEmpty\":false") && json.contains("\"backupsSize\":8"),
            "{json}"
        );
        assert!(!json.contains("pw-"), "no password in the plan: {json}");

        // passwords that cannot be read: no plan (only a forced delete)
        fs::write(state.db_file().unwrap(), "{ not json").unwrap();
        assert!(delete_plan(&kit, "Shop Online", "it").is_err());
    }

    #[test]
    fn delete_drops_the_databases_first() {
        let tmp = TempDir::new("projects-delete-dbs");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let state = project_with_dbs(&kit, "shop");
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        delete(&kit, "shop", "it", &dbs, false).unwrap();
        assert_eq!(
            dbs.calls(),
            [
                "start mariadb",
                "drop mariadb shop",
                "start mongodb",
                "drop mongodb shop"
            ]
        );
        assert!(!kit.projects().join("shop").exists());
        assert!(
            !state.dir().exists(),
            "nothing left in data\\projects\\shop"
        );

        // a server not installed is skipped; the rest goes
        let state = project_with_dbs(&kit, "shop");
        let dbs = FakeDbs::new(&["mongodb"]);
        delete(&kit, "shop", "it", &dbs, false).unwrap();
        assert_eq!(dbs.calls(), ["start mongodb", "drop mongodb shop"]);
        assert!(!kit.projects().join("shop").exists() && !state.dir().exists());
    }

    #[test]
    fn a_failed_drop_deletes_nothing_and_force_skips_the_servers() {
        let tmp = TempDir::new("projects-delete-fail");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let state = project_with_dbs(&kit, "shop");
        let mut dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        dbs.failing.push("mongodb");
        let e = delete(&kit, "shop", "it", &dbs, false).unwrap_err();
        assert!(format!("{e:#}").contains("mongodb did not start"), "{e:#}");
        assert!(!format!("{e:#}").contains("pw-"), "{e:#}");
        assert!(kit.projects().join("shop").is_dir(), "folder kept");
        assert_eq!(
            crate::db::auth::project_password(&kit, "shop", "mongodb").as_deref(),
            Some("pw-mongo"),
            "passwords kept"
        );
        assert_eq!(
            fs::read_dir(state.backups_dir().unwrap()).unwrap().count(),
            2
        );

        // forced: no server touched, the folders go
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        delete(&kit, "shop", "it", &dbs, true).unwrap();
        assert!(dbs.calls().is_empty(), "{:?}", dbs.calls());
        assert!(!kit.projects().join("shop").exists() && !state.dir().exists());

        // a non-empty folder stops before any server is touched
        let state = project_with_dbs(&kit, "shop");
        fs::write(kit.projects().join("shop").join("a.txt"), "x").unwrap();
        let dbs = FakeDbs::new(&["mariadb", "mongodb"]);
        assert!(delete(&kit, "shop", "it", &dbs, false).is_err());
        assert!(delete(&kit, "shop", "it", &dbs, true).is_err());
        assert!(dbs.calls().is_empty());
        fs::remove_file(kit.projects().join("shop").join("a.txt")).unwrap();

        // unreadable passwords: an error, unless forced
        fs::write(state.db_file().unwrap(), "{ not json").unwrap();
        assert!(delete(&kit, "shop", "it", &dbs, false).is_err());
        assert!(kit.projects().join("shop").is_dir());
        delete(&kit, "shop", "it", &dbs, true).unwrap();
        assert!(dbs.calls().is_empty());
        assert!(!kit.projects().join("shop").exists() && !state.dir().exists());
    }
}
