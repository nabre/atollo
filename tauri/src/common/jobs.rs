//! The long work of the app as the interface follows it (LEVELS.md §8.6):
//! the installs and updates of the setup task, each with its place in the
//! queue, and the preparation of a project's VM (`levels::prepare`). One
//! model for both: a title, the step running of how many with its label,
//! the bytes of a download when known, the state, the error. Every change
//! goes to the interface as the event `job` (`ipc::events::JOB`); the
//! jobs that ended stay a while for the Activity panel.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;

/// What a job does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobKind {
    /// Components installed (a tool, a line, the kit's required ones).
    Install,
    /// Components installed brought to their new version.
    Update,
    /// A project's VM made ready (`levels::prepare`).
    VmPrepare,
}

/// Where a job (or one of its steps) stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    /// Waiting for the one before it (the setup task runs one at a time).
    Queued,
    Running,
    Done,
    Error,
    Cancelled,
}

impl JobState {
    /// Done, failed or cancelled: nothing more happens to it.
    pub fn is_over(self) -> bool {
        matches!(self, JobState::Done | JobState::Error | JobState::Cancelled)
    }
}

/// One step of a job: `index` of `total`, from 1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobStep {
    pub index: usize,
    pub total: usize,
    pub label: String,
    pub state: JobState,
    pub error: Option<String>,
}

/// The bytes of the download running, `total` when the server said it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobBytes {
    pub done: u64,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub kind: JobKind,
    /// «Installo Node.js», «Preparazione VM · shop», already translated.
    pub title: String,
    /// The project of a VM's preparation.
    pub project: Option<String>,
    /// The components a setup job touches (its `only`, or its lines'):
    /// the rows of the Tools page that show its progress.
    pub items: Vec<String>,
    /// The step running (or the last one).
    pub step: Option<JobStep>,
    /// Every step so far, for the Activity panel.
    pub steps: Vec<JobStep>,
    pub bytes: Option<JobBytes>,
    /// The installer's last note (download, checksum…).
    pub note: Option<String>,
    pub state: JobState,
    pub error: Option<String>,
    /// When it was asked (Unix ms).
    pub created: u64,
    /// Started by the app (at start, for a VM), not by a click.
    pub automatic: bool,
}

impl Job {
    /// Step `index` of `total` begins with `label`: the one before it, if
    /// still running, is done.
    pub fn begin_step(&mut self, index: usize, total: usize, label: String) {
        if let Some(last) = self.steps.last_mut() {
            if last.state == JobState::Running && last.index != index {
                last.state = JobState::Done;
            }
        }
        let step = JobStep {
            index,
            total,
            label,
            state: JobState::Running,
            error: None,
        };
        match self.steps.last_mut() {
            // The same step said again (a new label: the image's download).
            Some(last) if last.index == index => *last = step.clone(),
            _ => self.steps.push(step.clone()),
        }
        self.step = Some(step);
        self.bytes = None;
        self.note = None;
        self.state = JobState::Running;
    }

    /// The step running ends with `state` (and its error).
    pub fn end_step(&mut self, state: JobState, error: Option<String>) {
        if let Some(last) = self.steps.last_mut() {
            last.state = state;
            last.error.clone_from(&error);
            self.step = Some(last.clone());
        }
        self.bytes = None;
    }

    /// The job ends: `state`, with `error` when it failed; a step still
    /// running ends with it.
    pub fn finish(&mut self, state: JobState, error: Option<String>) {
        if let Some(last) = self.steps.last_mut() {
            if last.state == JobState::Running {
                last.state = state;
                if state == JobState::Error && last.error.is_none() {
                    last.error.clone_from(&error);
                }
            }
            self.step = Some(last.clone());
        }
        self.state = state;
        self.error = error;
        self.bytes = None;
    }
}

/// Called with a job after every change of it.
pub type Emit = Box<dyn Fn(&Job) + Send + Sync>;

/// How many ended jobs are kept for the Activity panel.
pub const KEPT: usize = 20;

/// Every job of the app, the newest last.
pub struct Jobs {
    list: Mutex<Vec<Job>>,
    next: AtomicU64,
    emit: Mutex<Option<Emit>>,
}

impl Default for Jobs {
    fn default() -> Self {
        Jobs::new()
    }
}

impl Jobs {
    /// No one told yet: [`Jobs::set_emit`] once the app's window exists.
    pub fn new() -> Jobs {
        Jobs {
            list: Mutex::new(Vec::new()),
            next: AtomicU64::new(1),
            emit: Mutex::new(None),
        }
    }

    /// Who hears every change (the app's window).
    pub fn set_emit(&self, emit: Emit) {
        *self.emit.lock().unwrap_or_else(PoisonError::into_inner) = Some(emit);
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Job>> {
        self.list.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn tell(&self, job: &Job) {
        if let Some(emit) = self
            .emit
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
        {
            emit(job);
        }
    }

    /// A new job, `state` `Queued` or `Running`; its id.
    pub fn add(
        &self,
        kind: JobKind,
        title: String,
        project: Option<String>,
        items: Vec<String>,
        state: JobState,
        automatic: bool,
    ) -> String {
        let id = format!("job-{}", self.next.fetch_add(1, Ordering::Relaxed));
        let job = Job {
            id: id.clone(),
            kind,
            title,
            project,
            items,
            step: None,
            steps: Vec::new(),
            bytes: None,
            note: None,
            state,
            error: None,
            created: crate::common::time::now_ms(),
            automatic,
        };
        {
            let mut list = self.lock();
            list.push(job.clone());
            // The ended ones beyond the last `KEPT` go, the oldest first.
            let ended = list.iter().filter(|j| j.state.is_over()).count();
            if ended > KEPT {
                let mut extra = ended - KEPT;
                list.retain(|j| {
                    if extra > 0 && j.state.is_over() {
                        extra -= 1;
                        false
                    } else {
                        true
                    }
                });
            }
        }
        self.tell(&job);
        id
    }

    /// Changes job `id` with `change`, then tells; nothing for an id
    /// unknown (a job forgotten).
    pub fn update(&self, id: &str, change: impl FnOnce(&mut Job)) {
        let job = {
            let mut list = self.lock();
            let Some(job) = list.iter_mut().find(|j| j.id == id) else {
                return;
            };
            change(job);
            job.clone()
        };
        self.tell(&job);
    }

    pub fn get(&self, id: &str) -> Option<Job> {
        self.lock().iter().find(|j| j.id == id).cloned()
    }

    /// Every job kept, the oldest first.
    pub fn list(&self) -> Vec<Job> {
        self.lock().clone()
    }
}

#[cfg(test)]
#[path = "../../../test/rust/common/jobs.rs"]
mod tests;
