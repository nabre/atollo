//! Database servers of the kit (DECISIONS.md D-73): MySQL (D-110, the SQL
//! server) and MongoDB, one instance each for every project (as Laragon),
//! started from the Database page or with Atollo.
//!
//! - they listen on 127.0.0.1 only and ask for a password (D-74): MySQL's
//!   `root` and MongoDB's `admin` get a random one at the first start, also
//!   on data made by an older Atollo (`db/auth/`); each project gets its
//!   own database and user (`project_db_create`);
//! - data in `data\db\<id>\<version>`, one folder per version the server
//!   runs (D-81: a setting of the server, the newest by default), created
//!   at its first start as a copy of the nearest older version's (forward
//!   only; MongoDB one major at a time, then its feature compatibility), or
//!   new; output in `data\app\logs\<id>.log`;
//! - the other installed versions can run at the same time, each on a port
//!   of its own (D-102: MySQL 8.4 on 3307 beside 9.7 on 3306),
//!   for the projects that ask for them (`lines` of the project's
//!   `db.json`); their output in `data\app\logs\<id>-<version>.log`;
//! - each server runs in a job of its own, nested in the app's: closing the
//!   app ends it in any case; `stop` first asks the server to shut down
//!   (`mysqladmin shutdown`, mongosh `shutdownServer`), so no recovery
//!   is needed at the next start;
//! - port and "start with Atollo" of each server, and the port of each of
//!   its versions, in `data\app\services.json`;
//! - one start, stop or change of version of a server at a time (BUG-V5-02).
//!
//! Here the types, the folders and programs of each version and the
//! Database page's list; `services.json` and the ports in `settings`, the
//! start and stop in `process`, the versions the projects' databases are on
//! in `projects`; the data folders per version in `data`, the passwords and
//! the projects' databases (created, and dropped with the project) in
//! `access`, the copies in `backups`, the command-line tools in `tools`.

mod access;
mod backups;
mod data;
pub mod mysql;
mod process;
mod projects;
mod settings;
mod tools;

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::Child;
use std::sync::Mutex;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{auth, DbEngine};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::Job;
use crate::ports::responds;
use crate::setup::Catalog;
use access::admin_password;
pub use backups::Backup;
pub use data::{data_dir, data_dir_at, migrate_data};
pub use projects::{lines_in_use, project_client_env, project_db_port, project_line, ProjectDrops};
pub use settings::{line_port, line_ports, load_settings, server_line, server_lines};

/// The ids of the servers, in the order of the page: [`DbEngine::ALL`], for
/// the callers that still hold an id (`commands`, `projects`, `versions`).
pub const SERVERS: [&str; DbEngine::ALL.len()] = {
    let mut ids = [""; DbEngine::ALL.len()];
    let mut i = 0;
    while i < ids.len() {
        ids[i] = DbEngine::ALL[i].id();
        i += 1;
    }
    ids
};

/// [`DbEngine::default_port`] of `id`; an id that is not a server gets
/// MySQL's.
pub fn default_port(id: &str) -> u16 {
    DbEngine::from_id(id)
        .unwrap_or(DbEngine::MySql)
        .default_port()
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    /// None: the default port.
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub autostart: bool,
    /// The version (`12.3`) the server runs; None or not on disk: the
    /// newest on disk (D-81).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    /// The port of each version (D-102), version → port: stored at its
    /// start (the version the server runs too) or set on the page, so that
    /// choosing another version for the server moves none; one not stored
    /// gets `port` (the version the server runs) or the first free one
    /// after it ([`line_ports`]).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub ports: BTreeMap<String, u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Stopped,
    Starting,
    Running,
    Error,
}

/// What the Database page shows of a server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub state: State,
    pub port: u16,
    pub autostart: bool,
    pub data_dir: String,
    pub log: String,
    /// Connection string of the administrator (no password in it).
    pub url: String,
    /// `root` or `admin`, and its password once set (first start).
    pub user: String,
    pub password: Option<String>,
    /// Why the last start failed.
    pub message: Option<String>,
    /// The version it runs, and those installed (newest first).
    pub line: Option<String>,
    pub lines: Vec<String>,
    /// The other installed versions, each with its own port
    /// (D-102), newest first.
    pub others: Vec<LineInfo>,
}

/// Another version of a server, started beside the one it runs.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LineInfo {
    pub line: String,
    pub state: State,
    pub port: u16,
    pub data_dir: String,
    pub log: String,
    pub url: String,
    pub message: Option<String>,
}

struct Server {
    child: Child,
    port: u16,
    /// The environment it started with, for the tools that stop it.
    env: KitEnv,
    // Dropped last: ends the server and whatever it started.
    _job: Job,
}

/// A running server: its engine and the version it runs (empty: none known).
type Key = (DbEngine, String);

pub struct Services {
    kit: Kit,
    /// One operation at a time on each engine (BUG-V5-02): a start, a stop
    /// or a change of version holds it from its first check to the end, so
    /// two starts of the same version run one server and give the same
    /// port. Taken before `running` and `errors`, never while holding them.
    ops: BTreeMap<DbEngine, Mutex<()>>,
    running: Mutex<BTreeMap<Key, Server>>,
    errors: Mutex<BTreeMap<Key, String>>,
    phpmyadmin: crate::db::phpmyadmin::PhpMyAdmin,
    mongo_express: crate::db::mongo_express::MongoExpress,
}

/// The folder of version `line` of the versioned component `id` (None: the
/// version it runs).
pub(crate) fn line_dir(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    let Some(c) = Catalog::get().find(id) else {
        return kit.apps().join(id);
    };
    let line = line.map(str::to_owned).or_else(|| server_line(kit, id));
    crate::versions::dir_for(kit, c, line.as_deref()).unwrap_or_else(|| kit.apps().join(id))
}

/// The output of `line`: `<id>.log` for the version the server runs,
/// `<id>-<line>.log` for the others.
fn log_path(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    match line.filter(|l| Some(*l) != server_line(kit, id).as_deref()) {
        Some(l) => kit.logs().join(format!("{id}-{l}.log")),
        None => kit.logs().join(format!("{id}.log")),
    }
}

fn installed(kit: &Kit, id: &str) -> bool {
    Catalog::get().find(id).is_some_and(|c| c.is_installed(kit))
}

#[cfg(test)]
pub(crate) fn bin(kit: &Kit, id: &str, exe: &str) -> PathBuf {
    bin_at(kit, id, None, exe)
}

/// A program of version `line` of component `id` (None: the version it
/// runs): `exe` is a stem (`.exe` added on Windows) or, as the tests write
/// it, a Windows file name. A versioned component (the servers) has one
/// folder per line; the others (`mongosh`, `mongodb-tools`, folders of the
/// MongoDB installer, not in the catalog) one folder.
pub(crate) fn bin_at(kit: &Kit, id: &str, line: Option<&str>, exe: &str) -> PathBuf {
    let stem = exe.strip_suffix(".exe").unwrap_or(exe);
    match Catalog::get().find(id).filter(|c| c.versioned) {
        Some(c) => {
            let line = line.map(str::to_owned).or_else(|| server_line(kit, id));
            let dir = crate::versions::dir_for(kit, c, line.as_deref())
                .unwrap_or_else(|| kit.apps().join(id));
            crate::router::bins::db_bin_in(kit.os(), &dir, stem)
        }
        None => crate::router::bins::db_bin(kit, id, stem),
    }
}

/// The client of `engine` ([`DbEngine::client_bin`]), of version `line`
/// when it is in the server's folder.
pub(crate) fn client_at(kit: &Kit, engine: DbEngine, line: Option<&str>) -> PathBuf {
    bin_at(kit, engine.client_component(), line, engine.client_bin())
}

/// The last lines of a server's log, to tell why it stopped.
fn log_tail(kit: &Kit, id: &str, line: Option<&str>) -> String {
    let text = fs::read_to_string(log_path(kit, id, line)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(3)..].join("\n")
}

impl Services {
    pub fn new(kit: Kit) -> Services {
        Services {
            kit,
            ops: DbEngine::ALL.map(|e| (e, Mutex::new(()))).into(),
            running: Mutex::new(BTreeMap::new()),
            errors: Mutex::new(BTreeMap::new()),
            phpmyadmin: crate::db::phpmyadmin::PhpMyAdmin::default(),
            mongo_express: crate::db::mongo_express::MongoExpress::default(),
        }
    }

    /// The turn of `engine` ([`Services::ops`]); not reentrant: what holds
    /// it calls [`Services::stop_key`], never [`Services::stop_engine`].
    fn op(&self, engine: DbEngine) -> std::sync::MutexGuard<'_, ()> {
        self.ops[&engine]
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The version `line` names, or the one the server runs.
    fn resolve(&self, engine: DbEngine, line: Option<&str>) -> Option<String> {
        line.map(str::to_owned)
            .or_else(|| server_line(&self.kit, engine.id()))
    }

    fn key(engine: DbEngine, line: Option<&str>) -> Key {
        (engine, line.unwrap_or_default().to_owned())
    }

    /// State and port of a server under `key` (`port` when it is not
    /// running); one that ended by itself is an error with its log.
    fn state_of(
        &self,
        running: &mut BTreeMap<Key, Server>,
        errors: &mut BTreeMap<Key, String>,
        key: &Key,
        port: u16,
    ) -> (State, u16) {
        let alive = running
            .get_mut(key)
            .map(|s| (matches!(s.child.try_wait(), Ok(None)), s.port));
        match alive {
            Some((true, p)) => (
                if responds(p) {
                    State::Running
                } else {
                    State::Starting
                },
                p,
            ),
            Some((false, _)) => {
                running.remove(key);
                let line = (!key.1.is_empty()).then_some(key.1.as_str());
                errors.insert(key.clone(), log_tail(&self.kit, key.0.id(), line));
                (State::Error, port)
            }
            None if errors.contains_key(key) => (State::Error, port),
            None => (State::Stopped, port),
        }
    }

    /// The servers of the Database page, [`DbEngine::ALL`].
    pub fn list(&self) -> Vec<ServiceInfo> {
        let settings = load_settings(&self.kit);
        let secrets = auth::load(&self.kit);
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        DbEngine::ALL
            .into_iter()
            .map(|engine| {
                let id = engine.id();
                let setting = settings.get(id).cloned().unwrap_or_default();
                let line = server_line(&self.kit, id);
                let lines = server_lines(&self.kit, id);
                let ports = line_ports(&setting, id, line.as_deref(), &lines);
                let key = Self::key(engine, line.as_deref());
                let server = setting.port.unwrap_or_else(|| engine.default_port());
                let main = line.as_ref().and_then(|l| ports.get(l)).copied();
                let (state, port) =
                    self.state_of(&mut running, &mut errors, &key, main.unwrap_or(server));
                let others = lines
                    .iter()
                    .filter(|l| Some(l.as_str()) != line.as_deref())
                    .map(|l| {
                        let key = Self::key(engine, Some(l));
                        let stored = ports.get(l).copied().unwrap_or(port);
                        let (state, port) = self.state_of(&mut running, &mut errors, &key, stored);
                        LineInfo {
                            line: l.clone(),
                            state,
                            port,
                            data_dir: data_dir_at(&self.kit, id, Some(l)).display().to_string(),
                            log: log_path(&self.kit, id, Some(l)).display().to_string(),
                            url: engine.admin_url(port),
                            message: errors.get(&key).cloned(),
                        }
                    })
                    .collect();
                ServiceInfo {
                    id: id.to_string(),
                    name: Catalog::get()
                        .find(id)
                        .map_or_else(|| id.to_string(), |c| c.name.clone()),
                    installed: installed(&self.kit, id),
                    state,
                    port,
                    autostart: setting.autostart,
                    data_dir: data_dir(&self.kit, id).display().to_string(),
                    log: log_path(&self.kit, id, None).display().to_string(),
                    url: engine.admin_url(port),
                    user: engine.admin_user().to_owned(),
                    password: admin_password(&secrets, engine),
                    message: errors.get(&key).cloned(),
                    line,
                    lines,
                    others,
                }
            })
            .collect()
    }

    /// Servers started by the app and still alive.
    pub fn running(&self) -> usize {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        running.retain(|_, s| matches!(s.child.try_wait(), Ok(None)));
        running.len()
    }
}

/// A version of a server that is not installed: chosen for a project's
/// database and then uninstalled, or asked by name.
#[derive(Debug)]
pub struct LineMissing {
    pub id: String,
    pub line: String,
}

impl std::fmt::Display for LineMissing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} is not installed", self.id, self.line)
    }
}

impl std::error::Error for LineMissing {}

/// [`LineMissing`] unless version `line` of `id` is installed.
fn check_line(kit: &Kit, id: &str, line: &str) -> Result<()> {
    if server_lines(kit, id).iter().any(|x| x == line) {
        return Ok(());
    }
    Err(LineMissing {
        id: id.to_owned(),
        line: line.to_owned(),
    }
    .into())
}

/// The port of a server is taken by another program.
#[derive(Debug)]
pub struct PortInUse(pub u16);

impl std::fmt::Display for PortInUse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "port {} in use", self.0)
    }
}

impl std::error::Error for PortInUse {}

#[cfg(test)]
#[path = "../../../../test/rust/db/services/mod.rs"]
mod tests;
