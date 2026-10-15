//! Copies of the projects' databases (D-78), in
//! `data\projects\<project>\private\backups` (D-89): export and restore with
//! the servers' own tools. The folder is owner-only before anything is
//! written in it, so every copy is private from the start (by inheritance).

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;

use super::tools::run_with_input;
use super::{auth, Services};
use crate::os::jobs;
use crate::projects::state::ProjectState;

/// A copy of a project's database (D-78).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub file: String,
    pub engine: String,
    pub size: u64,
}

impl Services {
    /// Where a project's database copies go, created and private: also the
    /// folder a copy from outside is put in by hand to restore it (D-78).
    pub fn backup_dir(&self, project: &str) -> Result<PathBuf> {
        let Some(state) = ProjectState::of(&self.kit, project) else {
            bail!("not a project name: {project}");
        };
        Ok(state.backups_dir()?)
    }

    /// The copies of a project's database, newest first.
    pub fn backups(&self, project: &str) -> Vec<Backup> {
        let Ok(dir) = self.backup_dir(project) else {
            return Vec::new();
        };
        let mut out: Vec<Backup> = fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| {
                e.metadata()
                    .is_ok_and(|m| m.is_file() && !crate::os::safe_path::is_reparse(&m))
            })
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let engine = if name.ends_with(".sql") {
                    "mariadb"
                } else if name.ends_with(".archive.gz") {
                    "mongodb"
                } else {
                    return None;
                };
                Some(Backup {
                    file: name,
                    engine: engine.to_owned(),
                    size: e.metadata().map(|m| m.len()).unwrap_or(0),
                })
            })
            .collect();
        out.sort_by(|a, b| b.file.cmp(&a.file));
        out
    }

    /// A copy of the project's database in its backup folder (D-78); the
    /// server must run.
    pub fn export(&self, project: &str, engine: &str) -> Result<PathBuf> {
        let (port, env) = self.running_server(engine)?;
        let name = auth::project_db_name(&self.kit, project, engine)?;
        let dir = self.backup_dir(project)?;
        let stamp = crate::agents::sync::stamp();
        match engine {
            "mongodb" => {
                let file = dir.join(format!("{name}-{stamp}.archive.gz"));
                let (mut cmd, config) = self.mongo_tool("mongodump.exe", &env, port)?;
                cmd.arg(format!("--db={name}"))
                    .arg(format!("--archive={}", file.display()))
                    .arg("--gzip");
                let result = run_with_input(cmd, "", Duration::from_secs(600));
                let _ = fs::remove_file(config);
                result.context("mongodump")?;
                Ok(file)
            }
            _ => {
                let file = dir.join(format!("{name}-{stamp}.sql"));
                let out = fs::File::create(&file)?;
                let mut cmd = self.mariadb_tool("mariadb-dump.exe", &env);
                cmd.args(["--single-transaction", "--routines", "--triggers"])
                    .arg(&name)
                    .stdin(Stdio::null())
                    .stdout(Stdio::from(out))
                    .stderr(Stdio::piped());
                let child = jobs::spawn(&mut cmd)?;
                let done = child.wait_with_output()?;
                if !done.status.success() {
                    let _ = fs::remove_file(&file);
                    bail!(
                        "mariadb-dump: {}",
                        String::from_utf8_lossy(&done.stderr).trim()
                    );
                }
                Ok(file)
            }
        }
    }

    /// Puts a copy back into the project's database (what is there now is
    /// replaced by the copy's tables or collections).
    pub fn restore(&self, project: &str, file: &str) -> Result<()> {
        if file.contains(['\\', '/']) || file.starts_with('.') {
            bail!("not a copy of this project: {file}");
        }
        let dir = self.backup_dir(project)?;
        let path = dir.join(file);
        if !crate::os::safe_path::is_file(&dir, &path) {
            bail!("copy not found: {file}");
        }
        // Never over a database that another project uses as well.
        let name = |engine| -> Result<String> {
            let name = auth::project_db_name(&self.kit, project, engine)?;
            auth::check_name_free(&self.kit, project, engine, &name)?;
            Ok(name)
        };
        if file.ends_with(".archive.gz") {
            let name = name("mongodb")?;
            let (port, env) = self.running_server("mongodb")?;
            let (mut cmd, config) = self.mongo_tool("mongorestore.exe", &env, port)?;
            cmd.arg(format!("--archive={}", path.display()))
                .arg("--gzip")
                .arg("--drop")
                .arg(format!("--nsInclude={name}.*"));
            let result = run_with_input(cmd, "", Duration::from_secs(600));
            let _ = fs::remove_file(config);
            result.context("mongorestore")
        } else if file.ends_with(".sql") {
            let name = name("mariadb")?;
            // As the project's user, never root: the file may come from
            // outside (D-78) and name other databases or `mysql.*`.
            let password =
                auth::try_project_password(&self.kit, project, "mariadb")?.ok_or_else(|| {
                    anyhow!("the project has no MariaDB database yet: create it first")
                })?;
            let (port, env) = self.running_server("mariadb")?;
            let sql = fs::read_to_string(&path)?;
            let (mut cmd, login) =
                self.mariadb_as_project("mariadb.exe", &env, port, &name, &password)?;
            cmd.arg(&name);
            let result = run_with_input(cmd, &sql, Duration::from_secs(600));
            let _ = fs::remove_file(login);
            result.context("mariadb")
        } else {
            bail!("not a database copy: {file}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::auth::tests::assert_owner_only;
    use crate::kit::{tests::TempDir, Arch, Kit};

    #[test]
    fn copies_in_the_private_folder() {
        let tmp = TempDir::new("db-backups");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let s = Services::new(kit);
        assert!(s.backup_dir("..").is_err());
        assert!(s.backups(r"..\x").is_empty());

        let dir = s.backup_dir("shop").unwrap();
        assert_eq!(dir, tmp.0.join(r"data\projects\shop\private\backups"));
        assert!(s.backups("shop").is_empty());
        // a copy written there (as mariadb-dump's output) is private by
        // inheritance from private\
        for name in ["shop-1.sql", "shop-2.archive.gz", "notes.txt"] {
            fs::write(dir.join(name), "x").unwrap();
        }
        assert_owner_only(&dir.join("shop-1.sql"));
        let list = s.backups("shop");
        let names: Vec<(&str, &str)> = list
            .iter()
            .map(|b| (b.file.as_str(), b.engine.as_str()))
            .collect();
        assert_eq!(
            names,
            [("shop-2.archive.gz", "mongodb"), ("shop-1.sql", "mariadb")]
        );
        assert!(s.backups("other").is_empty(), "one folder per project");

        // no server: export fails before writing anything
        assert!(s.export("shop", "mariadb").is_err());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 3);

        // names outside the folder are refused before any server is asked
        let outside = tmp.0.join(r"data\projects\shop\private\x.sql");
        fs::write(&outside, "x").unwrap();
        for bad in [
            r"..\x.sql",
            "../x.sql",
            r"C:\x.sql",
            ".hidden.sql",
            "missing.sql",
        ] {
            let e = s.restore("shop", bad).unwrap_err().to_string();
            assert!(
                e.contains("not a copy") || e.contains("not found"),
                "{bad}: {e}"
            );
        }
        // a copy that is there gets past the check (then: no server)
        let e = s.restore("shop", "shop-1.sql").unwrap_err().to_string();
        assert!(!e.contains("not a copy") && !e.contains("not found"), "{e}");
    }
}
