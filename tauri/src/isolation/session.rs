//! The editor's terminals in a project's VM (D-175). VSCodium's terminal
//! profiles, its tasks and its agents' profiles all start the kit's
//! PowerShell with `data/kit/vm.ps1` (`editor::cfg`), which reads what the
//! app noted here, in `data/app/sandboxes/sessions/`:
//!
//! - `.level`: `vm` or `env`, the level of the status when an editor last
//!   opened, written before its window: fail closed, nothing runs on the
//!   PC (D-178): with `env` a terminal shows `.reason` (why the VM is off,
//!   what to do) and opens nothing; with `vm` a folder that is no
//!   project's, or a project not made ready, opens nothing;
//! - `<project>.pending`: the app is making the project's VM ready (its
//!   tools, databases and ports): the terminal waits;
//! - `<project>/`: ready, one argument per line (no argument holds a line
//!   break): `sbx` (its path), `env` (`KEY=VALUE` lines for `sbx`'s run),
//!   `shell` (the interactive session, its `-w` folder the project's),
//!   `run` (a task's line in place of [`LINE`]); `error`: why the VM could
//!   not be made ready;
//! - `<project>.<agent>.request`: an editor's profile of Claude Code,
//!   Codex or Pi asks for its session; the app makes its sandbox ready like
//!   the project's (tools, database, ports; Pi's is the project's own VM)
//!   and writes `<project>/<agent>`, the session's arguments, or
//!   `<project>/<agent>.error`;
//! - `<project>.pi.<pid>`: `vm.ps1` runs Pi's `sbx exec` for the project,
//!   `<pid>` its own PowerShell's (`$PID`); written before, removed when
//!   `sbx exec` returns ([`pi_markers`]). Beside `<project>/`, which the
//!   next editor's start rewrites: a running Pi keeps its marker. The
//!   folder is the PC's only, never mounted in a VM: the project's code
//!   cannot write one (D-178).
//!
//! The app clears them all at its start: until an editor opens, no level
//! is known and a terminal opens nothing.

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

/// Notes the level for the editor's terminals: `vm`, or `env` with `why`
/// (D-178: nothing of the editor opens on the PC, `vm.ps1` shows why the VM
/// is off and what to do). The reason is written first: a terminal that
/// reads `env` finds it.
pub fn set_level(kit: &Kit, vm: bool, why: &str) -> Result<()> {
    std::fs::create_dir_all(root(kit))?;
    let reason = root(kit).join(".reason");
    if vm {
        remove_file(&reason)?;
    } else {
        std::fs::write(&reason, format!("{why}\n"))
            .with_context(|| reason.display().to_string())?;
    }
    let file = root(kit).join(".level");
    let text = if vm { "vm\n" } else { "env\n" };
    std::fs::write(&file, text).with_context(|| file.display().to_string())
}

/// What `vm.ps1` shows when the VM is off (`.reason`), in `lang`: the
/// editor's terminals, tasks and agents start only in the project's VM,
/// then why it is off and what to do ([`super::why_no_vm`]).
pub fn no_vm_text(status: &super::Status, lang: &str) -> String {
    format!(
        "{}\n{}",
        crate::common::i18n::t(
            lang,
            "Senza VM: i terminali, i task e gli agenti dell'editor partono solo nella VM del progetto, e sul PC non si apre nulla.",
            &[],
        ),
        super::why_no_vm(status, lang)
    )
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
    remove_file(&pending_file(kit, project))
}

/// The agents' sessions the editor's profiles asked for
/// (`<project>.<agent>.request`, written by `vm.ps1`), each taken once:
/// only those that start in the VM (`claude`, `codex`, `pi`), of a project
/// with a session.
pub fn take_requests(kit: &Kit) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for entry in std::fs::read_dir(root(kit)).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".request") else {
            continue;
        };
        let _ = std::fs::remove_file(entry.path());
        let Some((project, agent)) = stem.rsplit_once('.') else {
            continue;
        };
        if vm::starts_in_vm(agent) && dir(kit, project).is_dir() {
            out.push((project.to_owned(), agent.to_owned()));
        }
    }
    out
}

/// The session of `agent` in its sandbox for `project` ([`Vm::agent_session_args`]),
/// for the editor's profile of that agent.
pub fn write_agent(kit: &Kit, project: &str, agent: &str, args: &[String]) -> Result<()> {
    let out = dir(kit, project);
    std::fs::create_dir_all(&out)?;
    remove_file(&out.join(format!("{agent}.error")))?;
    let text: String = args.iter().map(|a| format!("{a}\n")).collect();
    let path = out.join(agent);
    std::fs::write(&path, text).with_context(|| path.display().to_string())
}

/// The sandbox of `agent` for `project` could not be made ready.
pub fn agent_failed(kit: &Kit, project: &str, agent: &str, why: &str) -> Result<()> {
    let out = dir(kit, project);
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join(format!("{agent}.error")), format!("{why}\n"))?;
    Ok(())
}

/// What follows `<project>` in the name of a marker of an editor's Pi:
/// `<project>.pi.<pid>` (`vm.ps1`).
pub const PI_MARKER: &str = ".pi.";

/// The PowerShell pids of the markers of `project` among the file names
/// `names` (`<project>.pi.<pid>`, the pid digits only): another project's
/// (`shop.pi.5.pi.7` is `shop.pi.5`'s), a request or anything else is not
/// one.
pub fn pi_marker_pids<'a>(names: impl IntoIterator<Item = &'a str>, project: &str) -> Vec<u32> {
    let prefix = format!("{project}{PI_MARKER}");
    let mut pids: Vec<u32> = names
        .into_iter()
        .filter_map(|name| name.strip_prefix(&prefix))
        .filter(|pid| !pid.is_empty() && pid.bytes().all(|b| b.is_ascii_digit()))
        .filter_map(|pid| pid.parse().ok())
        .collect();
    pids.sort_unstable();
    pids.dedup();
    pids
}

/// The PowerShell pids of the editor's Pi sessions of `project` that
/// `vm.ps1` noted ([`pi_marker_pids`]); a stale marker is told apart by
/// its pid (`isolation::editor_pis_alive`).
pub fn pi_markers(kit: &Kit, project: &str) -> Vec<u32> {
    let names: Vec<String> = std::fs::read_dir(root(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    pi_marker_pids(names.iter().map(String::as_str), project)
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
