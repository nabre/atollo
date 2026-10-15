//! Copies of the projects' databases (D-78), in `data\dbackup\<project>`:
//! export and restore with the servers' own tools.

use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::tools::run_with_input;
use super::{auth, Services};
use crate::os::jobs;

/// A copy of a project's database (D-78).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub file: String,
    pub engine: String,
    pub size: u64,
}

impl Services {
    /// Where a project's database copies go: `data\db\backup\<project>`.
    pub fn backup_dir(&self, project: &str) -> PathBuf {
        self.kit.data().join(r"db\backup").join(project)
    }

    /// The copies of a project's database, newest first.
    pub fn backups(&self, project: &str) -> Vec<Backup> {
        let mut out: Vec<Backup> = fs::read_dir(self.backup_dir(project))
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
        let name = auth::db_name(project);
        let dir = self.backup_dir(project);
        fs::create_dir_all(&dir)?;
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
        let path = self.backup_dir(project).join(file);
        if !crate::os::safe_path::is_file(&self.backup_dir(project), &path) {
            bail!("copy not found: {file}");
        }
        let name = auth::db_name(project);
        if file.ends_with(".archive.gz") {
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
            let (_, env) = self.running_server("mariadb")?;
            let sql = fs::read_to_string(&path)?;
            let mut cmd = self.mariadb_tool("mariadb.exe", &env);
            cmd.arg(&name);
            run_with_input(cmd, &sql, Duration::from_secs(600)).context("mariadb")
        } else {
            bail!("not a database copy: {file}")
        }
    }
}
