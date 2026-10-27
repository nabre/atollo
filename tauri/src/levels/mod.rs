//! The three levels (D-179, `docs/app/LEVELS.md`): where every program of
//! Atollo runs. Level 1 (Manual: the kit's and the editor's terminals)
//! always on the PC; level 2 (Project code: actions, sites, tasks,
//! scaffolds, the project's terminal) in the project's VM when it is on,
//! sealed on the PC without it; level 3 (Agent) only in a VM, refused
//! without it.
//!
//! [`decide`] is the one decision: pure and total, the rules R1–R17 of
//! LEVELS.md §3.4 in order. The level comes from the origin
//! ([`Origin::level`]), never from the caller (I2); the place from a real
//! folder ([`place::place_of`]); the rest from [`facts::facts`], the only
//! read before the decision.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::isolation::vm;

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

    /// Where it runs in the VM: Claude Code and Codex in a sandbox of
    /// `sbx`'s own, Pi and Antigravity (their Linux builds from the kit) in
    /// the project's.
    pub fn runner(self) -> AgentRunner {
        match self {
            Agent::Claude | Agent::Codex => AgentRunner::SbxAgent,
            Agent::Pi | Agent::Antigravity => AgentRunner::ProjectVm,
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

/// Where an agent runs when the VM is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunner {
    /// An agent of `sbx` in a sandbox of its own (`atollo-a-`): Claude, Codex.
    SbxAgent,
    /// Its Linux build from the kit, in the project's sandbox (`atollo-p-`):
    /// Pi, Antigravity.
    ProjectVm,
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
            | Origin::EditorExtension { .. } => Level::Agent,
        }
    }

    /// The agent asked for: `Some` for the four level-3 origins only.
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
    /// For `AgentRunner::ProjectVm` agents: the catalog has a build for the
    /// VM's Linux target (`Component::is_available_on(guest)`), installed or
    /// installable. `false` for every other origin (not read then).
    pub linux_build: bool,
    /// The kit's tools for the VM's Linux (`kit.for_target(guest).apps()`),
    /// mounted read-only in every sandbox; empty where the PC has no VM.
    pub guest_tools: Vec<PathBuf>,
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
        /// Project or Agent.
        level: Level,
        project: String,
        sandbox: Sandbox,
        /// The working folder in the VM (project folder + `below`).
        workdir: String,
        /// Folders mounted: the project rw, the kit's Linux tools ro.
        mounts: Mounts,
        /// Which PC ports the sandbox may reach (numbers resolved by `prepare`).
        host: HostAccess,
        /// Whether a port of the VM is published on the PC (web actions).
        publish: bool,
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
    /// `atollo-p-<project>-<hash>` (`vm::name`).
    Project { name: String },
    /// `atollo-a-<agent>-<project>-<hash>` (`vm::agent_name`).
    Agent { name: String, agent: Agent },
}

impl Sandbox {
    pub fn name(&self) -> &str {
        match self {
            Sandbox::Project { name } | Sandbox::Agent { name, .. } => name,
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
    /// Interactive `bash -l`.
    Shell,
    /// An editor task: `sh -lc`.
    Line { command: String },
    /// The steps of a base (framework scaffolding, "Le mie basi"), each a
    /// `sh -lc` line (`start::scaffold_command`), in the new project's sandbox.
    Scaffold { base: String },
    /// `sh -lc`, tracked (`vm::tracked`).
    Action { action: String },
    /// A container of the VM's Docker (`vm::web_line`).
    Site { action: String },
    /// `sbx run --name <atollo-a-…>`.
    SbxAgent { agent: Agent },
    /// pi / agy, Linux build, `sbx exec -it`.
    AgentBinary { agent: Agent },
}

impl VmProgram {
    /// The agent it runs (`SbxAgent`, `AgentBinary`); `None` otherwise.
    pub fn agent(&self) -> Option<Agent> {
        match self {
            VmProgram::SbxAgent { agent } | VmProgram::AgentBinary { agent } => Some(*agent),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Refusal {
    /// Level 3 with the VM off: the agents run only in the VM.
    AgentsNeedVm {
        blockers: Vec<crate::isolation::Blocker>,
    },
    /// Level 3 outside a project folder.
    AgentOutsideProject,
    /// Level 2 outside a project folder with the VM on (an editor task out
    /// of `projects/`).
    NotAProject,
    /// The folder has no path in the VM (network share).
    NoPathInVm { dir: String },
    /// A Web Server pool with the VM on.
    PoolsOnlyWithoutVm,
    /// Pi or Antigravity has no build for the VM's Linux.
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
            Refusal::AgentOutsideProject | Refusal::NotAProject => Fix::OpenFromProject,
            Refusal::NoPathInVm { .. } => Fix::UseLocalDisk,
            Refusal::PoolsOnlyWithoutVm => Fix::StartFromProject,
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
    /// Open the agent from a project folder or the project's page.
    OpenFromProject,
    /// Move the project to a local disk.
    UseLocalDisk,
    /// Start the site from the project's page (it runs in the project's VM).
    StartFromProject,
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
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    R16,
    R17,
}

/// The one decision (D-179): pure, total, no I/O.
pub fn decide(req: &Request, facts: &Facts) -> Placement {
    decided(req, facts).1
}

/// What an origin asks for, as the rules tell origins apart: written once
/// with an exhaustive match, so that every origin has exactly one kind.
enum Kind<'a> {
    /// Level 1.
    Manual,
    /// Level 2, a Web Server pool.
    Pool,
    /// Level 2 or 3: what runs in a VM when it is on.
    Vm(InVm<'a>),
}

enum InVm<'a> {
    Shell,
    Line(&'a str),
    Scaffold(&'a str),
    Action(&'a str, bool),
    Site(&'a str),
    Agent(Agent),
}

impl<'a> Kind<'a> {
    fn of(origin: &'a Origin) -> Kind<'a> {
        match origin {
            Origin::KitTerminal { .. } | Origin::EditorTerminal => Kind::Manual,
            Origin::Pool { .. } => Kind::Pool,
            Origin::ProjectTerminal => Kind::Vm(InVm::Shell),
            Origin::EditorTask { command } => Kind::Vm(InVm::Line(command)),
            Origin::Scaffold { base } => Kind::Vm(InVm::Scaffold(base)),
            Origin::Action { action, web } => Kind::Vm(InVm::Action(action, *web)),
            Origin::Site { action } => Kind::Vm(InVm::Site(action)),
            Origin::AgentButton { agent }
            | Origin::EditorAgentProfile { agent }
            | Origin::Forwarded { agent, .. }
            | Origin::EditorExtension { agent, .. } => Kind::Vm(InVm::Agent(*agent)),
        }
    }
}

/// [`decide`] with the row of §3.4 that gave the placement. The rows are
/// read top to bottom; grouped here by what they look at, which gives the
/// same first row (the test `every_request_has_exactly_one_placement`
/// checks it against the table written row by row).
pub fn decided(req: &Request, facts: &Facts) -> (Rule, Placement) {
    let level = req.origin.level();
    let refused = |reason: Refusal| Placement::Refused {
        level,
        fix: reason.fix(),
        reason,
    };
    let on_the_pc = |project: Option<&String>| Placement::Pc {
        level,
        project: project.cloned(),
        dir: req.place.dir(),
    };
    let project = match &req.place {
        Place::Project { name, .. } => Some(name),
        Place::Folder { .. } => None,
    };
    let in_folder = project.is_none();
    match (Kind::of(&req.origin), &facts.vm) {
        (Kind::Vm(InVm::Agent(_)), VmState::Off { blockers }) => (
            Rule::R1,
            refused(Refusal::AgentsNeedVm {
                blockers: blockers.clone(),
            }),
        ),
        (Kind::Pool, VmState::On) => (Rule::R2, refused(Refusal::PoolsOnlyWithoutVm)),
        (Kind::Manual, _) => (
            if in_folder { Rule::R4 } else { Rule::R3 },
            on_the_pc(project),
        ),
        (Kind::Pool | Kind::Vm(_), VmState::Off { .. }) => (
            if in_folder { Rule::R6 } else { Rule::R5 },
            on_the_pc(project),
        ),
        (Kind::Vm(what), VmState::On) => in_vm(req, facts, what, refused),
    }
}

/// Rows R7–R17: level 2 (not a pool) or 3 with the VM on.
fn in_vm(
    req: &Request,
    facts: &Facts,
    what: InVm<'_>,
    refused: impl Fn(Refusal) -> Placement,
) -> (Rule, Placement) {
    let level = req.origin.level();
    let Place::Project { name, dir, below } = &req.place else {
        return match what {
            InVm::Agent(_) => (Rule::R8, refused(Refusal::AgentOutsideProject)),
            _ => (Rule::R7, refused(Refusal::NotAProject)),
        };
    };
    if facts.restart_pending {
        return (Rule::R9, refused(Refusal::DaemonRestartPending));
    }
    let Some(guest_dir) = &facts.guest_dir else {
        return (
            Rule::R10,
            refused(Refusal::NoPathInVm {
                dir: req.place.dir().display().to_string(),
            }),
        );
    };
    let project_sandbox = || Sandbox::Project {
        name: vm::name(name),
    };
    let databases_only = HostAccess {
        databases: true,
        local_models: false,
    };
    let placed =
        |sandbox: Sandbox, program: VmProgram, host: HostAccess, publish: bool| Placement::Vm {
            level,
            project: name.clone(),
            sandbox,
            workdir: guest_joined(guest_dir, below.as_deref()),
            mounts: Mounts {
                workspace_rw: dir.clone(),
                tools_ro: facts.guest_tools.clone(),
            },
            host,
            publish,
            program,
        };
    match what {
        InVm::Agent(agent) => match agent.runner() {
            AgentRunner::ProjectVm if !facts.linux_build => {
                (Rule::R11, refused(Refusal::NoLinuxBuild { agent }))
            }
            AgentRunner::SbxAgent => (
                Rule::R12,
                placed(
                    Sandbox::Agent {
                        name: vm::agent_name(name, agent.id()),
                        agent,
                    },
                    VmProgram::SbxAgent { agent },
                    databases_only,
                    false,
                ),
            ),
            AgentRunner::ProjectVm => (
                Rule::R13,
                placed(
                    project_sandbox(),
                    VmProgram::AgentBinary { agent },
                    HostAccess {
                        databases: true,
                        local_models: agent == Agent::Pi,
                    },
                    false,
                ),
            ),
        },
        InVm::Shell => (
            Rule::R14,
            placed(project_sandbox(), VmProgram::Shell, databases_only, false),
        ),
        InVm::Line(command) => (
            Rule::R15,
            placed(
                project_sandbox(),
                VmProgram::Line {
                    command: command.to_owned(),
                },
                databases_only,
                false,
            ),
        ),
        InVm::Scaffold(base) => (
            Rule::R15,
            placed(
                project_sandbox(),
                VmProgram::Scaffold {
                    base: base.to_owned(),
                },
                databases_only,
                false,
            ),
        ),
        InVm::Action(action, web) => (
            Rule::R16,
            placed(
                project_sandbox(),
                VmProgram::Action {
                    action: action.to_owned(),
                },
                databases_only,
                web,
            ),
        ),
        InVm::Site(action) => (
            Rule::R17,
            placed(
                project_sandbox(),
                VmProgram::Site {
                    action: action.to_owned(),
                },
                databases_only,
                true,
            ),
        ),
    }
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
