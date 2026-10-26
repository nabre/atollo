//! The levels at the app's commands (D-179, LEVELS.md §4.1, steps 3–4 of
//! §12.2): every command that starts a terminal, an action, a site, a pool,
//! a scaffold or an agent builds its `Request`, reads the facts, asks
//! `levels::decide`, and then starts on the PC with
//! `levels::env::environment`, in the VM once `levels::prepare` made it
//! ready ([`ready`], with the app's [`AppEffects`]), or returns the refusal
//! with its reason and fix.

use std::collections::BTreeMap;
use std::sync::Arc;

use tauri::{AppHandle, Runtime};

use crate::db::auth::ProjectDb;
use crate::editor::terminals::Terminal;
use crate::env::{Host, KitEnv};
use crate::isolation::Vm;
use crate::kit::Kit;
use crate::levels::env::{environment, Environment};
use crate::levels::prepare::{prepare, Effects, LocalModels, Ready};
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

/// Answers the request files of the processes outside the app while it
/// runs (LEVELS.md §4.3, E11, E12, §6.8): each through `decide` →
/// `environment` → `prepare` (`levels::requests::reply_to`, the app's
/// [`AppEffects`]), the facts read when it comes. The editor server's
/// environment, which a PC reply's changes start from, is
/// `environment(Pc{Manual})` of the kit's (`Editor::start_on`).
/// Antigravity is refused before `decide` until step 7 of §12.2.
pub(crate) fn serve_requests<R: Runtime>(app: AppHandle<R>) {
    use crate::levels::requests::{self, Reply};
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
        if let Some(agent) = asked.origin.agent() {
            if let Some(why) = super::agents::refused_on_project(agent.id(), lang) {
                return Reply::error(why);
            }
        }
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
        }
    }
}

impl<R: Runtime> Effects for AppEffects<R> {
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>) {
        super::terminals::ensure_guest_tools(&self.app, &self.task, guest, wanted);
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
        vm.start(env, lang)
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
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/levels.rs"]
mod tests;
