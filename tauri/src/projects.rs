//! The projects in `projects\` (port of nabre/atollo `core/projects.js`):
//! list with what each one is, new empty project, delete an empty one,
//! clone from GitHub or a Git URL.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::catalog::Catalog;
use crate::detect::{self, Project};
use crate::env::KitEnv;
use crate::kit::{validate_project_name, Kit};

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
    /// The kit's settings of the project (`project_settings.rs`).
    pub settings: crate::project_settings::ProjectSettings,
    /// Editor extensions of its languages and added by hand (`editor_ext.rs`).
    pub extensions: Vec<crate::editor_ext::ProjectExtension>,
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
                .is_ok_and(|m| m.is_dir() && !crate::safe_path::is_reparse(&m))
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
    let settings = crate::project_settings::load(kit, &id);
    let extensions = crate::editor_ext::of_project(
        &project.languages,
        &project.requires,
        &settings.extensions,
        &crate::editor_ext::installed(kit),
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

/// Deletes a project only while its folder is empty (never a link, never
/// files): `remove_dir` fails by itself if something appeared meanwhile.
pub fn delete_empty(kit: &Kit, name: &str, lang: &str) -> Result<()> {
    check_name(name, lang)?;
    let dir = kit.projects().join(name);
    let meta = fs::symlink_metadata(&dir).with_context(|| name.to_owned())?;
    if !meta.is_dir() || meta.file_type().is_symlink() {
        bail!(
            "{}",
            tr(
                lang,
                "{0} è un collegamento: non viene eliminato da qui.",
                &[name]
            )
        );
    }
    if fs::read_dir(&dir)?.next().is_some() {
        bail!(
            "{}",
            tr(
                lang,
                "La cartella {0} non è vuota: il progetto non viene eliminato.",
                &[name]
            )
        );
    }
    fs::remove_dir(&dir).with_context(|| dir.display().to_string())?;
    crate::project_settings::remove(kit, name);
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
/// credential helper: the kit's GitHub account, `github.rs`). Blocking.
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
    let mut cmd = crate::jobs::kit_command(&git, env);
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

use crate::safe_path::{lexical, no_links};

/// A file of the repository, read only when it is inside `root` and no link
/// leads to it (`safe_path`).
fn read_plain(root: &Path, path: &Path) -> Option<String> {
    crate::safe_path::read_text(root, path)
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
    if crate::safe_path::is_dir(root, &dot) {
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
    crate::safe_path::is_dir(root, &path).then_some(path)
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
    .filter(|f| crate::safe_path::is_file(dir, &dir.join(f)))
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
        crate::safe_path::tests::junction(&kit.projects().join("linked"), &outside);
        let names: Vec<String> = folders(&kit)
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["real"]);
        // Inside a project: a junction named like a manifest folder.
        crate::safe_path::tests::junction(&real.join("server"), &outside);
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
            Source::parse("nabre/atollo"),
            Some(Source::GitHub("nabre/atollo".into()))
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
            Source::parse("nabre/atollo").unwrap().default_name(),
            "atollo"
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
        assert!(delete_empty(&kit, "site", "it").is_err());
        delete_empty(&kit, "empty", "it").unwrap();
        assert!(!kit.projects().join("empty").exists());
        assert!(delete_empty(&kit, r"..\data", "it").is_err());
    }
}
