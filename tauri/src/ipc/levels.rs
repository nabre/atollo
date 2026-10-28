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
use crate::levels::report::{self, ActError, SandboxReport, VmAction, VmEffects, VmReport};
use crate::levels::{decide, facts, Agent, Facts, Placement, Request};
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

/// `levels-changed` to the window when the agents' availability flipped
/// with `status` (a status read by [`watch_levels`], or the notice's
/// forced check, `isolation_status`).
pub(crate) fn observed<R: Runtime>(app: &AppHandle<R>, status: &crate::isolation::Status) {
    let vm = crate::levels::agents::vm_state(status);
    if crate::levels::agents::TOLD.flipped(&vm) {
        tell_levels(app, &vm);
    }
}

fn tell_levels<R: Runtime>(app: &AppHandle<R>, vm: &crate::levels::VmState) {
    use tauri::{Emitter, Manager};
    let lang = app
        .try_state::<AppState>()
        .map_or("it", |state| state.lang());
    let payload = crate::levels::agents::LevelsState::of(vm, lang);
    crate::common::log::info(&format!(
        "levels: agents {}",
        if payload.agents_enabled {
            "enabled"
        } else {
            "disabled"
        }
    ));
    let _ = app.emit_to(super::MAIN, super::events::LEVELS_CHANGED, payload);
}

/// Re-enabling (LEVELS.md §7): `levels::agents::watch` asks the status at
/// its cadence from the app's start and tells each flip of the agents'
/// availability as `levels-changed`; the interface reads again its agents,
/// the Tools page and the quick panel. The editor's agent profiles follow
/// at its next start (`editor::cfg::write`).
pub(crate) fn watch_levels<R: Runtime>(app: AppHandle<R>) {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let (kit, host) = (state.kit.clone(), state.host.clone());
    let env_app = app.clone();
    crate::levels::agents::watch(
        kit,
        host,
        move || env_app.state::<AppState>().env(),
        move |vm| tell_levels(&app, vm),
    );
}

/// Answers the request files of the processes outside the app while it
/// runs (LEVELS.md §4.3, E11, E12, §6.8): each through `decide` →
/// `environment` → `prepare` (`levels::requests::reply_to`, the app's
/// [`AppEffects`]), the facts read when it comes. The editor server's
/// environment, which a PC reply's changes start from, is
/// `environment(Pc{Manual})` of the kit's (`Editor::start_on`).
pub(crate) fn serve_requests<R: Runtime>(app: AppHandle<R>) {
    use crate::levels::requests;
    use tauri::Manager;
    let kit = app.state::<AppState>().kit.clone();
    requests::serve(kit, move |asked| {
        let state = app.state::<AppState>();
        let (kit, host, base, lang) = (
            state.kit.clone(),
            state.host.clone(),
            state.env(),
            state.lang(),
        );
        let req = requests::request_of(&kit, &asked);
        let facts = facts_now(&kit, &host, &base, &req);
        let editor_env = crate::levels::env::pc_environment(
            &kit,
            base.clone(),
            crate::levels::Level::Manual,
            None,
            None,
        );
        let fx = AppEffects::of(&app, &state, base.clone());
        requests::reply_to(
            &fx,
            &kit,
            &base,
            &editor_env,
            &host,
            &req,
            &asked.args,
            &facts,
            lang,
        )
    });
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

/// The app's sessions running in `sandbox` (LEVELS.md §8.5): its
/// agents' terminals.
pub(crate) fn sessions_in<R: Runtime>(app: &AppHandle<R>, sandbox: &str) -> Vec<String> {
    use tauri::Manager;
    app.try_state::<crate::editor::terminal::Consoles>()
        .map(|c| c.ptys.list())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.running && p.sandbox.as_deref() == Some(sandbox))
        .map(|p| p.title)
        .collect()
}

/// Ends the app's sessions running in `sandbox`: its terminals closed as
/// «Chiudi» does, its actions stopped.
fn end_sessions_in<R: Runtime>(app: &AppHandle<R>, sandbox: &str) {
    use tauri::{Emitter, Manager};
    if let Some(consoles) = app.try_state::<crate::editor::terminal::Consoles>() {
        let ids: Vec<u64> = consoles
            .ptys
            .list()
            .into_iter()
            .filter(|p| p.running && p.sandbox.as_deref() == Some(sandbox))
            .map(|p| p.id)
            .collect();
        for id in ids {
            if let Err(e) = consoles.ptys.close(id, crate::editor::pty::GRACE) {
                crate::common::log::warn(&format!("{sandbox}: terminal {id}: {e:#}"));
            }
            let _ = app.emit_to(super::MAIN, super::events::PTY_CLOSED, id);
            if let Some(window) = app.get_webview_window(&crate::editor::terminal::label(id)) {
                let _ = window.close();
            }
        }
    }
}

/// The app's [`VmEffects`]: its sessions, the kit's `sbx` with the
/// environment built from `base`, `prepare` with [`AppEffects`].
pub(crate) struct AppVm<R: Runtime> {
    pub fx: AppEffects<R>,
    pub base: KitEnv,
    pub host: Host,
    pub lang: &'static str,
}

impl<R: Runtime> VmEffects for AppVm<R> {
    fn sessions(&self, sandbox: &str) -> Vec<String> {
        sessions_in(&self.fx.app, sandbox)
    }

    fn end_sessions(&self, sandbox: &str) {
        end_sessions_in(&self.fx.app, sandbox);
    }

    fn stop_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        let env = crate::isolation::sbx_session_env(&self.base, &self.host);
        crate::isolation::stop_sandbox(&self.fx.kit, &self.host, &env, sandbox)
    }

    fn remove_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        let env = crate::isolation::sbx_session_env(&self.base, &self.host);
        crate::isolation::remove_to_make_again(&self.fx.kit, &self.host, &env, sandbox)
    }

    fn prepare(&self, req: &Request, facts: &Facts) -> anyhow::Result<()> {
        ready(
            &self.fx,
            &self.fx.kit,
            &self.base,
            req,
            facts,
            &[],
            self.lang,
            &self.host,
        )
        .map(drop)
        .map_err(|e| anyhow::anyhow!("{e}"))
    }
}

/// An action's or a check's failure as the command's error: a refusal
/// with its reason and fix.
fn act_error(e: ActError, lang: &str) -> CmdError {
    match e {
        ActError::Refused { reason, fix } => CmdError::refused(reason, fix, lang),
        other => CmdError::from(report::act_error_text(&other, lang)),
    }
}

/// The agent of a sandbox the interface names (`claude`, `codex`): only
/// those have one of their own.
fn sandbox_agent(agent: Option<&str>) -> CmdResult<Option<Agent>> {
    match agent {
        None => Ok(None),
        Some(id) => match Agent::from_id(id) {
            Some(a) if a.runner() == crate::levels::AgentRunner::SbxAgent => Ok(Some(a)),
            _ => Err(format!("{id}: no sandbox of its own").into()),
        },
    }
}

/// The report of `project`'s VM now (LEVELS.md §8.1): the VM's status, the
/// sandboxes as `sbx` has them, the app's sessions.
fn report_now<R: Runtime>(app: &AppHandle<R>, state: &AppState, project: &str) -> VmReport {
    let (kit, host, env, lang) = (&state.kit, &state.host, state.env(), state.lang());
    let status = crate::isolation::status(kit, host, &env, false);
    let on = status.level == crate::isolation::Level::Vm;
    let ls = on
        .then(|| {
            crate::isolation::sandboxes_json(
                kit,
                host,
                &crate::isolation::sbx_session_env(&env, host),
            )
        })
        .and_then(Result::ok);
    let seen = report::Seen {
        on,
        reason: (!on).then(|| crate::isolation::why_no_vm(&status, lang)),
        ls,
        github: crate::projects::settings::load(kit, project).github_in_vm,
    };
    report::report(kit, project, seen, &|name| sessions_in(app, name))
}

/// «Verifica strumenti» in the sandboxes of `report` (all, or `agent`'s
/// only when `only` is `Some`), as a job (LEVELS.md §8.6): a step per
/// sandbox; each through `decide`.
fn check_now(
    state: &AppState,
    report: &mut VmReport,
    only: Option<Option<Agent>>,
) -> CmdResult<()> {
    let (kit, host, env, lang) = (&state.kit, &state.host, state.env(), state.lang());
    let dir = super::project_dir(kit, Some(&report.project), lang)?;
    let jobs = state.setup.jobs();
    let picked: Vec<usize> = (0..report.sandboxes.len())
        .filter(|i| only.is_none_or(|a| report.sandboxes[*i].agent == a))
        .collect();
    let id = jobs.add(
        JobKind::VmCheck,
        t(lang, "Verifica strumenti · {0}", &[&report.project]),
        Some(report.project.clone()),
        Vec::new(),
        JobState::Running,
        false,
    );
    let total = picked.len();
    let project = report.project.clone();
    for (n, i) in picked.into_iter().enumerate() {
        let s = &mut report.sandboxes[i];
        jobs.update(&id, |j| j.begin_step(n + 1, total, s.name.clone()));
        let req = report::sandbox_request(kit, &dir, s.agent);
        let facts = facts_now(kit, host, &env, &req);
        if let Err(e) = report::check(kit, host, &env, &req, &facts, &project, s, lang) {
            let e = act_error(e, lang);
            jobs.update(&id, |j| j.finish(JobState::Error, Some(e.to_string())));
            return Err(e);
        }
    }
    jobs.update(&id, |j| j.finish(JobState::Done, None));
    Ok(())
}

/// The report of `project`'s VM (LEVELS.md §8.1); with `check`,
/// «Verifica strumenti» in each of its sandboxes (§8.2).
#[tauri::command]
pub async fn project_vm_report<R: Runtime>(
    app: AppHandle<R>,
    webview: tauri::Webview<R>,
    project: String,
    check: bool,
) -> CmdResult<VmReport> {
    use tauri::Manager;
    super::ensure_main(&webview)?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        super::project_dir(&state.kit, Some(&project), state.lang())?;
        let mut report = report_now(&app, &state, &project);
        if check && report.vm == report::VmOnOff::On {
            check_now(&state, &mut report, None)?;
        }
        Ok(report)
    })
    .await
    .or_fail("vm")?
}

/// «Verifica strumenti» in one sandbox of `project`: its own, or `agent`'s
/// (`claude`, `codex`); the sandbox's report with its tools.
#[tauri::command]
pub async fn project_vm_check<R: Runtime>(
    app: AppHandle<R>,
    webview: tauri::Webview<R>,
    project: String,
    agent: Option<String>,
) -> CmdResult<SandboxReport> {
    use tauri::Manager;
    super::ensure_main(&webview)?;
    let agent = sandbox_agent(agent.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        super::project_dir(&state.kit, Some(&project), state.lang())?;
        let mut report = report_now(&app, &state, &project);
        if report.vm == report::VmOnOff::Off {
            return Err(act_error(ActError::NoVm, state.lang()));
        }
        check_now(&state, &mut report, Some(agent))?;
        report
            .sandboxes
            .into_iter()
            .find(|s| s.agent == agent)
            .ok_or_else(|| CmdError::from("vm: no such sandbox"))
    })
    .await
    .or_fail("vm")?
}

/// An action on a sandbox of `project` (LEVELS.md §8.5): its own, or
/// `agent`'s; «Riavvia» and «Ricrea» stop its sessions first. Through
/// `decide`, with the sandbox's lock; the preparation is a job. The
/// sessions that were stopped.
#[tauri::command]
pub async fn project_vm_action<R: Runtime>(
    app: AppHandle<R>,
    webview: tauri::Webview<R>,
    project: String,
    agent: Option<String>,
    action: VmAction,
) -> CmdResult<Vec<String>> {
    use tauri::Manager;
    super::ensure_main(&webview)?;
    let agent = sandbox_agent(agent.as_deref())?;
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        let (kit, host, lang) = (state.kit.clone(), state.host.clone(), state.lang());
        let dir = super::project_dir(&kit, Some(&project), lang)?;
        let base = super::ensure(&app, &state.setup, &kit, lang, &[])?;
        let req = report::sandbox_request(&kit, &dir, agent);
        let facts = facts_now(&kit, &host, &base, &req);
        let fx = AppVm {
            fx: AppEffects::of(&app, &state, base.clone()),
            base,
            host,
            lang,
        };
        report::act(&fx, &req, &facts, action).map_err(|e| act_error(e, lang))
    })
    .await
    .or_fail("vm")?
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/levels.rs"]
mod tests;
