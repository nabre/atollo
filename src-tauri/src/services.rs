//! Database servers of the kit (DECISIONS.md D-73): MariaDB and MongoDB,
//! one instance each for every project (as Laragon), started from the
//! Database page or with Dev-Kit.
//!
//! - they listen on 127.0.0.1 only and ask for a password (D-74): MariaDB's
//!   `root` and MongoDB's `admin` get a random one at the first start, also
//!   on data made by an older Dev-Kit (`db_auth.rs`); each project gets its
//!   own database and user (`project_db_create`);
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
use crate::db_auth;
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
    /// Connection string of the administrator (no password in it).
    pub url: String,
    /// `root` or `admin`, and its password once set (first start).
    pub user: String,
    pub password: Option<String>,
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

pub(crate) fn bin(kit: &Kit, id: &str, exe: &str) -> PathBuf {
    match id {
        "mongosh" => kit.apps().join(r"mongosh\bin").join(exe),
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
                        .env("DEVKIT_DB_PASSWORD", &password);
                    run_with_input(cmd, "", Duration::from_secs(60)).context(
                        "MongoDB already has users that NEXKORA does not know: \
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
                    .env("DEVKIT_DB_ADMIN", admin)
                    .env("DEVKIT_DB_NAME", &name)
                    .env("DEVKIT_DB_PASSWORD", &password);
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
        Ok(port)
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
                    .args(["--eval", db_auth::MONGO_SHUTDOWN_JS])
                    .env(
                        "DEVKIT_DB_ADMIN",
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
                            .env("DEVKIT_DB_ADMIN", db_auth::load(&kit).mongodb_admin.unwrap());
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
            assert_eq!(s.running(), 0);
            s.set(id, before.port, before.autostart).unwrap();
        }
    }
}
