//! Versions: the saved versions of the languages and tools (this file), the
//! versions side by side known to Gradle, Maven and dotnet (`side_by_side`,
//! D-81) and the relations matrix (`relations`) with its version ranges
//! (`ranges`).
//!
//! Saved versions of the languages and tools (DECISIONS.md D-80).
//!
//! Every version has its folder inside the component's own, one per minor
//! line (`major.minor`): `data\apps\<platform>\node\<line>`, with the full
//! version in `.devkit-version`. The default is the folder of the installed
//! version (`data\app\versions\<id>.txt`), the one updates replace:
//! - an update that changes the minor line installs next to the old one,
//!   which stays as a saved line (a new patch of the same line replaces);
//! - a line can be downloaded there from the Tools page;
//! - the kit uses the newest line on disk; a project the one in its
//!   `dev-settings.json` (`versions`), the newest when it was created;
//! - the lines each source offers are kept in `data\app\version-lines.json`,
//!   so the lists propose them also offline; choosing one that is not saved
//!   downloads it.
//!
//! A choice is applied by rewriting the kit environment: every value that
//! points into the default folder (PATH, `JAVA_HOME`, `DOTNET_ROOT`,
//! `CP_PHP`) points into the chosen one instead. The editor's terminals
//! start from the shared editor server, so each project with a choice gets
//! `data\projects\<project>\versions.json` (the rewrites), which the
//! kit's PowerShell profile applies.
//!
//! The tools always at their latest version (`minor_folder`: agents, Git,
//! GitHub CLI, PowerShell...) have one folder per line too, but only one:
//! a new patch replaces its line, a new line installs next to the old one,
//! which then goes (`drop_old_lines`; one in use at the next start).
//!
//! Here the folders and lines on disk and the Tools page; the choices in
//! `choice`, the lines offered in `offered`, removal and repairs in `disk`.

pub mod ranges;
pub mod relations;
pub mod side_by_side;

mod choice;
mod disk;
mod offered;

pub use choice::{
    apply_kit_choice, check, kit_choice, newest_line, pin_newest, project_choice, project_env,
    set_project_choice, write_project_files, Choice,
};
pub use disk::{drop_old_lines, migrate, remove, sync_with_disk};
pub(crate) use offered::offered_file;
pub use offered::{offered, save_offered, stale_offers, to_download};

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};

/// The file with the full version, inside a saved folder.
const VERSION_FILE: &str = ".devkit-version";

/// `major.minor` of a version: `v22.11.0` → `22.11`, `8.4.13 (Composer 2.8.1)`
/// → `8.4`, `21.0.5+11` → `21.0`. `None` without two numbers.
pub fn line_of(version: &str) -> Option<String> {
    let start = version.find(|c: char| c.is_ascii_digit())?;
    let mut parts = version[start..]
        .split(|c: char| !c.is_ascii_digit() && c != '.')
        .next()?
        .split('.');
    let (major, minor) = (parts.next()?, parts.next()?);
    if major.is_empty() || minor.is_empty() {
        return None;
    }
    Some(format!("{major}.{minor}"))
}

/// A line as the interface and the files write it: `22.11`.
pub fn valid_line(line: &str) -> bool {
    line_of(line).as_deref() == Some(line)
}

/// The versioned components of the catalog.
pub fn versioned() -> impl Iterator<Item = &'static Component> {
    Catalog::get().all().iter().filter(|c| c.versioned)
}

/// The components with one folder per line: versioned, or `minor_folder`.
pub fn lined() -> impl Iterator<Item = &'static Component> {
    Catalog::get().all().iter().filter(|c| c.per_line())
}

fn find(id: &str) -> Option<&'static Component> {
    Catalog::get().find(id).filter(|c| c.versioned)
}

/// `data\apps\<platform>\<folder>`: the folder of every version.
fn store(kit: &Kit, c: &Component) -> PathBuf {
    c.folder_path(kit).unwrap_or_else(|| kit.apps().join(&c.id))
}

/// The folder of `version`: its line, or the version itself (safe
/// characters only) when it has no `major.minor`.
pub fn dir_name(version: &str) -> String {
    line_of(version).unwrap_or_else(|| {
        version
            .trim()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' {
                    ch
                } else {
                    '_'
                }
            })
            .collect()
    })
}

/// Where `line` of `c` is (or goes).
pub fn saved_dir(kit: &Kit, c: &Component, line: &str) -> PathBuf {
    store(kit, c).join(line)
}

/// Where `version` of `c` is installed as the default.
pub fn install_dir(kit: &Kit, c: &Component, version: &str) -> PathBuf {
    saved_dir(kit, c, &dir_name(version))
}

/// The default folder of `c`: that of the installed version; the
/// component's folder itself while it has the layout of an older kit
/// (`migrate` not done yet) or nothing is installed.
pub fn default_dir(kit: &Kit, c: &Component) -> PathBuf {
    let store = store(kit, c);
    if let Some(version) = c.installed_version(kit) {
        let dir = store.join(dir_name(&version));
        if has_key(c, &dir) {
            return dir;
        }
    }
    store
}

/// `rel` of the catalog (`node/node.exe`) inside the default folder of the
/// component with one folder per line it starts with; `None` for the others.
pub fn resolve(kit: &Kit, rel: &str) -> Option<PathBuf> {
    let rel = rel.replace('/', "\\");
    let (head, rest) = rel.split_once('\\').unwrap_or((rel.as_str(), ""));
    let c = lined().find(|c| {
        c.folder
            .as_deref()
            .is_some_and(|f| f.eq_ignore_ascii_case(head))
    })?;
    let dir = default_dir(kit, c);
    Some(if rest.is_empty() { dir } else { dir.join(rest) })
}

/// The key of `c` is in `dir`, a copy of its own folder.
pub fn has_key(c: &Component, dir: &Path) -> bool {
    let Some(folder) = &c.folder else {
        return false;
    };
    let rel = c
        .key
        .strip_prefix(folder.as_str())
        .unwrap_or(&c.key)
        .trim_start_matches('/');
    let path = dir.join(rel.replace('/', "\\"));
    path.is_file() || (rel.is_empty() && fs::read_dir(dir).is_ok_and(|mut d| d.next().is_some()))
}

/// The line of the default folder, when it is installed.
pub fn default_line(kit: &Kit, c: &Component) -> Option<String> {
    if !c.is_installed(kit) {
        return None;
    }
    c.installed_version(kit).as_deref().and_then(line_of)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Line {
    pub line: String,
    /// The full version, when known.
    pub version: Option<String>,
}

/// The saved lines of `c` (not the default), newest first.
pub fn saved(kit: &Kit, c: &Component) -> Vec<Line> {
    let default = default_line(kit, c);
    let mut out: Vec<Line> = fs::read_dir(store(kit, c))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let line = e.file_name().to_string_lossy().into_owned();
            let other = valid_line(&line) && default.as_deref() != Some(line.as_str());
            (other && has_key(c, &e.path())).then(|| Line {
                version: version_in(&e.path()),
                line,
            })
        })
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(&l.line)));
    out
}

/// The full version written in a line's folder.
fn version_in(dir: &Path) -> Option<String> {
    fs::read_to_string(dir.join(VERSION_FILE))
        .ok()
        .map(|v| v.trim().to_owned())
        .filter(|v| !v.is_empty())
}

pub fn write_version(dir: &Path, version: &str) -> Result<()> {
    write_atomic(&dir.join(VERSION_FILE), version.as_bytes())
        .with_context(|| dir.display().to_string())
}

/// The folder used for `line` of `c`: the saved one, or the default folder
/// when `line` is the default's, is not chosen or is not saved (any more).
pub fn dir_for(kit: &Kit, c: &Component, line: Option<&str>) -> Option<PathBuf> {
    c.folder.as_ref()?;
    let main = default_dir(kit, c);
    Some(match line {
        Some(l) if Some(l) != default_line(kit, c).as_deref() => {
            let dir = saved_dir(kit, c, l);
            if has_key(c, &dir) {
                dir
            } else {
                main
            }
        }
        _ => main,
    })
}

/// `line` of `c` is on disk: the default folder or a saved one.
pub fn is_present(kit: &Kit, c: &Component, line: &str) -> bool {
    default_line(kit, c).as_deref() == Some(line) || has_key(c, &saved_dir(kit, c, line))
}

/// The projects that chose a line, per component id and line.
fn project_users(kit: &Kit) -> BTreeMap<(String, String), Vec<String>> {
    let mut out: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for project in crate::projects::state::all(kit) {
        for (id, line) in project_choice(kit, &project) {
            out.entry((id, line)).or_default().push(project.clone());
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub installed: bool,
    /// The default folder.
    pub default: Option<Line>,
    pub saved: Vec<Line>,
    /// The line the kit uses when newer than the default folder's.
    pub preferred: Option<String>,
    /// Lines can be downloaded (the installer knows the older ones).
    pub downloadable: bool,
    /// The lines its source offered the last time it was asked, newest
    /// first (also those already on disk).
    pub available: Vec<String>,
    /// When the source was asked (Unix seconds); `None`: never.
    pub checked: Option<u64>,
    /// The projects that chose each line.
    pub projects: BTreeMap<String, Vec<String>>,
}

pub fn tools(kit: &Kit) -> Vec<Tool> {
    let choice = kit_choice(kit);
    let mut offered = offered(kit);
    let users = project_users(kit);
    versioned()
        .map(|c| {
            let downloadable = crate::install::installer(&c.id).is_some_and(|i| i.has_lines());
            let offer = offered.remove(&c.id).filter(|_| downloadable);
            Tool {
                id: c.id.clone(),
                name: c.name.clone(),
                installed: c.is_installed(kit),
                default: default_line(kit, c).map(|line| Line {
                    line,
                    version: c.installed_version(kit),
                }),
                saved: saved(kit, c),
                preferred: choice.get(&c.id).cloned(),
                downloadable,
                checked: offer.as_ref().map(|o| o.checked),
                available: offer.map(|o| o.lines).unwrap_or_default(),
                projects: users
                    .iter()
                    .filter(|((id, _), _)| *id == c.id)
                    .map(|((_, line), p)| (line.clone(), p.clone()))
                    .collect(),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "../../../test/rust/versions/mod.rs"]
mod tests;
