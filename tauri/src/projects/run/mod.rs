//! Starting and stopping the projects' actions.
//!
//! - every action runs in a job of its own, nested in the app's: stopping it
//!   ends the whole tree (npm → node → esbuild…), closing the app ends all;
//! - commands go through `cmd.exe /d /s /c` (`/bin/sh -c` on macOS and
//!   Linux, `jobs::shell_command`) with the kit's environment plus
//!   `PORT`, `ROCKET_PORT`, `DB_PORT` (its MySQL database), `BROWSER=none`, `PYTHONUNBUFFERED=1`; a `static`
//!   action runs this exe as a file server (`Atollo.exe serve`), an
//!   `apache` action the kit's httpd with a configuration of the project,
//!   `caddy` actions the same with Caddy and PHP;
//! - web actions keep their port from one start to the next (bookmarks,
//!   cookies, localStorage stay valid): 7100-7999, remembered in
//!   `data\projects\<project>\ports.json`, or the one the user locked;
//! - output: last 3000 lines per action, without ANSI colours; a web action
//!   is "running" once its port answers, or the port it prints if it ignores
//!   the one assigned.
//!
//! The parts: `ports` (the actions' ports and their `ports.json`),
//! `output` (the log of a run, the address a server prints, the local
//! network), `start` (a start, until the action is running or ends),
//! `pool` (the sites served by a shared pool).

mod output;
mod pool;
mod ports;
mod start;

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use serde::Serialize;

use crate::common::time::now_ms;
use crate::detect::Kind;
use crate::kit::Kit;
use crate::platform::jobs::Job;

pub(crate) use output::{split_lines, take_utf8};
pub use pool::{site_mode, SiteMode, DEDICATED_SERVER};
pub(crate) use ports::saved_ports;
pub use ports::Ports;
use ports::{load_ports, split_key, ProjectPorts};

pub const PORT_FROM: u16 = 7100;
pub const PORT_TO: u16 = 7999;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Starting,
    Running,
    Finished,
    Error,
    Stopped,
}

/// What the interface sees of a started action.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInfo {
    pub key: String,
    pub project: String,
    pub action: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub command: String,
    pub port: Option<u16>,
    pub url: Option<String>,
    pub state: State,
    /// Milliseconds since 1970.
    pub start: u64,
    pub end: Option<u64>,
    pub code: Option<i32>,
    pub pid: Option<u32>,
    /// URLs of the site's aliases that were left out of its configuration.
    pub skipped_aliases: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Event {
    State(RunInfo),
    /// `seq`: the number of the last of `lines` (see [`RunLog`]).
    Log {
        key: String,
        lines: Vec<String>,
        seq: u64,
    },
}

/// The log of an action as read, for a reload merged with the events
/// that came while it was read (BUG-V7-30): every line written to a log
/// is numbered, over all the actions, and `seq` is the last number given
/// when it was read. The lines of a later event have higher numbers, by
/// position, never by content: a log repeats lines.
#[derive(Debug, Clone, Serialize)]
pub struct RunLog {
    pub lines: Vec<String>,
    pub seq: u64,
}

pub type Notify = Arc<dyn Fn(Event) + Send + Sync>;

struct Entry {
    info: RunInfo,
    /// The last [`MAX_LINES`](output::MAX_LINES): the oldest leave from the
    /// front without moving the others.
    lines: VecDeque<String>,
    /// Dropped to stop: ends every process of the action.
    job: Option<Job>,
    stopped: bool,
    actual_port: Option<u16>,
    detected_url: Option<String>,
    /// Served by this shared pool (`servers/mod.rs`): no process of its own.
    pool: Option<String>,
    /// The run this entry is for: the threads of a run that was replaced
    /// leave it alone (BUG-V6-10).
    run: u64,
}

pub struct Runner {
    kit: Kit,
    entries: Mutex<HashMap<String, Entry>>,
    ports: Mutex<Ports>,
    /// What the project files hold now, to write only those that changed.
    saved: Mutex<BTreeMap<String, ProjectPorts>>,
    /// Ports being probed for a start: two actions started together differ.
    pending: Mutex<HashSet<u16>>,
    /// Where automatic ports are taken: [`PORT_FROM`]..=[`PORT_TO`], a
    /// range of their own in the tests (TEST-V6-01).
    port_range: (u16, u16),
    /// The id of the last run started.
    runs: AtomicU64,
    /// The number of the last line written to a log ([`RunLog`]), only
    /// moved with `entries` held.
    seq: AtomicU64,
    /// One gate per action: its starts happen one after the other.
    starts: Mutex<HashMap<String, Arc<Mutex<()>>>>,
    notify: Notify,
}

pub fn key_of(project: &str, action: &str) -> String {
    format!("{project}|{action}")
}

impl Runner {
    pub fn new(kit: Kit, notify: Notify) -> Runner {
        let (ports, saved) = load_ports(&kit);
        Runner {
            kit,
            entries: Mutex::new(HashMap::new()),
            ports: Mutex::new(ports),
            saved: Mutex::new(saved),
            pending: Mutex::new(HashSet::new()),
            port_range: (PORT_FROM, PORT_TO),
            runs: AtomicU64::new(0),
            seq: AtomicU64::new(0),
            starts: Mutex::new(HashMap::new()),
            notify,
        }
    }

    /// The runner with automatic ports in `from..=to`: a test that checks
    /// a port is kept takes it where no other test takes one (TEST-V6-01).
    #[cfg(test)]
    pub fn with_port_range(mut self, from: u16, to: u16) -> Runner {
        self.port_range = (from, to);
        self
    }

    fn entries(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A new run id, never given before by this runner.
    fn next_run(&self) -> u64 {
        self.runs.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// Numbers `count` lines written to a log, `entries` held: the number
    /// of the last.
    fn number_lines(&self, count: usize) -> u64 {
        self.seq.fetch_add(count as u64, Ordering::Relaxed) + count as u64
    }

    /// The gate of the action `key`, held for a whole start.
    fn start_gate(&self, key: &str) -> Arc<Mutex<()>> {
        let mut starts = self.starts.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(starts.entry(key.to_owned()).or_default())
    }

    pub fn list(&self) -> Vec<RunInfo> {
        let mut out: Vec<RunInfo> = self.entries().values().map(|e| e.info.clone()).collect();
        out.sort_by(|a, b| a.key.cmp(&b.key));
        out
    }

    /// The log of `key` with the number of the last line written to any
    /// log (BUG-V7-30).
    pub fn log(&self, key: &str) -> RunLog {
        let entries = self.entries();
        RunLog {
            lines: entries
                .get(key)
                .map(|e| e.lines.iter().cloned().collect())
                .unwrap_or_default(),
            seq: self.seq.load(Ordering::Relaxed),
        }
    }

    /// A process of the action runs (one served by a pool has none: a
    /// dedicated start replaces it).
    fn alive(&self, key: &str) -> bool {
        self.entries().get(key).is_some_and(|e| {
            e.pool.is_none() && matches!(e.info.state, State::Starting | State::Running)
        })
    }

    /// Stops an action and everything it started. An action served by a
    /// shared pool is only marked stopped here: the caller takes its site
    /// out of the pool (`WebServers::detach`).
    pub fn stop(&self, key: &str) {
        let (job, unpooled) = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            e.stopped = true;
            let unpooled = (e.pool.is_some() && e.info.state == State::Running).then(|| {
                e.info.state = State::Stopped;
                e.info.end = Some(now_ms());
                e.info.clone()
            });
            (e.job.take(), unpooled)
        };
        // Closing the job's last handle ends its processes (KILL_ON_JOB_CLOSE).
        drop(job);
        if let Some(info) = unpooled {
            (self.notify)(Event::State(info));
        }
    }

    pub fn stop_all(&self) {
        let jobs: Vec<Job> = self
            .entries()
            .values_mut()
            .filter_map(|e| {
                e.stopped = true;
                if e.pool.is_some() && e.info.state == State::Running {
                    e.info.state = State::Stopped;
                    e.info.end = Some(now_ms());
                }
                e.job.take()
            })
            .collect();
        drop(jobs);
    }

    /// Actions still running (for the close confirmation); those served by
    /// a pool are counted with the pools.
    pub fn running(&self) -> usize {
        self.entries()
            .values()
            .filter(|e| {
                e.pool.is_none() && matches!(e.info.state, State::Starting | State::Running)
            })
            .count()
    }

    /// Forgets the output of an action that ended.
    pub fn clear(&self, key: &str) {
        let mut entries = self.entries();
        if entries.get(key).is_some_and(|e| {
            e.job.is_none() && (e.pool.is_none() || e.info.state != State::Running)
        }) {
            entries.remove(key);
        }
    }

    /// The ports of `from` are now `to`'s, whose files were moved on disk
    /// (`state::adopt`): nothing is written, and a later save does not
    /// replace the moved `ports.json`.
    pub fn rename_project(&self, from: &str, to: &str) {
        // Same order as `save_ports`: `saved`, then `ports`.
        let mut saved = self.saved.lock().unwrap_or_else(PoisonError::into_inner);
        let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
        let rekey = |key: &String| match split_key(key) {
            Some((p, action)) if p == from => key_of(to, action),
            _ => key.clone(),
        };
        ports.ports = std::mem::take(&mut ports.ports)
            .into_iter()
            .map(|(k, port)| (rekey(&k), port))
            .collect();
        ports.locked = ports.locked.iter().map(rekey).collect();
        if let Some(p) = saved.remove(from) {
            saved.insert(to.to_owned(), p);
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/run.rs"]
mod tests;
