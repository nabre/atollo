//! The projects in `projects\` (port of nabre/dev-kit `core/projects.js`):
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

/// Folders (or links to folders) in `projects\`, not hidden.
pub fn folders(kit: &Kit) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = fs::read_dir(kit.projects())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
        .map(|e| e.path())
        .filter(|p| p.is_dir())
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

/// A new, empty project folder.
pub fn create(kit: &Kit, name: &str, lang: &str) -> Result<PathBuf> {
    let name = name.trim();
    check_name(name, lang)?;
    let dir = kit.projects().join(name);
    if dir.exists() {
        bail!(
            "{}",
            tr(lang, "Esiste già una cartella {0} in projects.", &[name])
        );
    }
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

/// Git state of a project for its page (read only: commits and pushes stay
/// in the editor and the agents).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitInfo {
    pub branch: Option<String>,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    /// Changed, added, deleted or untracked files.
    pub changes: u32,
    pub remote: Option<String>,
}

/// From `git status --porcelain=v2 --branch`.
pub fn parse_git_status(text: &str) -> GitInfo {
    let mut info = GitInfo::default();
    for line in text.lines() {
        if let Some(head) = line.strip_prefix("# branch.head ") {
            info.branch = (head != "(detached)").then(|| head.to_owned());
        } else if let Some(up) = line.strip_prefix("# branch.upstream ") {
            info.upstream = Some(up.to_owned());
        } else if let Some(ab) = line.strip_prefix("# branch.ab ") {
            for part in ab.split_whitespace() {
                if let Some(n) = part.strip_prefix('+') {
                    info.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    info.behind = n.parse().unwrap_or(0);
                }
            }
        } else if !line.starts_with('#') && !line.trim().is_empty() {
            info.changes += 1;
        }
    }
    info
}

/// What a project's page shows besides its detection.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectStatus {
    /// None: not a Git repository (or Git not installed).
    pub git: Option<GitInfo>,
    /// Instruction files for the agents found at its root.
    pub instructions: Vec<String>,
}

pub fn status(kit: &Kit, env: &KitEnv, dir: &Path) -> ProjectStatus {
    let instructions = [
        "AGENTS.md",
        "CLAUDE.md",
        "GEMINI.md",
        ".github/copilot-instructions.md",
    ]
    .iter()
    .filter(|f| dir.join(f).is_file())
    .map(|f| (*f).to_owned())
    .collect();
    let git_exe = kit.apps().join(r"git\cmd\git.exe");
    let git = |args: &[&str]| {
        crate::jobs::kit_command(&git_exe, env)
            .arg("-C")
            .arg(dir)
            .args(args)
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    };
    let git_info = if dir.join(".git").exists() && git_exe.is_file() {
        git(&["status", "--porcelain=v2", "--branch"]).map(|text| {
            let mut info = parse_git_status(&text);
            info.remote = git(&["remote", "get-url", "origin"])
                .map(|u| u.trim().to_owned())
                .filter(|u| !u.is_empty());
            info
        })
    } else {
        None
    };
    ProjectStatus {
        git: git_info,
        instructions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn git_status_parsing() {
        let text = "# branch.oid 1234\n# branch.head main\n# branch.upstream origin/main\n\
                    # branch.ab +2 -1\n1 .M N... 100644 100644 100644 a b src/app.ts\n\
                    ? notes.txt\n";
        let info = parse_git_status(text);
        assert_eq!(info.branch.as_deref(), Some("main"));
        assert_eq!(info.upstream.as_deref(), Some("origin/main"));
        assert_eq!((info.ahead, info.behind, info.changes), (2, 1, 2));
        let detached = parse_git_status("# branch.head (detached)\n");
        assert_eq!(detached.branch, None);
        assert_eq!(detached.changes, 0);
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
        assert!(delete_empty(&kit, "site", "it").is_err());
        delete_empty(&kit, "empty", "it").unwrap();
        assert!(!kit.projects().join("empty").exists());
        assert!(delete_empty(&kit, r"..\data", "it").is_err());
    }
}
