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

use super::tools::{run_with_input, run_with_reader};
use super::{auth, project_line, Services};
use crate::db::DbEngine;
use crate::os::jobs;
use crate::projects::state::ProjectState;

/// The options of a MySQL dump (D-78): one consistent snapshot, with the
/// routines, triggers and events, and no GTID statement, which only an
/// administrator could restore (the restore runs as the project's user).
/// Binary columns as hexadecimal (BUG-V6-04): the copy is text a client
/// reads back whatever the bytes of a BLOB or a hash.
pub const DUMP_ARGS: [&str; 6] = [
    "--single-transaction",
    "--hex-blob",
    "--routines",
    "--events",
    "--triggers",
    "--set-gtid-purged=OFF",
];

/// A copy of a project's database (D-78).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub file: String,
    pub engine: String,
    pub size: u64,
}

/// A new copy of database `name` in `dir` (BUG-V6-06):
/// `<name>-<yyyymmdd-hhmmss><ms><ext>`, never the name of another copy (two
/// exports in the same second), and its `<file>.part`, created here (the
/// name is taken) and where the dump is written: not listed among the
/// copies ([`DbEngine::of_backup`]) before [`finish_copy`] renames it.
fn new_copy(
    dir: &std::path::Path,
    name: &str,
    kind: DbEngine,
) -> Result<(PathBuf, PathBuf, fs::File)> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis())
    };
    for _ in 0..1000 {
        // The stamp and the milliseconds of the same second.
        let before = millis();
        let stamp = crate::agents::sync::stamp();
        if millis() / 1000 != before / 1000 {
            continue;
        }
        let file = dir.join(format!(
            "{name}-{stamp}{:03}{}",
            before % 1000,
            kind.backup_ext()
        ));
        let part = part_of(&file);
        if !file.exists() {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&part)
            {
                Ok(out) => return Ok((file, part, out)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(anyhow!("{}: {e}", part.display())),
            }
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    bail!("{}: no free name for a copy", dir.display())
}

/// `<file>.part`, where a copy is written before it is one.
fn part_of(file: &std::path::Path) -> PathBuf {
    let mut name = file.as_os_str().to_owned();
    name.push(".part");
    PathBuf::from(name)
}

/// A dump done: its `.part` becomes the copy. A dump failed or cut short
/// (time limit, disk full): its `.part` goes, never left to be restored
/// with `--drop` (BUG-V6-06).
fn finish_copy(
    part: &std::path::Path,
    file: &std::path::Path,
    dumped: Result<()>,
) -> Result<PathBuf> {
    let renamed =
        dumped.and_then(|()| fs::rename(part, file).with_context(|| file.display().to_string()));
    if let Err(e) = renamed {
        let _ = fs::remove_file(part);
        return Err(e);
    }
    Ok(file.to_path_buf())
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
                let engine = DbEngine::of_backup(&name)?;
                Some(Backup {
                    file: name,
                    engine: engine.id().to_owned(),
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
        let kind = DbEngine::parse(engine)?;
        let engine = kind.id();
        let line = project_line(&self.kit, project, engine);
        let (port, env) = self.running_at(kind, line.as_deref())?;
        // The dump runs as root/admin: only the database the kit made for
        // this project, never one another project (or someone else) uses.
        let name = self.own_database(project, engine)?;
        let dir = self.backup_dir(project)?;
        let (file, part, out) = new_copy(&dir, &name, kind)?;
        let dump = kind.dump_bin();
        let written = || -> Result<()> {
            match kind {
                DbEngine::MongoDb => {
                    drop(out);
                    let (mut cmd, config) = self.mongo_tool(dump, &env, port)?;
                    cmd.arg(format!("--db={name}"))
                        .arg(format!("--archive={}", part.display()))
                        .arg("--gzip");
                    let result = run_with_input(cmd, "", Duration::from_secs(600));
                    let _ = fs::remove_file(config);
                    result.context(dump)
                }
                DbEngine::MySql => {
                    let mut cmd = self.sql_tool_at(kind, dump, &env, line.as_deref(), port);
                    cmd.args(DUMP_ARGS)
                        .arg(&name)
                        .stdin(Stdio::null())
                        .stdout(Stdio::from(out))
                        .stderr(Stdio::piped());
                    let child = jobs::spawn(&mut cmd)?;
                    let done = child.wait_with_output()?;
                    if !done.status.success() {
                        bail!("{dump}: {}", String::from_utf8_lossy(&done.stderr).trim());
                    }
                    Ok(())
                }
            }
        };
        finish_copy(&part, &file, written())
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
        let Some(engine) = DbEngine::of_backup(file) else {
            bail!("not a database copy: {file}")
        };
        let id = engine.id();
        let restore = engine.restore_bin();
        let name = self.own_database(project, id)?;
        match engine {
            DbEngine::MongoDb => {
                let line = project_line(&self.kit, project, id);
                let (port, env) = self.running_at(engine, line.as_deref())?;
                let (mut cmd, config) = self.mongo_tool(restore, &env, port)?;
                cmd.arg(format!("--archive={}", path.display()))
                    .arg("--gzip")
                    .arg("--drop")
                    .arg(format!("--nsInclude={name}.*"));
                let result = run_with_input(cmd, "", Duration::from_secs(600));
                let _ = fs::remove_file(config);
                result.context(restore)
            }
            DbEngine::MySql => {
                // As the project's user, never root: the file may come from
                // outside (D-78) and name other databases or `mysql.*`.
                let password = auth::try_project_password(&self.kit, project, id)?
                    .ok_or_else(|| anyhow!("the project has no {id} database yet"))?;
                let line = project_line(&self.kit, project, id);
                let (port, env) = self.running_at(engine, line.as_deref())?;
                // Streamed, never whole in memory nor read as UTF-8 text
                // (BUG-V6-04), without the `DEFINER` of root's dump: the
                // views, triggers, routines and events become the project
                // user's, who may create them (BUG-V6-01).
                let sql = super::mysql::WithoutDefiner::new(std::io::BufReader::new(
                    fs::File::open(&path).with_context(|| path.display().to_string())?,
                ));
                let (mut cmd, login) = self.sql_as_project(
                    engine,
                    restore,
                    &env,
                    line.as_deref(),
                    port,
                    &name,
                    &password,
                )?;
                cmd.arg(&name);
                let result = run_with_reader(cmd, sql, Duration::from_secs(600));
                let _ = fs::remove_file(login);
                result.context(restore)
            }
        }
    }

    /// The name of the database the kit made for `project` on `engine`, for
    /// a dump or a restore that runs with the server's administrator: only
    /// when the project has one (a stored password) and no other project
    /// uses the same name; never a database the kit did not make for it.
    fn own_database(&self, project: &str, engine: &str) -> Result<String> {
        if auth::try_project_password(&self.kit, project, engine)?.is_none() {
            bail!("the project has no {engine} database yet: create it first");
        }
        let name = auth::project_db_name(&self.kit, project, engine)?;
        auth::check_name_free(&self.kit, project, engine, &name)?;
        Ok(name)
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/db/services/backups.rs"]
mod tests;
