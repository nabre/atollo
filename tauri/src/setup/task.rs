//! The preparation started from the app window ("Linguaggi e strumenti"
//! page, and automatically at start when a required component is missing).
//!
//! One run at a time, in its own thread. The state is a snapshot the
//! interface reads at load (`setup_state`) and receives at every change
//! (event `setup`), so a page opened mid-run shows the same progress.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use super::catalog::Catalog;
use super::runner::{self, Busy, Event, Options, Outcome, Sink};
use crate::env::KitEnv;
use crate::install::{self, Ctx};
use crate::kit::Kit;
use crate::net::Net;
use crate::settings::Settings;

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub name: String,
    /// Last note of the installer (download, checksum…).
    pub note: Option<String>,
    pub done: Option<u64>,
    pub size: Option<u64>,
    /// `None` while the step runs.
    pub outcome: Option<Outcome>,
}

/// A step waiting: the component and the name its step will have.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Queued {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub running: bool,
    pub cancelling: bool,
    /// Started by the app at start, not by the user.
    pub automatic: bool,
    pub total: usize,
    pub steps: Vec<Step>,
    /// The steps not started yet, in order (empty when the run ends).
    pub queue: Vec<Queued>,
    /// Why the run could not start or stopped early.
    pub error: Option<String>,
}

pub struct SetupTask {
    kit: Kit,
    env: Mutex<KitEnv>,
    settings: Mutex<Settings>,
    github_token: Option<String>,
    state: Mutex<Snapshot>,
    cancel: AtomicBool,
}

/// Called with the new snapshot after every change.
pub type Notify = Box<dyn Fn(&Snapshot) + Send>;

/// System components (required, the parts of the app, and what they need)
/// this app can install that are not installed yet. Without them the editor
/// and the consoles do not open.
pub fn missing_required(kit: &Kit) -> Vec<String> {
    let catalog = Catalog::get();
    let locked = catalog.locked();
    catalog
        .all()
        .iter()
        .filter(|c| locked.contains(c.id.as_str()))
        .filter(|c| install::installer(&c.id).is_some() && !c.is_installed(kit))
        .map(|c| c.id.clone())
        .collect()
}

/// Parts of the app installed at another version than this release's (the
/// editor after an update of Atollo). Brought to the fixed version once, at
/// start; until then the installed one keeps working, so nothing waits for
/// it (a folder in use must not block the editor). Also the components
/// marked to be installed again because the kit moved and they cannot
/// follow it (conda environments on macOS/Linux, D-92).
pub fn outdated_app(kit: &Kit) -> Vec<String> {
    Catalog::get()
        .all()
        .iter()
        .filter(|c| install::installer(&c.id).is_some() && c.is_installed(kit))
        .filter(|c| c.outdated(kit) || c.needs_reinstall(kit))
        .map(|c| c.id.clone())
        .collect()
}

impl SetupTask {
    pub fn new(
        kit: Kit,
        env: KitEnv,
        settings: Settings,
        github_token: Option<String>,
    ) -> SetupTask {
        SetupTask {
            kit,
            env: Mutex::new(env),
            settings: Mutex::new(settings),
            github_token,
            state: Mutex::new(Snapshot::default()),
            cancel: AtomicBool::new(false),
        }
    }

    /// New environment and settings (Settings page), for the next runs.
    pub fn set_config(&self, env: KitEnv, settings: Settings) {
        *self.env.lock().unwrap_or_else(PoisonError::into_inner) = env;
        *self.settings.lock().unwrap_or_else(PoisonError::into_inner) = settings;
    }

    /// The language of the interface (`CP_LANG` of the kit environment).
    fn lang(&self) -> &'static str {
        let env = self.env.lock().unwrap_or_else(PoisonError::into_inner);
        crate::i18n::resolve(env.get("CP_LANG"), None, None)
    }

    fn lock(&self) -> MutexGuard<'_, Snapshot> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.lock().clone()
    }

    /// Starts a run in the background; refused while another one runs.
    pub fn start(
        self: &Arc<Self>,
        opts: Options,
        automatic: bool,
        notify: Notify,
    ) -> Result<(), String> {
        {
            let mut state = self.lock();
            if state.running {
                return Err(crate::i18n::t(
                    self.lang(),
                    "C'è già un'installazione in corso.",
                    &[],
                ));
            }
            *state = Snapshot {
                running: true,
                automatic,
                ..Snapshot::default()
            };
            notify(&state);
        }
        self.cancel.store(false, Ordering::Relaxed);
        crate::log::info(&format!(
            "setup from the window: {opts:?} automatic={automatic}"
        ));
        let task = Arc::clone(self);
        std::thread::spawn(move || {
            let error = task.run(&opts, &notify).err();
            let mut state = task.lock();
            state.running = false;
            state.cancelling = false;
            state.queue.clear();
            state.error = error;
            for step in &state.steps {
                crate::log::info(&format!("setup {}: {:?}", step.id, step.outcome));
            }
            notify(&state);
        });
        Ok(())
    }

    fn run(&self, opts: &Options, notify: &Notify) -> Result<(), String> {
        let settings = self
            .settings
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let env = self
            .env
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        // The PC's token, or the kit's GitHub account: GitHub allows 60
        // anonymous API calls an hour, and "Update all" asks for many.
        let token = self
            .github_token
            .clone()
            .or_else(|| crate::github::token(&self.kit, &env));
        let net = Net::new(&settings, token)
            .map_err(|e| format!("{e:#}"))?
            .with_github_cache(self.kit.data().join("cache").join("github"));
        let ctx = Ctx {
            kit: &self.kit,
            net: &net,
            env: &env,
            cancel: &self.cancel,
        };
        let mut sink = TaskSink { task: self, notify };
        match runner::run(&ctx, opts, &mut sink) {
            Ok(_) => Ok(()),
            Err(e) if e.is::<Busy>() => Err(crate::i18n::t(
                self.lang(),
                "Un'altra installazione è in corso in questo kit (un'altra finestra di Atollo o il menu del kit).",
                &[],
            )),
            Err(e) => Err(format!("{e:#}")),
        }
    }

    /// Stops during a download, otherwise at the end of the current step.
    pub fn cancel(&self, notify: &Notify) {
        let mut state = self.lock();
        if state.running {
            self.cancel.store(true, Ordering::Relaxed);
            state.cancelling = true;
            notify(&state);
        }
    }
}

struct TaskSink<'a> {
    task: &'a SetupTask,
    notify: &'a Notify,
}

impl Sink for TaskSink<'_> {
    fn emit(&mut self, event: Event) {
        let mut state = self.task.lock();
        match event {
            Event::Queue { steps } => {
                state.queue = steps
                    .into_iter()
                    .map(|(id, name)| Queued { id, name })
                    .collect();
            }
            Event::Step {
                total, id, name, ..
            } => {
                state.total = total;
                if let Some(i) = state
                    .queue
                    .iter()
                    .position(|q| q.id == id && q.name == name)
                {
                    state.queue.remove(i);
                }
                state.steps.push(Step {
                    id,
                    name,
                    ..Step::default()
                });
            }
            Event::Note { id, text } => {
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.note = Some(crate::env::mask_secrets(&text));
                }
            }
            Event::Bytes { id, done, total } => {
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.done = Some(done);
                    step.size = total;
                }
            }
            Event::Done { id, outcome } => {
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.outcome = Some(outcome);
                }
            }
        }
        (self.notify)(&state);
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/task.rs"]
mod tests;
