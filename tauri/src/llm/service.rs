//! The server of the local models: llama.cpp's `llama-server` in router
//! mode on the presets of the models on disk (`presets`), on 127.0.0.1
//! only, with the kit's key. It loads the model an agent or the chat asks
//! for, one at a time, and lets it go after ten minutes without requests.
//!
//! Like the database servers (D-140, D-141): one operation at a time (a
//! start, a stop, a restart after the models changed), started inside its
//! job with every process it starts (the router runs one per model), its
//! output in `data/app/logs/llm.log`, a port of its own that no other part
//! of Atollo takes ([`crate::ports::Owner::Llm`]); stopped with "Ferma
//! tutto", "Rimuovi l'unità" and when the app closes.

use std::fs::OpenOptions;
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use super::settings;
use crate::db::services::State;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::run::{is_port_free, responds};
use crate::setup::catalog::{kit_path, Catalog};

/// The program, as the catalog writes it.
pub const SERVER: &str = "llama/llama-server.exe";

/// How long the server has to answer after it starts (it only reads the
/// presets: a model is loaded later, when asked for).
const READY_WITHIN: Duration = Duration::from_secs(60);

/// A model the server has not been asked for in this long is unloaded.
const IDLE_SECONDS: u32 = 600;

struct Server {
    child: Child,
    port: u16,
    // Dropped last: ends the server and every model it runs.
    _job: Job,
}

/// What the interface shows of the server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmInfo {
    pub installed: bool,
    pub state: State,
    pub port: u16,
    /// `http://127.0.0.1:<port>`: the chat in the browser and the API.
    pub url: String,
    pub error: Option<String>,
    /// The models in memory now.
    pub loaded: Vec<String>,
    pub autostart: bool,
    pub pi_model: Option<String>,
}

/// A GPU the server can use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    pub name: String,
    /// Its memory, in bytes.
    pub memory: u64,
}

/// The PC, for "does this model fit".
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub ram: Option<u64>,
    pub gpus: Vec<Gpu>,
}

pub struct Llm {
    kit: Kit,
    /// One operation at a time; taken before `running` and `error`.
    op: Mutex<()>,
    running: Mutex<Option<Server>>,
    error: Mutex<Option<String>>,
    gpus: Mutex<Option<Vec<Gpu>>>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The GPUs `llama-server --list-devices` names: `Vulkan0: NVIDIA GeForce
/// RTX 2070 (8220 MiB, 7451 MiB free)`, `MTL0: Apple M2 (…)`.
pub fn parse_devices(text: &str) -> Vec<Gpu> {
    text.lines()
        .filter_map(|l| {
            let (_, rest) = l.trim().split_once(": ")?;
            let (name, mem) = rest.rsplit_once(" (")?;
            let mib: u64 = mem.split_whitespace().next()?.parse().ok()?;
            Some(Gpu {
                name: name.trim().to_owned(),
                memory: mib * 1024 * 1024,
            })
        })
        .collect()
}

/// `http://127.0.0.1:<port>`.
pub fn url_of(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

/// A request to the server, outside any proxy, with the key.
fn api(port: u16, key: &str, path: &str) -> Option<Value> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .proxy(None)
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .into();
    let mut answer = agent
        .get(format!("{}{path}", url_of(port)))
        .header("Authorization", format!("Bearer {key}"))
        .call()
        .ok()?;
    answer.body_mut().read_json().ok()
}

/// The models of `/models` whose status is `loaded`.
pub fn loaded_in(models: &Value) -> Vec<String> {
    models
        .get("data")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|m| m.pointer("/status/value").and_then(Value::as_str) == Some("loaded"))
        .filter_map(|m| m.get("id").and_then(Value::as_str).map(str::to_owned))
        .collect()
}

impl Llm {
    pub fn new(kit: Kit) -> Llm {
        Llm {
            kit,
            op: Mutex::new(()),
            running: Mutex::new(None),
            error: Mutex::new(None),
            gpus: Mutex::new(None),
        }
    }

    pub fn installed(&self) -> bool {
        Catalog::get()
            .find("llama")
            .is_some_and(|c| c.is_installed(&self.kit))
    }

    /// The port it has, or the first free one of [`settings::PORTS`] that
    /// no other part of Atollo keeps, saved.
    fn port(&self, lang: &str) -> Result<u16> {
        let saved = settings::load(&self.kit).port;
        if let Some(p) = saved {
            crate::ports::refuse_taken(
                &self.kit,
                p,
                |o| matches!(o, crate::ports::Owner::Llm),
                lang,
            )?;
            if !is_port_free(p) {
                bail!(
                    "{}",
                    crate::i18n::t(
                        lang,
                        "La porta {0} dei modelli locali è occupata da un altro programma.",
                        &[&p.to_string()]
                    )
                );
            }
            return Ok(p);
        }
        let taken = crate::ports::ports_taken(&self.kit);
        let Some(p) = settings::PORTS
            .clone()
            .find(|p| !taken.contains_key(p) && is_port_free(*p))
        else {
            bail!("no free port for the local models");
        };
        settings::update(&self.kit, |s| {
            s.port = Some(p);
            Ok(())
        })?;
        Ok(p)
    }

    /// Starts the server (nothing when it runs); its port.
    pub fn start(&self, env: &KitEnv, lang: &str) -> Result<u16> {
        let _turn = lock(&self.op);
        if let Some(p) = self.alive_port() {
            return Ok(p);
        }
        let result = self.launch(env, lang);
        *lock(&self.error) = result.as_ref().err().map(|e| format!("{e:#}"));
        result
    }

    /// The port of the server when it runs.
    fn alive_port(&self) -> Option<u16> {
        let mut running = lock(&self.running);
        let alive = running
            .as_mut()
            .is_some_and(|s| matches!(s.child.try_wait(), Ok(None)));
        if !alive {
            *running = None;
        }
        running.as_ref().map(|s| s.port)
    }

    fn launch(&self, env: &KitEnv, lang: &str) -> Result<u16> {
        if !self.installed() {
            bail!(
                "{}",
                crate::i18n::t(lang, "{0} non è installato.", &["llama.cpp"])
            );
        }
        let ids = super::presets::write(&self.kit)?;
        if ids.is_empty() {
            bail!(
                "{}",
                crate::i18n::t(
                    lang,
                    "Nessun modello locale: scaricane uno dalla pagina Modelli locali.",
                    &[]
                )
            );
        }
        settings::ensure_key(&self.kit)?;
        let port = self.port(lang)?;
        let exe = kit_path(&self.kit, SERVER);
        let log = self.kit.logs().join("llm.log");
        std::fs::create_dir_all(self.kit.logs())?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log)
            .with_context(|| log.display().to_string())?;
        let mut cmd = jobs::kit_command(&exe, &server_env(&self.kit, env, &exe));
        cmd.args(args(&self.kit, port))
            .current_dir(exe.parent().unwrap_or(self.kit.root()))
            .stdin(Stdio::null())
            .stdout(Stdio::from(file.try_clone()?))
            .stderr(Stdio::from(file));
        let job = Job::new()?;
        let child = jobs::spawn_in(&mut cmd, &job)?;
        crate::log::info(&format!(
            "llm: started on 127.0.0.1:{port} with {} models",
            ids.len()
        ));
        *lock(&self.running) = Some(Server {
            child,
            port,
            _job: job,
        });
        let start = Instant::now();
        while !responds(port) {
            if self.alive_port().is_none() {
                bail!("{}", log_tail(&self.kit));
            }
            if start.elapsed() > READY_WITHIN {
                self.stop_now();
                bail!(
                    "no answer on port {port} within {} s",
                    READY_WITHIN.as_secs()
                );
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        Ok(port)
    }

    /// Stops the server and the models it runs.
    pub fn stop(&self) {
        let _turn = lock(&self.op);
        self.stop_now();
        *lock(&self.error) = None;
    }

    fn stop_now(&self) {
        if let Some(mut server) = lock(&self.running).take() {
            let _ = server.child.kill();
            let _ = server.child.wait();
            crate::log::info("llm: stopped");
            // The job ends what the router started.
        }
    }

    /// For "Rimuovi l'unità" and the end of the app: without waiting for
    /// an operation under way (a start waits up to a minute).
    pub fn stop_all(&self) {
        self.stop_now();
    }

    /// Started again on the models on disk now, when it runs (a model was
    /// downloaded or removed): the router reads its presets at start.
    pub fn reload(&self, env: &KitEnv, lang: &str) -> Result<()> {
        let _turn = lock(&self.op);
        if self.alive_port().is_none() {
            return Ok(());
        }
        self.stop_now();
        let result = self.launch(env, lang).map(drop);
        *lock(&self.error) = result.as_ref().err().map(|e| format!("{e:#}"));
        result
    }

    /// Runs `work` with the server stopped, started again after when it
    /// was running: removing a model it may have open.
    pub fn without_server<T>(
        &self,
        env: &KitEnv,
        lang: &str,
        work: impl FnOnce() -> Result<T>,
    ) -> Result<T> {
        let _turn = lock(&self.op);
        let was = self.alive_port().is_some();
        self.stop_now();
        let out = work();
        if was {
            let result = self.launch(env, lang).map(drop);
            *lock(&self.error) = result.as_ref().err().map(|e| format!("{e:#}"));
        }
        out
    }

    /// 1 when the server runs: for the count of what is open.
    pub fn running(&self) -> usize {
        usize::from(self.alive_port().is_some())
    }

    pub fn status(&self) -> LlmInfo {
        let s = settings::load(&self.kit);
        let alive = self.alive_port();
        let port = alive.or(s.port).unwrap_or(*settings::PORTS.start());
        let (state, loaded) = match alive {
            Some(p) if responds(p) => {
                let loaded = settings::key(&self.kit)
                    .and_then(|k| api(p, &k, "/models"))
                    .map(|m| loaded_in(&m))
                    .unwrap_or_default();
                (State::Running, loaded)
            }
            Some(_) => (State::Starting, Vec::new()),
            None if lock(&self.error).is_some() => (State::Error, Vec::new()),
            None => (State::Stopped, Vec::new()),
        };
        LlmInfo {
            installed: self.installed(),
            state,
            port,
            url: url_of(port),
            error: lock(&self.error).clone(),
            loaded,
            autostart: s.autostart,
            pi_model: s.pi_model,
        }
    }

    /// The PC's RAM and the GPUs llama.cpp sees (asked once per run).
    pub fn hardware(&self, env: &KitEnv) -> Hardware {
        let mut gpus = lock(&self.gpus);
        if gpus.is_none() && self.installed() {
            let exe = kit_path(&self.kit, SERVER);
            let out = jobs::kit_command(&exe, &server_env(&self.kit, env, &exe))
                .arg("--list-devices")
                .stdin(Stdio::null())
                .output();
            *gpus = Some(match out {
                Ok(o) => parse_devices(&format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                )),
                Err(e) => {
                    crate::log::warn(&format!("llm: --list-devices: {e}"));
                    Vec::new()
                }
            });
        }
        Hardware {
            ram: crate::os::memory::total_ram(),
            gpus: gpus.clone().unwrap_or_default(),
        }
    }

    /// Started with the app when the user asked for it.
    pub fn autostart(&self, env: &KitEnv) {
        if settings::load(&self.kit).autostart && self.installed() {
            if let Err(e) = self.start(env, "en") {
                crate::log::warn(&format!("llm: automatic start: {e:#}"));
            }
        }
    }
}

impl Llm {
    /// Waits while the server runs (`llm start` of the command line).
    pub fn wait(&self) {
        while self.alive_port().is_some() {
            std::thread::sleep(Duration::from_millis(500));
        }
    }
}

/// A server of this kit on the saved port: whether it answers, and the
/// models it has in memory (`llm status` of the command line, which does
/// not own it).
pub fn probe(kit: &Kit) -> (u16, bool, Vec<String>) {
    let port = settings::load(kit).port.unwrap_or(*settings::PORTS.start());
    if !responds(port) {
        return (port, false, Vec::new());
    }
    let loaded = settings::key(kit)
        .and_then(|k| api(port, &k, "/models"))
        .map(|m| loaded_in(&m))
        .unwrap_or_default();
    (port, true, loaded)
}

/// Ends every `llama-server` of this kit, whoever started it (`llm stop`
/// of the command line); how many.
pub fn stop_everywhere(kit: &Kit) -> usize {
    let dir = kit_path(kit, SERVER)
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| kit.apps().join("llama"));
    let pids: Vec<u32> = crate::os::processes::running_under(&dir)
        .iter()
        .map(|p| p.pid)
        .collect();
    crate::os::sessions::stop(&pids);
    pids.len()
}

/// The server's arguments: router mode on the presets, 127.0.0.1 only,
/// the key from its file, one model at a time, let go when idle.
pub fn args(kit: &Kit, port: u16) -> Vec<String> {
    vec![
        "--models-preset".into(),
        super::presets::path(kit).display().to_string(),
        "--host".into(),
        "127.0.0.1".into(),
        "--port".into(),
        port.to_string(),
        "--api-key-file".into(),
        settings::key_file(kit).display().to_string(),
        "--models-max".into(),
        "1".into(),
        "--sleep-idle-seconds".into(),
        IDLE_SECONDS.to_string(),
    ]
}

/// The kit's environment, with the server's own libraries found next to it
/// on macOS and Linux.
fn server_env(kit: &Kit, env: &KitEnv, exe: &std::path::Path) -> KitEnv {
    let mut env = env.clone();
    if !kit.os().is_windows() {
        if let Some(dir) = exe.parent() {
            let var = if kit.os() == crate::kit::Os::Darwin {
                "DYLD_FALLBACK_LIBRARY_PATH"
            } else {
                "LD_LIBRARY_PATH"
            };
            let sep = env.path_sep();
            let value = match env.get(var) {
                Some(old) if !old.is_empty() => format!("{}{sep}{old}", dir.display()),
                _ => dir.display().to_string(),
            };
            env.set(var, value);
        }
    }
    env
}

/// The last lines of the server's output, to tell why it stopped.
fn log_tail(kit: &Kit) -> String {
    let text = std::fs::read_to_string(kit.logs().join("llm.log")).unwrap_or_default();
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(3)..].join("\n")
}

#[cfg(test)]
#[path = "../../../test/rust/llm/service.rs"]
mod tests;
