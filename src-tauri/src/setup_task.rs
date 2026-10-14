//! The preparation started from the app window ("Linguaggi e strumenti"
//! page, and automatically at start when a required component is missing).
//!
//! One run at a time, in its own thread. The state is a snapshot the
//! interface reads at load (`setup_state`) and receives at every change
//! (event `setup`), so a page opened mid-run shows the same progress.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use crate::catalog::{Catalog, Group};
use crate::env::KitEnv;
use crate::install::{self, Ctx};
use crate::kit::Kit;
use crate::net::Net;
use crate::settings::Settings;
use crate::setup_runner::{self, Busy, Event, Options, Outcome, Sink};

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

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub running: bool,
    pub cancelling: bool,
    /// Started by the app at start, not by the user.
    pub automatic: bool,
    pub total: usize,
    pub steps: Vec<Step>,
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

/// Required components this app can install that are not installed yet.
pub fn missing_required(kit: &Kit) -> Vec<String> {
    Catalog::get()
        .all()
        .iter()
        .filter(|c| c.group == Group::Required)
        .filter(|c| install::installer(&c.id).is_some() && !c.is_installed(kit))
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
                return Err("an installation is already running".into());
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
        let net = Net::new(&settings, self.github_token.clone()).map_err(|e| format!("{e:#}"))?;
        let ctx = Ctx {
            kit: &self.kit,
            net: &net,
            env: &env,
            cancel: &self.cancel,
        };
        let mut sink = TaskSink { task: self, notify };
        match setup_runner::run(&ctx, opts, &mut sink) {
            Ok(_) => Ok(()),
            Err(e) if e.is::<Busy>() => Err("another installation is running in this kit".into()),
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
            Event::Step {
                total, id, name, ..
            } => {
                state.total = total;
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use std::fs;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn missing_required_lists_only_installable_ones() {
        let tmp = TempDir::new("task-missing");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let missing = missing_required(&kit);
        for id in ["git", "node", "uv", "gh", "pwsh", "vscodium-server"] {
            assert!(missing.contains(&id.to_owned()), "{id}");
        }
        assert!(missing.contains(&"uv".to_owned()) && missing.contains(&"gh".to_owned()));
        assert!(!missing.contains(&"python".to_owned()), "not required");

        let pwsh = Catalog::get().find("pwsh").unwrap().key_path(&kit);
        fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
        fs::write(&pwsh, b"").unwrap();
        assert!(!missing_required(&kit).contains(&"pwsh".to_owned()));
    }

    /// No network: every required component is already installed.
    #[test]
    fn run_reports_steps_and_ends() {
        let tmp = TempDir::new("task-run");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let catalog = Catalog::get();
        let required: Vec<_> = catalog
            .all()
            .iter()
            .filter(|c| c.group == Group::Required)
            .collect();
        for c in &required {
            for key in c.key_paths(&kit) {
                fs::create_dir_all(key.parent().unwrap()).unwrap();
                fs::write(&key, b"").unwrap();
            }
            c.save_version(&kit, "1.0").unwrap();
        }

        let task = Arc::new(SetupTask::new(
            kit,
            KitEnv::default(),
            Settings::default(),
            None,
        ));
        let (tx, rx) = mpsc::channel();
        let notify: Notify = Box::new(move |s| {
            let _ = tx.send(s.clone());
        });
        let opts = Options {
            skip: catalog.all().iter().map(|c| c.id.clone()).collect(),
            ..Options::default()
        };
        task.start(opts, false, notify).unwrap();
        let last = loop {
            let s = rx.recv_timeout(Duration::from_secs(30)).unwrap();
            if !s.running {
                break s;
            }
        };
        assert_eq!(last.error, None);
        assert_eq!(last.steps.len(), required.len());
        assert!(last.steps.iter().all(|s| s.outcome
            == Some(Outcome::UpToDate {
                version: Some("1.0".into())
            })));
    }
}
