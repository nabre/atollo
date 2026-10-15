//! Database servers of the kit (DECISIONS.md D-73): MariaDB and MongoDB,
//! one instance each for every project (as Laragon), started from the
//! Database page or with Dev-Kit.
//!
//! - they listen on 127.0.0.1 only and ask for a password (D-74): MariaDB's
//!   `root` and MongoDB's `admin` get a random one at the first start, also
//!   on data made by an older Dev-Kit (`db/auth.rs`); each project gets its
//!   own database and user (`project_db_create`);
//! - data in `data\db\<id>\<version>`, one folder per version the server
//!   runs (D-81: a setting of the server, the newest by default), created
//!   at its first start as a copy of the nearest older version's (forward
//!   only; MongoDB one major at a time, then its feature compatibility), or
//!   new (MariaDB with its `mariadb-install-db`); output in
//!   `data\app\logs\<id>.log`;
//! - each server runs in a job of its own, nested in the app's: closing the
//!   app ends it in any case; `stop` first asks the server to shut down
//!   (`mariadb-admin shutdown`, mongosh `shutdownServer`), so no recovery
//!   is needed at the next start;
//! - after a major update of MariaDB the system tables are brought up to
//!   date by hand with `mariadb-upgrade` (run automatically it hung at
//!   "Fixing views", tried 14/10/2026);
//! - port and "start with Dev-Kit" of each server in `data\app\services.json`.
//!
//! Here the settings and the life of a server (start, state, stop); the data
//! folders per version in `data`, the passwords and the projects' databases
//! (created, and dropped with the project) in `access`, the copies in `backups`, the command-line tools in `tools`.

mod access;
mod backups;
mod data;
mod tools;

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::auth;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::run::{is_port_free, responds};
use crate::setup::catalog::Catalog;
use access::{admin_password, admin_user};
pub use backups::Backup;
use data::seed;
pub use data::{data_dir, migrate_data};
use tools::run_tool;

/// The servers, in the order of the page.
pub const SERVERS: [&str; 2] = ["mariadb", "mongodb"];

const READY_WITHIN: Duration = Duration::from_secs(90);
const STOP_WITHIN: Duration = Duration::from_secs(20);

pub fn default_port(id: &str) -> u16 {
    match id {
        "mongodb" => 27017,
        _ => 3306,
    }
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
}

struct Server {
    child: Child,
    port: u16,
    /// The environment it started with, for the tools that stop it.
    env: KitEnv,
    // Dropped last: ends the server and whatever it started.
    _job: Job,
}

pub struct Services {
    kit: Kit,
    running: Mutex<BTreeMap<String, Server>>,
    errors: Mutex<BTreeMap<String, String>>,
    adminer: crate::db::adminer::Adminer,
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

fn log_path(kit: &Kit, id: &str) -> PathBuf {
    kit.logs().join(format!("{id}.log"))
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

pub(crate) fn bin(kit: &Kit, id: &str, exe: &str) -> PathBuf {
    let versioned = Catalog::get().find(id).filter(|c| c.versioned);
    match (id, versioned) {
        ("mongosh", _) => kit.apps().join(r"mongosh\bin").join(exe),
        // The version the server runs.
        (_, Some(c)) => crate::versions::dir_for(kit, c, server_line(kit, id).as_deref())
            .unwrap_or_else(|| kit.apps().join(id))
            .join("bin")
            .join(exe),
        _ => kit.apps().join(id).join("bin").join(exe),
    }
}

pub fn url(id: &str, port: u16) -> String {
    match id {
        "mongodb" => format!("mongodb://admin@127.0.0.1:{port}/?authSource=admin"),
        _ => format!("mysql://root@127.0.0.1:{port}/"),
    }
}

/// The last lines of a server's log, to tell why it stopped.
fn log_tail(kit: &Kit, id: &str) -> String {
    let text = fs::read_to_string(log_path(kit, id)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(3)..].join("\n")
}

impl Services {
    pub fn new(kit: Kit) -> Services {
        Services {
            kit,
            running: Mutex::new(BTreeMap::new()),
            errors: Mutex::new(BTreeMap::new()),
            adminer: crate::db::adminer::Adminer::default(),
        }
    }

    pub(crate) fn kit(&self) -> &Kit {
        &self.kit
    }

    fn port(&self, id: &str) -> u16 {
        load_settings(&self.kit)
            .get(id)
            .and_then(|s| s.port)
            .unwrap_or_else(|| default_port(id))
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        let settings = load_settings(&self.kit);
        let secrets = auth::load(&self.kit);
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        SERVERS
            .iter()
            .map(|id| {
                let setting = settings.get(*id).cloned().unwrap_or_default();
                let mut port = setting.port.unwrap_or_else(|| default_port(id));
                let alive = running
                    .get_mut(*id)
                    .map(|s| (matches!(s.child.try_wait(), Ok(None)), s.port));
                let state = match alive {
                    Some((true, p)) => {
                        port = p;
                        if responds(p) {
                            State::Running
                        } else {
                            State::Starting
                        }
                    }
                    Some((false, _)) => {
                        running.remove(*id);
                        errors.insert(id.to_string(), log_tail(&self.kit, id));
                        State::Error
                    }
                    None if errors.contains_key(*id) => State::Error,
                    None => State::Stopped,
                };
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
                    log: log_path(&self.kit, id).display().to_string(),
                    url: url(id, port),
                    user: admin_user(id).to_owned(),
                    password: admin_password(&secrets, id),
                    message: errors.get(*id).cloned(),
                    line: server_line(&self.kit, id),
                    lines: server_lines(&self.kit, id),
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

    fn command(&self, id: &str, env: &KitEnv, data: &std::path::Path, port: u16) -> Command {
        match id {
            "mongodb" => {
                let mut cmd = jobs::kit_command(bin(&self.kit, id, "mongod.exe"), env);
                cmd.arg("--dbpath")
                    .arg(data)
                    .arg("--port")
                    .arg(port.to_string())
                    .arg("--bind_ip")
                    .arg("127.0.0.1")
                    .arg("--auth");
                cmd
            }
            _ => {
                let mut cmd = jobs::kit_command(bin(&self.kit, id, "mariadbd.exe"), env);
                // The ini written by mariadb-install-db first (it must be the
                // first option); port and address from here win over it.
                cmd.arg(format!("--defaults-file={}", data.join("my.ini").display()))
                    .arg(format!("--datadir={}", data.display()))
                    .arg(format!("--port={port}"))
                    .arg("--bind-address=127.0.0.1")
                    .arg("--console");
                cmd
            }
        }
    }

    /// Starts `id` and waits until it answers on its port. Errors: not
    /// installed, port in use (`PortInUse`), or what the log says.
    pub fn start(&self, id: &str, env: &KitEnv) -> Result<u16> {
        if !SERVERS.contains(&id) {
            bail!("unknown server {id}");
        }
        if !installed(&self.kit, id) {
            bail!("{id} is not installed");
        }
        if let Some(s) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(id)
        {
            if matches!(s.child.try_wait(), Ok(None)) {
                return Ok(s.port);
            }
        }
        let port = self.port(id);
        if !is_port_free(port) {
            return Err(PortInUse(port).into());
        }
        let result = self.launch(id, env, port);
        let mut errors = self.errors.lock().unwrap_or_else(|e| e.into_inner());
        match &result {
            Ok(_) => {
                errors.remove(id);
            }
            Err(e) => {
                errors.insert(id.to_owned(), format!("{e:#}"));
            }
        }
        result
    }

    fn launch(&self, id: &str, env: &KitEnv, port: u16) -> Result<u16> {
        // The first start of a version: the data of the nearest older one.
        let line = server_line(&self.kit, id);
        let seeded = match &line {
            Some(l) => seed(&self.kit, id, l)?,
            None => None,
        };
        let (data, fresh) = self.prepare(id, env, port)?;
        let log = log_path(&self.kit, id);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
            .with_context(|| log.display().to_string())?;
        let mut cmd = self.command(id, env, &data, port);
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(file.try_clone()?))
            .stderr(Stdio::from(file));
        let child = jobs::spawn(&mut cmd)?;
        let job = Job::new()?;
        job.assign(&child)?;
        crate::log::info(&format!("{id}: started on 127.0.0.1:{port}"));
        self.running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(
                id.to_owned(),
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
                .get_mut(id)
                .is_none_or(|s| !matches!(s.child.try_wait(), Ok(None)));
            if exited {
                self.running
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(id);
                bail!("{}", log_tail(&self.kit, id));
            }
            if start.elapsed() > READY_WITHIN {
                self.stop(id);
                bail!(
                    "no answer on port {port} within {} s",
                    READY_WITHIN.as_secs()
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        if let Err(e) = self.secure(id, env, port, fresh) {
            self.stop(id);
            return Err(e);
        }
        if let (Some(from), Some(line)) = (&seeded, &line) {
            self.after_copy(id, env, port, from, line);
        }
        Ok(port)
    }

    /// The server's port and environment, when it runs.
    pub(crate) fn running_server(&self, engine: &str) -> Result<(u16, KitEnv)> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        match running.get_mut(engine) {
            Some(s) => match s.child.try_wait() {
                Ok(None) => Ok((s.port, s.env.clone())),
                _ => bail!("{engine} is not running"),
            },
            None => bail!("{engine} is not running"),
        }
    }

    /// The address that opens Adminer logged in to the running MariaDB
    /// (D-85).
    pub fn adminer_url(&self) -> Result<String> {
        let (port, env) = self.running_server("mariadb")?;
        self.adminer.open(&self.kit, &env, port)
    }

    /// Asks the server to shut down, then ends whatever is left.
    pub fn stop(&self, id: &str) {
        if id == "mariadb" {
            self.adminer.stop();
        }
        let Some(mut server) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(id)
        else {
            return;
        };
        let ask = match id {
            "mongodb" => {
                let mut cmd =
                    jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &server.env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(server.port.to_string())
                    .args(["--eval", auth::MONGO_SHUTDOWN_JS])
                    .env(
                        "DEVKIT_DB_ADMIN",
                        auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
                    );
                cmd
            }
            _ => {
                let mut cmd = self.mariadb_tool("mariadb-admin.exe", &server.env);
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
            crate::log::warn(&format!("{id}: did not stop, ended"));
        }
        crate::log::info(&format!("{id}: stopped"));
        // Dropping the server closes its job.
    }

    pub fn stop_all(&self) {
        for id in SERVERS {
            self.stop(id);
        }
    }

    /// Port (None: default) and "start with Dev-Kit" of `id`.
    pub fn set(&self, id: &str, port: Option<u16>, autostart: bool) -> Result<()> {
        if !SERVERS.contains(&id) {
            bail!("unknown server {id}");
        }
        if port.is_some_and(|p| p < 1024) {
            bail!("port below 1024");
        }
        let mut all = load_settings(&self.kit);
        let line = all.get(id).and_then(|s| s.line.clone());
        all.insert(
            id.to_owned(),
            Setting {
                port: port.filter(|p| *p != default_port(id)),
                autostart,
                line,
            },
        );
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    /// The version `id` runs (None: the newest on disk), while it is
    /// stopped: each version has its own data (D-81).
    pub fn set_line(&self, id: &str, line: Option<&str>) -> Result<()> {
        if !SERVERS.contains(&id) {
            bail!("unknown server {id}");
        }
        if let Some(l) = line {
            if !server_lines(&self.kit, id).iter().any(|x| x == l) {
                bail!("{id} {l} is not installed");
            }
        }
        if self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get_mut(id)
            .is_some_and(|s| matches!(s.child.try_wait(), Ok(None)))
        {
            bail!("{id} is running: stop it first");
        }
        let mut all = load_settings(&self.kit);
        let entry = all.entry(id.to_owned()).or_default();
        entry.line = line.map(str::to_owned);
        crate::fsx::write_json(&settings_path(&self.kit), &all)
    }

    /// The installed servers marked "start with Dev-Kit".
    pub fn autostart(&self, env: &KitEnv) {
        for (id, s) in load_settings(&self.kit) {
            if s.autostart && SERVERS.contains(&id.as_str()) && installed(&self.kit, &id) {
                if let Err(e) = self.start(&id, env) {
                    crate::log::warn(&format!("{id}: automatic start: {e:#}"));
                }
            }
        }
    }
}

/// The kit's servers for the deletion of a project (PLAN-DATA.md WP-5),
/// started with `env` when stopped.
pub struct ProjectDrops<'a> {
    pub services: &'a Services,
    pub env: KitEnv,
}

impl crate::projects::ProjectDatabases for ProjectDrops<'_> {
    fn installed(&self, engine: &str) -> bool {
        installed(&self.services.kit, engine)
    }

    fn drop_db(&self, engine: &str, database: &str) -> Result<()> {
        self.services.db_drop(engine, database, &self.env)
    }
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn settings_round_trip_and_defaults() {
        let tmp = TempDir::new("services");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let s = Services::new(kit.clone());
        assert_eq!(s.port("mariadb"), 3306);
        assert_eq!(s.port("mongodb"), 27017);
        s.set("mariadb", Some(3307), true).unwrap();
        s.set("mongodb", Some(27017), false).unwrap();
        let all = load_settings(&kit);
        assert_eq!(
            all["mariadb"],
            Setting {
                port: Some(3307),
                autostart: true,
                line: None
            }
        );
        assert_eq!(all["mongodb"].port, None, "the default port is not stored");
        assert!(s.set("mariadb", Some(80), false).is_err());
        assert!(s.set("redis", None, false).is_err());
        let list = s.list();
        assert_eq!(list[0].port, 3307);
        assert_eq!(list[0].url, "mysql://root@127.0.0.1:3307/");
        assert_eq!(
            list[1].url,
            "mongodb://admin@127.0.0.1:27017/?authSource=admin"
        );
        assert!(list.iter().all(|i| i.password.is_none()), "no password yet");
        assert!(s.project_dbs("shop").is_empty());
        assert!(
            s.project_db_create("shop", "mariadb").is_err(),
            "server not running"
        );
        assert!(list
            .iter()
            .all(|i| i.state == State::Stopped && !i.installed));
        assert!(
            s.start("mariadb", &KitEnv::default()).is_err(),
            "not installed"
        );
        assert!(
            s.project_db_drop("shop", "mariadb", &KitEnv::default())
                .is_err(),
            "not installed: nothing dropped"
        );
        assert_eq!(s.running(), 0);
    }

    /// A real MariaDB and MongoDB of the kit given by DEVKIT_TEST_KIT, on
    /// free ports: start, answer, stop with nothing left.
    #[test]
    fn real_databases_start_and_stop() {
        let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
            eprintln!("DEVKIT_TEST_KIT not set: skipped");
            return;
        };
        let kit = Kit::at(PathBuf::from(root), Arch::detect());
        kit.prepare().unwrap();
        let env = KitEnv::build(
            &kit,
            &crate::settings::Settings::default(),
            "en",
            &crate::env::Host::capture(),
        );
        let s = Services::new(kit.clone());
        for (id, port) in [("mariadb", 13306), ("mongodb", 37017)] {
            if !installed(&kit, id) {
                eprintln!("{id} not installed: skipped");
                continue;
            }
            let before = load_settings(&kit).get(id).cloned().unwrap_or_default();
            s.set(id, Some(port), false).unwrap();
            assert_eq!(s.start(id, &env).unwrap(), port, "{id}");
            assert!(responds(port), "{id} answers");
            let info = s.list().into_iter().find(|i| i.id == id).unwrap();
            assert_eq!(info.state, State::Running);
            assert!(info.password.is_some(), "{id}: administrator password set");
            let db = s.project_db_create("Prova Kit", id).unwrap();
            assert_eq!(db.database, "prova_kit");
            let again = s.project_db_create("Prova Kit", id).unwrap();
            assert_eq!(again.password, db.password, "{id}: same password");
            assert_eq!(
                s.project_dbs("Prova Kit")
                    .iter()
                    .filter(|d| d.engine == id)
                    .count(),
                1
            );
            // Export, lose the data, restore: the data is back (D-78).
            let admin = |script: &str| -> String {
                match id {
                    "mongodb" => {
                        let mut cmd = jobs::kit_command(bin(&kit, "mongosh", "mongosh.exe"), &env);
                        cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                            .arg(port.to_string())
                            .args(["--eval", &format!(
                                "db.getSiblingDB('admin').auth('admin', process.env.DEVKIT_DB_ADMIN); {script}"
                            )])
                            .env("DEVKIT_DB_ADMIN", auth::load(&kit).mongodb_admin.unwrap());
                        let out = cmd.output().unwrap();
                        String::from_utf8_lossy(&out.stdout).trim().to_owned()
                    }
                    _ => {
                        let mut cmd = s.mariadb_tool("mariadb.exe", &env);
                        cmd.args(["-N", "-e", script]);
                        let out = cmd.output().unwrap();
                        String::from_utf8_lossy(&out.stdout).trim().to_owned()
                    }
                }
            };
            let (fill, count, lose) = match id {
                "mongodb" => (
                    "db.getSiblingDB('prova_kit').t.insertOne({x: 7})",
                    "print(db.getSiblingDB('prova_kit').t.countDocuments())",
                    "db.getSiblingDB('prova_kit').t.drop()",
                ),
                _ => (
                    "CREATE TABLE IF NOT EXISTS prova_kit.t (x INT); INSERT INTO prova_kit.t VALUES (7);",
                    "SELECT COUNT(*) FROM prova_kit.t",
                    "DROP TABLE prova_kit.t",
                ),
            };
            admin(fill);
            let n = admin(count);
            assert!(
                n.parse::<u32>().is_ok_and(|n| n > 0),
                "{id}: data written, count {n:?}"
            );
            // The explorer sees the database, the table and its rows (D-79).
            let dbs = crate::db::browse::tree(&s, id).unwrap();
            let db = dbs
                .iter()
                .find(|d| d.name == "prova_kit")
                .expect("database listed");
            assert!(
                db.tables.iter().any(|t| t.name == "t"),
                "{id}: table listed"
            );
            assert!(
                dbs.iter().any(|d| d.system),
                "{id}: system databases listed"
            );
            let page = crate::db::browse::rows(&s, id, "prova_kit", "t", 0).unwrap();
            assert!(page.total > 0 && !page.rows.is_empty(), "{id}: rows read");
            assert!(page.columns.iter().any(|c| c.name == "x"), "{id}: column x");
            if id == "mariadb" {
                assert!(crate::db::browse::rows(&s, id, "prova_kit", "no`; DROP", 0).is_err());
            }
            // Adminer logged in with the token only, ended with MariaDB (D-85).
            let adminer = (id == "mariadb" && installed(&kit, "adminer")).then(|| {
                let url = s.adminer_url().unwrap();
                crate::db::adminer::tests::assert_logs_in(&url);
                assert_eq!(s.adminer_url().unwrap(), url, "the same Adminer");
                url
            });
            let file = s.export("Prova Kit", id).unwrap();
            assert!(file.is_file(), "{id}: copy written");
            admin(lose);
            assert_ne!(admin(count), n, "{id}: data lost before restore");
            let name = file.file_name().unwrap().to_string_lossy().into_owned();
            assert!(s
                .backups("Prova Kit")
                .iter()
                .any(|b| b.file == name && b.engine == id));
            s.restore("Prova Kit", &name).unwrap();
            assert_eq!(admin(count), n, "{id}: data back after restore");
            assert!(s.restore("Prova Kit", r"..\x.sql").is_err());
            let _ = fs::remove_file(&file);
            // Deleting the project drops its database and user: a stopped
            // server is started for it and stopped again (PLAN-DATA.md WP-5).
            s.stop(id);
            s.project_db_drop("Prova Kit", id, &env).unwrap();
            assert!(!responds(port), "{id}: stopped again after the drop");
            s.start(id, &env).unwrap();
            assert!(
                !crate::db::browse::tree(&s, id)
                    .unwrap()
                    .iter()
                    .any(|d| d.name == "prova_kit"),
                "{id}: database dropped"
            );
            s.project_db_drop("Prova Kit", id, &env).unwrap();
            assert!(responds(port), "{id}: a running server stays running");
            s.stop(id);
            assert!(!responds(port), "{id} stopped");
            if let Some(url) = adminer {
                assert!(ureq::get(&url).call().is_err(), "Adminer stopped");
            }
            assert_eq!(s.running(), 0);
            s.set(id, before.port, before.autostart).unwrap();
        }
    }
}
