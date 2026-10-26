//! The facts of a request (LEVELS.md §3.3): the only read before
//! [`super::decide`], which then needs nothing else. Reading only: nothing
//! here starts, installs or writes.

use std::path::PathBuf;

use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm};
use crate::kit::Kit;

use super::{AgentRunner, Facts, Origin, Place, Request, VmState};

/// Reads the facts for `req`: `isolation::status(kit, host, env, false)`
/// (cached at most 60 s, `FRESH`), `vm::guest_path`,
/// `isolation::restart_pending`, and for a ProjectVm agent the catalog's
/// availability on `vm::guest_target`.
pub fn facts(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts {
    let status = isolation::status(kit, host, env, false);
    let vm = match status.level {
        isolation::Level::Vm => VmState::On,
        isolation::Level::Env => VmState::Off {
            blockers: status.blockers,
        },
    };
    Facts {
        vm,
        guest_dir: guest_dir(kit, &req.place),
        restart_pending: isolation::restart_pending(kit),
        linux_build: linux_build(kit, &req.origin),
        guest_tools: guest_tools(kit),
    }
}

/// The folder of `place` as the VM sees it: a project's own folder (its
/// `below` is added by `decide`), or the folder itself.
pub fn guest_dir(kit: &Kit, place: &Place) -> Option<String> {
    let dir = match place {
        Place::Project { dir, .. } | Place::Folder { dir } => dir,
    };
    vm::guest_path(kit.os().is_windows(), dir)
}

/// True when `origin` asks for a ProjectVm agent (Pi, Antigravity) and the
/// catalog has its build for the VM's Linux; `false` for every other origin.
pub fn linux_build(kit: &Kit, origin: &Origin) -> bool {
    let Some(agent) = origin
        .agent()
        .filter(|a| a.runner() == AgentRunner::ProjectVm)
    else {
        return false;
    };
    let Some(guest) = vm::guest_target(kit.target()) else {
        return false;
    };
    crate::setup::Catalog::get()
        .find(agent.id())
        .is_some_and(|c| c.is_available_on(guest))
}

/// The kit's tools for the VM's Linux, mounted read-only (D-170); none where
/// the PC has no VM.
pub fn guest_tools(kit: &Kit) -> Vec<PathBuf> {
    vm::guest_target(kit.target())
        .map(|t| vec![kit.for_target(t).apps()])
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "../../../test/rust/levels/facts.rs"]
mod tests;
