//! The facts of a request (LEVELS.md §3.3): the only read before
//! [`super::decide`], which then needs nothing else. Reading only: nothing
//! here starts, installs or writes.

use std::path::PathBuf;

use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm};
use crate::kit::Kit;

use super::{Facts, Origin, Place, Request};

/// Reads the facts for `req`: `isolation::status(kit, host, env, false)`
/// (cached at most 60 s, `FRESH`), `vm::guest_path`,
/// `isolation::restart_pending`, and for a ProjectVm agent the catalog's
/// availability on `vm::guest_target`.
pub fn facts(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts {
    let status = isolation::status(kit, host, env, false);
    let workspace = workspace(kit, host, &req.place);
    // A folder's sandbox mounts its canonical path (D-187).
    let guest_dir = match &workspace {
        Some(dir) => vm::guest_path(kit.os().is_windows(), dir),
        None => guest_dir(kit, &req.place),
    };
    Facts {
        vm: super::agents::vm_state(&status),
        guest_dir,
        restart_pending: isolation::restart_pending(kit),
        linux_build: linux_build(kit, &req.origin),
        guest_tools: guest_tools(kit),
        workspace,
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

/// For a folder that is not a project: its canonical path when it may be a
/// sandbox's workspace (`isolation::vm_workspace`, D-187); `None` for a
/// project and for a folder that may not.
pub fn workspace(kit: &Kit, host: &Host, place: &Place) -> Option<PathBuf> {
    match place {
        Place::Folder { dir } => isolation::vm_workspace(kit, host, dir),
        Place::Project { .. } => None,
    }
}

/// True when `origin` asks for an agent and the catalog has its build for
/// the VM's Linux (D-184: every agent runs its kit's build); `false` for
/// every other origin.
pub fn linux_build(kit: &Kit, origin: &Origin) -> bool {
    let Some(agent) = origin.agent() else {
        return false;
    };
    let Some(guest) = vm::guest_target(kit.target()) else {
        return false;
    };
    crate::setup::Catalog::get()
        .find(agent.id())
        .is_some_and(|c| c.is_available_on(guest))
}

/// The kit's tools for the VM's Linux, mounted read-only (D-170): its
/// `data/apps/<linux>` and `data/npm-global` (pnpm and yarn, D-180), the
/// folders the tools sandbox writes, and the `php.ini` of its PHP
/// (`install::guest::vm_mounts`, D-181); none where the PC has no VM.
pub fn guest_tools(kit: &Kit) -> Vec<PathBuf> {
    vm::guest_target(kit.target())
        .map(|t| crate::install::guest::vm_mounts(&kit.for_target(t)))
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "../../../test/rust/levels/facts.rs"]
mod tests;
