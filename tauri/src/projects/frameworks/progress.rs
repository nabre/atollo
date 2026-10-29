//! Where the creation running now (or the last one) is: what was chosen,
//! the command it is at, the output, how it ended. Kept by the app, so the
//! wizard finds it again after another page or a reload of the window.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use super::EnvVar;

/// The lines of output kept; the wizard shows the same.
pub const MAX_LOG: usize = 600;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creation {
    /// 0 = none since the app started; one more at every creation.
    pub id: u64,
    pub running: bool,
    pub base: String,
    pub addons: Vec<String>,
    pub name: String,
    /// The variables of `.env` as chosen (`{password}` not filled in).
    pub env: Vec<EnvVar>,
    /// 1-based number of the command; 0 = installing the kit's tools.
    pub step: usize,
    pub total: usize,
    /// Lines of output since the start, the ones dropped from `log` too.
    pub lines: usize,
    /// The last [`MAX_LOG`] lines; a command starting is `> command`.
    pub log: Vec<String>,
    /// Why it stopped, as the command's error reaches the interface (a
    /// text, or a refusal of the levels with its fix).
    pub error: Option<serde_json::Value>,
    /// The project made, once it is done.
    pub created: Option<String>,
}

static CREATION: Mutex<Creation> = Mutex::new(Creation {
    id: 0,
    running: false,
    base: String::new(),
    addons: Vec::new(),
    name: String::new(),
    env: Vec::new(),
    step: 0,
    total: 0,
    lines: 0,
    log: Vec::new(),
    error: None,
    created: None,
});
static NEXT: AtomicU64 = AtomicU64::new(1);

fn lock() -> MutexGuard<'static, Creation> {
    CREATION.lock().unwrap_or_else(PoisonError::into_inner)
}

pub fn snapshot() -> Creation {
    lock().clone()
}

/// A new creation: what it was asked for, nothing done yet. Returns its id.
pub fn start(base: &str, addons: &[String], name: &str, env: &[EnvVar], total: usize) -> u64 {
    let id = NEXT.fetch_add(1, Ordering::SeqCst);
    *lock() = Creation {
        id,
        running: true,
        base: base.to_owned(),
        addons: addons.to_vec(),
        name: name.to_owned(),
        env: env.to_vec(),
        total,
        ..Creation::default()
    };
    id
}

/// The creation is at `step`, with `text` as a new line of its output
/// (`> command` for a command starting). Returns the number of lines so
/// far (the event's `seq`).
pub fn record(id: u64, step: usize, text: Option<String>) -> usize {
    let mut c = lock();
    if c.id != id {
        return c.lines;
    }
    c.step = step;
    if let Some(text) = text {
        c.log.push(text);
        c.lines += 1;
        if c.log.len() > MAX_LOG {
            let extra = c.log.len() - MAX_LOG;
            c.log.drain(..extra);
        }
    }
    c.lines
}

/// The end of the creation: the project made, or why it stopped.
pub fn finish(id: u64, result: Result<String, serde_json::Value>) -> Creation {
    let mut c = lock();
    if c.id == id {
        c.running = false;
        match result {
            Ok(name) => c.created = Some(name),
            Err(e) => c.error = Some(e),
        }
    }
    c.clone()
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/frameworks/progress.rs"]
mod tests;
