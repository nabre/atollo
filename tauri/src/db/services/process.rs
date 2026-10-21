//! The life of a server: started in a job of its own, waited for, stopped; the automatic start.

use std::fs::OpenOptions;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use super::access::admin_password;
use super::data::seed;
use super::tools::run_tool;
use super::{
    auth, bin_at, check_line, client_at, installed, line_port, lines_in_use, load_settings,
    log_path, log_tail, server_line, Key, PortInUse, Server, Services,
};
use crate::db::DbEngine;
use crate::env::KitEnv;
use crate::os::jobs::{self, Job};
use crate::projects::run::{is_port_free, responds};

const READY_WITHIN: Duration = Duration::from_secs(90);
const STOP_WITHIN: Duration = Duration::from_secs(20);

impl Services {
    /// The versions of `engine` running now.
    pub(super) fn running_lines(&self, engine: DbEngine) -> Vec<String> {
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
        // To the end: a second start of this version waits for the first,
        // then finds it running.
        let _turn = self.op(engine);
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
        if let Some(l) = &line {
            self.keep_port(id, l, port)?;
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
        // Started inside its job (BUG-V6-25): a job that cannot be made or
        // joined never leaves a server running outside it, with its port
        // and data folder taken.
        let job = Job::new()?;
        let child = jobs::spawn_in(&mut cmd, &job)?;
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
                // The turn of `start_engine` is held.
                self.stop_key(&key);
                bail!(
                    "no answer on port {port} within {} s",
                    READY_WITHIN.as_secs()
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        if let Err(e) = self.secure(engine, line, main, env, port, fresh) {
            self.stop_key(&key);
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
        let password = admin_password(&auth::try_load(&self.kit)?, engine)
            .ok_or_else(|| anyhow::anyhow!("{engine} has no password of root yet"))?;
        self.phpmyadmin.open(&self.kit, &env, port, &password, lang)
    }

    /// The address that opens mongo-express logged in to the running
    /// MongoDB (D-147), version `line` (None: the one the server runs).
    pub fn mongo_express_url(&self, line: Option<&str>) -> Result<String> {
        let engine = DbEngine::MongoDb;
        let (port, env) = self.running_at(engine, line)?;
        let password = admin_password(&auth::try_load(&self.kit)?, engine)
            .ok_or_else(|| anyhow::anyhow!("{engine} has no password of admin yet"))?;
        self.mongo_express.open(&self.kit, &env, port, &password)
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
        let _turn = self.op(engine);
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
                // mongo-express logged in to this version ends with it.
                self.mongo_express.stop_for(server.port);
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
            DbEngine::MySql => {
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

    /// Every server and every version of them. Without the turns of
    /// [`Services::ops`]: on closing, a start still waiting for its server
    /// (up to 90 s) does not hold the app open; that start then finds its
    /// server gone, and the job ends it in any case.
    pub fn stop_all(&self) {
        self.phpmyadmin.stop();
        self.mongo_express.stop();
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
