//! The editor's terminals in a project's VM (D-175). VSCodium's terminal
//! profiles, its tasks and its agents' profiles all start the kit's
//! PowerShell with `data/kit/vm.ps1` (`editor::cfg`), which reads what the
//! app noted here, in `data/app/sandboxes/sessions/`:
//!
//! - `.vm`: the VM is on (the level of the status when an editor opened):
//!   a terminal outside a project's folder is refused, never the PC's;
//! - `<project>.pending`: the app is making the project's VM ready (its
//!   tools, databases and ports): the terminal waits;
//! - `<project>/`: ready, one argument per line (no argument holds a line
//!   break): `sbx` (its path), `env` (`KEY=VALUE` lines for `sbx`'s run),
//!   `shell` (the interactive session, its `-w` folder the project's),
//!   `run` (a task's line in place of [`LINE`]), `claude` and `codex`
//!   (their sandboxes); `error`: why the VM could not be made ready.
//!
//! Without `.vm` nor a session the terminal is the kit's on the PC, as
//! everywhere without the VM. The app clears them all at its start.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::{kit, vm, Vm};
use crate::env::KitEnv;
use crate::kit::Kit;

/// What `vm.ps1` puts a task's line in place of, in `run`.
pub const LINE: &str = "__ATOLLO_LINE__";

fn root(kit: &Kit) -> PathBuf {
    kit::root(kit).join("sessions")
}

fn dir(kit: &Kit, project: &str) -> PathBuf {
    root(kit).join(project)
}

fn pending_file(kit: &Kit, project: &str) -> PathBuf {
    root(kit).join(format!("{project}.pending"))
}

/// Notes whether the VM is on for the editor's terminals.
pub fn mark_vm(kit: &Kit, on: bool) -> Result<()> {
    let file = root(kit).join(".vm");
    if on {
        std::fs::create_dir_all(root(kit))?;
        std::fs::write(&file, "1\n").with_context(|| file.display().to_string())
    } else {
        remove_file(&file)
    }
}

/// The VM of `project` is being made ready: its terminals wait.
pub fn pending(kit: &Kit, project: &str) -> Result<()> {
    clear(kit, project)?;
    std::fs::create_dir_all(root(kit))?;
    let file = pending_file(kit, project);
    std::fs::write(&file, "1\n").with_context(|| file.display().to_string())
}

/// The session of `project` in `vm` (started, its tools and databases
/// ready), for `vm.ps1`; `env` is the one of the project's programs.
pub fn write(kit: &Kit, project: &str, dir_on_pc: &Path, vm: &Vm, env: &KitEnv) -> Result<()> {
    let out = dir(kit, project);
    remove_dir(&out)?;
    std::fs::create_dir_all(&out)?;
    let lines = |name: &str, list: &[String]| -> Result<()> {
        let path = out.join(name);
        let text: String = list.iter().map(|a| format!("{a}\n")).collect();
        std::fs::write(&path, text).with_context(|| path.display().to_string())
    };
    lines("sbx", &[vm.sbx.path().display().to_string()])?;
    let session = vm.session_env(env);
    let wanted = ["TEMP", "TMP", "TMPDIR"]
        .into_iter()
        .chain(super::daemon::ENV.iter().map(|(k, _)| *k));
    let vars: Vec<String> = wanted
        .filter_map(|k| session.get(k).map(|v| format!("{k}={v}")))
        .collect();
    lines("env", &vars)?;
    let guest = vm::guest_path(kit.target().is_windows(), dir_on_pc)
        .with_context(|| format!("{}: no path in the VM", dir_on_pc.display()))?;
    lines(
        "shell",
        &vm::kit_terminal_args(&vm.name, &guest, vm.tools()),
    )?;
    lines("run", &vm::exec_args(&vm.name, None, LINE, &[], vm.tools()))?;
    for agent in ["claude", "codex"] {
        if let Some(args) = vm::agent_args(project, agent, dir_on_pc) {
            lines(agent, &args)?;
        }
    }
    remove_file(&pending_file(kit, project))
}

/// The VM of `project` could not be made ready: its terminals say why.
pub fn failed(kit: &Kit, project: &str, why: &str) -> Result<()> {
    let out = dir(kit, project);
    remove_dir(&out)?;
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("error"), format!("{why}\n"))?;
    remove_file(&pending_file(kit, project))
}

/// No session for `project`: its terminals are the kit's on the PC.
pub fn clear(kit: &Kit, project: &str) -> Result<()> {
    remove_dir(&dir(kit, project))?;
    remove_file(&pending_file(kit, project))
}

/// Every session gone (the app's start: nothing of the last run holds).
pub fn clear_all(kit: &Kit) -> Result<()> {
    remove_dir(&root(kit))
}

fn remove_file(path: &Path) -> Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| path.display().to_string())
        }
        _ => Ok(()),
    }
}

fn remove_dir(path: &Path) -> Result<()> {
    match std::fs::remove_dir_all(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| path.display().to_string())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/session.rs"]
mod tests;
