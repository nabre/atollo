//! From [`Ready`] to what runs (LEVELS.md §3.6): the session of a terminal
//! (a shell of the PC, an agent in its VM), an action or a site, a line of a
//! creation. The only way the app opens a terminal of levels 1–3, starts an
//! action, or runs a creation's line (I1, test `no_process_outside_levels`).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use tauri::{AppHandle, Runtime};

use crate::editor::pty::Launch;
use crate::env::KitEnv;
use crate::isolation::{self, vm};
use crate::kit::Kit;
use crate::projects::run::{RunInfo, Runner};

use super::env::Environment;
use super::labels::SessionLabel;
use super::prepare::Ready;
use super::{Agent, Placement, VmProgram};

/// The arguments of `sbx` for the agent's session of the `Vm` placement of
/// `ready` (its sandbox started by `prepare`): the agent's Linux build from
/// the kit in the project's sandbox (D-184, `vm::agent_session_args`).
/// `extra`: what the user typed after the agent's command (a forwarder,
/// §6.8), after the program.
pub fn vm_session_args(kit: &Kit, ready: &Ready, extra: &[String]) -> Result<Vec<String>> {
    let (
        Placement::Vm {
            workdir, program, ..
        },
        Some(vm),
        Environment::Vm { .. },
    ) = (&ready.placement, &ready.vm, &ready.env)
    else {
        bail!("levels: not a session in a VM");
    };
    match program {
        VmProgram::Agent { agent } => {
            let program = vm::guest_target(kit.target())
                .and_then(|t| isolation::agent_in_vm(kit, &kit.for_target(t), agent.id()))
                .ok_or_else(|| anyhow!("{}: no build for the VM's Linux", agent.id()))?;
            Ok(vm::agent_session_args(
                &vm.name,
                workdir,
                agent.id(),
                &program,
                vm.tools(),
                extra,
            ))
        }
        VmProgram::Tools => bail!("levels: the sandbox's tools open no session"),
    }
}

/// The pty session of a terminal-like placement (a shell of the PC, or an
/// agent in a VM), with the environment and the folder it opens with; its
/// `PtyInfo` carries the placement's level, label and sandbox (§8.3).
pub fn start_terminal(kit: &Kit, ready: &Ready, lang: &str) -> Result<(Launch, KitEnv, PathBuf)> {
    let at = SessionLabel::of(&ready.placement).ok_or_else(|| anyhow!("levels: refused"))?;
    let (mut launch, env, dir) = match (&ready.placement, &ready.env, &ready.vm) {
        (Placement::Pc { dir, .. }, Environment::Pc(env), None) => {
            // The agents' commands are the forwarders, first on its PATH
            // (§6.8): they say why when they cannot forward.
            (
                crate::editor::pty::shell(kit, dir),
                env.clone(),
                dir.clone(),
            )
        }
        (Placement::Vm { program, .. }, Environment::Vm { sbx_env, .. }, Some(vm)) => {
            let args = vm_session_args(kit, ready, &[])?;
            let what = program.agent().map_or("VM", Agent::id);
            let launch = vm.launch(kit, args, what, lang)?;
            (launch, vm.session_env(sbx_env), vm.dir.clone())
        }
        _ => bail!("levels: not ready for a terminal"),
    };
    launch.at = at;
    Ok((launch, env, dir))
}

/// [`start_terminal`] opened in a session of the app's terminal. Pi's
/// session keeps its local models' port open while it runs (D-178).
pub fn open_terminal<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    ready: &Ready,
    lang: &str,
) -> Result<()> {
    let (launch, env, dir) = start_terminal(kit, ready, lang)?;
    let pi = match (&ready.placement, &ready.vm) {
        (
            Placement::Vm {
                program: VmProgram::Agent { agent: Agent::Pi },
                ..
            },
            Some(vm),
        ) => Some(isolation::pi_session(&vm.name)),
        _ => None,
    };
    let at = launch.at.clone();
    crate::editor::terminal::open_launch_then(
        app,
        &env,
        &dir,
        launch,
        at,
        Box::new(move || drop(pi)),
    )
}

/// Starts `action` of `project` (folder `dir`) on the PC with the
/// environment `ready` gave it (R2).
pub fn start_action(
    runner: &Arc<Runner>,
    ready: &Ready,
    project: &str,
    dir: &Path,
    action: &crate::detect::Action,
    lang: &str,
) -> Result<RunInfo> {
    match &ready.env {
        Environment::Pc(env) => runner.start_in(project, dir, action, env, lang),
        Environment::Vm { .. } => bail!("levels: not ready for an action"),
    }
}

/// One line of a creation, in `at` with `vars` in its environment: the
/// kit's PowerShell on the PC with the placement's environment (R2).
pub fn scaffold_command(
    kit: &Kit,
    ready: &Ready,
    line: &str,
    at: &Path,
    vars: &[(String, String)],
) -> Result<std::process::Command> {
    match &ready.env {
        Environment::Pc(env) => Ok(crate::projects::frameworks::line_command(
            kit, env, vars, at, line,
        )),
        Environment::Vm { .. } => bail!("levels: not ready for a creation"),
    }
}
