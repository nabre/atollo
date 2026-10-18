//! The kit's files of one project, all in `data\projects\<project>\` (D-89),
//! never in the project's folder (D-63):
//!
//! ```text
//! dev-settings.json   agent, last opened, extensions, versions
//! ports.json          ports of its actions, those locked by the user
//! apache\  nginx\  caddy\
//!                     configurations of its own web servers
//! private\            owner-only DACL, set before anything is written in it
//!   db.json           passwords of its databases
//!   backups\          copies of its databases
//! ```
//!
//! Deleting, renaming or saving a project is an operation on one folder.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::kit::{validate_project_name, Kit};

/// The kit's files of one project: `data\projects\<project>\`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectState {
    dir: PathBuf,
}

impl ProjectState {
    /// `None` when `project` is not a valid project name (`..`, `a/b`, `con`).
    pub fn of(kit: &Kit, project: &str) -> Option<ProjectState> {
        validate_project_name(project).ok()?;
        Some(ProjectState {
            dir: kit.project_data().join(project),
        })
    }

    pub fn dir(&self) -> &PathBuf {
        &self.dir
    }
    pub fn settings_file(&self) -> PathBuf {
        self.dir.join("dev-settings.json")
    }
    pub fn ports_file(&self) -> PathBuf {
        self.dir.join("ports.json")
    }
    pub fn apache_dir(&self) -> PathBuf {
        self.dir.join("apache")
    }
    pub fn nginx_dir(&self) -> PathBuf {
        self.dir.join("nginx")
    }
    pub fn caddy_dir(&self) -> PathBuf {
        self.dir.join("caddy")
    }

    /// `private\`, created and made owner-only before it is returned, so
    /// every file written in it is private from the start (as `db/auth.rs`).
    /// A disk without permissions (FAT32/exFAT) only leaves a warning.
    pub fn private_dir(&self) -> io::Result<PathBuf> {
        let dir = self.dir.join("private");
        fs::create_dir_all(&dir)?;
        if let Err(e) = crate::router::fs::restrict_dir_to_owner(&dir) {
            crate::log::warn(&format!("{}: permissions: {e:#}", dir.display()));
        }
        Ok(dir)
    }
    pub fn db_file(&self) -> io::Result<PathBuf> {
        Ok(self.private_dir()?.join("db.json"))
    }
    pub fn backups_dir(&self) -> io::Result<PathBuf> {
        let dir = self.private_dir()?.join("backups");
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    /// Everything the kit keeps for the project goes; a missing folder is
    /// not an error.
    pub fn remove(self) -> io::Result<()> {
        match fs::remove_dir_all(&self.dir) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        }
    }

    /// The files follow a project renamed to `to`; refused when `to` is not
    /// a valid name or already has files of the kit.
    pub fn rename(self, kit: &Kit, to: &str) -> io::Result<ProjectState> {
        let target = ProjectState::of(kit, to)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "not a project name"))?;
        if target.dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} already exists", target.dir.display()),
            ));
        }
        if self.dir.exists() {
            if let Some(parent) = target.dir.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::rename(&self.dir, &target.dir)?;
        }
        Ok(target)
    }
}

/// The projects with files of the kit (valid names only), sorted.
pub fn all(kit: &Kit) -> Vec<String> {
    let mut out: Vec<String> = fs::read_dir(kit.project_data())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|n| validate_project_name(n).is_ok())
        .collect();
    out.sort();
    out
}

/// At least one file (or link) anywhere under `dir`: folders alone are not
/// data (reading passwords or copies creates an empty `private\`). Links
/// count as files and are never followed.
pub fn has_files(dir: &Path) -> bool {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| match e.file_type() {
            Ok(t) if t.is_dir() => has_files(&e.path()),
            _ => true,
        })
}

/// Removes `dir` and the folders in it that hold no file, bottom up; a file
/// or a link is never removed (nor the folders above it). True when `dir`
/// itself went.
pub fn remove_empty_dirs(dir: &Path) -> bool {
    for e in fs::read_dir(dir).into_iter().flatten().flatten() {
        if e.file_type().is_ok_and(|t| t.is_dir()) {
            remove_empty_dirs(&e.path());
        }
    }
    // Refused by Windows while anything is left in it.
    fs::remove_dir(dir).is_ok()
}

/// The folders of `data\projects\` whose project is gone from `projects\`
/// (renamed or deleted outside the app) and that still hold files of the
/// kit, sorted (PLAN-DATA.md WP-7). Never deleted automatically.
pub fn orphans(kit: &Kit) -> Vec<String> {
    all(kit)
        .into_iter()
        .filter(|p| !kit.projects().join(p).is_dir())
        .filter(|p| has_files(&kit.project_data().join(p)))
        .collect()
}

/// What the kit keeps for a project, without creating anything (`private\`
/// is only read) and without a password.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Holdings {
    pub project: String,
    /// `dev-settings.json` is there.
    pub settings: bool,
    /// Ports remembered for its actions.
    pub ports: usize,
    /// Engines with a user of the project (`mariadb`, `mongodb`).
    pub databases: Vec<String>,
    /// Copies of its databases.
    pub backups: usize,
}

pub fn holdings(kit: &Kit, project: &str) -> Option<Holdings> {
    let state = ProjectState::of(kit, project)?;
    let private = state.dir.join("private");
    let json = |path: &Path| crate::fsx::read_json::<serde_json::Value>(path);
    let ports = json(&state.ports_file())
        .and_then(|v| v.get("ports")?.as_object().map(|m| m.len()))
        .unwrap_or(0);
    let databases = json(&private.join("db.json"))
        .and_then(|v| {
            let engines = v.as_object()?.iter().filter(|(_, pw)| pw.is_string());
            Some(engines.map(|(k, _)| k.clone()).collect())
        })
        .unwrap_or_default();
    let backups = fs::read_dir(private.join("backups"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .count();
    Some(Holdings {
        project: project.to_owned(),
        settings: state.settings_file().is_file(),
        ports,
        databases,
        backups,
    })
}

/// Gives the files of `orphan` to `project` (the same project renamed
/// outside the app): refused when `orphan` is not an orphan, when
/// `project` has no folder in `projects\` or already has files of the kit
/// (empty folders are not files: they are removed first).
///
/// The databases on the servers keep their name and user, made from the
/// old name: the passwords stay valid for them and the project's `.env`
/// keeps working as it is. The names travel in `db.json` (`names`); data
/// from before they were stored gets them first, by the rule of the old
/// name (`db::auth::keep_legacy_names`), so the kit goes on using, and on
/// deletion drops, those databases and not new ones named after `project`.
///
/// A link (junction) in place of either folder is refused before anything
/// is read or moved (D-77).
pub fn adopt(kit: &Kit, orphan: &str, project: &str) -> io::Result<ProjectState> {
    let invalid = |m: &str| io::Error::new(io::ErrorKind::InvalidInput, m.to_owned());
    let from = ProjectState::of(kit, orphan).ok_or_else(|| invalid("not a project name"))?;
    let to = ProjectState::of(kit, project).ok_or_else(|| invalid("not a project name"))?;
    for state in [&from, &to] {
        if crate::os::safe_path::is_link(state.dir()) {
            return Err(invalid(&format!(
                "{}: a link, left alone",
                state.dir().display()
            )));
        }
    }
    if !orphans(kit).iter().any(|o| o == orphan) {
        return Err(invalid(&format!("{orphan}: not data without a project")));
    }
    if !kit.projects().join(project).is_dir() {
        return Err(invalid(&format!("{project}: no such project")));
    }
    if has_files(&to.dir) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("{project} already has files of the kit"),
        ));
    }
    // Before anything moves: a `db.json` that cannot be read stops here.
    crate::db::auth::keep_legacy_names(kit, orphan)
        .map_err(|e| io::Error::other(format!("{orphan}: databases: {e:#}")))?;
    if to.dir.exists() && !remove_empty_dirs(&to.dir) {
        return Err(io::Error::other(format!(
            "{}: not removed",
            to.dir.display()
        )));
    }
    from.rename(kit, project)
}

#[cfg(test)]
#[path = "../../../test/rust/projects/state.rs"]
mod tests;
