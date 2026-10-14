//! Database servers of the kit (DECISIONS.md D-73): MariaDB and MongoDB,
//! one instance each for every project (as Laragon), started from the
//! Database page or with Dev-Kit.
//!
//! - they listen on 127.0.0.1 only; MariaDB's `root` has no password, which
//!   is acceptable only because nothing outside this PC can reach it;
//! - data in `data\db\<id>`, created at the first start (MariaDB with its
//!   `mariadb-install-db`); output in `data\app\logs\<id>.log`;
//! - each server runs in a job of its own, nested in the app's: closing the
//!   app ends it in any case; `stop` first asks the server to shut down
//!   (`mariadb-admin shutdown`, mongosh `shutdownServer`), so no recovery
//!   is needed at the next start;
//! - after a major update of MariaDB the system tables are brought up to
//!   date by hand with `mariadb-upgrade` (run automatically it hung at
//!   "Fixing views", tried 14/10/2026);
//! - port and "start with Dev-Kit" of each server in `data\app\services.json`.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::env::KitEnv;
use crate::jobs::{self, Job};
use crate::kit::{write_atomic, Kit};
use crate::run::{is_port_free, responds};

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
    /// Connection string for the projects.
    pub url: String,
    /// Why the last start failed.
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

pub struct Services {
    kit: Kit,
    running: Mutex<BTreeMap<String, Server>>,
    errors: Mutex<BTreeMap<String, String>>,
}

pub fn data_dir(kit: &Kit, id: &str) -> PathBuf {
    kit.data().join("db").join(id)
}

fn log_path(kit: &Kit, id: &str) -> PathBuf {
    kit.logs().join(format!("{id}.log"))
}

fn settings_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("services.json")
}

pub fn load_settings(kit: &Kit) -> BTreeMap<String, Setting> {
    fs::read_to_string(settings_path(kit))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn installed(kit: &Kit, id: &str) -> bool {
    Catalog::get().find(id).is_some_and(|c| c.is_installed(kit))
}

fn bin(kit: &Kit, id: &str, exe: &str) -> PathBuf {
    match id {
        "mongosh" => kit.apps().join(r"mongosh\bin").join(exe),
        _ => kit.apps().join(id).join("bin").join(exe),
    }
}

pub fn url(id: &str, port: u16) -> String {
    match id {
        "mongodb" => format!("mongodb://127.0.0.1:{port}/"),
        _ => format!("mysql://root@127.0.0.1:{port}/"),
    }
}

/// The last lines of a server's log, to tell why it stopped.
fn log_tail(kit: &Kit, id: &str) -> String {
    let text = fs::read_to_string(log_path(kit, id)).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(3)..].join("\n")
}

/// Runs a tool of a server and waits at most `limit`.
fn run_tool(mut cmd: Command, limit: Duration) -> Result<()> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = jobs::spawn(&mut cmd)?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(anyhow!("{:?}: exit {status}", cmd.get_program()))
            };
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

impl Services {
    pub fn new(kit: Kit) -> Services {
        Services {
            kit,
            running: Mutex::new(BTreeMap::new()),
            errors: Mutex::new(BTreeMap::new()),
        }
    }

    fn port(&self, id: &str) -> u16 {
        load_settings(&self.kit)
            .get(id)
            .and_then(|s| s.port)
            .unwrap_or_else(|| default_port(id))
    }

    pub fn list(&self) -> Vec<ServiceInfo> {
        let settings = load_settings(&self.kit);
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
                    message: errors.get(*id).cloned(),
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

    /// Creates the data folder the first time.
    fn prepare(&self, id: &str, env: &KitEnv, port: u16) -> Result<PathBuf> {
        let data = data_dir(&self.kit, id);
        if id == "mariadb" && !data.join("mysql").is_dir() {
            // mariadb-install-db wants a folder that does not exist yet.
            if data.is_dir() && fs::read_dir(&data)?.next().is_none() {
                fs::remove_dir(&data)?;
            }
            if let Some(parent) = data.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut cmd = jobs::kit_command(bin(&self.kit, id, "mariadb-install-db.exe"), env);
            cmd.arg(format!("--datadir={}", data.display()))
                .arg(format!("--port={port}"));
            run_tool(cmd, Duration::from_secs(120)).context("mariadb-install-db")?;
        }
        fs::create_dir_all(&data)?;
        Ok(data)
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
                    .arg("127.0.0.1");
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
        let data = self.prepare(id, env, port)?;
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
        Ok(port)
    }

    /// Asks the server to shut down, then ends whatever is left.
    pub fn stop(&self, id: &str) {
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
                    .args(["--eval", "db.getSiblingDB('admin').shutdownServer()"]);
                cmd
            }
            _ => {
                let mut cmd =
                    jobs::kit_command(bin(&self.kit, id, "mariadb-admin.exe"), &server.env);
                cmd.args(["--user=root", "--host=127.0.0.1", "--protocol=tcp"])
                    .arg(format!("--port={}", server.port))
                    .arg("shutdown");
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
        all.insert(
            id.to_owned(),
            Setting {
                port: port.filter(|p| *p != default_port(id)),
                autostart,
            },
        );
        let text = serde_json::to_string_pretty(&all)?;
        write_atomic(&settings_path(&self.kit), text.as_bytes())?;
        Ok(())
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
                autostart: true
            }
        );
        assert_eq!(all["mongodb"].port, None, "the default port is not stored");
        assert!(s.set("mariadb", Some(80), false).is_err());
        assert!(s.set("redis", None, false).is_err());
        let list = s.list();
        assert_eq!(list[0].port, 3307);
        assert_eq!(list[0].url, "mysql://root@127.0.0.1:3307/");
        assert_eq!(list[1].url, "mongodb://127.0.0.1:27017/");
        assert!(list
            .iter()
            .all(|i| i.state == State::Stopped && !i.installed));
        assert!(
            s.start("mariadb", &KitEnv::default()).is_err(),
            "not installed"
        );
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
            assert_eq!(
                s.list().iter().find(|i| i.id == id).unwrap().state,
                State::Running
            );
            s.stop(id);
            assert!(!responds(port), "{id} stopped");
            assert_eq!(s.running(), 0);
            s.set(id, before.port, before.autostart).unwrap();
        }
    }
}
