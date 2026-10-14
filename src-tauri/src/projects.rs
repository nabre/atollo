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
fn folders(kit: &Kit) -> Vec<PathBuf> {
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
    ProjectInfo {
        id: dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        modified: modified(dir),
        empty,
        missing,
        project,
    }
}

/// Every project, most recently changed first.
pub fn list(kit: &Kit, lang: &str) -> Vec<ProjectInfo> {
    let mut out: Vec<ProjectInfo> = folders(kit).iter().map(|d| info(kit, d, lang)).collect();
    out.sort_by(|a, b| b.modified.cmp(&a.modified).then_with(|| a.id.cmp(&b.id)));
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
    fs::remove_dir(&dir).with_context(|| dir.display().to_string())
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
/// credential helper: `gh` when signed in). Blocking.
pub fn clone(
    kit: &Kit,
    env: &KitEnv,
    source: &str,
    name: Option<&str>,
    lang: &str,
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
    let out = crate::jobs::kit_command(&git, env)
        .args(["clone", "--"])
        .arg(src.url())
        .arg(&dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .context("git clone")?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

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
