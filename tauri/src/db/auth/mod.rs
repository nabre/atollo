//! Passwords of the kit's databases (DECISIONS.md D-74): the `root` of
//! MySQL, MongoDB's `admin` and one user per project and database, each
//! with a random password.
//!
//! - the administrators' in `data\app\db\secrets.json`, a project's in
//!   `data\projects\<project>\private\db.json` (D-89), readable by the owner
//!   only where the disk has permissions (NTFS; FAT32/exFAT sticks have none);
//! - `data\app\db\mysql.cnf` holds the `[client]` login of MySQL's `root`
//!   (D-110): the kit's `mysql`, `mysqldump` and `mysqladmin` launchers
//!   (`data\kit\bin`) pass it with `--defaults-extra-file` (read after the
//!   other files, it wins) and the app's tools with `--defaults-file`, so
//!   they log in without a password on the command line;
//! - passwords never go on a command line (other users can read those):
//!   SQL goes through stdin, mongosh reads them from its environment;
//! - a project's `.env` gets its connection only after the user saw the
//!   lines that change (`env_changes`, then `env_apply`).
//!
//! Here the administrators' passwords and MySQL's login files; the lock of
//! `db.json` in `lock`, a project's `db.json` in `project`, the names of
//! its databases in `names`, the SQL and mongosh scripts in `scripts`, its
//! `.env` in `env`.

mod env;
mod lock;
mod names;
mod project;
mod scripts;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::fsx::try_read_json;
use crate::kit::Kit;
pub use env::{
    env_apply, env_block, env_changes, env_ignored, env_set, env_source, env_wanted, project_db,
    EnvChange, ProjectDb,
};
pub(crate) use lock::db_json_lock;
#[allow(unused_imports)] // the tests, here and elsewhere
pub use names::db_name;
pub(crate) use names::{add_legacy_names, keep_legacy_names};
pub use names::{check_name_free, is_reserved, is_valid_name, project_db_name};
pub use project::{
    add_project_used_line, drop_project, ensure_project_db, project_line, project_passwords,
    project_used_lines, set_project_line, try_project_password, try_project_passwords,
};
#[allow(unused_imports)] // the tests and callers on other branches
pub use project::{project_password, set_project_password};
pub use scripts::{
    mysql_root_sql, project_drop_sql, project_sql, MONGO_ADMIN_JS, MONGO_DROP_JS, MONGO_FCV_JS,
    MONGO_PROJECT_JS, MONGO_SHUTDOWN_JS,
};

/// The administrators' passwords. The projects' are in their own `db.json`;
/// keys this version does not know stay as they are, the `projects` of an
/// older kit included (moved to the projects' files by the migration).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secrets {
    /// MySQL's `root` (D-110).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mysql_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mongodb_admin: Option<String>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

pub fn dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("db")
}

pub fn secrets_path(kit: &Kit) -> PathBuf {
    dir(kit).join("secrets.json")
}

/// The administrators' passwords, none when nothing is saved yet. A file
/// that is there but cannot be read or understood (damaged, held a moment
/// by an antivirus) is an error, never "no password" (SEC-V6-01): a new
/// one written over it would lock the kit out of its own data.
pub fn try_load(kit: &Kit) -> Result<Secrets> {
    Ok(try_read_json(&secrets_path(kit))?.unwrap_or_default())
}

/// [`try_load`] for what only shows the passwords or checks for one: a
/// file that cannot be read gives none. Never the start of a write
/// ([`update_secrets`]).
pub fn load(kit: &Kit) -> Secrets {
    try_load(kit).unwrap_or_default()
}

/// Reads `secrets.json`, lets `change` modify it and writes it when it
/// changed, holding the lock of `db.json` ([`db_json_lock`]): the first
/// starts of MySQL and MongoDB at once (each with its own turn) never write
/// back what they read and lose the other's password. Refused, nothing
/// written, when the file is there but cannot be read (SEC-V6-01); an error
/// from `change` leaves the file as it is.
pub fn update_secrets<T>(kit: &Kit, change: impl FnOnce(&mut Secrets) -> Result<T>) -> Result<T> {
    let _one = db_json_lock(kit)?;
    let before = try_load(kit)?;
    let mut secrets = before.clone();
    let out = change(&mut secrets)?;
    if secrets != before {
        save(kit, &secrets)?;
    }
    Ok(out)
}

/// Written in a folder made private first, so neither the temporary file of
/// `write_atomic` nor the file is ever readable by others, not even for a
/// moment (security review of D-74); the file's own DACL is set again after.
fn write_private(path: &Path, text: &str) -> Result<()> {
    crate::router::write_private(path, text)
}

/// The atomic write and the file's own DACL, in a folder already private
/// (`ProjectState::private_dir` sets its DACL before returning it).
fn write_in_private(path: &Path, text: &str) -> Result<()> {
    crate::router::fs::write_in_private(path, text)
}

/// A private file of `data\app\db` (a tool's config with a password).
pub fn write_private_text(path: &Path, text: &str) -> Result<()> {
    write_private(path, text)
}

pub fn save(kit: &Kit, secrets: &Secrets) -> Result<()> {
    write_private(&secrets_path(kit), &serde_json::to_string_pretty(secrets)?)
}

/// 32 hex characters: nothing to escape in SQL, ini files or URLs.
pub fn new_password() -> Result<String> {
    Ok(crate::editor::random_token()?[..32].to_owned())
}

/// The `[client]` login of `root` for the kit's MySQL, on the port of the
/// version the server runs (D-110, [`write_mysql_cnf`]).
pub fn mysql_cnf(kit: &Kit) -> PathBuf {
    dir(kit).join("mysql.cnf")
}

/// The login of another version of MySQL, on its own port
/// (D-102): the `ATOLLO_MYSQL_CNF` of the programs of a project whose
/// database is on it (`services::project_client_env`).
pub fn mysql_line_cnf(kit: &Kit, line: &str) -> PathBuf {
    dir(kit).join("mysql-lines").join(line).join("mysql.cnf")
}

/// MySQL's `[client]` login: TCP on 127.0.0.1 (never a socket or a named
/// pipe of another MySQL of the PC), no paths, so nothing to escape (a
/// `\` is an escape in MySQL's option files). LF ends: read the same on
/// every system.
pub fn mysql_cnf_text(port: u16, password: &str) -> String {
    format!(
        "# Atollo: login of the kit's MySQL (written by the app).\n\
         [client]\nuser=root\npassword={password}\nhost=127.0.0.1\n\
         port={port}\nprotocol=TCP\n"
    )
}

/// [`mysql_cnf`] for `port`.
pub fn write_mysql_cnf(kit: &Kit, port: u16, password: &str) -> Result<()> {
    write_private(&mysql_cnf(kit), &mysql_cnf_text(port, password))
}

/// [`mysql_line_cnf`] for version `line` on its `port`.
pub fn write_mysql_line_cnf(kit: &Kit, line: &str, port: u16, password: &str) -> Result<()> {
    if !crate::versions::valid_line(line) {
        bail!("not a version line: {line}");
    }
    write_private(&mysql_line_cnf(kit, line), &mysql_cnf_text(port, password))
}

#[cfg(test)]
#[path = "../../../../test/rust/db/auth/mod.rs"]
pub(crate) mod tests;
