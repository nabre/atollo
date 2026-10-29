//! The agents Atollo knows, as the catalog names them: a plain type the
//! tools (the kit's files, the installs) and the levels share (D-189).

use serde::{Deserialize, Serialize};

use super::catalog::{Component, Group};
use crate::kit::Target;

/// The agents Atollo knows (the catalog's `agents` group).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Codex,
    Pi,
    Antigravity,
}

impl Agent {
    pub const ALL: [Agent; 4] = [Agent::Claude, Agent::Codex, Agent::Pi, Agent::Antigravity];

    /// The catalog id: "claude", "codex", "pi", "antigravity".
    pub fn id(self) -> &'static str {
        match self {
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Pi => "pi",
            Agent::Antigravity => "antigravity",
        }
    }

    /// The name the user reads: "Claude Code", "Codex", "Pi", "Antigravity".
    pub fn name(self) -> &'static str {
        match self {
            Agent::Claude => "Claude Code",
            Agent::Codex => "Codex",
            Agent::Pi => "Pi",
            Agent::Antigravity => "Antigravity",
        }
    }

    /// The commands a user types for it: ["claude"], ["codex"], ["pi"],
    /// ["agy", "antigravity"].
    pub fn commands(self) -> &'static [&'static str] {
        match self {
            Agent::Claude => &["claude"],
            Agent::Codex => &["codex"],
            Agent::Pi => &["pi"],
            Agent::Antigravity => &["agy", "antigravity"],
        }
    }

    pub fn from_id(id: &str) -> Option<Agent> {
        Agent::ALL.into_iter().find(|a| a.id() == id)
    }

    #[cfg(test)]
    pub fn from_command(cmd: &str) -> Option<Agent> {
        Agent::ALL.into_iter().find(|a| a.commands().contains(&cmd))
    }
}

/// The agent `c` is, if it is one (the catalog's `agents` group).
pub fn agent_of(c: &Component) -> Option<Agent> {
    (c.group == Group::Agents)
        .then(|| Agent::from_id(&c.id))
        .flatten()
}

/// The kit keeps `agent` for `target` (Q11, D-184): every agent for the
/// VM's Linux only (every VM is Linux), none for the PC.
pub fn kept_on(_agent: Agent, target: Target) -> bool {
    target.is_linux()
}

/// `c` may be installed or updated for `target` with the agents `enabled`
/// (the VM on, `levels::agents_enabled`): any component but an agent; an
/// agent only where the kit keeps it ([`kept_on`]) and while enabled. The
/// one filter of the runner's plans and of the updates listed.
pub fn installable(c: &Component, target: Target, enabled: bool) -> bool {
    agent_of(c).is_none_or(|agent| kept_on(agent, target) && enabled)
}

/// `c` is an agent the kit never keeps for `target`: what is on disk of it
/// there (an install of an earlier Atollo for the PC) goes at the next run
/// that removes. Never decided by the VM's state.
pub fn obsolete(c: &Component, target: Target) -> bool {
    agent_of(c).is_some_and(|agent| !kept_on(agent, target))
}
