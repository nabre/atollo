//! Passwords of the kit's databases (DECISIONS.md D-74): MariaDB's `root`,
//! MongoDB's `admin` and one user per project and database, each with a
//! random password.
//!
//! - kept in `data\app\db\secrets.json`, readable by the owner only where
//!   the disk has permissions (NTFS; FAT32/exFAT sticks have none);
//! - `data\app\db\my.ini` holds the `[client]` login of `root`: the kit's
//!   `MYSQL_HOME` points there, so `mariadb` in the kit's terminal and the
//!   app's own tools log in without a password on the command line;
//! - passwords never go on a command line (other users can read those):
//!   SQL goes through stdin, mongosh reads them from its environment;
//! - a project's `.env` gets its connection only after the user saw the
//!   lines that change (`env_changes`, then `env_apply`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::kit::{write_atomic, Kit};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secrets {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mariadb_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mongodb_admin: Option<String>,
    /// project → engine (`mariadb`, `mongodb`) → password of its user.
    #[serde(default)]
    pub projects: BTreeMap<String, BTreeMap<String, String>>,
}

pub fn dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("db")
}

fn secrets_path(kit: &Kit) -> PathBuf {
    dir(kit).join("secrets.json")
}

pub fn client_ini(kit: &Kit) -> PathBuf {
    dir(kit).join("my.ini")
}

pub fn load(kit: &Kit) -> Secrets {
    fs::read_to_string(secrets_path(kit))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Written in a folder made private first, so neither the temporary file of
/// `write_atomic` nor the file is ever readable by others, not even for a
/// moment (security review of D-74); the file's own DACL is set again after.
fn write_private(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        if let Err(e) = crate::editor::restrict_dir_to_owner(parent) {
            crate::log::warn(&format!("{}: permissions: {e:#}", parent.display()));
        }
    }
    write_atomic(path, text.as_bytes())?;
    if let Err(e) = crate::editor::restrict_to_owner(path) {
        crate::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
    Ok(())
}

pub fn save(kit: &Kit, secrets: &Secrets) -> Result<()> {
    write_private(&secrets_path(kit), &serde_json::to_string_pretty(secrets)?)
}

/// 32 hex characters: nothing to escape in SQL, ini files or URLs.
pub fn new_password() -> Result<String> {
    Ok(crate::editor::random_token()?[..32].to_owned())
}

/// The `[client]` login of `root` for the kit's MariaDB on `port`.
pub fn write_client_ini(kit: &Kit, port: u16, password: &str) -> Result<()> {
    write_private(
        &client_ini(kit),
        &format!(
            "# Atollo: login of the kit's MariaDB (written by the app).\r\n\
             [client]\r\nuser=root\r\npassword={password}\r\nhost=127.0.0.1\r\n\
             port={port}\r\nprotocol=TCP\r\n"
        ),
    )
}

/// SQL that gives every `root` account the password (Windows creates
/// `localhost`, `127.0.0.1` and `::1`).
pub fn root_password_sql(password: &str) -> String {
    format!(
        "ALTER USER IF EXISTS 'root'@'localhost' IDENTIFIED BY '{password}', \
         'root'@'127.0.0.1' IDENTIFIED BY '{password}', \
         'root'@'::1' IDENTIFIED BY '{password}';\nFLUSH PRIVILEGES;\n"
    )
}

/// Name of a project's database and user: lower case letters, digits and
/// `_`, at most 64 characters (MariaDB's limit for databases).
pub fn db_name(project: &str) -> String {
    let mut name: String = project
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    name.truncate(64);
    if name.trim_matches('_').is_empty() {
        name = "progetto".into();
    }
    name
}

/// SQL that creates (or brings back to `password`) a project's database and
/// user, with every right on that database only.
pub fn project_sql(name: &str, password: &str) -> String {
    let mut sql = format!("CREATE DATABASE IF NOT EXISTS `{name}` CHARACTER SET utf8mb4;\n");
    for host in ["localhost", "127.0.0.1", "::1"] {
        sql.push_str(&format!(
            "CREATE USER IF NOT EXISTS '{name}'@'{host}' IDENTIFIED BY '{password}';\n\
             ALTER USER '{name}'@'{host}' IDENTIFIED BY '{password}';\n\
             GRANT ALL PRIVILEGES ON `{name}`.* TO '{name}'@'{host}';\n"
        ));
    }
    sql.push_str("FLUSH PRIVILEGES;\n");
    sql
}

/// mongosh script for a project's user (names and passwords from the
/// environment: `ATOLLO_DB_ADMIN`, `ATOLLO_DB_NAME`, `ATOLLO_DB_PASSWORD`).
pub const MONGO_PROJECT_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); \
    const n = process.env.ATOLLO_DB_NAME, p = process.env.ATOLLO_DB_PASSWORD; \
    const d = db.getSiblingDB(n); \
    if (d.getUser(n)) { d.updateUser(n, { pwd: p }); } \
    else { d.createUser({ user: n, pwd: p, roles: [{ role: 'dbOwner', db: n }] }); }";

/// mongosh script for the first `admin` (MongoDB's localhost exception: it
/// works while the server has no users).
pub const MONGO_ADMIN_JS: &str = "db.getSiblingDB('admin').createUser({ user: 'admin', \
    pwd: process.env.ATOLLO_DB_PASSWORD, roles: ['root'] })";

pub const MONGO_SHUTDOWN_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); a.shutdownServer()";

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

pub fn project_db(engine: &str, project: &str, password: &str, port: u16) -> ProjectDb {
    let name = db_name(project);
    let url = match engine {
        "mongodb" => {
            format!("mongodb://{name}:{password}@127.0.0.1:{port}/{name}?authSource={name}")
        }
        _ => format!("mysql://{name}:{password}@127.0.0.1:{port}/{name}"),
    };
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
/// file uses them (MariaDB), otherwise `DATABASE_URL` / `MONGODB_URI`.
pub fn env_wanted(text: &str, db: &ProjectDb) -> Vec<(String, String)> {
    let laravel = text
        .lines()
        .any(|l| l.trim_start().starts_with("DB_CONNECTION="));
    let pairs: Vec<(&str, String)> = match db.engine.as_str() {
        "mongodb" => vec![("MONGODB_URI", db.url.clone())],
        _ if laravel => vec![
            ("DB_CONNECTION", "mysql".into()),
            ("DB_HOST", db.host.clone()),
            ("DB_PORT", db.port.to_string()),
            ("DB_DATABASE", db.database.clone()),
            ("DB_USERNAME", db.user.clone()),
            ("DB_PASSWORD", db.password.clone()),
        ],
        _ => vec![("DATABASE_URL", db.url.clone())],
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
    let wanted = env_wanted(text, db);
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
    crate::safe_path::read_text(project_dir, &project_dir.join(".env"))
        .or_else(|| crate::safe_path::read_text(project_dir, &project_dir.join(".env.example")))
        .unwrap_or_default()
}

/// `.env` is listed in the project's `.gitignore` (it will hold a password).
pub fn env_ignored(project_dir: &Path) -> bool {
    crate::safe_path::read_text(project_dir, &project_dir.join(".gitignore"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .any(|l| matches!(l, ".env" | "/.env" | ".env*" | "*.env" | ".env.*"))
}

pub fn env_apply(project_dir: &Path, db: &ProjectDb) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("no project folder");
    }
    if crate::safe_path::is_link(&project_dir.join(".env")) {
        bail!(".env is a link: left alone");
    }
    let text = env_apply_text(&env_source(project_dir), db);
    write_atomic(&project_dir.join(".env"), text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_and_sql() {
        assert_eq!(db_name("Negozio-Online"), "negozio_online");
        assert_eq!(db_name("app & co (1)"), "app___co__1_");
        assert_eq!(db_name("---"), "progetto");
        assert_eq!(db_name(&"x".repeat(80)).len(), 64);
        let pw = "0123456789abcdef0123456789abcdef";
        let sql = project_sql("shop", pw);
        assert!(sql.contains("CREATE DATABASE IF NOT EXISTS `shop`"));
        assert!(sql.contains("GRANT ALL PRIVILEGES ON `shop`.* TO 'shop'@'127.0.0.1'"));
        assert!(root_password_sql(pw).contains("'root'@'::1' IDENTIFIED BY"));
    }

    #[test]
    fn env_for_laravel_and_others() {
        let db = project_db("mariadb", "shop", "pw1", 3307);
        let laravel =
            "APP_NAME=Shop\r\n# DB_HOST=old\r\nDB_CONNECTION=sqlite\r\nDB_HOST=10.0.0.1\r\n";
        let changes = env_changes(laravel, &db);
        assert_eq!(changes[0].key, "DB_CONNECTION");
        assert_eq!(changes[0].old.as_deref(), Some("sqlite"));
        assert!(changes
            .iter()
            .any(|c| c.key == "DB_PASSWORD" && c.old.is_none()));
        let out = env_apply_text(laravel, &db);
        assert!(
            out.contains("# DB_HOST=old\r\n"),
            "comments untouched: {out}"
        );
        assert!(out.contains("DB_HOST=127.0.0.1\r\nDB_PORT=3307") || out.contains("DB_PORT=3307"));
        assert!(out.contains("DB_DATABASE=shop\r\n"));
        assert!(out.starts_with("APP_NAME=Shop\r\n"));
        assert!(env_changes(&out, &db).is_empty(), "applied: nothing left");
        let node = project_db("mongodb", "shop", "pw2", 27017);
        let out = env_apply_text("PORT=3000\n", &node);
        assert_eq!(
            out,
            "PORT=3000\nMONGODB_URI=mongodb://shop:pw2@127.0.0.1:27017/shop?authSource=shop\n"
        );
        let other = env_apply_text("", &db);
        assert_eq!(
            other.trim(),
            "DATABASE_URL=mysql://shop:pw1@127.0.0.1:3307/shop"
        );
    }

    #[test]
    fn secrets_round_trip_and_gitignore() {
        let tmp = crate::kit::tests::TempDir::new("db-auth");
        let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64);
        kit.prepare().unwrap();
        let mut s = load(&kit);
        assert_eq!(s, Secrets::default());
        s.mariadb_root = Some(new_password().unwrap());
        s.projects
            .entry("shop".into())
            .or_default()
            .insert("mariadb".into(), "x".into());
        save(&kit, &s).unwrap();
        assert_eq!(load(&kit), s);
        assert_eq!(s.mariadb_root.as_ref().unwrap().len(), 32);
        write_client_ini(&kit, 3306, "abc").unwrap();
        assert!(fs::read_to_string(client_ini(&kit))
            .unwrap()
            .contains("password=abc\r\nhost=127.0.0.1\r\nport=3306"));
        let p = tmp.0.join("p");
        fs::create_dir_all(&p).unwrap();
        assert!(!env_ignored(&p));
        fs::write(p.join(".gitignore"), "/vendor\n.env\n").unwrap();
        assert!(env_ignored(&p));
        fs::write(p.join(".env.example"), "DB_CONNECTION=mysql\n").unwrap();
        env_apply(&p, &project_db("mariadb", "p", "z", 3306)).unwrap();
        assert!(fs::read_to_string(p.join(".env"))
            .unwrap()
            .contains("DB_PASSWORD=z"));
    }
}
