//! The levels at the app's commands (D-179, LEVELS.md §4.1, steps 3–4 of
//! §12.2): every command that starts a terminal, an action, a site, a pool,
//! a scaffold or an agent builds its `Request`, reads the facts, asks
//! `levels::decide`, and then starts on the PC with
//! `levels::env::environment`, in the VM once `levels::prepare` made it
//! ready ([`ready`], with the app's [`AppEffects`]), or returns the refusal
//! with its reason and fix.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, PoisonError};

use tauri::{AppHandle, Runtime};

use crate::common::i18n::t;
use crate::common::jobs::{JobBytes, JobKind, JobState, Jobs};
use crate::db::auth::ProjectDb;
use crate::editor::terminals::Terminal;
use crate::env::{Host, KitEnv};
use crate::isolation::Vm;
use crate::kit::Kit;
use crate::levels::env::{environment, Environment};
use crate::levels::prepare::{prepare, Effects, LocalModels, PrepareProgress, Ready};
use crate::levels::{decide, facts, Facts, Placement, Request};
use crate::setup::task::SetupTask;

use super::{AppState, CmdError, CmdResult, OrFail};

/// A placement on the PC (it says the folder) with its environment.
#[derive(Debug)]
pub(crate) struct Placed {
    pub placement: Placement,
    pub env: KitEnv,
}

/// `decide(req, facts)`, a refusal as the command's error: its text in
/// `lang`, its reason and its fix (LEVELS.md §4.1).
pub(crate) fn admitted(req: &Request, facts: &Facts, lang: &str) -> CmdResult<Placement> {
    match decide(req, facts) {
        Placement::Refused { reason, fix, .. } => Err(CmdError::refused(reason, fix, lang)),
        placement => Ok(placement),
    }
}

/// The environment of the `Pc` placement `placement`, built from the kit's
/// `base` by `levels::env::environment` (`terminal`: a kit terminal's tool
/// choice); an error for any other placement.
pub(crate) fn pc_env(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<KitEnv> {
    match environment(kit, base, placement, terminal, lang, host) {
        Some(Environment::Pc(env)) => Ok(env),
        _ => Err("levels: not a placement on the PC".into()),
    }
}

/// `req` placed on the PC with `facts`: [`admitted`], then the placement
/// with its environment ([`pc_env`] from `base`). For the entry points that
/// only ever start on the PC (a kit terminal, a pool); a `Vm` placement is
/// an error there (the VM's go through [`ready`]).
pub(crate) fn placed(
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    facts: &Facts,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<Placed> {
    let placement = admitted(req, facts, lang)?;
    if !matches!(placement, Placement::Pc { .. }) {
        return Err("levels: placed in a VM, made ready by levels::prepare".into());
    }
    let env = pc_env(kit, base, &placement, terminal, lang, host)?;
    Ok(Placed { placement, env })
}

/// The facts of `req` read now (`levels::facts::facts`: the VM's status at
/// the moment of the request; `env` the kit's environment that asks `sbx`).
pub(crate) fn facts_now(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts {
    facts::facts(kit, host, env, req)
}

/// [`placed`] with the facts read now.
pub(crate) fn place_now(
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> CmdResult<Placed> {
    let facts = facts_now(kit, host, base, req);
    placed(kit, base, req, &facts, terminal, lang, host)
}

/// `req` made ready with `facts` (LEVELS.md §3.6): [`admitted`], its
/// environment from the kit's `base`, then `levels::prepare` with `fx`
/// (`needs`: the tools the program needs besides the project's). The one
/// way of every entry point that may start in a VM: the project terminal
/// (E3), an action or a site (E4/E5), a creation (E9), an agent (E10, E11),
/// the editor's sessions until step 5.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ready(
    fx: &dyn Effects,
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    facts: &Facts,
    needs: &[String],
    lang: &str,
    host: &Host,
) -> CmdResult<Ready> {
    let placement = admitted(req, facts, lang)?;
    let env = environment(kit, base, &placement, None, lang, host)
        .ok_or_else(|| CmdError::from("levels: refused"))?;
    prepare(fx, kit, host, placement, env, needs, lang).or_fail_chain("vm")
}

/// [`ready`] with the facts read now.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ready_now(
    fx: &dyn Effects,
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    needs: &[String],
    lang: &str,
    host: &Host,
) -> CmdResult<Ready> {
    let facts = facts_now(kit, host, base, req);
    ready(fx, kit, base, req, &facts, needs, lang, host)
}

/// The app's [`Effects`]: the setup task installs for the VM's Linux, the
/// database servers and llama-server start with the kit's environment
/// (`kit_env`), `sbx` starts the sandbox.
pub(crate) struct AppEffects<R: Runtime> {
    pub app: AppHandle<R>,
    pub task: Arc<SetupTask>,
    pub services: Arc<crate::db::services::Services>,
    pub kit: Kit,
    pub kit_env: KitEnv,
    /// The job of the preparation (LEVELS.md §8.6), once it began, with
    /// its step running (index, total).
    pub job: VmJob,
}

/// The job of a VM's preparation as the app tells it: its id and step.
#[derive(Default)]
pub(crate) struct VmJob {
    id: Mutex<Option<String>>,
    step: Mutex<(usize, usize)>,
}

impl VmJob {
    /// `event` in the job of `jobs` (made at `Begin`), in `lang`.
    pub(crate) fn tell(&self, jobs: &Jobs, event: PrepareProgress, lang: &str) {
        let mut id = self.id.lock().unwrap_or_else(PoisonError::into_inner);
        match event {
            PrepareProgress::Begin {
                project,
                agent,
                total,
                ..
            } => {
                let title = match agent {
                    Some(agent) => t(
                        lang,
                        "Preparazione VM · {0} · {1}",
                        &[project.as_str(), agent.id()],
                    ),
                    None => t(lang, "Preparazione VM · {0}", &[&project]),
                };
                *id = Some(jobs.add(
                    JobKind::VmPrepare,
                    title,
                    Some(project),
                    Vec::new(),
                    JobState::Running,
                    true,
                ));
                *self.step.lock().unwrap_or_else(PoisonError::into_inner) = (0, total);
            }
            PrepareProgress::Step {
                index,
                total,
                label,
            } => {
                *self.step.lock().unwrap_or_else(PoisonError::into_inner) = (index, total);
                if let Some(id) = id.as_deref() {
                    jobs.update(id, |j| j.begin_step(index, total, label));
                }
            }
            PrepareProgress::Done => {
                if let Some(id) = id.take() {
                    jobs.update(&id, |j| j.finish(JobState::Done, None));
                }
            }
            PrepareProgress::Failed { error } => {
                if let Some(id) = id.take() {
                    jobs.update(&id, |j| j.finish(JobState::Error, Some(error)));
                }
            }
        }
    }

    /// The step running said again with `label` (the image's download).
    pub(crate) fn relabel(&self, jobs: &Jobs, label: String) {
        let (index, total) = *self.step.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(id) = self
            .id
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_deref()
        {
            jobs.update(id, |j| j.begin_step(index, total, label));
        }
    }

    /// The download of the setup task's run in the step running: its
    /// component as the note, its bytes.
    pub(crate) fn follow(&self, jobs: &Jobs, snapshot: &crate::setup::task::Snapshot) {
        let Some(step) = snapshot.steps.last() else {
            return;
        };
        if let Some(id) = self
            .id
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_deref()
        {
            jobs.update(id, |j| {
                j.note = Some(step.name.clone());
                j.bytes = step.done.map(|done| JobBytes {
                    done,
                    total: step.size,
                });
            });
        }
    }
}

impl<R: Runtime> AppEffects<R> {
    /// The effects of the app of `state`, the kit's environment `kit_env`.
    pub fn of(app: &AppHandle<R>, state: &AppState, kit_env: KitEnv) -> AppEffects<R> {
        AppEffects {
            app: app.clone(),
            task: Arc::clone(&state.setup),
            services: Arc::clone(&state.services),
            kit: state.kit.clone(),
            kit_env,
            job: VmJob::default(),
        }
    }

    fn lang(&self) -> &'static str {
        use tauri::Manager;
        self.app
            .try_state::<AppState>()
            .map_or("it", |state| state.lang())
    }
}

impl<R: Runtime> Effects for AppEffects<R> {
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>) {
        let jobs = self.task.jobs();
        super::terminals::ensure_guest_tools(&self.app, &self.task, guest, wanted, &|snapshot| {
            self.job.follow(jobs, snapshot);
        });
    }

    fn start_databases(&self, project: &str) -> anyhow::Result<Vec<(ProjectDb, u16)>> {
        self.services
            .project_dbs(project)
            .into_iter()
            .map(|db| {
                let port = self
                    .services
                    .start_for(project, &db.engine, &self.kit_env)?;
                Ok((db, port))
            })
            .collect()
    }

    fn start_local_models(&self, lang: &str) -> Option<LocalModels> {
        use tauri::Manager;
        crate::agents::sync::pi::local_model(&self.kit)?;
        let port = crate::llm::settings::load(&self.kit).port?;
        let key = crate::llm::settings::key(&self.kit)?;
        if let Some(state) = self.app.try_state::<AppState>() {
            if let Err(e) = state.llm.start(&state.env(), lang) {
                crate::common::log::warn(&format!("pi: local models: {e:#}"));
            }
        }
        Some(LocalModels { port, key })
    }

    fn start_sandbox(&self, vm: &Vm, env: &KitEnv, lang: &str) -> anyhow::Result<()> {
        // `sbx` says nothing while it pulls the image: the step says it.
        vm.start(env, lang, &|| {
            self.job.relabel(
                self.task.jobs(),
                t(lang, "Scarico l'immagine della VM…", &[]),
            );
        })
    }

    fn watch_local_models(
        &self,
        vm: &Vm,
        env: &KitEnv,
        project: &str,
        port: u16,
        db_ports: &[u16],
    ) {
        crate::isolation::watch_pi_port(
            vm.clone(),
            env.clone(),
            project.to_owned(),
            port,
            db_ports.to_vec(),
        );
    }

    fn progress(&self, event: PrepareProgress) {
        self.job.tell(self.task.jobs(), event, self.lang());
    }
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/levels.rs"]
mod tests;
