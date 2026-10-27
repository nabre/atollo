//! The preparation started from the app window ("Linguaggi e strumenti"
//! page, and automatically at start when a required component is missing).
//!
//! One run at a time, in its own thread; a run asked meanwhile waits in a
//! queue and starts when the one before it ends, each with its job
//! (`common::jobs`), cancelled alone. The state is a snapshot the
//! interface reads at load (`setup_state`) and receives at every change
//! (event `setup`), so a page opened mid-run shows the same progress.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use serde::Serialize;

use super::catalog::Catalog;
use super::runner::{self, Busy, Event, Options, Outcome, Sink};
use crate::common::i18n::t;
use crate::common::jobs::{JobKind, JobState, Jobs};
use crate::common::settings::Settings;
use crate::env::KitEnv;
use crate::install::{self, Ctx};
use crate::kit::Kit;
use crate::net::Net;

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
    /// Target (`os-arch`) of this run.
    pub target: String,
    pub total: usize,
    pub steps: Vec<Step>,
    /// The steps not started yet, in order (empty when the run ends).
    pub queue: Vec<Queued>,
    /// Why the run could not start or stopped early.
    pub error: Option<String>,
    /// The job of the run (`common::jobs`).
    pub job: Option<String>,
    /// The runs asked while this one runs, in order: each starts when the
    /// one before it ends (LEVELS.md §8.6).
    pub pending: Vec<PendingRun>,
}

/// A run waiting for the one before it: its job, its title and the
/// components it touches (empty: the whole choice).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingRun {
    pub job: String,
    pub title: String,
    pub items: Vec<String>,
}

/// A run asked while another one runs.
struct Pending {
    job: String,
    opts: Options,
    automatic: bool,
    notify: Notify,
}

pub struct SetupTask {
    kit: Kit,
    /// The PC's variables, for the kit's `sbx` (the tools sandbox, D-180).
    host: crate::env::Host,
    env: Mutex<KitEnv>,
    settings: Mutex<Settings>,
    github_token: Option<String>,
    state: Mutex<Snapshot>,
    /// Taken before the snapshot's lock is let go, held while the snapshot
    /// is told ([`SetupTask::tell`]): the changes reach the window in the
    /// order they were made, and nobody reading the snapshot waits for the
    /// telling.
    order: Mutex<()>,
    cancel: AtomicBool,
    pending: Mutex<VecDeque<Pending>>,
    jobs: Arc<Jobs>,
}

/// Called with the new snapshot after every change.
pub type Notify = Box<dyn Fn(&Snapshot) + Send>;

/// System components (required, the parts of the app, and what they need)
/// this app can install on the kit's target that are not installed yet:
/// installed at start. Docker Sandboxes, whose builds are for some systems
/// only, is skipped on the others.
pub fn missing_required(kit: &Kit) -> Vec<String> {
    let catalog = Catalog::get();
    let locked = catalog.locked();
    let target = kit.target();
    catalog
        .all()
        .iter()
        .filter(|c| locked.contains(c.id.as_str()))
        .filter(|c| crate::router::target::installable_on(c, target))
        .filter(|c| install::installer(&c.id).is_some() && !c.is_installed(kit))
        .map(|c| c.id.clone())
        .collect()
}

/// [`missing_required`] without Docker Sandboxes: without the others the
/// editor and the consoles do not open; without it they run on the PC, and
/// the notice of the isolation says so (D-167).
pub fn missing_to_open(kit: &Kit) -> Vec<String> {
    let mut ids = missing_required(kit);
    ids.retain(|id| id != crate::isolation::COMPONENT);
    ids
}

/// Parts of the app installed at another version than this release's (the
/// editor after an update of Atollo). Brought to the fixed version once, at
/// start; until then the installed one keeps working, so nothing waits for
/// it (a folder in use must not block the editor). Also the components
/// marked to be installed again because the kit moved and they cannot
/// follow it (conda environments on macOS/Linux, D-92) or because their
/// bootstrap was pre-downloaded on another OS (`router::bootstrap`), also
/// when only saved lines of theirs are marked (`setup::runner::run`
/// reinstalls them).
pub fn outdated_app(kit: &Kit) -> Vec<String> {
    Catalog::get()
        .all()
        .iter()
        .filter(|c| install::installer(&c.id).is_some())
        .filter(|c| {
            (c.is_installed(kit) && (c.outdated(kit) || c.needs_reinstall(kit)))
                || crate::router::bootstrap::is_staged(kit, c)
                || !c.lines_to_reinstall(kit).is_empty()
        })
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
        let target = kit.target().as_str().to_owned();
        SetupTask {
            kit,
            host: crate::env::Host::default(),
            env: Mutex::new(env),
            settings: Mutex::new(settings),
            github_token,
            state: Mutex::new(Snapshot {
                target,
                ..Snapshot::default()
            }),
            order: Mutex::new(()),
            cancel: AtomicBool::new(false),
            pending: Mutex::new(VecDeque::new()),
            jobs: Arc::new(Jobs::new()),
        }
    }

    /// The PC's variables `sbx` runs with: the tools sandbox (D-180) needs
    /// them; without, the components that run Linux programs are staged.
    pub fn with_host(mut self, host: crate::env::Host) -> SetupTask {
        self.host = host;
        self
    }

    /// The app's jobs: this task's runs, and the VMs' preparations the
    /// app adds (`ipc::levels::AppEffects`).
    pub fn jobs(&self) -> &Arc<Jobs> {
        &self.jobs
    }

    /// New environment and settings (Settings page), for the next runs.
    pub fn set_config(&self, env: KitEnv, settings: Settings) {
        *self.env.lock().unwrap_or_else(PoisonError::into_inner) = env;
        *self.settings.lock().unwrap_or_else(PoisonError::into_inner) = settings;
    }

    /// The language of the interface (`CP_LANG` of the kit environment).
    fn lang(&self) -> &'static str {
        let env = self.env.lock().unwrap_or_else(PoisonError::into_inner);
        crate::common::i18n::resolve(env.get("CP_LANG"), None, None)
    }

    fn lock(&self) -> MutexGuard<'_, Snapshot> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn queue(&self) -> MutexGuard<'_, VecDeque<Pending>> {
        self.pending.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn snapshot(&self) -> Snapshot {
        self.lock().clone()
    }

    /// Tells `notify` the snapshot `state` holds, after letting `state` go:
    /// a notify is the window's event and, at the end of a run, the kit's
    /// environment built again (`ipc::setup::setup_notify`), and the
    /// commands that read the snapshot (`setup_state`, `setup_start`…) must
    /// not wait for it. The order lock, taken before `state` is let go,
    /// keeps the snapshots told in the order of their changes.
    fn tell(&self, state: MutexGuard<'_, Snapshot>, notify: &Notify) {
        let _order = self.order.lock().unwrap_or_else(PoisonError::into_inner);
        let snapshot = state.clone();
        drop(state);
        notify(&snapshot);
    }

    /// Starts a run in the background, or, while another one runs, puts it
    /// in the queue (it starts when the ones before it end); its job's id.
    pub fn start(
        self: &Arc<Self>,
        opts: Options,
        automatic: bool,
        notify: Notify,
    ) -> Result<String, String> {
        let target = opts.target.unwrap_or_else(|| self.kit.target());
        let (kind, title, items) = describe(&self.kit, &opts, automatic, self.lang());
        let mut state = self.lock();
        if state.running {
            let job = self.jobs.add(
                kind,
                title.clone(),
                None,
                items.clone(),
                JobState::Queued,
                automatic,
            );
            crate::common::log::info(&format!("setup queued ({job}): {opts:?}"));
            state.pending.push(PendingRun {
                job: job.clone(),
                title,
                items,
            });
            // Told with the snapshot still held: the run must reach the
            // queue before the run going on can end without it, and
            // `notify` moves there. A running snapshot is only an event.
            let order = self.order.lock().unwrap_or_else(PoisonError::into_inner);
            notify(&state);
            drop(order);
            self.queue().push_back(Pending {
                job: job.clone(),
                opts,
                automatic,
                notify,
            });
            return Ok(job);
        }
        let job = self
            .jobs
            .add(kind, title, None, items, JobState::Running, automatic);
        *state = Snapshot {
            running: true,
            automatic,
            target: target.as_str().to_owned(),
            job: Some(job.clone()),
            ..Snapshot::default()
        };
        self.cancel.store(false, Ordering::Relaxed);
        self.tell(state, &notify);
        crate::common::log::info(&format!(
            "setup from the window: {opts:?} automatic={automatic}"
        ));
        let task = Arc::clone(self);
        let first = Pending {
            job: job.clone(),
            opts,
            automatic,
            notify,
        };
        std::thread::spawn(move || task.chain(first));
        Ok(job)
    }

    /// Runs `first`, then each run of the queue, in this thread; the
    /// snapshot stays `running` until the queue is empty, its steps
    /// appended run after run.
    fn chain(&self, first: Pending) {
        let mut current = first;
        loop {
            let from = self.lock().steps.len();
            // A panic in an installer ends the run like an error: otherwise
            // `running` stays and every next run is refused until restart.
            let run = std::panic::AssertUnwindSafe(|| {
                self.run(&current.opts, &current.job, &current.notify)
            });
            let error = std::panic::catch_unwind(run).unwrap_or_else(|p| Err(panic_text(&*p)));
            let cancelled = self.cancel.load(Ordering::Relaxed);
            let mut state = self.lock();
            for step in &state.steps[from.min(state.steps.len())..] {
                crate::common::log::info(&format!("setup {}: {:?}", step.id, step.outcome));
            }
            let failed = state.steps[from.min(state.steps.len())..]
                .iter()
                .find_map(|s| match &s.outcome {
                    Some(o) if o.is_failure() => {
                        Some(format!("{}: {}", s.name, outcome_text(o, self.lang())))
                    }
                    _ => None,
                });
            let (end, why) = match (&error, cancelled) {
                (_, true) => (JobState::Cancelled, None),
                (Err(e), _) => (JobState::Error, Some(e.clone())),
                (Ok(()), _) if failed.is_some() => (JobState::Error, failed),
                _ => (JobState::Done, None),
            };
            self.jobs.update(&current.job, |j| j.finish(end, why));
            state.error = error.err();
            let next = self.queue().pop_front();
            let (mut state, next) = match next {
                Some(next) => (state, next),
                None => match self.end(state, &current.notify) {
                    Some(more) => more,
                    None => return,
                },
            };
            // The next run of the queue: the same snapshot, its steps
            // appended to the ones before (the rows keep their outcome).
            state.pending.retain(|p| p.job != next.job);
            state.running = true;
            state.cancelling = false;
            state.automatic = next.automatic;
            state.target = next
                .opts
                .target
                .unwrap_or_else(|| self.kit.target())
                .as_str()
                .to_owned();
            state.total = 0;
            state.queue.clear();
            state.job = Some(next.job.clone());
            self.cancel.store(false, Ordering::Relaxed);
            self.jobs.update(&next.job, |j| j.state = JobState::Running);
            crate::common::log::info(&format!(
                "setup from the queue ({}): {:?}",
                next.job, next.opts
            ));
            self.tell(state, &next.notify);
            current = next;
        }
    }

    /// The queue is empty: the end told to `notify` (the window's event,
    /// and the kit's environment built again, `ipc::setup::setup_notify`)
    /// before the snapshot says it. Until then `running` stays: `ensure`
    /// and the others that wait for the end read the new environment
    /// right after it, and the snapshot's lock is not held while the
    /// environment is built (seconds on a slow disk), so `setup_state`
    /// and the commands that start or cancel do not wait for it. Not under
    /// the order lock for the same reason. A run asked meanwhile is
    /// returned to go on with; a cancel asked meanwhile, which told a
    /// running snapshot, is followed by the end told once more.
    fn end<'a>(
        &'a self,
        state: MutexGuard<'a, Snapshot>,
        notify: &Notify,
    ) -> Option<(MutexGuard<'a, Snapshot>, Pending)> {
        let ended = |state: &mut Snapshot| {
            state.running = false;
            state.cancelling = false;
            state.queue.clear();
            state.pending.clear();
        };
        let mut told = state.clone();
        ended(&mut told);
        drop(state);
        // A panic there must not leave `running` for good.
        let tell = std::panic::AssertUnwindSafe(|| notify(&told));
        if let Err(p) = std::panic::catch_unwind(tell) {
            panic_text(&*p);
        }
        let mut state = self.lock();
        if let Some(next) = self.queue().pop_front() {
            return Some((state, next));
        }
        let again = state.cancelling;
        ended(&mut state);
        if again {
            self.tell(state, notify);
        }
        None
    }

    fn run(&self, opts: &Options, job: &str, notify: &Notify) -> Result<(), String> {
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
        let kit = match opts.target {
            Some(t) => self.kit.for_target(t),
            None => self.kit.clone(),
        };
        // The PC's token, or the kit's GitHub account: GitHub allows 60
        // anonymous API calls an hour, and "Update all" asks for many.
        let token = self
            .github_token
            .clone()
            .or_else(|| crate::github::token(&self.kit, &env));
        let net = Net::new(&settings, token)
            .map_err(|e| format!("{e:#}"))?
            .with_github_cache(self.kit.data().join("cache").join("github"));
        // The tools sandbox when this run prepares the VM's Linux from a PC
        // that cannot run it (D-180), made only then.
        let guest_for = |target: crate::kit::Target| {
            crate::install::guest::Guest::for_target(&self.kit, &self.host, &env, target)
        };
        let own_guest = opts.target.and_then(guest_for);
        let ctx = Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &self.cancel,
            guest: own_guest.as_ref(),
        };
        let mut sink = TaskSink {
            task: self,
            notify,
            job,
            lang: self.lang(),
        };
        // With the VM on (D-173): the PC's side first, then the chosen
        // tools for the VM's Linux, one task.
        let both = runner::run(&ctx, opts, &mut sink).and_then(|first| {
            let Some(guest) = opts.then_tools_for else {
                return Ok(first);
            };
            let guest_kit = self.kit.for_target(guest);
            let tools_sandbox = guest_for(guest);
            let guest_ctx = Ctx {
                kit: &guest_kit,
                guest: tools_sandbox.as_ref(),
                ..ctx
            };
            // Asked only with the VM on (`ipc::setup::vm_tools_target`):
            // Pi's and Antigravity's builds for its Linux too.
            let tools = Options {
                target: Some(guest),
                update_all: opts.update_all,
                force: opts.force,
                scope: runner::Scope::Tools,
                vm: Some(crate::levels::VmState::On),
                ..Options::default()
            };
            runner::run(&guest_ctx, &tools, &mut sink)
        });
        match both {
            Ok(_) => Ok(()),
            Err(e) if e.is::<Busy>() => Err(t(
                self.lang(),
                "Un'altra installazione è in corso in questo kit (un'altra finestra di Atollo o il menu del kit).",
                &[],
            )),
            Err(e) => Err(format!("{e:#}")),
        }
    }

    /// Stops the run during a download, otherwise at the end of the
    /// current step; the runs of the queue are cancelled too.
    pub fn cancel(&self, notify: &Notify) {
        let mut state = self.lock();
        let waiting: Vec<Pending> = self.queue().drain(..).collect();
        for p in &waiting {
            self.jobs
                .update(&p.job, |j| j.finish(JobState::Cancelled, None));
        }
        state.pending.clear();
        if state.running {
            self.cancel.store(true, Ordering::Relaxed);
            state.cancelling = true;
            self.tell(state, notify);
        }
    }

    /// Cancels job `id` alone: out of the queue if it waits there, stopped
    /// as [`SetupTask::cancel`] stops a run if it runs; the others go on.
    /// False when it is neither (ended, or not this task's).
    pub fn cancel_job(&self, id: &str, notify: &Notify) -> bool {
        let mut state = self.lock();
        let waiting = {
            let mut queue = self.queue();
            let at = queue.iter().position(|p| p.job == id);
            at.and_then(|i| queue.remove(i))
        };
        if waiting.is_some() {
            state.pending.retain(|p| p.job != id);
            self.jobs
                .update(id, |j| j.finish(JobState::Cancelled, None));
            self.tell(state, notify);
            return true;
        }
        if state.running && state.job.as_deref() == Some(id) {
            self.cancel.store(true, Ordering::Relaxed);
            state.cancelling = true;
            self.tell(state, notify);
            return true;
        }
        false
    }

    /// Job `id` ended (done, failed or cancelled), or is not known.
    pub fn is_over(&self, id: &str) -> bool {
        self.jobs.get(id).is_none_or(|j| j.state.is_over())
    }

    /// [`SetupTask::start`], then waits for its job to end, calling `tick`
    /// with the snapshot while it waits (a VM's preparation shows the
    /// download's progress in its own job). Blocking: off the window's
    /// thread.
    pub fn start_and_wait(
        self: &Arc<Self>,
        opts: Options,
        automatic: bool,
        notify: Notify,
        tick: &dyn Fn(&Snapshot),
    ) -> Result<(), String> {
        let job = self.start(opts, automatic, notify)?;
        while !self.is_over(&job) {
            let snapshot = self.snapshot();
            if snapshot.job.as_deref() == Some(job.as_str()) {
                tick(&snapshot);
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        Ok(())
    }
}

/// What a run is for the interface: its kind, its title (already in
/// `lang`) and the components it touches (empty: the whole choice).
pub fn describe(
    kit: &Kit,
    opts: &Options,
    automatic: bool,
    lang: &str,
) -> (JobKind, String, Vec<String>) {
    let catalog = Catalog::get();
    let name = |id: &str| {
        catalog
            .find(id)
            .map_or_else(|| id.to_owned(), |c| c.name.clone())
    };
    let (kind, title, items) = if !opts.lines.is_empty() {
        let items: Vec<String> = opts.lines.iter().map(|(id, _)| id.clone()).collect();
        let title = match opts.lines.as_slice() {
            [(id, line)] => t(lang, "Scarico {0} {1}", &[&name(id), line]),
            more => t(lang, "Scarico {0} versioni", &[&more.len().to_string()]),
        };
        (JobKind::Install, title, items)
    } else if !opts.only.is_empty() {
        let update = opts.update_all;
        let kind = if update {
            JobKind::Update
        } else {
            JobKind::Install
        };
        let title = match opts.only.as_slice() {
            [id] if update => t(lang, "Aggiorno {0}", &[&name(id)]),
            [id] => match opts.at.iter().find(|(a, _)| a == id) {
                Some((_, line)) => t(lang, "Installo {0} {1}", &[&name(id), line]),
                None => t(lang, "Installo {0}", &[&name(id)]),
            },
            _ if automatic && !update => t(lang, "Installo i componenti del kit", &[]),
            more if update => t(lang, "Aggiorno {0} strumenti", &[&more.len().to_string()]),
            more => t(lang, "Installo {0} strumenti", &[&more.len().to_string()]),
        };
        (kind, title, opts.only.clone())
    } else if opts.update_all {
        (
            JobKind::Update,
            t(lang, "Aggiorno gli strumenti", &[]),
            Vec::new(),
        )
    } else {
        (
            JobKind::Install,
            t(lang, "Applico la scelta degli strumenti", &[]),
            Vec::new(),
        )
    };
    // Another system than the kit's: the VM's Linux, or one prepared
    // from this PC.
    let title = match opts.target.filter(|t| *t != kit.target()) {
        Some(other) if crate::isolation::vm::guest_target(kit.target()) == Some(other) => {
            format!("{title} · {}", t(lang, "VM", &[]))
        }
        Some(other) => format!("{title} · {}", other.as_str()),
        None => title,
    };
    (kind, title, items)
}

/// An outcome said in a job's error.
fn outcome_text(o: &Outcome, lang: &str) -> String {
    match o {
        Outcome::Failed { message, .. } => message.clone(),
        Outcome::InUse => t(lang, "in uso: chiudi i programmi del kit e riprova", &[]),
        _ => String::new(),
    }
}

struct TaskSink<'a> {
    task: &'a SetupTask,
    notify: &'a Notify,
    job: &'a str,
    lang: &'static str,
}

impl Sink for TaskSink<'_> {
    fn emit(&mut self, event: Event) {
        let mut state = self.task.lock();
        let jobs = &self.task.jobs;
        match event {
            Event::Queue { steps } => {
                state.queue = steps
                    .into_iter()
                    .map(|(id, name)| Queued { id, name })
                    .collect();
            }
            Event::Step {
                index,
                total,
                id,
                name,
            } => {
                state.total = total;
                if let Some(i) = state
                    .queue
                    .iter()
                    .position(|q| q.id == id && q.name == name)
                {
                    state.queue.remove(i);
                }
                jobs.update(self.job, |j| j.begin_step(index, total, name.clone()));
                state.steps.push(Step {
                    id,
                    name,
                    ..Step::default()
                });
            }
            Event::Note { id, text } => {
                let text = crate::env::mask_secrets(&text);
                jobs.update(self.job, |j| j.note = Some(text.clone()));
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.note = Some(text);
                }
            }
            Event::Bytes { id, done, total } => {
                jobs.update(self.job, |j| {
                    j.bytes = Some(crate::common::jobs::JobBytes { done, total });
                });
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.done = Some(done);
                    step.size = total;
                }
            }
            Event::Done { id, outcome } => {
                let (end, why) = match &outcome {
                    o if o.is_failure() => (JobState::Error, Some(outcome_text(o, self.lang))),
                    Outcome::Cancelled => (JobState::Cancelled, None),
                    _ => (JobState::Done, None),
                };
                jobs.update(self.job, |j| j.end_step(end, why));
                if let Some(step) = state.steps.iter_mut().rfind(|s| s.id == id) {
                    step.outcome = Some(outcome);
                }
            }
        }
        self.task.tell(state, self.notify);
    }
}

/// The message of a panic caught with `catch_unwind`, logged.
pub(crate) fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    let message = payload
        .downcast_ref::<&str>()
        .map(|s| (*s).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "unknown".to_owned());
    let text = format!("internal error: {message}");
    crate::common::log::error(&text);
    text
}

#[cfg(test)]
#[path = "../../../test/rust/setup/task.rs"]
mod tests;
