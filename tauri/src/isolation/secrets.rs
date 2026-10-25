//! The kit's secrets for the VMs (D-172): what the proxy of Docker
//! Sandboxes adds to a VM's requests (`sbx secret`), never inside the VM.
//!
//! `sbx` keeps its secrets on the PC, not in the kit (Windows' Credential
//! Manager, `com.docker.sandboxes:…`: tried with sbx 0.49.2). So the kit
//! gives it no copy: it registers a command of its own (`--command`),
//! which `sbx` runs on the PC when it needs the value: `Atollo.exe secret
//! github` prints the token the kit's `gh` holds, and only when the kit's
//! `sbx` asks ([`asked_by_sbx`]). Signed out in the kit, the command fails
//! and nothing is added. On another PC, or after the kit moved, the
//! command is registered again.
//!
//! Never global: a VM runs the project's code, which could act as the
//! user on GitHub through the proxy. The secret is given to the sandbox of
//! a project the user allowed (`github_in_vm`), that one only
//! (`--sandbox`), and taken back when it is turned off. A secret `sbx` has
//! that Atollo did not register (the user's own) is never touched: Atollo
//! notes, per PC and sandbox, the commands it registered
//! (`data\app\sandboxes\secrets.json`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kit::Kit;

/// True when `sbx` (the kit's) is among the processes that started this
/// one (`ancestors`, nearest first): `secret github` answers it only, not
/// an agent or a project's program on the PC that runs the app (D-172).
pub fn asked_by_sbx(ancestors: &[&crate::platform::processes::Proc], sbx: &Path) -> bool {
    let key = crate::platform::processes::path_key(&sbx.display().to_string());
    ancestors
        .iter()
        .any(|p| crate::platform::processes::path_key(&p.path.display().to_string()) == key)
}

/// What Atollo registered on one PC for one service, for one sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Noted {
    pub computer: String,
    pub sandbox: String,
    pub command: String,
}

fn file(kit: &Kit) -> std::path::PathBuf {
    super::kit::root(kit).join("secrets.json")
}

/// The commands Atollo registered, by service.
pub fn noted(kit: &Kit) -> BTreeMap<String, Vec<Noted>> {
    std::fs::read_to_string(file(kit))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// What Atollo noted for `service` on `computer` for `sandbox`.
pub fn mine(kit: &Kit, service: &str, computer: &str, sandbox: &str) -> Option<Noted> {
    noted(kit)
        .remove(service)?
        .into_iter()
        .find(|n| n.computer == computer && n.sandbox == sandbox)
}

/// Notes `command` registered for `service` on `computer` for `sandbox`
/// (one entry each), or with `None` that it was taken back.
pub fn note(
    kit: &Kit,
    service: &str,
    computer: &str,
    sandbox: &str,
    command: Option<&str>,
) -> anyhow::Result<()> {
    let mut all = noted(kit);
    let list = all.entry(service.to_owned()).or_default();
    list.retain(|n| !(n.computer == computer && n.sandbox == sandbox));
    if let Some(command) = command {
        list.push(Noted {
            computer: computer.to_owned(),
            sandbox: sandbox.to_owned(),
            command: command.to_owned(),
        });
    }
    let path = file(kit);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, serde_json::to_string_pretty(&all)? + "\n")?;
    Ok(())
}

/// The command `sbx` runs for the kit's GitHub token: the app with
/// `secret github`, its path quoted as `sbx` splits the command (a path
/// with spaces unquoted fails: tried), for the PC's system.
pub fn github_command(exe: &Path, windows: bool) -> String {
    let path = exe.display().to_string();
    let quoted = if windows {
        format!("\"{path}\"")
    } else {
        crate::platform::sh_quote(&path)
    };
    format!("{quoted} secret github")
}

/// What to do with `service`, from whether `sbx` lists it (`listed`) and
/// what Atollo noted for this PC (`mine`) against the command now
/// (`command`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Nothing: registered already, or the user's own.
    Keep,
    /// Registered for the first time.
    Register,
    /// Registered again over Atollo's own (the kit moved).
    Replace,
}

pub fn step(listed: bool, mine: Option<&Noted>, command: &str) -> Step {
    match (listed, mine) {
        (false, _) => Step::Register,
        (true, Some(n)) if n.command != command => Step::Replace,
        (true, _) => Step::Keep,
    }
}

/// True when `sbx secret ls` lists the secret of `service` for `scope`
/// (a sandbox's name, or `(global)`).
pub fn listed(ls: &str, service: &str, scope: &str) -> bool {
    ls.lines().any(|l| {
        let words: Vec<&str> = l.split_whitespace().collect();
        words.first() == Some(&scope) && words.contains(&service)
    })
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/secrets.rs"]
mod tests;
