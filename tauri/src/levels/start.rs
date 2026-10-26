//! From [`Ready`] to what runs (LEVELS.md §3.6): the session of a terminal
//! (a shell of the PC, the project's terminal in its VM, an agent), an
//! action or a site, a line of a creation. The only way the app opens a
//! terminal of levels 1–3, starts an action, or runs a creation's line
//! (I1, test `no_process_outside_levels`).

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

/// The arguments of `sbx` for the terminal-like session of the `Vm`
/// placement of `ready` (its sandbox started by `prepare`): the project's
/// terminal (`Shell`), an editor's task (`Line`, `sh -lc` in the folder of
/// the request), Claude Code or Codex in their sandbox (`SbxAgent`, which
/// asks the sandbox its image's `PATH`), Pi's Linux build in the project's
/// (`AgentBinary`). Antigravity waits for step 7 of §12.2. `extra`: what
/// the user typed after the agent's command (a forwarder, §6.8), after `--`
/// for `sbx run` **[unverified, Q4]**; nothing else takes it.
pub fn vm_session_args(kit: &Kit, ready: &Ready, extra: &[String]) -> Result<Vec<String>> {
    let (
        Placement::Vm {
            workdir, program, ..
        },
        Some(vm),
        Environment::Vm { sbx_env, .. },
    ) = (&ready.placement, &ready.vm, &ready.env)
    else {
        bail!("levels: not a session in a VM");
    };
    let agent = matches!(
        program,
        VmProgram::SbxAgent { .. } | VmProgram::AgentBinary { .. }
    );
    if !agent && !extra.is_empty() {
        bail!("levels: arguments only after an agent's command");
    }
    match program {
        VmProgram::Shell => Ok(vm::kit_terminal_args(&vm.name, workdir, vm.tools())),
        VmProgram::Line { command } => Ok(vm::exec_args(
            &vm.name,
            Some(workdir),
            command,
            &[],
            vm.tools(),
        )),
        VmProgram::SbxAgent { .. } => {
            let mut args = vm.agent_session_args(sbx_env)?;
            if !extra.is_empty() {
                args.push("--".into());
                args.extend(extra.iter().cloned());
            }
            Ok(args)
        }
        VmProgram::AgentBinary { agent: Agent::Pi } => {
            let pi = vm::guest_target(kit.target())
                .and_then(|t| isolation::pi_in_vm(kit, &kit.for_target(t)))
                .ok_or_else(|| anyhow!("pi: no build for the VM's Linux"))?;
            Ok(vm::pi_session_args(
                &vm.name,
                workdir,
                &pi,
                vm.tools(),
                extra,
            ))
        }
        // Step 7 of LEVELS.md §12.2 (`vm::agy_session_args`); until then
        // `ipc::agents::refused_on_project` refuses it before `decide`.
        VmProgram::AgentBinary { agent } => bail!("{}: not in the VM yet", agent.id()),
        other => bail!("levels: {other:?} is not a terminal"),
    }
}

/// The pty session of a terminal-like placement (a shell of the PC, or
/// `Shell`, `SbxAgent`, `AgentBinary` in a VM), with the environment and
/// the folder it opens with; its `PtyInfo` carries the placement's level,
/// label and sandbox (§8.3).
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
            let what = match program {
                VmProgram::SbxAgent { agent } | VmProgram::AgentBinary { agent } => agent.id(),
                _ => "VM",
            };
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
                program: VmProgram::AgentBinary { agent: Agent::Pi },
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

/// Starts `action` of `project` (folder `dir`) as `ready` placed it: on
/// the PC with its environment (R5), or in the project's VM (R16, R17)
/// through `sbx exec`, its port published there.
pub fn start_action(
    runner: &Arc<Runner>,
    ready: &Ready,
    project: &str,
    dir: &Path,
    action: &crate::detect::Action,
    lang: &str,
) -> Result<RunInfo> {
    match (&ready.env, &ready.vm) {
        (Environment::Pc(env), None) => runner.start_in(project, dir, action, env, lang, None),
        (Environment::Vm { sbx_env, .. }, Some(vm)) => {
            runner.start_in(project, dir, action, sbx_env, lang, Some(vm))
        }
        _ => bail!("levels: not ready for an action"),
    }
}

/// What a creation's lines get in the VM besides the databases' variables:
/// `vars` (its `.env`'s values, the database's connection), in a file
/// readable by its owner only, never on a command line.
pub const SCAFFOLD_ENV: &str = "scaffold.env";

/// One line of a creation, in `at` with `vars` in its environment: on the
/// PC the kit's PowerShell with the placement's environment (R5); in the
/// new project's VM (R15) the kit's PowerShell for the VM's Linux through
/// `sbx exec`, `at` inside the project's folder (its workspace).
pub fn scaffold_command(
    kit: &Kit,
    ready: &Ready,
    line: &str,
    at: &Path,
    vars: &[(String, String)],
) -> Result<std::process::Command> {
    match (&ready.env, &ready.vm) {
        (Environment::Pc(env), None) => Ok(crate::projects::frameworks::line_command(
            kit, env, vars, at, line,
        )),
        (Environment::Vm { sbx_env, .. }, Some(vm)) => {
            let windows = kit.target().is_windows();
            let guest = vm::guest_target(kit.target())
                .map(|t| kit.for_target(t))
                .ok_or_else(|| anyhow!("levels: no VM on this system"))?;
            let pwsh = crate::setup::kit_path(&guest, r"pwsh\pwsh.exe");
            let pwsh = vm::guest_path(windows, &pwsh)
                .ok_or_else(|| anyhow!("{}: no path in the VM", pwsh.display()))?;
            let dir = vm::guest_path(windows, at)
                .ok_or_else(|| anyhow!("{}: no path in the VM", at.display()))?;
            let mut tools = vm.tools().clone();
            let file = isolation::kit::dir(kit, &vm.name).join(SCAFFOLD_ENV);
            let mut text = tools
                .env_file
                .as_ref()
                .and_then(|f| std::fs::read_to_string(f).ok())
                .unwrap_or_default();
            for (k, v) in vars {
                text.push_str(&format!("{k}={v}\n"));
            }
            if let Some(parent) = file.parent() {
                std::fs::create_dir_all(parent)?;
            }
            crate::platform::files::replace_private(&file, &text)?;
            tools.env_file = Some(file);
            let args = vm::scaffold_args(
                &vm.name,
                &dir,
                &pwsh,
                &crate::projects::frameworks::encoded_script(line),
                &crate::projects::frameworks::QUIET,
                &tools,
            );
            Ok(vm.command(sbx_env, &args))
        }
        _ => bail!("levels: not ready for a creation"),
    }
}

/// The creation is over: its variables' file in the VM's kit goes.
pub fn scaffold_end(kit: &Kit, ready: &Ready) {
    if let Some(vm) = &ready.vm {
        let file = isolation::kit::dir(kit, &vm.name).join(SCAFFOLD_ENV);
        if let Err(e) = std::fs::remove_file(&file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                crate::common::log::warn(&format!("{}: {e}", file.display()));
            }
        }
    }
}
