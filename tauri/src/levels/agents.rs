//! Agents only with the VM (D-179, LEVELS.md §7, step 6 of §12.2; Q11).
//!
//! An agent runs only in a VM, so without the VM everything about agents
//! is disabled with the reason: nothing of level 3 is installed, updated,
//! launched or offered in the editor (I6). One function decides it,
//! [`agents_enabled`]; the Tools page, the runner's plans, the agents'
//! buttons, the palette and the editor's profiles all read it.
//!
//! Where the kit keeps an agent (Q11, decided by the user, 26/10/2026):
//! only for the VM. Pi and Antigravity as their builds for the VM's Linux
//! ([`kept_on`]); Claude Code and Codex not at all, the `sbx` agent images
//! bring them ([`AgentRow::Provided`]). An agent's folder for any other
//! system (an install of an earlier Atollo for the PC) is [`obsolete`]:
//! never on a `PATH` (`env::strip_agents`), removed by the next «Applica»
//! of the Tools page, which lists it among what goes.
//!
//! Re-enabling: [`watch`] asks the isolation's status at its cadence (once
//! a minute, `isolation::status`'s `FRESH`) and tells when the agents'
//! availability flips ([`Told`]); the app sends it as `levels-changed`
//! (`ipc::levels::watch_levels`), and the interface reads again what it
//! shows. The editor's agent profiles follow at its next start
//! (`editor::cfg::write`, written at every start).

use std::sync::{Mutex, PoisonError};
use std::time::Duration;

use serde::Serialize;

use crate::common::i18n::t;
use crate::env::{Host, KitEnv};
use crate::isolation;
use crate::kit::{Kit, Target};
use crate::setup::{Catalog, Component, Group};

use super::{Agent, AgentRunner, VmState};

/// True exactly when the VM is on (LEVELS.md §7: `agentsEnabled`).
pub fn agents_enabled(vm: &VmState) -> bool {
    matches!(vm, VmState::On)
}

/// The VM as `status` saw it (the one mapping, also `facts`').
pub fn vm_state(status: &isolation::Status) -> VmState {
    match status.level {
        isolation::Level::Vm => VmState::On,
        isolation::Level::Env => VmState::Off {
            blockers: status.blockers.clone(),
        },
    }
}

/// The VM as the last status saw it, without asking `sbx` (the commands
/// that answer at once: the Tools page's list, the updates); off with no
/// reason before the first status, which [`watch`] asks at the app's start
/// and then tells.
pub fn known_vm() -> VmState {
    isolation::last_status().map_or(
        VmState::Off {
            blockers: Vec::new(),
        },
        |s| vm_state(&s),
    )
}

/// The agent `c` is, if it is one (the catalog's `agents` group).
pub fn agent_of(c: &Component) -> Option<Agent> {
    (c.group == Group::Agents)
        .then(|| Agent::from_id(&c.id))
        .flatten()
}

/// The kit keeps `agent` for `target` (Q11): Pi and Antigravity for the
/// VM's Linux only (every VM is Linux); Claude Code and Codex for no
/// system, the `sbx` agent images bring them.
pub fn kept_on(agent: Agent, target: Target) -> bool {
    agent.runner() == AgentRunner::ProjectVm && target.is_linux()
}

/// `c` may be installed or updated for `target` with the VM as `vm` says:
/// any component but an agent; an agent only where the kit keeps it
/// ([`kept_on`]) and while the agents are enabled. The one filter of the
/// runner's plans (first start, «Aggiorna tutto», an install, the tools of
/// a VM) and of the updates listed.
pub fn installable(c: &Component, target: Target, vm: &VmState) -> bool {
    agent_of(c).is_none_or(|agent| kept_on(agent, target) && agents_enabled(vm))
}

/// `c` is an agent the kit never keeps for `target`: what is on disk of it
/// there (an install of an earlier Atollo for the PC) goes at the next run
/// that removes. Never decided by the VM's state: Pi's Linux build stays
/// while the VM is off for a moment.
pub fn obsolete(c: &Component, target: Target) -> bool {
    agent_of(c).is_some_and(|agent| !kept_on(agent, target))
}

/// Why the agents are disabled, in `lang`: the VM's blockers, each with
/// what to do; `None` while the VM is on.
pub fn off_reason(vm: &VmState, lang: &str) -> Option<String> {
    let VmState::Off { blockers } = vm else {
        return None;
    };
    Some(if blockers.is_empty() {
        t(
            lang,
            "Gli agenti girano solo nella VM, che non è attiva.",
            &[],
        )
    } else {
        let why: Vec<String> = blockers.iter().map(|b| b.explain(lang)).collect();
        t(
            lang,
            "Gli agenti girano solo nella VM, che non è attiva: {0}",
            &[&why.join(" ")],
        )
    })
}

/// Why the agent `c` is not installed now (an install asked anyway), in
/// `lang`: brought by the VM (Claude Code, Codex), the VM off, or a
/// system that is not the VM's Linux.
pub fn refused_install(c: &Component, vm: &VmState, lang: &str) -> String {
    match agent_of(c).map(Agent::runner) {
        Some(AgentRunner::SbxAgent) => t(
            lang,
            "{0} arriva con la sandbox di Docker Sandboxes: Atollo non lo scarica.",
            &[&c.name],
        ),
        _ => off_reason(vm, lang).unwrap_or_else(|| {
            t(
                lang,
                "{0} si installa solo per il Linux della VM.",
                &[&c.name],
            )
        }),
    }
}

/// What an agent's row of the Tools page (and its tile in the quick panel)
/// says instead of an install (LEVELS.md §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AgentRow {
    /// Claude Code, Codex: «fornito dalla VM»; the kit downloads nothing.
    Provided,
    /// Pi, Antigravity without the VM: «serve la VM», nothing installed or
    /// updated; `reason`: [`off_reason`].
    NeedsVm { reason: String },
}

/// The row of `c` with the VM as `vm` says; `None` for every other
/// component, and for Pi and Antigravity while the VM is on (installed for
/// its Linux as any tool).
pub fn agent_row(c: &Component, vm: &VmState, lang: &str) -> Option<AgentRow> {
    let agent = agent_of(c)?;
    match agent.runner() {
        AgentRunner::SbxAgent => Some(AgentRow::Provided),
        AgentRunner::ProjectVm => off_reason(vm, lang).map(|reason| AgentRow::NeedsVm { reason }),
    }
}

/// The levels as the interface reads them (LEVELS.md §4.1): the payload
/// of `levels-changed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelsState {
    /// "on" or "off".
    pub vm: &'static str,
    /// Why the VM is off and what to do, `None` when on.
    pub reason: Option<String>,
    /// False exactly when `vm` is "off" (I6).
    pub agents_enabled: bool,
}

impl LevelsState {
    pub fn of(vm: &VmState, lang: &str) -> LevelsState {
        LevelsState {
            vm: if agents_enabled(vm) { "on" } else { "off" },
            reason: off_reason(vm, lang),
            agents_enabled: agents_enabled(vm),
        }
    }
}

/// The agents' availability last told, to tell only its flips.
#[derive(Debug, Default)]
pub struct Told(Mutex<Option<bool>>);

impl Told {
    /// `vm` seen now: true when the availability differs from the one told
    /// last, or none was told yet; it is then the one told.
    pub fn flipped(&self, vm: &VmState) -> bool {
        let now = agents_enabled(vm);
        let mut told = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        let flipped = *told != Some(now);
        *told = Some(now);
        flipped
    }
}

/// The app's: every status seen ([`watch`], a forced check of the notice)
/// goes through it.
pub static TOLD: Told = Told(Mutex::new(None));

/// How often [`watch`] looks: the status's own cadence (`sbx` is asked at
/// most once a minute, `isolation::status`).
pub const WATCH_EVERY: Duration = Duration::from_secs(60);

/// A thread that asks the isolation's status now and then every
/// [`WATCH_EVERY`] (`env`: the kit's environment then), and calls `tell`
/// with the VM's state each time the agents' availability flips (the first
/// time too).
pub fn watch(
    kit: Kit,
    host: Host,
    env: impl Fn() -> KitEnv + Send + 'static,
    tell: impl Fn(&VmState) + Send + 'static,
) {
    std::thread::spawn(move || loop {
        let vm = vm_state(&isolation::status(&kit, &host, &env(), false));
        if TOLD.flipped(&vm) {
            tell(&vm);
        }
        std::thread::sleep(WATCH_EVERY);
    });
}

/// The catalog's agents in the order of the catalog.
pub fn catalog_agents() -> impl Iterator<Item = (&'static Component, Agent)> {
    Catalog::get()
        .all()
        .iter()
        .filter_map(|c| agent_of(c).map(|a| (c, a)))
}

#[cfg(test)]
#[path = "../../../test/rust/levels/agents.rs"]
mod tests;
