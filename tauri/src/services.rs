//! Database servers of the kit (DECISIONS.md D-73): MariaDB and MongoDB,
//! one instance each for every project (as Laragon), started from the
//! Database page or with Atollo.
//!
//! - they listen on 127.0.0.1 only and ask for a password (D-74): MariaDB's
//!   `root` and MongoDB's `admin` get a random one at the first start, also
//!   on data made by an older Atollo (`db_auth.rs`); each project gets its
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
//! - port and "start with Atollo" of each server in `data\app\services.json`.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::catalog::Catalog;
use crate::db_auth;
use crate::env::KitEnv;
use crate::jobs::{self, Job};
use crate::kit::Kit;
use crate::projects::run::{is_port_free, responds};

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
    adminer: crate::adminer::Adminer,
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

/// `data\db\<id>`: the data of every version of the server.
fn data_root(kit: &Kit, id: &str) -> PathBuf {
    kit.data().join("db").join(id)
}

/// The data of the version the server runs, `data\db\<id>\<line>`: data
/// of a newer version does not open in an older one (D-81).
pub fn data_dir(kit: &Kit, id: &str) -> PathBuf {
    match server_line(kit, id) {
        Some(line) => data_root(kit, id).join(line),
        None => data_root(kit, id),
    }
}

/// The data of a version that has it yet (MariaDB's `mysql`, MongoDB's
/// `WiredTiger`).
fn has_data(id: &str, dir: &std::path::Path) -> bool {
    match id {
        "mariadb" => dir.join("mysql").is_dir(),
        _ => dir.join("WiredTiger").is_file(),
    }
}

/// The data of `line` the first time it starts: a copy of that of the
/// nearest older version, if the server can take it (MariaDB from any
/// older one; MongoDB from the same or the previous major only). The old
/// data stays as it was, for its own version. Returns the version copied.
fn seed(kit: &Kit, id: &str, line: &str) -> Result<Option<String>> {
    let root = data_root(kit, id);
    let target = root.join(line);
    if has_data(id, &target) {
        return Ok(None);
    }
    let wanted = crate::install::version_key(line);
    let mut older: Vec<(Vec<u64>, String)> = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let key = crate::install::version_key(&name);
            (crate::versions::valid_line(&name) && key < wanted && has_data(id, &e.path()))
                .then_some((key, name))
        })
        .collect();
    older.sort();
    let Some((key, from)) = older.pop() else {
        return Ok(None);
    };
    if id == "mongodb"
        && wanted.first().copied().unwrap_or(0) > key.first().copied().unwrap_or(0) + 1
    {
        crate::log::warn(&format!(
            "mongodb {line}: the data of {from} cannot jump more than one major version; \
             starting with empty data (the {from} data stays)"
        ));
        return Ok(None);
    }
    if target.exists() {
        crate::archive::remove_dir(&target)?;
    }
    copy_dir(&root.join(&from), &target)?;
    crate::log::info(&format!("{id}: data of {from} copied for {line}"));
    Ok(Some(from))
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)?.flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).with_context(|| target.display().to_string())?;
        }
    }
    Ok(())
}

/// Data of an older kit, `data\db\<id>` itself: into the folder of the
/// installed version, `data\db\<id>\<line>`. At start (versions::migrate).
pub fn migrate_data(kit: &Kit) {
    for id in SERVERS {
        let root = data_root(kit, id);
        if !has_data(id, &root) {
            continue;
        }
        let Some(line) = Catalog::get()
            .find(id)
            .and_then(|c| crate::versions::default_line(kit, c))
        else {
            continue;
        };
        let moving = kit.data().join("db").join(format!("{id}.atollo-move"));
        let moved = fs::rename(&root, &moving)
            .and_then(|()| fs::create_dir_all(&root))
            .and_then(|()| fs::rename(&moving, root.join(&line)));
        match moved {
            Ok(()) => crate::log::info(&format!(
                "{id}: data moved to {}",
                root.join(&line).display()
            )),
            Err(e) => {
                if moving.exists() && !root.join(&line).exists() {
                    let _ = fs::remove_dir(&root);
                    let _ = fs::rename(&moving, &root);
                }
                crate::log::warn(&format!("{id}: data not moved ({e})"));
            }
        }
    }
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

fn admin_user(id: &str) -> &'static str {
    match id {
        "mongodb" => "admin",
        _ => "root",
    }
}

fn admin_password(secrets: &db_auth::Secrets, id: &str) -> Option<String> {
    match id {
        "mongodb" => secrets.mongodb_admin.clone(),
        _ => secrets.mariadb_root.clone(),
    }
}

/// Runs a tool with `input` on its stdin (SQL with passwords: never on the
/// command line) and waits at most `limit`; fails with its error output.
fn run_with_input(mut cmd: Command, input: &str, limit: Duration) -> Result<()> {
    use std::io::{Read, Write};
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = jobs::spawn(&mut cmd)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let mut err = String::new();
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_string(&mut err);
            }
            return if status.success() {
                Ok(())
            } else {
                Err(anyhow!("{}", err.trim()))
            };
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(100));
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
            adminer: crate::adminer::Adminer::default(),
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
        let secrets = db_auth::load(&self.kit);
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

    /// Creates the data folder the first time; true when it is new (no
    /// password set in it yet).
    fn prepare(&self, id: &str, env: &KitEnv, port: u16) -> Result<(PathBuf, bool)> {
        let data = data_dir(&self.kit, id);
        let fresh = match id {
            "mariadb" => !data.join("mysql").is_dir(),
            _ => !data.join("WiredTiger").is_file(),
        };
        if id == "mariadb" && fresh {
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
        Ok((data, fresh))
    }

    /// The administrator's password, set on a new data folder or on one made
    /// before D-74 (no password yet), and MariaDB's `[client]` login.
    fn secure(&self, id: &str, env: &KitEnv, port: u16, fresh: bool) -> Result<()> {
        let mut secrets = db_auth::load(&self.kit);
        let known = admin_password(&secrets, id);
        let password = match &known {
            Some(p) => p.clone(),
            None => db_auth::new_password()?,
        };
        if fresh || known.is_none() {
            match id {
                "mongodb" => {
                    let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), env);
                    cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                        .arg(port.to_string())
                        .args(["--eval", db_auth::MONGO_ADMIN_JS])
                        .env("ATOLLO_DB_PASSWORD", &password);
                    run_with_input(cmd, "", Duration::from_secs(60)).context(
                        "MongoDB already has users that Atollo does not know: \
                         its administrator could not be created",
                    )?;
                    secrets.mongodb_admin = Some(password.clone());
                }
                _ => {
                    // No password yet: log in without one, ignoring my.ini.
                    let mut cmd = jobs::kit_command(bin(&self.kit, id, "mariadb.exe"), env);
                    cmd.args([
                        "--no-defaults",
                        "--user=root",
                        "--host=127.0.0.1",
                        "--protocol=tcp",
                    ])
                    .arg(format!("--port={port}"));
                    run_with_input(
                        cmd,
                        &db_auth::root_password_sql(&password),
                        Duration::from_secs(60),
                    )
                    .context("MariaDB: setting the password of root")?;
                    secrets.mariadb_root = Some(password.clone());
                }
            }
            db_auth::save(&self.kit, &secrets)?;
            crate::log::info(&format!("{id}: administrator password set"));
        }
        if id == "mariadb" {
            db_auth::write_client_ini(&self.kit, port, &password)?;
        }
        Ok(())
    }

    /// A tool of MariaDB logged in as `root` (the kit's `[client]` login).
    pub(crate) fn mariadb_tool(&self, exe: &str, env: &KitEnv) -> Command {
        let mut cmd = jobs::kit_command(bin(&self.kit, "mariadb", exe), env);
        // It must be the first option.
        cmd.arg(format!(
            "--defaults-extra-file={}",
            db_auth::client_ini(&self.kit).display()
        ));
        cmd
    }

    /// The project's databases already created, at the servers' ports.
    pub fn project_dbs(&self, project: &str) -> Vec<db_auth::ProjectDb> {
        db_auth::load(&self.kit)
            .projects
            .get(project)
            .map(|engines| {
                engines
                    .iter()
                    .map(|(engine, pw)| db_auth::project_db(engine, project, pw, self.port(engine)))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Creates (or repairs) the project's database and user on a running
    /// server; the password stays the same once generated.
    pub fn project_db_create(&self, project: &str, engine: &str) -> Result<db_auth::ProjectDb> {
        let (port, env) = {
            let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
            match running.get_mut(engine) {
                Some(s) => match s.child.try_wait() {
                    Ok(None) => (s.port, s.env.clone()),
                    _ => bail!("{engine} is not running"),
                },
                None => bail!("{engine} is not running"),
            }
        };
        let mut secrets = db_auth::load(&self.kit);
        let password = match secrets.projects.get(project).and_then(|e| e.get(engine)) {
            Some(p) => p.clone(),
            None => db_auth::new_password()?,
        };
        let name = db_auth::db_name(project);
        match engine {
            "mongodb" => {
                let admin = secrets
                    .mongodb_admin
                    .clone()
                    .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
                let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(port.to_string())
                    .args(["--eval", db_auth::MONGO_PROJECT_JS])
                    .env("ATOLLO_DB_ADMIN", admin)
                    .env("ATOLLO_DB_NAME", &name)
                    .env("ATOLLO_DB_PASSWORD", &password);
                run_with_input(cmd, "", Duration::from_secs(60))?;
            }
            _ => {
                let cmd = self.mariadb_tool("mariadb.exe", &env);
                run_with_input(
                    cmd,
                    &db_auth::project_sql(&name, &password),
                    Duration::from_secs(60),
                )?;
            }
        }
        secrets
            .projects
            .entry(project.to_owned())
            .or_default()
            .insert(engine.to_owned(), password.clone());
        db_auth::save(&self.kit, &secrets)?;
        Ok(db_auth::project_db(engine, project, &password, port))
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

    /// After the data of `from` was copied for `line`: MongoDB takes the
    /// features of the new major (its next one needs it); MariaDB's system
    /// tables are brought up to date by hand (`mariadb-upgrade`, D-73).
    fn after_copy(&self, id: &str, env: &KitEnv, port: u16, from: &str, line: &str) {
        let major = |l: &str| l.split('.').next().unwrap_or("").to_owned();
        if major(from) == major(line) {
            return;
        }
        if id != "mongodb" {
            crate::log::info(&format!(
                "mariadb {line}: data of {from}; run mariadb-upgrade to update the system tables"
            ));
            return;
        }
        let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), env);
        cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
            .arg(port.to_string())
            .args(["--eval", db_auth::MONGO_FCV_JS])
            .env(
                "ATOLLO_DB_ADMIN",
                db_auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
            )
            .env("ATOLLO_DB_FCV", line);
        match run_tool(cmd, Duration::from_secs(60)) {
            Ok(_) => crate::log::info(&format!("mongodb {line}: feature compatibility {line}")),
            Err(e) => crate::log::warn(&format!(
                "mongodb {line}: setFeatureCompatibilityVersion: {e:#}"
            )),
        }
    }

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
                    .is_ok_and(|m| m.is_file() && !crate::safe_path::is_reparse(&m))
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

    /// mongodump/mongorestore logged in as `admin`, the password in a config
    /// file of the private folder (never on the command line), removed after.
    fn mongo_tool(&self, exe: &str, env: &KitEnv, port: u16) -> Result<(Command, PathBuf)> {
        let admin = db_auth::load(&self.kit)
            .mongodb_admin
            .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
        let config = db_auth::dir(&self.kit).join(format!("{exe}-{}.yaml", std::process::id()));
        db_auth::write_private_text(&config, &format!("password: {admin}\n"))?;
        let mut cmd = jobs::kit_command(self.kit.apps().join(r"mongodb-tools\bin").join(exe), env);
        cmd.arg(format!("--config={}", config.display()))
            .args([
                "--host=127.0.0.1",
                "--username=admin",
                "--authenticationDatabase=admin",
            ])
            .arg(format!("--port={port}"));
        Ok((cmd, config))
    }

    /// A copy of the project's database in its backup folder (D-78); the
    /// server must run.
    pub fn export(&self, project: &str, engine: &str) -> Result<PathBuf> {
        let (port, env) = self.running_server(engine)?;
        let name = db_auth::db_name(project);
        let dir = self.backup_dir(project);
        fs::create_dir_all(&dir)?;
        let stamp = crate::agents_sync::stamp();
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
        if !crate::safe_path::is_file(&self.backup_dir(project), &path) {
            bail!("copy not found: {file}");
        }
        let name = db_auth::db_name(project);
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
                    .args(["--eval", db_auth::MONGO_SHUTDOWN_JS])
                    .env(
                        "ATOLLO_DB_ADMIN",
                        db_auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
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

    /// Port (None: default) and "start with Atollo" of `id`.
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

    /// The installed servers marked "start with Atollo".
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

/// A copy of a project's database (D-78).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub file: String,
    pub engine: String,
    pub size: u64,
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

    /// `id` installed at `version`, the default folder of its line.
    fn fake_server(kit: &Kit, id: &str, version: &str) {
        let c = Catalog::get().find(id).unwrap();
        let dir = crate::versions::install_dir(kit, c, version);
        let folder = c.folder.as_deref().unwrap();
        for k in std::iter::once(&c.key).chain(&c.keys) {
            if let Some(rel) = k.strip_prefix(folder) {
                let file = dir.join(rel.trim_start_matches('/').replace('/', "\\"));
                fs::create_dir_all(file.parent().unwrap()).unwrap();
                fs::write(&file, b"").unwrap();
            }
        }
        c.save_version(kit, version).unwrap();
    }

    #[test]
    fn each_version_has_its_data() {
        let tmp = TempDir::new("services-lines");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        // Data of an older kit, then the server's own folder per version.
        fake_server(&kit, "mariadb", "11.8.3");
        let root = kit.data().join(r"db\mariadb");
        fs::create_dir_all(root.join("mysql")).unwrap();
        fs::write(root.join("ibdata1"), b"x").unwrap();
        migrate_data(&kit);
        assert!(root.join(r"11.8\mysql").is_dir() && !root.join("mysql").exists());
        assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));

        // A newer version starts with a copy; the old data stays.
        let c = Catalog::get().find("mariadb").unwrap();
        let newer = crate::versions::saved_dir(&kit, c, "12.3");
        fs::create_dir_all(newer.join("bin")).unwrap();
        for k in ["mariadbd.exe", "mariadb.exe"] {
            fs::write(newer.join("bin").join(k), b"").unwrap();
        }
        assert_eq!(server_line(&kit, "mariadb").as_deref(), Some("12.3"));
        assert_eq!(server_lines(&kit, "mariadb"), ["12.3", "11.8"]);
        assert_eq!(
            seed(&kit, "mariadb", "12.3").unwrap().as_deref(),
            Some("11.8")
        );
        assert!(root.join(r"12.3\ibdata1").is_file() && root.join(r"11.8\ibdata1").is_file());
        assert_eq!(
            seed(&kit, "mariadb", "12.3").unwrap(),
            None,
            "only the first time"
        );
        // Back to the older version: its own data, a setting of the server.
        let s = Services::new(kit.clone());
        s.set_line("mariadb", Some("11.8")).unwrap();
        assert_eq!(data_dir(&kit, "mariadb"), root.join("11.8"));
        assert!(
            s.set_line("mariadb", Some("10.6")).is_err(),
            "not installed"
        );
        assert_eq!(
            bin(&kit, "mariadb", "mariadbd.exe")
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .file_name()
                .unwrap(),
            "11.8"
        );

        // MongoDB: one major at a time.
        let mongo = kit.data().join(r"db\mongodb");
        fs::create_dir_all(mongo.join("7.0")).unwrap();
        fs::write(mongo.join(r"7.0\WiredTiger"), b"").unwrap();
        assert_eq!(
            seed(&kit, "mongodb", "9.0").unwrap(),
            None,
            "7.0 cannot jump to 9.0"
        );
        assert_eq!(
            seed(&kit, "mongodb", "8.0").unwrap().as_deref(),
            Some("7.0")
        );
    }

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
    }

    /// A real MariaDB and MongoDB of the kit given by ATOLLO_TEST_KIT, on
    /// free ports: start, answer, stop with nothing left.
    #[test]
    fn real_databases_start_and_stop() {
        let Ok(root) = std::env::var("ATOLLO_TEST_KIT") else {
            eprintln!("ATOLLO_TEST_KIT not set: skipped");
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
                                "db.getSiblingDB('admin').auth('admin', process.env.ATOLLO_DB_ADMIN); {script}"
                            )])
                            .env("ATOLLO_DB_ADMIN", db_auth::load(&kit).mongodb_admin.unwrap());
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
            let dbs = crate::db_browse::tree(&s, id).unwrap();
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
            let page = crate::db_browse::rows(&s, id, "prova_kit", "t", 0).unwrap();
            assert!(page.total > 0 && !page.rows.is_empty(), "{id}: rows read");
            assert!(page.columns.iter().any(|c| c.name == "x"), "{id}: column x");
            if id == "mariadb" {
                assert!(crate::db_browse::rows(&s, id, "prova_kit", "no`; DROP", 0).is_err());
            }
            // Adminer logged in with the token only, ended with MariaDB (D-85).
            let adminer = (id == "mariadb" && installed(&kit, "adminer")).then(|| {
                let url = s.adminer_url().unwrap();
                crate::adminer::tests::assert_logs_in(&url);
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
