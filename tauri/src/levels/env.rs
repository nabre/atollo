//! The one `PATH`/environment builder (LEVELS.md §3.5): what a
//! [`Placement`] runs with. On the PC the kit's environment at the right
//! lines, sealed (D-167), without the agents (I4), the forwarders of their
//! commands first, with the level and the project named; in the VM only
//! the kit's Linux tools, as the VM sees them, and the environment of
//! `sbx` on the PC.
//!
//! Each step of the PC's environment is a function of its own, in the
//! order of §3.5: [`crate::editor::terminals::apply_tools`] (a user
//! terminal's lines and tools), [`project_lines`], [`seal`],
//! [`strip_agents`], [`forward_first`], [`mark`].

use std::path::PathBuf;

use crate::editor::terminals::Terminal;
use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm, GuestTools};
use crate::kit::Kit;
use crate::versions::Choice;

use super::{Level, Placement, VmProgram};

/// The variable every level-1/2 process on the PC gets: "manual" or
/// "project"; in the VM: "project" or "agent". To be read by profile.ps1
/// and bashrc for their banner (step 8 of LEVELS.md §12.2).
pub const LEVEL_VAR: &str = "ATOLLO_LEVEL";
/// The project of a PC process, when it has one.
pub const PROJECT_VAR: &str = "ATOLLO_PROJECT";
/// A kit terminal's name (`Origin::KitTerminal`): its banner, and the
/// kit's profile does not apply the project's lines again.
pub const TERMINAL_VAR: &str = "ATOLLO_TERMINAL";
/// `data/kit/forward`: the forwarders of the agents' commands, first on
/// every PC `PATH` (LEVELS.md §6.8, written by `setup::kit_files`).
pub const FORWARD_DIR: &str = "forward";

/// What a project's programs never get from the kit's environment: the
/// way to the user's real profile, the user's SSH keys (the agent's
/// socket), the desktop's session bus (the Secret Service), and the root
/// login of the kit's MySQL, replaced by the project's own user.
pub const SEALED: [&str; 4] = [
    "CP_USER_PROFILE",
    "SSH_AUTH_SOCK",
    "DBUS_SESSION_BUS_ADDRESS",
    "ATOLLO_MYSQL_CNF",
];

/// What of [`SEALED`] level 1 keeps (decided by the user, 26/10/2026): the
/// user's SSH agent, so Git over SSH in a terminal the user types in uses
/// the keys the user loaded. `SSH_AUTH_SOCK` is all an SSH client needs
/// (`SSH_AGENT_PID` only serves `ssh-agent -k`, and the kit's environment
/// never takes it from the PC); on Windows the OpenSSH agent is a named
/// pipe found without a variable.
pub const SSH_AGENT: [&str; 1] = ["SSH_AUTH_SOCK"];

/// What a placement runs with.
#[derive(Debug, Clone)]
pub enum Environment {
    /// The environment of the PC process.
    Pc(KitEnv),
    /// The environment of `sbx` on the PC ([`isolation::sbx_session_env`]:
    /// the kit's environment, the PC's temp, `daemon::ENV`) and what goes
    /// into the VM. `guest.env_file` is set by `prepare` (the databases'
    /// variables, D-171), never here.
    Vm {
        sbx_env: KitEnv,
        // What the model says goes into the VM; `prepare` gives the sandbox
        // its own (`Vm::tools`), so only the tests read it.
        #[cfg_attr(not(test), allow(dead_code))]
        guest: GuestTools,
    },
}

/// The environment of `placement`, from the kit's `base` (built for the
/// PC's target). `terminal`: the kit terminal's tool choice, for
/// `Origin::KitTerminal` only. `host`: the PC's variables (the kit's
/// environment for the VM's Linux is built from them, and `sbx` gets the
/// PC's temporary folder). `None` exactly for `Placement::Refused`.
pub fn environment(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&Terminal>,
    lang: &str,
    host: &Host,
) -> Option<Environment> {
    match placement {
        Placement::Pc { level, project, .. } => Some(Environment::Pc(pc_environment(
            kit,
            base.clone(),
            *level,
            project.as_deref(),
            terminal,
        ))),
        Placement::Vm {
            level,
            project,
            program,
            ..
        } => Some(Environment::Vm {
            sbx_env: isolation::sbx_session_env(base, host),
            guest: guest_tools(kit, *level, project.as_deref(), program, lang, host),
        }),
        Placement::Refused { .. } => None,
    }
}

/// What goes into the VM of a `Vm` placement of `level` in `project`
/// running `program` (§3.5, `Vm` steps 1–3): the kit's tools for the VM's
/// Linux at the project's lines, sealed and without the agents, as the VM
/// sees them; for an agent's session the agent's own folder back, last; the
/// level. Read from the disk: `levels::prepare` asks again once it has
/// installed what was missing (D-170).
pub fn guest_tools(
    kit: &Kit,
    level: Level,
    project: Option<&str>,
    program: &VmProgram,
    lang: &str,
    host: &Host,
) -> GuestTools {
    let settings = crate::common::settings::Settings::load(kit).settings;
    let mut guest = vm::guest_target(kit.target())
        .map(|target| {
            let guest = kit.for_target(target);
            let base_guest = KitEnv::build(&guest, &settings, lang, host);
            let mut tools = isolation::project_guest_tools(kit, &guest, project, base_guest);
            if let VmProgram::Agent { agent } = program {
                agent_back(kit, &guest, *agent, &mut tools);
            }
            tools
        })
        .unwrap_or_default();
    guest
        .vars
        .push((LEVEL_VAR.to_owned(), level_value(level).to_owned()));
    guest
}

/// The environment of a process on the PC at `level` (Manual or Project),
/// in `project` when `Some`, for the kit terminal `terminal` when `Some`:
/// the steps of LEVELS.md §3.5 in order. Applied again to its own result
/// it changes nothing.
pub fn pc_environment(
    kit: &Kit,
    base: KitEnv,
    level: Level,
    project: Option<&str>,
    terminal: Option<&Terminal>,
) -> KitEnv {
    let mut env = base;
    // 2. A user terminal's lines and tools (none for the default one).
    let lines = match terminal {
        Some(terminal) => crate::editor::terminals::apply_tools(kit, terminal, &mut env),
        None => crate::versions::kit_choice(kit),
    };
    // 3. The project's lines over them (Q1: the project's for the tools it
    //    chose, the terminal's for the others).
    if let Some(project) = project {
        project_lines(kit, project, &lines, &mut env);
    }
    seal(kit, level, project, &mut env);
    strip_agents(kit, &mut env);
    forward_first(kit, &mut env);
    mark(&mut env, level, project, terminal);
    env
}

/// Step 3: the lines `project` chose (`versions::project_choice`), from
/// `lines`, the ones `env` is on: a tool the project chose is at its line
/// in any terminal opened in its folder; the others keep `env`'s.
pub fn project_lines(kit: &Kit, project: &str, lines: &Choice, env: &mut KitEnv) {
    crate::versions::swap_lines(
        kit,
        env,
        lines,
        &crate::versions::project_choice(kit, project),
    );
}

/// Step 4: seals `env` for a process of `level` in `project`, or of no
/// project when `None`: the [`SEALED`] variables out (at level 1 but the
/// user's [`SSH_AGENT`]), then the project's own MySQL login in
/// (`db::services::project_client_env`, D-167).
pub fn seal(kit: &Kit, level: Level, project: Option<&str>, env: &mut KitEnv) {
    for name in SEALED {
        if !(level == Level::Manual && SSH_AGENT.contains(&name)) {
            env.remove(name);
        }
    }
    if let Some(project) = project {
        for (name, value) in crate::db::services::project_client_env(kit, project) {
            env.set(name, value.to_string_lossy());
        }
    }
}

/// The folders of the agents' programs and launchers in `kit` (of the
/// PC's target, or of the VM's Linux for `kit.for_target(guest)`): their
/// components' folders in `data/apps/<target>` (Claude Code, Codex with
/// its `codex-path`, Antigravity, Pi) and the launchers' `data/kit/agents`.
pub fn agent_dirs(kit: &Kit) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = super::Agent::ALL
        .iter()
        .map(|agent| kit.apps().join(agent.id()))
        .collect();
    dirs.push(kit.kit_files().join(crate::setup::kit_files::AGENTS_DIR));
    dirs
}

/// Step 5: `env` with no agent's command (D-178, I4): the folders of
/// [`agent_dirs`] leave the `PATH`, with the variables that point into them
/// (`CP_ANTIGRAVITY`); so does [`forward_dir`], put back first by
/// [`forward_first`]. Applied again it changes nothing.
pub fn strip_agents(kit: &Kit, env: &mut KitEnv) {
    let mut dirs = agent_dirs(kit);
    dirs.push(forward_dir(kit));
    crate::editor::terminals::strip(env, &dirs);
}

/// The folder first on every PC `PATH`, so that `claude`, `codex`, `agy`,
/// `antigravity` and `pi` never start one the PC installed elsewhere
/// (Homebrew, npm): the forwarders of `data/kit/forward` ([`FORWARD_DIR`]),
/// which ask the app to run the agent in the project's VM, or say why not
/// (LEVELS.md §6.8).
pub fn forward_dir(kit: &Kit) -> PathBuf {
    kit.kit_files().join(FORWARD_DIR)
}

/// Step 6: [`forward_dir`] first on `env`'s `PATH`, once.
pub fn forward_first(kit: &Kit, env: &mut KitEnv) {
    let first = forward_dir(kit);
    crate::editor::terminals::strip(env, std::slice::from_ref(&first));
    let first = first.display().to_string();
    let rest = env.get("PATH").unwrap_or_default().to_owned();
    let path = if rest.is_empty() {
        first
    } else {
        format!("{first}{}{rest}", env.path_sep())
    };
    env.set("PATH", path);
}

/// Step 7: [`LEVEL_VAR`], [`PROJECT_VAR`] (removed without a project) and
/// [`TERMINAL_VAR`] (a kit terminal's name; removed for any other
/// process).
pub fn mark(env: &mut KitEnv, level: Level, project: Option<&str>, terminal: Option<&Terminal>) {
    env.set(LEVEL_VAR, level_value(level));
    match project {
        Some(project) => env.set(PROJECT_VAR, project),
        None => env.remove(PROJECT_VAR),
    }
    match terminal {
        Some(terminal) => env.set(TERMINAL_VAR, terminal.name.clone()),
        None => env.remove(TERMINAL_VAR),
    }
}

/// The value of [`LEVEL_VAR`] for `level`.
pub fn level_value(level: Level) -> &'static str {
    match level {
        Level::Manual => "manual",
        Level::Project => "project",
        Level::Agent => "agent",
    }
}

/// For `VmProgram::Agent { agent }`: the agent's own Linux folder
/// back on `tools`' `PATH`, last: it is the program being run, not a
/// command offered to others.
fn agent_back(kit: &Kit, guest: &Kit, agent: super::Agent, tools: &mut GuestTools) {
    let dir = guest.apps().join(agent.id());
    if let Some(dir) = vm::guest_path(kit.target().is_windows(), &dir) {
        if !tools.path.contains(&dir) {
            tools.path.push(dir);
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/levels/env.rs"]
mod tests;
