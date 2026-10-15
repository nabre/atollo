//! The kit's files of one project, all in `data\projects\<project>\` (D-89),
//! never in the project's folder (D-63):
//!
//! ```text
//! dev-settings.json   agent, last opened, extensions, versions
//! ports.json          ports of its actions, those locked by the user
//! apache\  nginx\     configurations of its own web servers
//! private\            owner-only DACL, set before anything is written in it
//!   db.json           passwords of its databases
//!   backups\          copies of its databases
//! ```
//!
//! Deleting, renaming or saving a project is an operation on one folder.
// Contract of PLAN-DATA.md WP-0, used from wave 1 on; removed in WP-14.
#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::PathBuf;

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

    /// `private\`, created and made owner-only before it is returned, so
    /// every file written in it is private from the start (as `db/auth.rs`).
    /// A disk without permissions (FAT32/exFAT) only leaves a warning.
    pub fn private_dir(&self) -> io::Result<PathBuf> {
        let dir = self.dir.join("private");
        fs::create_dir_all(&dir)?;
        if let Err(e) = crate::editor::restrict_dir_to_owner(&dir) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn one_folder_per_project() {
        let tmp = TempDir::new("project-state");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        for bad in ["", "..", r"..\x", "a/b", "con", "x."] {
            assert!(ProjectState::of(&kit, bad).is_none(), "{bad:?}");
        }
        let s = ProjectState::of(&kit, "shop").unwrap();
        assert_eq!(s.dir(), &tmp.0.join(r"data\projects\shop"));
        assert_eq!(
            s.settings_file(),
            tmp.0.join(r"data\projects\shop\dev-settings.json")
        );
        assert_eq!(s.ports_file(), tmp.0.join(r"data\projects\shop\ports.json"));
        assert_eq!(s.apache_dir(), tmp.0.join(r"data\projects\shop\apache"));
        let backups = s.backups_dir().unwrap();
        assert!(backups.is_dir() && backups.ends_with(r"private\backups"));
        assert_eq!(
            s.db_file().unwrap(),
            tmp.0.join(r"data\projects\shop\private\db.json")
        );
        assert_eq!(all(&kit), ["shop"]);

        // renamed with its files; never over another project's files
        fs::write(s.settings_file(), "{}").unwrap();
        let s = s.rename(&kit, "store").unwrap();
        assert!(s.settings_file().is_file());
        assert_eq!(all(&kit), ["store"]);
        ProjectState::of(&kit, "other")
            .unwrap()
            .private_dir()
            .unwrap();
        assert!(s.clone().rename(&kit, "other").is_err());
        assert!(s.clone().rename(&kit, "..").is_err());

        s.remove().unwrap();
        ProjectState::of(&kit, "never").unwrap().remove().unwrap();
        assert_eq!(all(&kit), ["other"]);
    }
}
