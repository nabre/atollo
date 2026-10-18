//! Database servers of the kit (DECISIONS.md D-73): MySQL (D-110, the SQL
//! server of new kits), MariaDB (kept on the kits that run it, until its
//! data moves to MySQL, issue #3) and MongoDB, one instance each for every
//! project (as Laragon), started from the Database page or with Atollo.
//!
//! - a project's SQL database is on the server it was created on (a
//!   password stored for `mysql` or `mariadb`); a new one goes to the kit's
//!   SQL server ([`sql_engine`]): the callers that still say `mariadb`
//!   (detection, frameworks) reach it through [`project_engine`];
//!
//! - they listen on 127.0.0.1 only and ask for a password (D-74): MariaDB's
//!   `root` and MongoDB's `admin` get a random one at the first start, also
//!   on data made by an older Atollo (`db/auth.rs`); each project gets its
//!   own database and user (`project_db_create`);
//! - data in `data\db\<id>\<version>`, one folder per version the server
//!   runs (D-81: a setting of the server, the newest by default), created
//!   at its first start as a copy of the nearest older version's (forward
//!   only; MongoDB one major at a time, then its feature compatibility), or
//!   new (MariaDB with its `mariadb-install-db`); output in
//!   `data\app\logs\<id>.log`;
//! - the other installed versions can run at the same time, each on a port
//!   of its own (D-102: MariaDB 10.6 on 3307 beside 11.4 on 3306),
//!   for the projects that ask for them (`lines` of the project's
//!   `db.json`); their output in `data\app\logs\<id>-<version>.log`;
//! - each server runs in a job of its own, nested in the app's: closing the
//!   app ends it in any case; `stop` first asks the server to shut down
//!   (`mariadb-admin shutdown`, mongosh `shutdownServer`), so no recovery
//!   is needed at the next start;
//! - after a major update of MariaDB the system tables are brought up to
//!   date by hand with `mariadb-upgrade` (run automatically it hung at
//!   "Fixing views", tried 14/10/2026);
//! - port and "start with Atollo" of each server, and the ports of its
//!   other versions, in `data\app\services.json`.
//!
//! Here the settings and the life of a server (start, state, stop); the data
//! folders per version in `data`, the passwords and the projects' databases
//! (created, and dropped with the project) in `access`, the copies in `backups`, the command-line tools in `tools`.

mod access;
mod backups;
mod data;
pub mod mysql;
mod tools;

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::{auth, DbEngine};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::run::{is_port_free, responds};
use crate::setup::catalog::Catalog;
use access::admin_password;
pub use backups::Backup;
use data::seed;
pub use data::{data_dir, data_dir_at, migrate_data};
use tools::run_tool;

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

const READY_WITHIN: Duration = Duration::from_secs(90);
const STOP_WITHIN: Duration = Duration::from_secs(20);

/// [`DbEngine::default_port`] of `id`; an id that is not a server gets
/// MariaDB's, as before the engines were an enum.
pub fn default_port(id: &str) -> u16 {
    DbEngine::from_id(id)
        .unwrap_or(DbEngine::MariaDb)
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
    /// The ports of the other versions (D-102), version → port:
    /// stored at their first start or set on the page; one not stored gets
    /// the first free one after `port` ([`line_ports`]).
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
    running: Mutex<BTreeMap<Key, Server>>,
    errors: Mutex<BTreeMap<Key, String>>,
    phpmyadmin: crate::db::phpmyadmin::PhpMyAdmin,
}

/// The version of `id` the server runs: the one in its settings while it is
/// on disk, otherwise the newest on disk.
pub fn server_line(kit: &Kit, id: &str) -> Option<String> {
    let c = Catalog::get().find(id)?;
    load_settings(kit)
        .get(id)
        .and_then(|s| s.line.clone())
        .filter(|l| crate::versions::is_present(kit, c, l))
        .or_else(|| crate::versions::newest_line(kit, c))
}

/// The installed versions of `id`, newest first.
pub fn server_lines(kit: &Kit, id: &str) -> Vec<String> {
    let Some(c) = Catalog::get().find(id) else {
        return Vec::new();
    };
    let mut lines: Vec<String> = crate::versions::default_line(kit, c)
        .into_iter()
        .chain(crate::versions::saved(kit, c).into_iter().map(|l| l.line))
        .collect();
    lines.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
    lines
}

/// The port of every version in `lines`: `main` (the version the server
/// runs) on the server's port, the others on their stored port, or on the
/// first free one after it (3307, 3308…), in the order of `lines`. A
/// stored port that another version already has is given again.
pub fn line_ports(
    setting: &Setting,
    id: &str,
    main: Option<&str>,
    lines: &[String],
) -> BTreeMap<String, u16> {
    let port = setting.port.unwrap_or_else(|| default_port(id));
    let mut taken = vec![port];
    let mut out = BTreeMap::new();
    let others: Vec<&String> = lines.iter().filter(|l| Some(l.as_str()) != main).collect();
    for line in &others {
        if let Some(p) = setting.ports.get(*line).filter(|p| !taken.contains(p)) {
            taken.push(*p);
            out.insert((*line).clone(), *p);
        }
    }
    let mut next = port;
    for line in others {
        if out.contains_key(line) {
            continue;
        }
        loop {
            next = next.checked_add(1).unwrap_or(1024).max(1024);
            if !taken.contains(&next) {
                break;
            }
        }
        taken.push(next);
        out.insert(line.clone(), next);
    }
    if let Some(m) = main {
        out.insert(m.to_owned(), port);
    }
    out
}

/// The port of version `line` of `id` (None: the version the server runs).
pub fn line_port(kit: &Kit, id: &str, line: Option<&str>) -> u16 {
    let setting = load_settings(kit).get(id).cloned().unwrap_or_default();
    let main = server_line(kit, id);
    match line.filter(|l| Some(*l) != main.as_deref()) {
        Some(l) => line_ports(&setting, id, main.as_deref(), &server_lines(kit, id))
            .get(l)
            .copied()
            .unwrap_or_else(|| setting.port.unwrap_or_else(|| default_port(id))),
        None => setting.port.unwrap_or_else(|| default_port(id)),
    }
}

/// The version of `engine` the project's database is on: the one chosen
/// for it (`lines` of its `db.json`), otherwise the one the server runs.
pub fn project_line(kit: &Kit, project: &str, engine: &str) -> Option<String> {
    auth::project_line(kit, project, engine)
        .unwrap_or_else(|e| {
            crate::log::warn(&format!("{project}: {engine}: {e:#}"));
            None
        })
        .or_else(|| server_line(kit, engine))
}

/// The kit's SQL server (D-110, issue #3): MySQL, unless the kit runs
/// MariaDB and has no MySQL, where MariaDB stays until its data moves
/// (issue #3): a new project there gets its database next to the others.
pub fn sql_engine(kit: &Kit) -> DbEngine {
    sql_engine_with(|id| installed(kit, id))
}

/// [`sql_engine`] with the installed components given by `installed`.
pub fn sql_engine_with(installed: impl Fn(&str) -> bool) -> DbEngine {
    if !installed(DbEngine::MySql.component()) && installed(DbEngine::MariaDb.component()) {
        DbEngine::MariaDb
    } else {
        DbEngine::MySql
    }
}

/// The component a need for a SQL server installs (issue #3): `mariadb` or
/// `mysql` (of phpMyAdmin's `needs`, a framework's `database`, the
/// detection of a project, its actions, its containers) is the kit's SQL
/// server ([`sql_engine`]), so a new kit gets MySQL and a kit that runs
/// only MariaDB keeps it; any other need is itself. An explicit choice of
/// a component (`prepare --only mariadb`, its card) is never resolved.
#[allow(dead_code)] // the tests; the needs outside `db` call it (issue #3)
pub fn sql_need(kit: &Kit, need: &str) -> String {
    sql_need_with(need, |id| installed(kit, id)).to_string()
}

/// [`sql_need`] with the installed components given by `installed`.
pub fn sql_need_with(need: &str, installed: impl Fn(&str) -> bool) -> &str {
    match need {
        "mariadb" | "mysql" => sql_engine_with(installed).component(),
        other => other,
    }
}

/// The SQL server of the project's database: the one it was created on (a
/// password stored, MySQL first), otherwise the kit's ([`sql_engine`]).
pub fn project_sql_engine(kit: &Kit, project: &str) -> DbEngine {
    project_sql_db(kit, project).unwrap_or_else(|| sql_engine(kit))
}

/// The server a request for the project's `engine` reaches: for a SQL one
/// (`mariadb` of the detection and the frameworks, `mysql`), the project's
/// SQL server ([`project_sql_engine`]); MongoDB as it is.
pub fn project_engine(kit: &Kit, project: &str, engine: DbEngine) -> DbEngine {
    if engine.is_sql() {
        project_sql_engine(kit, project)
    } else {
        engine
    }
}

/// The SQL server the project's database is on, when it has one.
fn project_sql_db(kit: &Kit, project: &str) -> Option<DbEngine> {
    let stored = auth::project_passwords(kit, project);
    DbEngine::SQL
        .into_iter()
        .find(|e| stored.contains_key(e.id()))
}

/// The port of the project's SQL database, when it has one: the `DB_PORT`
/// of its programs (D-102).
pub fn project_db_port(kit: &Kit, project: &str) -> Option<u16> {
    let id = project_sql_db(kit, project)?.id();
    Some(line_port(
        kit,
        id,
        project_line(kit, project, id).as_deref(),
    ))
}

/// The login of the project's programs when its SQL database is on another
/// version than the server's, written at that version's start (`mariadb`
/// or `mysql` of its scripts then log in on its port): `MYSQL_HOME` with
/// MariaDB's `my.ini`, or `ATOLLO_MYSQL_CNF` (read by the kit's launchers,
/// `setup/kit_files.rs`) and `CP_MYSQL`, the folder of that version. Empty:
/// the kit's own (`env.rs`), on the server's port.
pub fn project_client_env(kit: &Kit, project: &str) -> Vec<(&'static str, PathBuf)> {
    let Some(engine) = project_sql_db(kit, project) else {
        return Vec::new();
    };
    let id = engine.id();
    let Some(line) = auth::project_line(kit, project, id).ok().flatten() else {
        return Vec::new();
    };
    if Some(&line) == server_line(kit, id).as_ref() {
        return Vec::new();
    }
    match engine {
        DbEngine::MariaDb => {
            let dir = auth::line_client_dir(kit, &line);
            if dir.join("my.ini").is_file() {
                vec![("MYSQL_HOME", dir)]
            } else {
                Vec::new()
            }
        }
        DbEngine::MySql => {
            let cnf = auth::mysql_line_cnf(kit, &line);
            if cnf.is_file() {
                vec![
                    ("ATOLLO_MYSQL_CNF", cnf),
                    ("CP_MYSQL", line_dir(kit, id, Some(&line))),
                ]
            } else {
                Vec::new()
            }
        }
        DbEngine::MongoDb => Vec::new(),
    }
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

/// Shown on the Database page: every server but MariaDB, which only a kit
/// that has it (installed, data, settings) shows (D-110).
fn shown(kit: &Kit, engine: DbEngine, settings: &BTreeMap<String, Setting>) -> bool {
    match engine {
        DbEngine::MariaDb => {
            installed(kit, engine.id())
                || data::has_any_data(kit, engine)
                || settings.contains_key(engine.id())
        }
        DbEngine::MySql | DbEngine::MongoDb => true,
    }
}

/// The output of `line`: `<id>.log` for the version the server runs,
/// `<id>-<line>.log` for the others.
fn log_path(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    match line.filter(|l| Some(*l) != server_line(kit, id).as_deref()) {
        Some(l) => kit.logs().join(format!("{id}-{l}.log")),
        None => kit.logs().join(format!("{id}.log")),
    }
}

fn settings_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("services.json")
}

pub fn load_settings(kit: &Kit) -> BTreeMap<String, Setting> {
    crate::fsx::read_json(&settings_path(kit)).unwrap_or_default()
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
            running: Mutex::new(BTreeMap::new()),
            errors: Mutex::new(BTreeMap::new()),
            phpmyadmin: crate::db::phpmyadmin::PhpMyAdmin::default(),
        }
    }

    pub(crate) fn kit(&self) -> &Kit {
        &self.kit
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

    pub fn list(&self) -> Vec<ServiceInfo> {
        let settings = load_settings(&self.kit);
        let secrets = auth::load(&self.kit);
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        DbEngine::ALL
            .into_iter()
            .filter(|e| shown(&self.kit, *e, &settings))
            .map(|engine| {
                let id = engine.id();
                let setting = settings.get(id).cloned().unwrap_or_default();
                let line = server_line(&self.kit, id);
                let lines = server_lines(&self.kit, id);
                let ports = line_ports(&setting, id, line.as_deref(), &lines);
                let key = Self::key(engine, line.as_deref());
                let (state, port) = self.state_of(
                    &mut running,
                    &mut errors,
                    &key,
                    setting.port.unwrap_or_else(|| engine.default_port()),
                );
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

    /// The versions of `engine` running now.
    fn running_lines(&self, engine: DbEngine) -> Vec<String> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        running
            .iter_mut()
            .filter_map(|((e, l), s)| {
                (*e == engine && matches!(s.child.try_wait(), Ok(None))).then(|| l.clone())
            })
            .collect()
    }

    fn command(
        &self,
        engine: DbEngine,
        line: Option<&str>,
        env: &KitEnv,
        data: &std::path::Path,
        port: u16,
    ) -> Command {
        let server = bin_at(&self.kit, engine.component(), line, engine.server_bin());
        let mut cmd = jobs::kit_command(server, env);
        match engine {
            DbEngine::MongoDb => {
                cmd.arg("--dbpath")
                    .arg(data)
                    .arg("--port")
                    .arg(port.to_string())
                    .arg("--bind_ip")
                    .arg("127.0.0.1")
                    .arg("--auth");
            }
            DbEngine::MariaDb => {
                // The ini written by mariadb-install-db first (it must be the
                // first option); port and address from here win over it.
                cmd.arg(format!("--defaults-file={}", data.join("my.ini").display()))
                    .arg(format!("--datadir={}", data.display()))
                    .arg(format!("--port={port}"))
                    .arg("--bind-address=127.0.0.1")
                    .arg("--console");
            }
            DbEngine::MySql => {
                // Only the kit's my.cnf (`data::prepare`, port and address
                // in it); errors on stderr, which the log gets (on Windows
                // they would go to a file of the data folder).
                cmd.arg(format!("--defaults-file={}", data.join("my.cnf").display()));
                if self.kit.os().is_windows() {
                    cmd.arg("--console");
                }
            }
        }
        cmd
    }

    /// The server `engine` names for the project (`mariadb` of the callers
    /// that know no other: its SQL server, [`project_engine`]).
    pub(crate) fn engine_for(&self, project: &str, engine: &str) -> Result<DbEngine> {
        Ok(project_engine(&self.kit, project, DbEngine::parse(engine)?))
    }

    /// Starts `id` and waits until it answers on its port. Errors: not
    /// installed, port in use (`PortInUse`), or what the log says.
    pub fn start(&self, id: &str, env: &KitEnv) -> Result<u16> {
        self.start_line(id, None, env)
    }

    /// Starts version `line` of `id` (None: the version the server runs),
    /// on its own port beside the others (D-102).
    pub fn start_line(&self, id: &str, line: Option<&str>, env: &KitEnv) -> Result<u16> {
        self.start_engine(DbEngine::parse(id)?, line, env)
    }

    /// [`Services::start_line`] of `engine`.
    pub(crate) fn start_engine(
        &self,
        engine: DbEngine,
        line: Option<&str>,
        env: &KitEnv,
    ) -> Result<u16> {
        let id = engine.id();
        // The version named first: one uninstalled is told as such
        // (`LineMissing`), the interface offers to install it.
        if let Some(l) = line {
            check_line(&self.kit, id, l)?;
        }
        if !installed(&self.kit, id) {
            bail!("{id} is not installed");
        }
        let line = self.resolve(engine, line);
        let key = Self::key(engine, line.as_deref());
        if let Some(s) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(&key)
        {
            if matches!(s.child.try_wait(), Ok(None)) {
                return Ok(s.port);
            }
        }
        let port = line_port(&self.kit, id, line.as_deref());
        if !is_port_free(port) {
            return Err(PortInUse(port).into());
        }
        let main = line == server_line(&self.kit, id);
        if !main {
            if let Some(l) = &line {
                self.keep_port(id, l, port)?;
            }
        }
        let result = self.launch(engine, line.as_deref(), main, env, port);
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        match &result {
            Ok(_) => {
                errors.remove(&key);
            }
            Err(e) => {
                errors.insert(key, format!("{e:#}"));
            }
        }
        result
    }

    /// The port given to another version at its first start, stored so
    /// that versions installed later do not move it.
    fn keep_port(&self, id: &str, line: &str, port: u16) -> Result<()> {
        let mut all = load_settings(&self.kit);
        let entry = all.entry(id.to_owned()).or_default();
        if entry.ports.get(line) == Some(&port) {
            return Ok(());
        }
        entry.ports.insert(line.to_owned(), port);
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    fn launch(
        &self,
        engine: DbEngine,
        line: Option<&str>,
        main: bool,
        env: &KitEnv,
        port: u16,
    ) -> Result<u16> {
        let id = engine.id();
        // The first start of a version: the data of the nearest older one,
        // never while that one runs (its files are in use).
        let seeded = match line {
            Some(l) => seed(&self.kit, id, l, &self.running_lines(engine))?,
            None => None,
        };
        let key = Self::key(engine, line);
        let (data, fresh) = self.prepare(engine, line, env, port)?;
        let log = log_path(&self.kit, id, line);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
            .with_context(|| log.display().to_string())?;
        let mut cmd = self.command(engine, line, env, &data, port);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(file.try_clone()?))
            .stderr(Stdio::from(file));
        let child = jobs::spawn(&mut cmd)?;
        let job = Job::new()?;
        job.assign(&child)?;
        crate::log::info(&format!(
            "{id} {}: started on 127.0.0.1:{port}",
            line.unwrap_or_default()
        ));
        self.running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                key.clone(),
                Server {
                    child,
                    port,
                    env: env.clone(),
                    _job: job,
                },
            );
        let start = Instant::now();
        loop {
            if responds(port) {
                break;
            }
            let exited = self
                .running
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .get_mut(&key)
                .is_none_or(|s| !matches!(s.child.try_wait(), Ok(None)));
            if exited {
                self.running
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&key);
                bail!("{}", log_tail(&self.kit, id, line));
            }
            if start.elapsed() > READY_WITHIN {
                self.stop_engine(engine, line);
                bail!(
                    "no answer on port {port} within {} s",
                    READY_WITHIN.as_secs()
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        if let Err(e) = self.secure(engine, line, main, env, port, fresh) {
            self.stop_engine(engine, line);
            return Err(e);
        }
        if let (Some(from), Some(line)) = (&seeded, line) {
            self.after_copy(engine, env, port, from, line);
        }
        Ok(port)
    }

    /// The server's port and environment, when it runs.
    pub(crate) fn running_server(&self, engine: DbEngine) -> Result<(u16, KitEnv)> {
        self.running_at(engine, None)
    }

    /// Port and environment of version `line` of `engine` (None: the
    /// version the server runs), when it runs.
    pub(crate) fn running_at(&self, engine: DbEngine, line: Option<&str>) -> Result<(u16, KitEnv)> {
        let line = self.resolve(engine, line);
        let key = Self::key(engine, line.as_deref());
        let name = match &line {
            Some(l) => format!("{engine} {l}"),
            None => engine.id().to_owned(),
        };
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        match running.get_mut(&key) {
            Some(s) => match s.child.try_wait() {
                Ok(None) => Ok((s.port, s.env.clone())),
                _ => bail!("{name} is not running"),
            },
            None => bail!("{name} is not running"),
        }
    }

    /// The address that opens phpMyAdmin logged in to the running SQL
    /// `engine` (D-107), version `line` (None: the one the server runs), in
    /// `lang`.
    pub fn phpmyadmin_url(
        &self,
        engine: DbEngine,
        line: Option<&str>,
        lang: &str,
    ) -> Result<String> {
        if !engine.is_sql() {
            bail!("phpMyAdmin does not open {engine}");
        }
        let (port, env) = self.running_at(engine, line)?;
        let password = admin_password(&auth::load(&self.kit), engine)
            .ok_or_else(|| anyhow::anyhow!("{engine} has no password of root yet"))?;
        self.phpmyadmin.open(&self.kit, &env, port, &password, lang)
    }

    /// Asks the server to shut down, then ends whatever is left.
    #[allow(dead_code)] // the tests; the commands name the version
    pub fn stop(&self, id: &str) {
        self.stop_line(id, None);
    }

    /// [`Services::stop`] for version `line` (None: the one it runs); an id
    /// that is not a server has nothing running.
    pub fn stop_line(&self, id: &str, line: Option<&str>) {
        if let Some(engine) = DbEngine::from_id(id) {
            self.stop_engine(engine, line);
        }
    }

    /// [`Services::stop_line`] of `engine`.
    pub(crate) fn stop_engine(&self, engine: DbEngine, line: Option<&str>) {
        let line = self.resolve(engine, line);
        self.stop_key(&Self::key(engine, line.as_deref()));
    }

    fn stop_key(&self, key: &Key) {
        let Some(mut server) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(key)
        else {
            return;
        };
        let (engine, line) = (key.0, (!key.1.is_empty()).then_some(key.1.as_str()));
        let id = engine.id();
        let ask = match engine {
            DbEngine::MongoDb => {
                let mut cmd = jobs::kit_command(client_at(&self.kit, engine, None), &server.env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(server.port.to_string())
                    .args(["--eval", auth::MONGO_SHUTDOWN_JS])
                    .env(
                        "ATOLLO_DB_ADMIN",
                        auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
                    );
                cmd
            }
            DbEngine::MariaDb | DbEngine::MySql => {
                // phpMyAdmin logged in to this version ends with it.
                self.phpmyadmin.stop_for(server.port);
                let admin = engine.admin_bin().unwrap_or(engine.client_bin());
                let mut cmd = self.sql_tool_at(engine, admin, &server.env, line, server.port);
                cmd.arg("shutdown");
                cmd
            }
        };
        // mongosh fails when the connection drops on shutdown: not an error.
        let _ = run_tool(ask, Duration::from_secs(15));
        let start = Instant::now();
        while matches!(server.child.try_wait(), Ok(None)) && start.elapsed() < STOP_WITHIN {
            std::thread::sleep(Duration::from_millis(200));
        }
        if matches!(server.child.try_wait(), Ok(None)) {
            crate::log::warn(&format!("{id} {}: did not stop, ended", key.1));
        }
        crate::log::info(&format!("{id} {}: stopped", key.1));
        // Dropping the server closes its job.
    }

    /// Every server and every version of them.
    pub fn stop_all(&self) {
        self.phpmyadmin.stop();
        let keys: Vec<Key> = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .keys()
            .cloned()
            .collect();
        for key in keys {
            self.stop_key(&key);
        }
    }

    /// Port (None: default) and "start with Atollo" of `id`.
    pub fn set(&self, id: &str, port: Option<u16>, autostart: bool) -> Result<()> {
        let engine = DbEngine::parse(id)?;
        if port.is_some_and(|p| p < 1024) {
            bail!("port below 1024");
        }
        let mut all = load_settings(&self.kit);
        let entry = all.entry(id.to_owned()).or_default();
        entry.port = port.filter(|p| *p != engine.default_port());
        entry.autostart = autostart;
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    /// The port of another version of `id` (None: the first free one after
    /// the server's), from its next start. Not the version the server runs
    /// (its port is the server's, [`Services::set`]) nor a port another
    /// version has.
    pub fn set_line_port(&self, id: &str, line: &str, port: Option<u16>) -> Result<()> {
        DbEngine::parse(id)?;
        let lines = server_lines(&self.kit, id);
        if !lines.iter().any(|x| x == line) {
            bail!("{id} {line} is not installed");
        }
        let main = server_line(&self.kit, id);
        if main.as_deref() == Some(line) {
            bail!("{id} {line} is the version the server runs: its port is the server's");
        }
        if port.is_some_and(|p| p < 1024) {
            bail!("port below 1024");
        }
        let mut all = load_settings(&self.kit);
        let entry = all.entry(id.to_owned()).or_default();
        if let Some(p) = port {
            let ports = line_ports(entry, id, main.as_deref(), &lines);
            if let Some((other, _)) = ports.iter().find(|(l, q)| **q == p && *l != line) {
                bail!("port {p} is the one of {id} {other}");
            }
            entry.ports.insert(line.to_owned(), p);
        } else {
            entry.ports.remove(line);
        }
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    /// The version `id` runs (None: the newest on disk), while it is
    /// stopped: each version has its own data (D-81).
    pub fn set_line(&self, id: &str, line: Option<&str>) -> Result<()> {
        let engine = DbEngine::parse(id)?;
        if let Some(l) = line {
            check_line(&self.kit, id, l)?;
        }
        // Any version, not only the one the server runs: another one on its
        // own port would become the server while every caller takes the
        // server's port for it.
        if self.running_server(engine).is_ok() || !self.running_lines(engine).is_empty() {
            bail!("{id} is running: stop it first");
        }
        let mut all = load_settings(&self.kit);
        let entry = all.entry(id.to_owned()).or_default();
        entry.line = line.map(str::to_owned);
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    /// The installed servers marked "start with Atollo": the version each
    /// runs, then the other installed versions the projects' databases are
    /// on ([`lines_in_use`]), each on its own port.
    pub fn autostart(&self, env: &KitEnv) {
        for (id, s) in load_settings(&self.kit) {
            let Some(engine) = DbEngine::from_id(&id) else {
                continue;
            };
            if s.autostart && installed(&self.kit, &id) {
                if let Err(e) = self.start(&id, env) {
                    crate::log::warn(&format!("{id}: automatic start: {e:#}"));
                }
                for line in lines_in_use(&self.kit, &id) {
                    if let Err(e) = self.start_engine(engine, Some(&line), env) {
                        crate::log::warn(&format!("{id} {line}: automatic start: {e:#}"));
                    }
                }
            }
        }
    }
}

/// The installed versions of `id`, other than the one the server runs, that
/// a project's database is on (`lines` of its `db.json`, with a database
/// created): those the automatic start starts beside the server. Newest
/// first; a version chosen and then uninstalled is left out.
pub fn lines_in_use(kit: &Kit, id: &str) -> Vec<String> {
    let main = server_line(kit, id);
    let wanted: std::collections::BTreeSet<String> = crate::projects::state::all(kit)
        .into_iter()
        .filter(|p| auth::project_passwords(kit, p).contains_key(id))
        .filter_map(|p| auth::project_line(kit, &p, id).ok().flatten())
        .filter(|l| Some(l) != main.as_ref())
        .collect();
    server_lines(kit, id)
        .into_iter()
        .filter(|l| wanted.contains(l))
        .collect()
}

/// The versions of `engine` to drop the project's database from: every one
/// it was created on (`used` of its `db.json`) and the one it is on now,
/// in that order, each once.
pub fn project_drop_lines(kit: &Kit, project: &str, engine: &str) -> Vec<String> {
    let mut lines = auth::project_used_lines(kit, project, engine).unwrap_or_else(|e| {
        crate::log::warn(&format!("{project}: {engine}: {e:#}"));
        Vec::new()
    });
    if let Some(now) = project_line(kit, project, engine) {
        lines.push(now);
    }
    let mut seen = std::collections::BTreeSet::new();
    lines.retain(|l| seen.insert(l.clone()));
    lines
}

/// The kit's servers for the deletion of `project` (D-89),
/// started with `env` when stopped: every version its databases are on
/// ([`project_drop_lines`]); a version no longer installed is skipped and
/// named in the log.
pub struct ProjectDrops<'a> {
    pub services: &'a Services,
    pub env: KitEnv,
    pub project: String,
}

impl crate::projects::ProjectDatabases for ProjectDrops<'_> {
    fn installed(&self, engine: &str) -> bool {
        installed(&self.services.kit, engine)
    }

    fn drop_db(&self, engine: &str, database: &str) -> Result<()> {
        let kit = &self.services.kit;
        let lines = project_drop_lines(kit, &self.project, engine);
        if lines.is_empty() {
            return self.services.db_drop_at(engine, None, database, &self.env);
        }
        let installed = server_lines(kit, engine);
        for line in &lines {
            if installed.contains(line) {
                self.services
                    .db_drop_at(engine, Some(line), database, &self.env)
                    .with_context(|| format!("{engine} {line}"))?;
            } else {
                crate::log::warn(&format!(
                    "{}: {engine} {line} not installed: database and user {database} left there",
                    self.project
                ));
            }
        }
        Ok(())
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
