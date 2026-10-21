//! A project's `.env`: the connection to its database, shown as changes, then written.

use std::path::Path;

use anyhow::{bail, Result};
use serde::Serialize;

use crate::db::DbEngine;
use crate::fsx::write_atomic;

/// A project's connection to one of the kit's databases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDb {
    pub engine: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub url: String,
}

/// The connection to the database and user `name` ([`project_db_name`]).
pub fn project_db(engine: &str, name: &str, password: &str, port: u16) -> ProjectDb {
    let name = name.to_owned();
    // An id this version does not know: a SQL server, as before the engines
    // were an enum.
    let url = DbEngine::from_id(engine)
        .unwrap_or(DbEngine::MySql)
        .project_url(&name, password, port);
    ProjectDb {
        engine: engine.to_owned(),
        database: name.clone(),
        user: name,
        password: password.to_owned(),
        host: "127.0.0.1".into(),
        port,
        url,
    }
}

/// A line of `.env` that changes: `old` None = added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvChange {
    pub key: String,
    pub old: Option<String>,
    pub new: String,
}

fn env_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.trim().split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}

/// The keys `db` sets in a `.env` like `text`: Laravel's `DB_*` when the
/// file uses them (MySQL), otherwise `DATABASE_URL` / `MONGODB_URI`.
pub fn env_wanted(text: &str, db: &ProjectDb) -> Vec<(String, String)> {
    let laravel = text
        .lines()
        .any(|l| l.trim_start().starts_with("DB_CONNECTION="));
    // An id this version does not know: a SQL server, as before the engines
    // were an enum.
    let engine = DbEngine::from_id(&db.engine).unwrap_or(DbEngine::MySql);
    let pairs: Vec<(&str, String)> = match engine {
        DbEngine::MongoDb if laravel => vec![
            ("DB_CONNECTION", "mongodb".into()),
            ("DB_DATABASE", db.database.clone()),
            ("MONGODB_URI", db.url.clone()),
        ],
        DbEngine::MongoDb => vec![("MONGODB_URI", db.url.clone())],
        DbEngine::MySql if laravel => vec![
            ("DB_CONNECTION", "mysql".into()),
            ("DB_HOST", db.host.clone()),
            ("DB_PORT", db.port.to_string()),
            ("DB_DATABASE", db.database.clone()),
            ("DB_USERNAME", db.user.clone()),
            ("DB_PASSWORD", db.password.clone()),
        ],
        DbEngine::MySql => vec![("DATABASE_URL", db.url.clone())],
    };
    pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}

pub fn env_changes(text: &str, db: &ProjectDb) -> Vec<EnvChange> {
    env_wanted(text, db)
        .into_iter()
        .filter_map(|(key, new)| {
            let old = env_value(text, &key);
            (old.as_deref() != Some(new.as_str())).then_some(EnvChange { key, old, new })
        })
        .collect()
}

/// `text` with the keys of `db` set: lines replaced where they are,
/// missing keys added at the end; comments and other lines untouched.
pub fn env_apply_text(text: &str, db: &ProjectDb) -> String {
    env_set_text(text, &env_wanted(text, db))
}

/// `text` with the keys of `wanted` set: a line with the key is replaced
/// (comments stay), a missing key is added at the end.
pub fn env_set_text(text: &str, wanted: &[(String, String)]) -> String {
    let mut done: Vec<bool> = vec![false; wanted.len()];
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out: Vec<String> = text
        .lines()
        .map(|line| {
            let key = line
                .trim()
                .split_once('=')
                .map(|(k, _)| k.trim().to_owned());
            match key.and_then(|k| wanted.iter().position(|(w, _)| *w == k)) {
                Some(i) if !line.trim_start().starts_with('#') => {
                    done[i] = true;
                    format!("{}={}", wanted[i].0, wanted[i].1)
                }
                _ => line.to_owned(),
            }
        })
        .collect();
    for (i, (k, v)) in wanted.iter().enumerate() {
        if !done[i] {
            out.push(format!("{k}={v}"));
        }
    }
    let mut s = out.join(newline);
    s.push_str(newline);
    s
}

/// The project's `.env`, created from `.env.example` when missing.
pub fn env_source(project_dir: &Path) -> String {
    crate::os::safe_path::read_text(project_dir, &project_dir.join(".env"))
        .or_else(|| crate::os::safe_path::read_text(project_dir, &project_dir.join(".env.example")))
        .unwrap_or_default()
}

/// `.env` is listed in the project's `.gitignore` (it will hold a password).
pub fn env_ignored(project_dir: &Path) -> bool {
    crate::os::safe_path::read_text(project_dir, &project_dir.join(".gitignore"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .any(|l| matches!(l, ".env" | "/.env" | ".env*" | "*.env" | ".env.*"))
}

pub fn env_apply(project_dir: &Path, db: &ProjectDb) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("no project folder");
    }
    if crate::os::safe_path::is_link(&project_dir.join(".env")) {
        bail!(".env is a link: left alone");
    }
    let text = env_apply_text(&env_source(project_dir), db);
    write_atomic(&project_dir.join(".env"), text.as_bytes())?;
    Ok(())
}

/// Sets `wanted` in the project's `.env` (made from `.env.example`, or new).
pub fn env_set(project_dir: &Path, wanted: &[(String, String)]) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("no project folder");
    }
    if crate::os::safe_path::is_link(&project_dir.join(".env")) {
        bail!(".env is a link: left alone");
    }
    let text = env_set_text(&env_source(project_dir), wanted);
    write_atomic(&project_dir.join(".env"), text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/db/auth/env.rs"]
mod tests;
