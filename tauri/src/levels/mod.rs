//! Where every program of Atollo runs (D-179, D-183, `docs/app/LEVELS.md`):
//! two places. On the PC level 1 (Manual: the kit's and the editor's
//! terminals) and level 2 (Project code: actions, sites, tasks, scaffolds,
//! the project's terminal, sealed); in the VM level 3 (Agent) only,
//! refused without it.
//!
//! [`decide`] is the one decision: pure and total, the rules R1–R8 of
//! LEVELS.md §3.4 in order. The level comes from the origin
//! ([`Origin::level`]), never from the caller (I2); the place from a real
//! folder ([`place::place_of`]); the rest from [`facts::facts`], the only
//! read before the decision.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::isolation::vm;

pub use crate::setup::agents::Agent;

pub mod agents;
pub mod env;
pub mod facts;

pub use agents::{agents_enabled, installable};
pub mod labels;
pub mod place;
pub mod prepare;
pub mod report;
pub mod requests;
pub mod start;

/// The three levels (D-179). The number is the user-facing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Manual = 1,
    Project = 2,
    Agent = 3,
}

/// Who asks. The level follows from it (I2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Origin {
    // Level 1
    KitTerminal {
        terminal: String,
    },
    EditorTerminal,
    // Level 2
    ProjectTerminal,
    EditorTask {
        command: String,
    },
    /// A command line of the project; `web`: it listens on a port (`Kind::Web`).
    Action {
        action: String,
        web: bool,
    },
    /// An action served by a web server of the kit (static, Apache, Caddy).
    Site {
        action: String,
    },
    Pool {
        pool: String,
    },
    Scaffold {
        base: String,
    },
    // Level 3
    AgentButton {
        agent: Agent,
    },
    EditorAgentProfile {
        agent: Agent,
    },
    Forwarded {
        agent: Agent,
        from: Level,
    },
    EditorExtension {
        extension: String,
        agent: Agent,
    },
    /// The project's sandbox itself, for its tools: «Verifica strumenti»,
    /// «Riavvia», «Ricrea», «Aggiorna» (LEVELS.md §8.5). No session.
    Sandbox,
}

impl Origin {
    /// The level of the origin (LEVELS.md §3.4, column "Level").
    pub const fn level(&self) -> Level {
        match self {
            Origin::KitTerminal { .. } | Origin::EditorTerminal => Level::Manual,
            Origin::ProjectTerminal
            | Origin::EditorTask { .. }
            | Origin::Action { .. }
            | Origin::Site { .. }
            | Origin::Pool { .. }
            | Origin::Scaffold { .. } => Level::Project,
            Origin::AgentButton { .. }
            | Origin::EditorAgentProfile { .. }
            | Origin::Forwarded { .. }
            | Origin::EditorExtension { .. }
            | Origin::Sandbox => Level::Agent,
        }
    }

    /// The agent asked for: `Some` for the four level-3 origins that start
    /// one, `None` for the others (and [`Origin::Sandbox`]).
    pub fn agent(&self) -> Option<Agent> {
        match self {
            Origin::AgentButton { agent }
            | Origin::EditorAgentProfile { agent }
            | Origin::Forwarded { agent, .. }
            | Origin::EditorExtension { agent, .. } => Some(*agent),
            _ => None,
        }
    }
}

/// Where the request is, resolved from a folder by [`place::place_of`] (never from
/// a name the caller typed: a project is a real folder under `projects/`,
/// not a link, D-77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// `dir` is `projects/<name>`; `below` the part under it ("web/src"),
    /// `/`-separated, `None` in the project's folder itself.
    Project {
        name: String,
        dir: PathBuf,
        below: Option<String>,
    },
    /// Any other folder of the PC.
    Folder { dir: PathBuf },
}

impl Place {
    /// The folder the place was resolved for: the project's folder with
    /// `below`, or the folder itself.
    pub fn dir(&self) -> PathBuf {
        match self {
            Place::Project { dir, below, .. } => joined(dir, below.as_deref()),
            Place::Folder { dir } => dir.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub origin: Origin,
    pub place: Place,
}

/// The VM as the last status saw it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmState {
    On,
    Off {
        blockers: Vec<crate::isolation::Blocker>,
    },
}

/// Everything `decide` needs besides the request, read once by
/// [`facts::facts`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub vm: VmState,
    /// The place's folder as the VM sees it (`vm::guest_path`): for a
    /// project its folder (without `below`); `None` for a path the VM cannot
    /// mount (a network share).
    pub guest_dir: Option<String>,
    /// `isolation::restart_pending`: the kit daemon must restart before any
    /// VM opens.
    pub restart_pending: bool,
    /// For an agent: the catalog has a build for the VM's Linux target (`Component::is_available_on(guest)`), installed or
    /// installable. `false` for every other origin (not read then).
    pub linux_build: bool,
    /// The kit's tools for the VM's Linux (`kit.for_target(guest).apps()`),
    /// mounted read-only in every sandbox; empty where the PC has no VM.
    pub guest_tools: Vec<PathBuf>,
    /// For a `Place::Folder`: its canonical path when it may be a folder
    /// sandbox's workspace (`isolation::vm_workspace`, D-168, D-187), `None`
    /// when it may not (the kit, the profile, a system folder, a drive's
    /// root, `projects/`, a link) and for a project.
    pub workspace: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "where", rename_all = "camelCase")]
pub enum Placement {
    Pc {
        /// Manual or Project, never Agent (I3).
        level: Level,
        /// The place's project, if any.
        project: Option<String>,
        /// Where the process starts.
        dir: PathBuf,
    },
    Vm {
        /// Agent, always.
        level: Level,
        /// The project, `None` for a folder's sandbox (D-187).
        project: Option<String>,
        sandbox: Sandbox,
        /// The working folder in the VM (project folder + `below`, or the
        /// folder).
        workdir: String,
        /// Folders mounted: the project or the folder rw, the kit's Linux
        /// tools ro.
        mounts: Mounts,
        /// Which PC ports the sandbox may reach (numbers resolved by `prepare`).
        host: HostAccess,
        program: VmProgram,
    },
    Refused {
        level: Level,
        reason: Refusal,
        fix: Fix,
    },
}

impl Placement {
    /// The level of the request placed.
    pub fn level(&self) -> Level {
        match self {
            Placement::Pc { level, .. }
            | Placement::Vm { level, .. }
            | Placement::Refused { level, .. } => *level,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Sandbox {
    /// `atollo-p-<project>-<hash>` (`vm::name`): the project's one sandbox,
    /// every agent in it (D-184).
    Project { name: String },
    /// `atollo-f-<folder>-<hash>` (`vm::folder_name`): a folder that is not
    /// a project, its agents in it (D-187).
    Folder { name: String },
}

impl Sandbox {
    pub fn name(&self) -> &str {
        match self {
            Sandbox::Project { name } | Sandbox::Folder { name } => name,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mounts {
    /// The project folder.
    pub workspace_rw: PathBuf,
    /// `data/apps/linux-x64` or `linux-arm64`.
    pub tools_ro: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HostAccess {
    /// The project's databases on the kit's servers (D-171).
    pub databases: bool,
    /// llama-server, Pi only, when Pi uses a local model (D-178).
    pub local_models: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VmProgram {
    /// The agent's build for the VM's Linux from the kit, `sbx exec -it`
    /// (`vm::agent_session_args`).
    Agent { agent: Agent },
    /// The sandbox made ready for its tools, no session ([`Origin::Sandbox`]).
    Tools,
}

impl VmProgram {
    /// The agent it runs; `None` for [`VmProgram::Tools`].
    pub fn agent(&self) -> Option<Agent> {
        match self {
            VmProgram::Agent { agent } => Some(*agent),
            VmProgram::Tools => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Refusal {
    /// An agent with the VM off: the agents run only in the VM.
    AgentsNeedVm {
        blockers: Vec<crate::isolation::Blocker>,
    },
    /// An agent in a folder that may not be a sandbox's workspace: the
    /// kit, the profile, a system folder, a drive's root, `projects/`, a
    /// link (D-168, D-187).
    FolderNotAllowed { dir: String },
    /// The folder has no path in the VM (network share).
    NoPathInVm { dir: String },
    /// The agent has no build for the VM's Linux.
    NoLinuxBuild { agent: Agent },
    /// The kit's daemon must restart for a security setting
    /// (`isolation::restart_pending`).
    DaemonRestartPending,
}

impl Refusal {
    /// What the user does about it: one fix per reason (I7).
    pub fn fix(&self) -> Fix {
        match self {
            Refusal::AgentsNeedVm { .. } => Fix::OpenVmNotice,
            Refusal::FolderNotAllowed { .. } => Fix::OpenFromProject,
            Refusal::NoPathInVm { .. } => Fix::UseLocalDisk,
            Refusal::NoLinuxBuild { .. } => Fix::None,
            Refusal::DaemonRestartPending => Fix::RestartDaemon,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
    /// Open the "No VM" notice (its reasons and their fixes).
    OpenVmNotice,
    /// Open the agent from a project, or from a folder that may be a
    /// sandbox's workspace.
    OpenFromProject,
    /// Move the project to a local disk.
    UseLocalDisk,
    /// Close the VM sessions, then "Check again" in the VM window.
    RestartDaemon,
    /// Nothing the user can do in Atollo (no Linux build published).
    None,
}

/// The rows of LEVELS.md §3.4: which one placed a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Rule {
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
}

/// The one decision (D-179, D-183, D-184): pure, total, no I/O.
pub fn decide(req: &Request, facts: &Facts) -> Placement {
    decided(req, facts).1
}

/// [`decide`] with the row of §3.4 that gave the placement: everything but
/// an agent on the PC (R2, R3); an agent refused with the VM off (R1), in
/// the project's sandbox otherwise (R4–R8).
pub fn decided(req: &Request, facts: &Facts) -> (Rule, Placement) {
    let level = req.origin.level();
    let project = match &req.place {
        Place::Project { name, .. } => Some(name),
        Place::Folder { .. } => None,
    };
    if level != Level::Agent {
        return (
            if project.is_some() {
                Rule::R2
            } else {
                Rule::R3
            },
            Placement::Pc {
                level,
                project: project.cloned(),
                dir: req.place.dir(),
            },
        );
    };
    let refused = |reason: Refusal| Placement::Refused {
        level,
        fix: reason.fix(),
        reason,
    };
    if let VmState::Off { blockers } = &facts.vm {
        return (
            Rule::R1,
            refused(Refusal::AgentsNeedVm {
                blockers: blockers.clone(),
            }),
        );
    }
    // The workspace: the project's folder, or the folder itself when it may
    // be one (D-187).
    let (name, dir, below) = match &req.place {
        Place::Project { name, dir, below } => (Some(name), dir, below.as_deref()),
        Place::Folder { dir } => match &facts.workspace {
            Some(workspace) => (None, workspace, None),
            None => {
                return (
                    Rule::R4,
                    refused(Refusal::FolderNotAllowed {
                        dir: dir.display().to_string(),
                    }),
                )
            }
        },
    };
    if facts.restart_pending {
        return (Rule::R5, refused(Refusal::DaemonRestartPending));
    }
    let Some(guest_dir) = &facts.guest_dir else {
        return (
            Rule::R6,
            refused(Refusal::NoPathInVm {
                dir: req.place.dir().display().to_string(),
            }),
        );
    };
    let placed = |sandbox: Sandbox, program: VmProgram, local_models: bool| Placement::Vm {
        level,
        project: name.cloned(),
        sandbox,
        workdir: guest_joined(guest_dir, below),
        mounts: Mounts {
            workspace_rw: dir.clone(),
            tools_ro: facts.guest_tools.clone(),
        },
        host: HostAccess {
            // A folder has no project's databases (D-187).
            databases: name.is_some(),
            local_models,
        },
        program,
    };
    let sandbox = match name {
        Some(name) => Sandbox::Project {
            name: vm::name(name),
        },
        None => Sandbox::Folder {
            name: vm::folder_name(dir),
        },
    };
    let Some(agent) = req.origin.agent() else {
        return (Rule::R8, placed(sandbox, VmProgram::Tools, false));
    };
    if !facts.linux_build {
        return (Rule::R7, refused(Refusal::NoLinuxBuild { agent }));
    }
    (
        Rule::R8,
        placed(sandbox, VmProgram::Agent { agent }, agent == Agent::Pi),
    )
}

/// `dir` with `below` (`/`-separated), one component at a time.
fn joined(dir: &Path, below: Option<&str>) -> PathBuf {
    let mut out = dir.to_path_buf();
    for part in below
        .unwrap_or_default()
        .split('/')
        .filter(|p| !p.is_empty())
    {
        out.push(part);
    }
    out
}

/// The VM's folder `guest` with `below`.
fn guest_joined(guest: &str, below: Option<&str>) -> String {
    match below.filter(|b| !b.is_empty()) {
        Some(below) if guest.ends_with('/') => format!("{guest}{below}"),
        Some(below) => format!("{guest}/{below}"),
        None => guest.to_owned(),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/levels/decide.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../test/rust/levels/rule.rs"]
mod rule;
