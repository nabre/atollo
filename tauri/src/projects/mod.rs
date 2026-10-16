//! Projects: the list and its operations (this file), the kit's settings of
//! a project (`settings`, `dev-settings.json`), its MCP servers (`mcp`) and
//! editor settings (`vscode`), new projects from frameworks (`frameworks`),
//! the actions started by the kit (`run`), the static file server (`serve`)
//! and a project's own Apache (`apache`). What a project is: `detect`.
//!
//! The projects in `projects\` (port of nabre/atollo `core/projects.js`):
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

/// What can be added from "Linguaggi e strumenti": not installed, and
/// available for the kit's platform (no PHP, Apache, Nginx off Windows, no
/// R on Windows ARM64).
fn addable(kit: &Kit, c: &crate::setup::catalog::Component) -> bool {
    !c.is_installed(kit) && c.is_available_on(kit.target())
}

pub fn info(kit: &Kit, dir: &Path, lang: &str) -> ProjectInfo {
    let project = detect::detect(dir, lang);
    let catalog = Catalog::get();
    let missing = project
        .requires
        .iter()
        .filter(|id| catalog.find(id).is_some_and(|c| addable(kit, c)))
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
    // No folder at all: the data of the kit left by a project that is gone
    // (an orphan, WP-7) is deleted with its databases.
    let no_folder = matches!(
        fs::symlink_metadata(&dir),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound
    );
    if !no_folder {
        check_deletable(&dir, name, lang)?;
    }
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
    if !no_folder {
        fs::remove_dir(&dir).with_context(|| dir.display().to_string())?;
    }
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
    let git = crate::setup::catalog::kit_path(kit, r"git\cmd\git.exe");
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
#[path = "../../../test/rust/projects/mod.rs"]
mod tests;
