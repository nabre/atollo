# The three levels: where every program of Atollo runs

English · [Italiano](LEVELS.it.md)

Normative document of decision D-179 (DECISIONS.md, 26/10/2026). It is the single source of truth for: which program runs where, with which `PATH`, with which agents, with which network and mounts; the code that decides it; the IPC that carries it; and how the user can check it. Code, tests, the Guide and ARCHITECTURE.md follow this file; where they differ, this file wins and the difference is a bug.

Status: **design, phase 1**; step 1 of 12.2 (types, `place_of`, `facts`, `decide`, labels) implemented, not yet called by the app; step 2 (`levels::env`, the one environment builder) implemented and used by today's callers; step 3 (the PC's entry points E1, E3, E5–E9, E20 through `decide` and `environment`, `PtyInfo.level/label/sandbox`, `CmdError.refusal/fix`, the pools sealed) implemented; step 4 (`levels::prepare` with `Effects`, `levels::start`, the VM's entry points E3, E4, E9, E10 through `decide` → `prepare` → `start`, the editor's E11/E12 sessions prepared the same way until step 5, the rule test `no_process_outside_levels`) implemented; step 5 (`levels::requests`, the clients `launch.ps1`/`launch.sh`, the editor's profiles: terminals level 1 on the PC, tasks and agents through the clients, only Atollo's profiles; the editor server at level 1; the forwarders of `data/kit/forward`) implemented; step 6 (`levels::agents`: `agents_enabled`, `installable`, the runner's filter, `AgentInfo.available`, `levels_state`, `levels-changed` and `levels::agents::watch`, the editor's agent profiles gated, the interface; agents installed only for the VM, Q11; the PC's agent code removed) implemented; step 7 (Antigravity in the project's VM) implemented; step 9 (the editor's extensions that run an agent: `level: agent`, never installed, uninstalled at every editor start, the editor's message; the Q2 wrapper ready, not wired) implemented; step 8 (`levels::report`: the VM report, «Verifica strumenti» and the actions on a sandbox; `project_vm_report`, `project_vm_check`, `project_vm_action`; `atollo vm`; the panel «VM del progetto»; the read-only `atollo` and the agents' stand-ins in every sandbox; the shells' level line) implemented; step 10 (the documentation: ARCHITECTURE, TESTING, PRIVACY, the Guide, DECISIONS D-179 «Prove», CHANGELOG) done, without the rename of `isolation::Level`, as 12.2 says. Nothing else here is implemented yet unless the text says "today". Every statement that was not verified on a real system is marked **[unverified]**; every choice made by this document that the user has not confirmed is marked **[assumption]**.

> **D-183 (28/10/2026) supersedes this document where they differ: two places.** Everything but an agent runs on the PC, with or without the VM: the kit's and the editor's terminals (level 1), the project's code (level 2: its terminal, actions, sites, editor tasks, project creation, sealed as D-167 says) and the Web Server pools. The VM runs the agents (level 3) only. `decide` has nine rules (§3.4). The VM rows of level 2 below (R14–R17 of the old table, §6.3–6.5 in the VM, sites in containers, published ports, scaffolding in the VM) are removed from the code; the rest of this document still holds for the agents.

Contents:

1. [Purpose and threat model](#1-purpose-and-threat-model)
2. [The constraint table](#2-the-constraint-table)
3. [The code model: `tauri/src/levels/`](#3-the-code-model-taurisrclevels)
4. [IPC contract](#4-ipc-contract)
5. [Entry-point inventory](#5-entry-point-inventory)
6. [Flows](#6-flows)
7. [Agents disabled without the VM](#7-agents-disabled-without-the-vm)
8. [Trust features](#8-trust-features)
9. [Antigravity](#9-antigravity)
10. [Editor extensions that run an agent](#10-editor-extensions-that-run-an-agent)
11. [Secrets: none from the kit enters a VM](#11-secrets-none-from-the-kit-enters-a-vm)
12. [Migration](#12-migration)
13. [Today's code that contradicts this document](#13-todays-code-that-contradicts-this-document)
14. [Open questions and unverified points](#14-open-questions-and-unverified-points)

---

## 1. Purpose and threat model

Atollo starts three kinds of programs, and they deserve three different levels of trust:

- **Level 1, Manual.** Commands the user types: the kit's terminals and the editor's integrated terminal. The user is the author of every command; the risk is the user's own, as in any terminal of the PC. These run on the PC, always, with or without the VM.
- **Level 2, Project code.** Code that comes with a project and runs without the user reading it line by line: `npm run dev`, a `postinstall` script, a framework's scaffolding (`npm create`, `composer create-project`), a site's PHP, an editor task. Third-party code: it may be hostile (a cloned repository, a compromised package). With the VM it runs in the project's microVM; without the VM it runs on the PC in the sealed environment (D-167), and the status bar says "No VM".
- **Level 3, Agent.** An autonomous program that decides by itself which commands to run and which files to write: Claude Code, Codex, Pi, Antigravity (`agy`), and editor extensions that run an agent. It is the most dangerous level ("pericoloso è l'agente", the user, 26/10/2026): it acts on its own, reads what it finds and can be steered by text inside the project (prompt injection). **It runs only in a VM.** Without the VM no agent runs anywhere, and everything that would start one is disabled with the reason.

What the levels protect, and what they do not:

| Threat                                          | Level 1 (PC)                                                                                                                                                                                                            | Level 2 with VM                                                                               | Level 2 without VM                                                                                                      | Level 3 (VM only)       |
| ----------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- | ----------------------- |
| Reading the user's files outside the project    | possible (it is the user)                                                                                                                                                                                               | no: only the project folder (rw) and the kit's Linux tools (ro) are mounted                   | possible: same user; the sealed environment only removes the easy ways (SSH agent, root MySQL login, real profile path) | no, as level 2 with VM  |
| Using the user's SSH keys, keyring, kit secrets | SSH keys: yes, the user's SSH agent is kept (`SSH_AUTH_SOCK`, Git over SSH; **decided by the user, 26/10/2026**); the rest not given (`seal`: no real profile path, session bus, MySQL root), but reachable as the user | no                                                                                            | reduced (`seal`, the SSH agent too), not prevented                                                                      | no                      |
| Reaching the PC's network services              | yes                                                                                                                                                                                                                     | only the ports Atollo opens for that sandbox (its databases, D-171; Pi's local models, D-178) | yes                                                                                                                     | same as level 2 with VM |
| Changing the kit's programs                     | possible                                                                                                                                                                                                                | no (tools mounted read-only; the kit and the profile are never a workspace, D-168)            | possible                                                                                                                | no                      |
| Acting on GitHub as the user                    | possible                                                                                                                                                                                                                | only if the project turned on "GitHub in the VM" (D-172), through the `sbx` proxy             | possible                                                                                                                | as level 2 with VM      |

The boundary is the VM (D-167). Without it Atollo does not pretend a boundary exists: level 2 on the PC is "sealed", not "isolated", and the interface says so.

A fourth category is **not** a level: the **kit's own programs** (database servers, phpMyAdmin, mongo-express, llama.cpp, Git for cloning, the editor server process itself, `sbx`, the app). They never run project code by design and stay on the PC (D-175, "Kit on the PC"). Section 5 lists them so the inventory is complete.

## 2. The constraint table

The same table holds for the app and for the editor. "VM on" means `isolation::status(...).level == Vm` at the moment of the request (see 3.3).

| Who (origin)                                                                             | Level | Where, VM on                                                           | Where, VM off                | `PATH`                                                                                             | Agent commands (`claude`, `codex`, `pi`, `agy`, `antigravity`)                                                                                         | Network and ports                                                                 | Mounts                                            |
| ---------------------------------------------------------------------------------------- | ----- | ---------------------------------------------------------------------- | ---------------------------- | -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------- | ------------------------------------------------- |
| Kit terminal ("All tools", user terminals), any folder                                   | 1     | PC                                                                     | PC                           | kit tools; in a project folder at the project's lines (3.5)                                        | **forwarders** first on `PATH`: in a project folder they forward to the agent in the project's VM; elsewhere, or VM off, they say why and exit 1       | the PC's                                                                          | none (PC)                                         |
| Editor integrated terminal (every Atollo profile except agents)                          | 1     | PC                                                                     | PC                           | as the kit terminal; the project of the folder the terminal opens in                               | forwarders, as above                                                                                                                                   | the PC's                                                                          | none (PC)                                         |
| Project terminal (project page)                                                          | 2     | project VM `atollo-p-…`                                                | PC, sealed                   | VM: kit Linux tools at the project's lines, then the image's; PC: kit tools at the project's lines | VM: none (the agents' Linux folders are stripped, `sbx`'s agents are not in the `shell` image **[unverified]**); PC: forwarders, which refuse (VM off) | VM: `sbx` policy `balanced`, its databases' ports, GitHub if chosen; PC: the PC's | VM: project rw, kit Linux tools ro                |
| Action (`npm run dev`…)                                                                  | 2     | project VM                                                             | PC, sealed                   | as project terminal                                                                                | none / forwarders                                                                                                                                      | VM: its web port published on `127.0.0.1` (or `0.0.0.0` with "Local network")     | as project terminal                               |
| Site of an action (static, Apache, Caddy)                                                | 2     | a container of the VM's Docker                                         | PC: its own server or a pool | —                                                                                                  | none                                                                                                                                                   | as action                                                                         | as action, plus the project's own aliases         |
| Web Server pool                                                                          | 2     | **refused** (each site runs in its project's VM)                       | PC                           | kit tools, no agents                                                                               | forwarders (refuse)                                                                                                                                    | the PC's                                                                          | none                                              |
| Editor task (`tasks.json`)                                                               | 2     | project VM                                                             | PC, sealed                   | as project terminal                                                                                | as project terminal                                                                                                                                    | as project terminal                                                               | as project terminal                               |
| Project creation (framework scaffolding, "Le mie basi")                                  | 2     | project VM (the new project's sandbox) **[new]**                       | PC, sealed                   | as project terminal                                                                                | none / forwarders                                                                                                                                      | VM: `balanced` (package registries)                                               | VM: the new project folder rw, kit Linux tools ro |
| Claude Code, Codex (project page, palette, editor profile, forwarder, extension wrapper) | 3     | own `sbx` sandbox `atollo-a-<agent>-<project>-<hash>`                  | **refused**                  | kit Linux tools at the project's lines, then the image's                                           | the agent itself only                                                                                                                                  | as project VM (same database ports, same GitHub rule)                             | project rw, kit Linux tools ro                    |
| Pi, Antigravity (`agy`)                                                                  | 3     | project VM `atollo-p-…`, their Linux build from the kit                | **refused**                  | as project terminal                                                                                | the agent itself only                                                                                                                                  | as project VM; Pi: plus the local models' port when Pi uses them                  | as project terminal                               |
| Agentic editor extensions (section 10)                                                   | 3     | uninstalled at every editor start (Q2: the forwarder once proven)      | uninstalled                  | —                                                                                                  | —                                                                                                                                                      | —                                                                                 | —                                                 |

Invariants that follow from the table (normative; each has a test, section 12.3):

- **I1** No code path starts a process for levels 1–3 without going through `levels::decide()`. The kit's own programs (section 5, table "Kit") are the only exception and are listed by name.
- **I2** The level is a function of the origin (`Origin::level()`), never a parameter a caller chooses.
- **I3** An `Agent` placement is never `Pc`.
- **I4** A `Pc` placement never has a folder of an agent program on `PATH` (`levels::env::agent_dirs` of the PC's target), and has the forwarders' folder first.
- **I5** A `Vm` placement never carries a kit secret: not in arguments, not in `-e`, not in `--env-file` (section 11 defines "kit secret").
- **I6** Without the VM, nothing of level 3 can be installed, updated, launched or configured in the editor; the kit's agent data stays on disk untouched.
- **I7** A refusal always carries a reason and a fix (`Refusal`, `Fix`); nothing silently falls back from the VM to the PC.
- **I8** `decide()` is pure and total: same inputs, same output; every input combination has a row of 3.4, and the first row that matches gives its one placement.

## 3. The code model: `tauri/src/levels/`

### 3.1 Files

| File                 | Content                                                                                                                                                                                                                                                                       | Comes from                                                                                                                                                                                        |
| -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `levels/mod.rs`      | `Level`, `Agent`, `AgentRunner`, `Origin`, `Place`, `Request`, `VmState`, `Facts`, `Placement`, `Sandbox`, `Mounts`, `VmProgram`, `HostAccess`, `Refusal`, `Fix`, `Rule`; `decide()`, `decided()`                                                                             | new                                                                                                                                                                                               |
| `levels/agents.rs`   | `agents_enabled`, `installable`, `kept_on`, `obsolete`, `agent_of`, `vm_state`, `known_vm`, `off_reason`, `refused_install`, `AgentRow`, `agent_row`, `LevelsState`, `Told`, `TOLD`, `watch` (section 7; step 6)                                                              | `ipc::agents::no_vm_for_agents`' reason, the agents' gates scattered in the UI                                                                                                                    |
| `levels/place.rs`    | `place_of(kit, dir) -> Place` (which project a folder belongs to)                                                                                                                                                                                                             | the logic of `runtime/vm.ps1` (`$projects`, `$below`) and `ipc::project_dir`                                                                                                                      |
| `levels/facts.rs`    | `facts(kit, host, env, request) -> Facts` (the only impure read before `decide`)                                                                                                                                                                                              | `isolation::status`, `vm::guest_path`, `isolation::restart_pending`, catalog availability                                                                                                         |
| `levels/env.rs`      | `Environment`, `environment()`, `pc_environment()` (its `Pc` part, each step a function: `project_lines`, `seal`, `strip_agents`, `forward_first`, `mark`), `SEALED`, `SSH_AGENT`, `agent_dirs()`, `forward_dir()`, `FORWARD_DIR`, `LEVEL_VAR`, `PROJECT_VAR`, `TERMINAL_VAR` | `isolation::{SEALED, seal, agent_dirs, without_agents, agents_notice, with_agents_notice}`, `versions::{project_env, choice_env}` (their sealing part), `editor::terminals::env` (its agent part) |
| `levels/prepare.rs`  | `Effects`, `Ready`, `prepare()`: tools for the VM's Linux, databases and their ports (`db.env`), Pi's local models (`pi.env`), the sandbox built and started, `info.json` (§8.4)                                                                                              | `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs}`, `ipc::agents::{agent_vm_ready, pi_local_models}`, `isolation::vm_for`                                                     |
| `levels/start.rs`    | `vm_session_args()`, `start_terminal()`, `open_terminal()`, `start_action()`, `scaffold_command()`, `scaffold_end()`: from `Ready` to a pty session, an action, or a creation's `Command`                                                                                     | the tails of `terminal_launch`, `terminal_open`, `agent_launch`, `run_start`, `frameworks_create`                                                                                                 |
| `levels/requests.rs` | the file protocol for processes outside the app (editor, forwarders)                                                                                                                                                                                                          | replaces `isolation/session.rs`                                                                                                                                                                   |
| `levels/report.rs` | `VmReport`, `SandboxReport`, `ToolCheck`, `report()`, `check_specs()`, `check_script()`, `parse_check()`, `check()`; `VmAction`, `VmEffects`, `act()` (§8.5) | new (step 8) |
| `levels/labels.rs`   | `LevelLabel`, `label_of(placement)`, `label(placement, lang)` texts, `banner_line(placement, lang)`, `refusal_text(reason, lang)`                                                                                                                                             | new; the refusal texts of `ipc::agents::no_vm_for_agents` and `ipc::servers::refused_with_vm`                                                                                                     |

`isolation/` keeps what is about `sbx` itself: `status`, `Blocker`, `Sbx`, `Vm` (sandbox lifecycle, mounts, ports, publish, stop), `vm.rs` (argument builders), `daemon.rs`, `kit.rs`, `secrets.rs`. It no longer decides who goes where.

`isolation::Level { Vm, Env }` is renamed `isolation::Availability { Vm, NoVm }` (serialized `"vm"`/`"env"` unchanged for the UI until step 10 of 12.2), so that `levels::Level` is the only "level" in the code.

### 3.2 Types

```rust
// levels/mod.rs

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
    pub fn id(self) -> &'static str;
    /// The name the user reads: "Claude Code", "Codex", "Pi", "Antigravity".
    pub fn name(self) -> &'static str;
    /// The commands a user types for it: ["claude"], ["codex"], ["pi"], ["agy", "antigravity"].
    pub fn commands(self) -> &'static [&'static str];
    /// Where it runs in the VM.
    pub fn runner(self) -> AgentRunner;
    pub fn from_id(id: &str) -> Option<Agent>;
    pub fn from_command(cmd: &str) -> Option<Agent>;
}

/// Where an agent runs when the VM is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunner {
    /// An agent of `sbx` in a sandbox of its own (`atollo-a-`): Claude, Codex.
    SbxAgent,
    /// Its Linux build from the kit, in the project's sandbox (`atollo-p-`): Pi, Antigravity.
    ProjectVm,
}

/// Who asks. The level follows from it (I2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Origin {
    // Level 1
    KitTerminal { terminal: String },
    EditorTerminal,
    // Level 2
    ProjectTerminal,
    EditorTask { command: String },
    /// A command line of the project; `web`: it listens on a port (`Kind::Web`).
    Action { action: String, web: bool },
    /// An action served by a web server of the kit (static, Apache, Caddy).
    Site { action: String },
    Pool { pool: String },
    Scaffold { base: String },
    // Level 3
    AgentButton { agent: Agent },
    EditorAgentProfile { agent: Agent },
    Forwarded { agent: Agent, from: Level },
    EditorExtension { extension: String, agent: Agent },
}

impl Origin {
    pub const fn level(&self) -> Level; // table in 3.4, column "Level"
    pub fn agent(&self) -> Option<Agent>; // Some for the four level-3 origins only
}

/// Where the request is, resolved from a folder by `place_of` (never from
/// a name the caller typed: a project is a real folder under `projects/`,
/// not a link, D-77).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// `dir` is `projects/<name>`; `below` the part under it ("web/src"), `/`-separated.
    Project { name: String, dir: PathBuf, below: Option<String> },
    /// Any other folder of the PC.
    Folder { dir: PathBuf },
}

impl Place {
    /// The folder resolved: the project's folder with `below`, or the folder.
    pub fn dir(&self) -> PathBuf;
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
    Off { blockers: Vec<crate::isolation::Blocker> },
}

/// Everything `decide` needs besides the request, read once by `levels::facts`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub vm: VmState,
    /// The place's folder as the VM sees it (`vm::guest_path`): for a project
    /// its own folder, without `below`; `None` for a path the VM cannot mount
    /// (a network share).
    pub guest_dir: Option<String>,
    /// `isolation::restart_pending`: the kit daemon must restart before any VM opens.
    pub restart_pending: bool,
    /// For `AgentRunner::ProjectVm` agents: the catalog has a build for the VM's
    /// Linux target (`Component::is_available_on(guest)`), installed or installable.
    /// `false` for every other origin (not read then).
    pub linux_build: bool,
    /// The kit's tools for the VM's Linux (`kit.for_target(guest).apps()`),
    /// mounted read-only; empty where the PC has no VM. In `Facts` because
    /// `decide` has no `Kit`.
    pub guest_tools: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "where", rename_all = "camelCase")]
pub enum Placement {
    Pc {
        level: Level,                // Manual or Project, never Agent (I3)
        project: Option<String>,     // the place's project, if any
        dir: PathBuf,                // where the process starts
    },
    Vm {
        level: Level,                // Project or Agent
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Sandbox {
    /// `atollo-p-<project>-<hash>` (`vm::name`).
    Project { name: String },
    /// `atollo-a-<agent>-<project>-<hash>` (`vm::agent_name`).
    Agent { name: String, agent: Agent },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mounts {
    pub workspace_rw: PathBuf,       // the project folder
    pub tools_ro: Vec<PathBuf>,      // data/apps/linux-x64 or linux-arm64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HostAccess {
    pub databases: bool,             // the project's databases on the kit's servers (D-171)
    pub local_models: bool,          // llama-server, Pi only, when Pi uses a local model (D-178)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum VmProgram {
    Shell,                                   // interactive `bash -l`
    Line { command: String },                // editor task: `sh -lc`
    Scaffold { base: String },               // the base's lines, each the kit's PowerShell for Linux through `sbx exec` (`start::scaffold_command`)
    Action { action: String },               // `sh -lc`, tracked (vm::tracked)
    Site { action: String },                 // a container of the VM's Docker (vm::web_line)
    SbxAgent { agent: Agent },               // `sbx run --name <atollo-a-…>`
    AgentBinary { agent: Agent },            // pi / agy, Linux build, `sbx exec -it`
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Refusal {
    /// Level 3 with the VM off: the agents run only in the VM.
    AgentsNeedVm { blockers: Vec<crate::isolation::Blocker> },
    /// Level 3 outside a project folder.
    AgentOutsideProject,
    /// Level 2 outside a project folder with the VM on (an editor task in `projects/`).
    NotAProject,
    /// The folder has no path in the VM (network share).
    NoPathInVm { dir: String },
    /// A Web Server pool with the VM on.
    PoolsOnlyWithoutVm,
    /// Pi or Antigravity has no build for the VM's Linux.
    NoLinuxBuild { agent: Agent },
    /// The kit's daemon must restart for a security setting (isolation::restart_pending).
    DaemonRestartPending,
}

impl Refusal {
    /// The one fix of each reason (I7): AgentsNeedVm → OpenVmNotice;
    /// AgentOutsideProject, NotAProject → OpenFromProject; NoPathInVm →
    /// UseLocalDisk; PoolsOnlyWithoutVm → StartFromProject; NoLinuxBuild →
    /// None; DaemonRestartPending → RestartDaemon.
    pub fn fix(&self) -> Fix;
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
```

### 3.3 Inputs of `decide` and where they come from

```rust
// levels/place.rs
/// The project `dir` belongs to: the first folder under `kit.projects()`,
/// read without following links (`safe_path::is_dir`, and no link from
/// `projects/` down to `dir`: a folder reached through a link is no
/// project's); `Folder` otherwise. `dir` is read as text (`\\?\` dropped,
/// `.`/`..` resolved); names are compared as the PC's file system does,
/// without regard to case on Windows and macOS, the project's name being
/// the one on disk.
pub fn place_of(kit: &Kit, dir: &Path) -> Place;

// levels/facts.rs
/// Reads the facts for `req`: `isolation::status(kit, host, env, false)`
/// (cached at most 60 s, `FRESH`), `vm::guest_path`, `isolation::restart_pending`,
/// and for a ProjectVm agent the catalog's availability on `vm::guest_target`.
pub fn facts(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts;
```

A scaffold's caller creates the new project's folder (empty) before `place_of`, so that the scaffold has a `Place::Project` and its sandbox (R15).

The status used is the one at the moment of the request: no level is cached in files any more (today's `.level` of `isolation::session`, written when the editor opened, goes away).

### 3.4 `decide`: the decision table

```rust
/// The one decision (D-179): pure, total, no I/O.
pub fn decide(req: &Request, facts: &Facts) -> Placement;
/// `decide` with the row (`Rule::R1` … `Rule::R9`) that gave the placement:
/// for the tests and the log.
pub fn decided(req: &Request, facts: &Facts) -> (Rule, Placement);
```

`Origin::level()`:

| Origin                                                                | Level     |
| --------------------------------------------------------------------- | --------- |
| `KitTerminal`, `EditorTerminal`                                       | 1 Manual  |
| `ProjectTerminal`, `EditorTask`, `Action`, `Site`, `Pool`, `Scaffold` | 2 Project |
| `AgentButton`, `EditorAgentProfile`, `Forwarded`, `EditorExtension`   | 3 Agent   |

Rules (D-183), evaluated top to bottom; the first that matches gives the placement. "Project" in the Place column means `Place::Project`, "Folder" means `Place::Folder`, "any" means either.

| #   | Level | Origin                             | Place   | `vm` | Other facts         | Placement                                                                                                                  |
| --- | ----- | ---------------------------------- | ------- | ---- | ------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| R1  | 3     | any                                | any     | Off  | —                   | `Refused { AgentsNeedVm { blockers }, OpenVmNotice }`                                                                      |
| R2  | 1, 2  | any                                | Project | any  | —                   | `Pc { level, Some(name), dir+below }`                                                                                      |
| R3  | 1, 2  | any                                | Folder  | any  | —                   | `Pc { level, None, dir }` (sealed for no project at level 2: a pool, a task outside a project)                             |
| R4  | 3     | any                                | Folder  | On   | —                   | `Refused { AgentOutsideProject, OpenFromProject }`                                                                         |
| R5  | 3     | any                                | Project | On   | `restart_pending`   | `Refused { DaemonRestartPending, RestartDaemon }`                                                                          |
| R6  | 3     | any                                | Project | On   | `guest_dir == None` | `Refused { NoPathInVm { dir }, UseLocalDisk }`                                                                             |
| R7  | 3     | agent with `runner() == ProjectVm` | Project | On   | `!linux_build`      | `Refused { NoLinuxBuild { agent }, None }`                                                                                 |
| R8  | 3     | agent with `runner() == SbxAgent`  | Project | On   | —                   | `Vm { Agent, sandbox: Agent{agent_name(project, id)}, program: SbxAgent, host: {databases: true, local_models: false} }`  |
| R9  | 3     | agent with `runner() == ProjectVm` | Project | On   | —                   | `Vm { Agent, sandbox: Project{name(project)}, program: AgentBinary, host: {databases: true, local_models: agent == Pi} }` |

Totality: levels 1 and 2 are covered by R2/R3 for every place and VM state; level 3 by R1, R4–R9. In every `Vm` placement `mounts` is `{ workspace_rw: project dir, tools_ro: facts.guest_tools }` (`[kit.for_target(guest).apps()]`) and `workdir` is `guest_dir` joined with `below`. A `Pc` placement's `dir` is the place's folder (`Place::dir`: the project's folder joined with `below`, or the folder). A refusal's `fix` is always `reason.fix()`. Rows can overlap (R5 and R6): the first wins. The test `every_request_has_exactly_one_placement` enumerates all origins × {project, project subfolder, folder} × every fact combination (VM on, off with and without blockers, `restart_pending`, `guest_dir`, `linux_build`, `guest_tools`) and checks that a row matches, that the first row matching (the table written row by row in the test) is the one `decided` reports, and the invariants (12.3).

Note on `Forwarded { from }`: `from` is kept for the log and the label only; it never changes the row. A forwarder only exists on the PC (level 1, or level 2 without the VM); level-2 sessions in the VM have no forwarders, so an agent cannot be started from inside a project VM (section 6.8).

### 3.5 `environment`: the one `PATH`/environment builder

```rust
// levels/env.rs

/// The variable every level-1/2 process on the PC gets: "manual" or "project";
/// in the VM: "project" or "agent". Read by profile.ps1 and bashrc (banner).
pub const LEVEL_VAR: &str = "ATOLLO_LEVEL";
/// The project of a PC process, when it has one (profile.ps1/bashrc no longer guess it from the folder).
pub const PROJECT_VAR: &str = "ATOLLO_PROJECT";
/// A kit terminal's name (`Origin::KitTerminal`): its banner, and the kit's
/// profile does not apply the project's lines a second time.
pub const TERMINAL_VAR: &str = "ATOLLO_TERMINAL";
/// `data/kit/forward`: the forwarders, first on every PC `PATH` (replaces `data/kit/no-agents`).
pub const FORWARD_DIR: &str = "forward";

/// What a placement runs with.
pub enum Environment {
    /// The environment of the PC process.
    Pc(KitEnv),
    /// The environment of `sbx` on the PC (`isolation::sbx_session_env`, the
    /// body of `Sbx::session_env`: the kit's environment, the PC's temp,
    /// `daemon::ENV`) and what goes into the VM.
    Vm { sbx_env: KitEnv, guest: GuestTools },
}

/// Builds the environment of `placement` from the kit's `base` (built for
/// the PC's target). `terminal`: the kit terminal's tool choice, for
/// `Origin::KitTerminal` only. `host`: the PC's variables, for the kit's
/// environment of the VM's Linux and `sbx`'s temporary folder.
/// `None` exactly for `Placement::Refused`.
pub fn environment(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&crate::editor::terminals::Terminal>,
    lang: &str,
    host: &Host,
) -> Option<Environment>;

/// The `Pc` part (steps 1–7 below), for the callers that have no
/// `Placement` yet (12.2 steps 2–4).
pub fn pc_environment(
    kit: &Kit,
    base: KitEnv,
    level: Level,
    project: Option<&str>,
    terminal: Option<&crate::editor::terminals::Terminal>,
) -> KitEnv;
```

`Pc { level, project, .. }`, in this order (each step is a named function, tested on its own); applied again to its own result it changes nothing (`the_environment_is_idempotent`):

1. `base` cloned.
2. If `terminal` is a user terminal: its tool choice and the left-out tools stripped (`editor::terminals::apply_tools`: `versions::swap_lines` from the kit's lines, then `editor::terminals::strip`), as today. It returns the lines the environment is on (the kit's, `versions::kit_choice`, with the terminal's on top); the default terminal changes nothing.
3. If `project` is `Some`: `project_lines`, the project's lines (`versions::project_choice`) swapped **from the lines of step 2**, so a tool the project chose is at the project's line in any terminal opened in its folder; tools the project did not choose keep the terminal's lines (Q1, decided). A tool the terminal left out stays out: the project chooses lines, not the terminal's tools.
4. `seal(kit, level, project, env)`: the `SEALED` variables out; the project's own MySQL login in when `project` is `Some` (D-167). Level 2 keeps none of them. Level 1 keeps only the user's SSH agent (`SSH_AGENT` = `SSH_AUTH_SOCK`), so Git over SSH in a terminal the user types in uses the keys the user loaded (**decided by the user, 26/10/2026**); an SSH client needs nothing else (`SSH_AGENT_PID` only serves `ssh-agent -k` and the kit's environment never takes it from the PC; on Windows the OpenSSH agent is a named pipe found without a variable). The real profile path, the session bus and the kit's MySQL root stay out of level 1 too: the default kit terminal, which kept them before D-179, loses them; the user terminals, which lost the SSH agent before D-179, get it back.
5. `strip_agents(kit, env)`: every `agent_dirs(kit)` entry leaves `PATH`, with the variables pointing into them (`CP_ANTIGRAVITY`); `agent_dirs` = `data/apps/<pc-target>/{claude,codex,antigravity,pi}` and `data/kit/agents`. `forward_dir(kit)` leaves too, so that step 6 puts it once.
6. `forward_first`: `forward_dir(kit)` first on `PATH`: `data/kit/forward` from step 5 of 12.2; until then the stand-ins of `data/kit/no-agents`.
7. `mark`: `LEVEL_VAR` = `manual` or `project`; `PROJECT_VAR` = the project when `Some`, removed otherwise; `TERMINAL_VAR` = the kit terminal's name when `terminal` is `Some`, removed otherwise.

`Vm { project, sandbox, program, .. }`:

Steps 1–3 are `levels::env::guest_tools(kit, level, project, program, lang, host)`, which `prepare` calls again once it has installed what was missing (a tool installed then is on the `PATH`):

1. `guest = kit.for_target(vm::guest_target(kit.target()))`; `base_guest = KitEnv::build(&guest, settings, lang, host)`, `settings` read with `Settings::load(kit)`.
2. `isolation::project_guest_tools(kit, &guest, project, base_guest)`: `pc_environment` of `guest` at level 2 (the project's lines, sealed, agents stripped: step 5 above applied to `guest`), then `guest_view`: only `PATH` entries and variables inside `data/apps/linux-…`, as the VM sees them. For `AgentBinary { agent }` the agent's own Linux folder is added back, last, to `guest.path` (it is the program being run, not a command offered to others).
3. `-e ATOLLO_LEVEL=project|agent` added to `guest.vars`.
4. `sbx_env = isolation::sbx_session_env(base, host)`.
5. `guest.env_file` is set by `prepare` (database variables, D-171), never by `environment`.

`Refused` → `None`.

The old functions (step 2 of 12.2): `versions::project_env(kit, p, env)` was a thin wrapper of `pc_environment(kit, env, Project, Some(p), None)` for its many callers, removed with its last callers by step 4; `versions::choice_env` and `editor::terminals::env` are gone: `terminal_launch` calls `pc_environment(kit, base, Manual, place_of(dir)'s project, Some(terminal))`.

### 3.6 `prepare` and `start`

```rust
// levels/prepare.rs

/// What `prepare` needs from the app, so that it can be tested without
/// Tauri and without `sbx` (the app's: `ipc::levels::AppEffects`).
pub trait Effects {
    /// `ipc::terminals::ensure_guest_tools`: installs for the VM's Linux what `wanted` lacks.
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>);
    /// Starts the project's database servers on the PC, with the kit's
    /// environment the effects hold (the servers are the kit's own programs);
    /// each database with its port.
    fn start_databases(&self, project: &str) -> anyhow::Result<Vec<(crate::db::auth::ProjectDb, u16)>>;
    /// Starts llama-server on the PC when Pi uses a local model; its port and key.
    fn start_local_models(&self, lang: &str) -> Option<LocalModels>;
    /// `Vm::start` with `sbx`'s environment (made the first time, kit, mounts,
    /// PC ports and GitHub synced).
    fn start_sandbox(&self, vm: &Vm, env: &KitEnv, lang: &str) -> anyhow::Result<()>;
    /// `isolation::watch_pi_port`: closes the models' port once no Pi runs there.
    fn watch_local_models(&self, vm: &Vm, env: &KitEnv, project: &str, port: u16, db_ports: &[u16]);
    /// Where the preparation stands (§8.6): Begin, each Step before it runs, Done or Failed.
    fn progress(&self, event: PrepareProgress);
}

pub struct LocalModels { pub port: u16, pub key: String }

/// A placement made ready to start.
pub struct Ready {
    pub placement: Placement,     // never Refused
    pub env: Environment,         // Vm: `guest` as the sandbox got it, with its `env_file`
    pub vm: Option<crate::isolation::Vm>,   // started, ports synced, GitHub synced
}

/// For `Vm`, in this order: the tools for the VM's Linux (D-170: the kit's
/// choice, `needs`, the kit's PowerShell for a `Scaffold`, the agent's own
/// build for `AgentBinary`, at the project's lines) through
/// `fx.ensure_guest_tools`, `guest_tools` read again; the databases
/// (`fx.start_databases`) and `db.env` (D-171); the `isolation::Vm` built from
/// the placement (`isolation::project_vm`, then `Vm::for_agent` for an agent's
/// sandbox; no kit `sbx`: an error, never the PC); Pi's local models
/// (`fx.start_local_models`, `pi.env`, the port, D-178); `info.json` (§8.4);
/// `fx.start_sandbox`; `fx.watch_local_models` for a port not a database's.
/// For `Pc`: nothing (the caller's `ensure` ran before `environment`, which is
/// built from what it returns). `Refused`: an error.
pub fn prepare(
    fx: &dyn Effects,
    kit: &Kit,
    host: &Host,
    placement: Placement,
    env: Environment,
    needs: &[String],
    lang: &str,
) -> anyhow::Result<Ready>;
```

```rust
// levels/start.rs

/// The `sbx` arguments of a terminal-like `Vm` session: `Shell`
/// (`vm::kit_terminal_args`), `SbxAgent` (`Vm::agent_session_args`),
/// `AgentBinary { Pi }` (`vm::pi_session_args`), `AgentBinary { Antigravity }`
/// (`vm::agy_session_args`, step 7).
pub fn vm_session_args(kit: &Kit, ready: &Ready) -> anyhow::Result<Vec<String>>;

/// The pty session of a terminal-like placement (a PC shell, or Shell,
/// SbxAgent, AgentBinary in a VM), with the environment and the folder it
/// opens with. Its `PtyInfo` carries `level`, `label`, `sandbox` (section 8.3).
pub fn start_terminal(kit: &Kit, ready: &Ready, lang: &str) -> anyhow::Result<(crate::editor::pty::Launch, KitEnv, PathBuf)>;

/// `start_terminal` opened in the app's terminal; Pi's session holds its
/// models' port open while it runs (`isolation::pi_session`).
pub fn open_terminal<R: Runtime>(app: &AppHandle<R>, kit: &Kit, ready: &Ready, lang: &str) -> anyhow::Result<()>;

/// An action or a site as `ready` placed it: `Runner::start_in` with the Pc
/// environment, or in the VM with `sbx`'s environment and the started `Vm`
/// (`in_vm`: `vm::exec_args`/`vm::web_line` through `Vm::command`, the port
/// published with `Vm::publish`; the sandbox is no longer started there).
pub fn start_action(runner: &Arc<Runner>, ready: &Ready, project: &str, dir: &Path, action: &crate::detect::Action, lang: &str) -> anyhow::Result<RunInfo>;

/// One line of a creation, in `at` with `vars`: PC: `frameworks::line_command`
/// (the kit's PowerShell, the Pc environment); VM: `vm::scaffold_args`, the
/// kit's PowerShell for the VM's Linux through `sbx exec -w <at in the VM>`,
/// `vars` with the databases' variables in `scaffold.env` (owner only, never
/// on the command line). Run by `frameworks::run_command`.
pub fn scaffold_command(kit: &Kit, ready: &Ready, line: &str, at: &Path, vars: &[(String, String)]) -> anyhow::Result<std::process::Command>;

/// The creation is over: `scaffold.env` goes.
pub fn scaffold_end(kit: &Kit, ready: &Ready);
```

A creation in the VM (R15): the catalog's lines are PowerShell (`frameworks.json`: "one command run with the kit's PowerShell"), so each runs with the kit's PowerShell for the VM's Linux, not `sh -lc`. Its `create` lines make `{name}` in the current folder, which on the PC is `projects/`; the VM reaches only the project's folder, so they run in `<project>/.atollo-new` (`ipc::frameworks::VM_STAGING`), and what they made is moved into the project (kept files listed, as D-90), the staging folder removed. The project's folder, the sandbox's workspace, is never removed and made again as on the PC. A database of the plan is created on the kit's server, then `prepare` runs again (its port opened for the sandbox, `db.env` rewritten) and the steps get `ATOLLO_DB_*` with the VM's way to the PC.

What stays where:

| Today                                                                                                        | Fate                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `isolation::vm_for`                                                                                          | done in step 4: `isolation::project_vm` (`pub(crate)`, no status asked: the decision did), called only by `levels::prepare` (`Vm` is built only from a `Placement::Vm`)                                                                                                                                                                                                                                                                                                                               |
| `isolation::Vm::for_agent`, `agent_session_args`, `launch`, `start`, `publish`, `stop_run`                   | stay in `isolation` (sandbox lifecycle); called only from `levels::prepare`/`start` and `projects::run` (stop)                                                                                                                                                                                                                                                                                                                                                                                        |
| `isolation::vm::*` builders                                                                                  | stay (pure `sbx` argument builders, D-175 rule)                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| `isolation::seal`, `SEALED`, `agent_dirs`, `without_agents`                                                  | move to `levels::env` (`without_agents` renamed `strip_agents`, and no longer prepends stand-ins: step 6 does)                                                                                                                                                                                                                                                                                                                                                                                        |
| `isolation::agents_notice`, `with_agents_notice`, `AGENTS_NOTICE_VAR`                                        | removed; the forwarders get the reason from the reply (section 4.3)                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `isolation::session` (whole file)                                                                            | replaced by `levels::requests`                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}`                   | done in step 4: the first three moved into `levels::prepare`; `ensure_guest_tools` stays in `ipc::terminals` as the app's `Effects::ensure_guest_tools` (also «Installa i mancanti» with the VM on)                                                                                                                                                                                                                                                                                                   |
| `ipc::agents::{agent_vm_ready, refused_on_project, no_vm_for_agents, pi_local_models, watch_agent_requests}` | step 4: `agent_vm_ready`, `pi_local_models` → `levels::prepare` (the local models' watch kept: `pi_port_touch` in `prepare`, `watch_pi_port` behind `Effects`, `pi_session` in `start::open_terminal`); `no_vm_for_agents` → R1's text in `levels::labels`; `refused_on_project` kept for Antigravity until step 7, which removed it; `watch_agent_requests` goes through `decide(EditorAgentProfile)` → `prepare` → `start::vm_session_args` until step 5 replaces it with `levels::requests::serve` |
| `ipc::editing::{editor_vm_begin, editor_vm_session}`                                                         | removed in step 5 (no per-editor VM session: each task/agent request is decided when it comes); in step 4 they go through `decide(ProjectTerminal)` → `prepare`                                                                                                                                                                                                                                                                                                                                       |
| `Runner::start` (the PC wrapper of `start_in`), `frameworks::run_line`                                       | removed in step 4: `levels::start::start_action` calls `Runner::start_in`; `frameworks::run_line` split into `line_command` (built by `levels::start::scaffold_command`) and `run_command`                                                                                                                                                                                                                                                                                                            |
| `editor::terminal::open_console`                                                                             | removed in step 4: `levels::start::start_terminal` (`pty::shell`) and `open_terminal` (`open_launch_then`)                                                                                                                                                                                                                                                                                                                                                                                            |
| `ipc::servers::refused_with_vm`                                                                              | removed (R2)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
| `vm::starts_in_vm`, `vm::sbx_agent`                                                                          | replaced by `Agent::runner()`: `starts_in_vm` removed in step 7; `sbx_agent` stays as the id `Vm::for_agent` gives `sbx`                                                                                                                                                                                                                                                                                                                                                                              |
| `setup::kit_files::{NO_AGENTS_DIR, AGENT_COMMANDS, no_agent_cmd, no_agent_sh, AGENTS_NOTICE_FALLBACK}`       | replaced by `FORWARD_DIR` and the forwarder files (6.8)                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| `runtime/vm.ps1`                                                                                             | replaced by `runtime/launch.ps1` and `runtime/launch.sh` (request clients only, 4.3)                                                                                                                                                                                                                                                                                                                                                                                                                  |

## 4. IPC contract

Three channels, each with its own contract: Tauri commands and events (the app's windows), the request files (processes outside the app: the editor's profiles and the forwarders), and the CLI.

### 4.1 Tauri commands

TypeScript types (`ui/src/ipc/levels.ts`, new):

```ts
export type Level = "manual" | "project" | "agent";
export type AgentId = "claude" | "codex" | "pi" | "antigravity";

/** The label shown on every terminal (8.3). */
export type LevelLabel = "pc" | "projectVm" | "agentVm";

export interface LevelsState {
  vm: "on" | "off";
  /** Why the VM is off and what to do (the notice's text), null when on. */
  reason: string | null;
  /** False exactly when vm is "off" (I6). */
  agentsEnabled: boolean;
}

export type Refusal =
  | { code: "agents_need_vm"; blockers: IsolationBlocker[] }
  | { code: "agent_outside_project" }
  | { code: "not_a_project" }
  | { code: "no_path_in_vm"; dir: string }
  | { code: "pools_only_without_vm" }
  | { code: "no_linux_build"; agent: AgentId }
  | { code: "daemon_restart_pending" };

export type Fix =
  "open_vm_notice" | "open_from_project" | "use_local_disk" | "start_from_project" | "restart_daemon" | "none";

/** The error of a refused command: `CmdError` with `refusal` set. */
export interface RefusedError {
  message: string;
  refusal: Refusal;
  fix: Fix;
}
```

| Command                                                                                     | Request                               | Response                                                                                                                                                                                                                                                                                                                                                                                         | Status                                                                                                                                                                          |
| ------------------------------------------------------------------------------------------- | ------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `project_vm_report` | `{ project: string; check: boolean }` | `VmReport` (8.1); with `check`, «Verifica strumenti» in each sandbox | **new** (step 8) |
| `project_vm_check` | `{ project: string; agent: "claude" \| "codex" \| null }` | `SandboxReport` with its `tools` (8.2) | **new** (step 8) |
| `project_vm_action` | `{ project: string; agent: "claude" \| "codex" \| null; action: "restart" \| "recreate" \| "refresh" }` | `string[]`: the sessions it stopped (8.5) | **new** (step 8) |
| `terminal_launch`                                                                           | `{ id: string; dir?: string }`        | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept; goes through `decide(KitTerminal)`                                                                                                                                        |
| `terminal_open`                                                                             | `{ project: string }`                 | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept; `decide(ProjectTerminal)`                                                                                                                                                 |
| `run_start`                                                                                 | `{ project: string; action: string }` | `RunInfo`                                                                                                                                                                                                                                                                                                                                                                                        | kept; `decide(Action                                                                                                                                                            | Site)`, pools after R5 |
| `server_start`, `server_reload`                                                             | `{ id: string }`                      | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept; `decide(Pool)` (R2 replaces `refused_with_vm`)                                                                                                                            |
| `frameworks_create`                                                                         | as today                              | `string`                                                                                                                                                                                                                                                                                                                                                                                         | kept; `decide(Scaffold)` (VM when on: new)                                                                                                                                      |
| `agent_launch`                                                                              | `{ agent: AgentId; project: string }` | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept; `decide(AgentButton)`                                                                                                                                                     |
| `agents_status`                                                                             | `{}`                                  | `AgentInfo[]` with new fields `available: boolean`, `unavailableReason: string \| null`, `runsIn: "sbxSandbox" \| "projectVm"`, `provided: boolean` (Claude Code, Codex: brought by the VM), `kitLogin: boolean` (a sign-in of the PC an earlier Atollo saved in the kit's data); `installed`, `version`, `signedIn`, `signInKind` are the VM's (§13 item 12); `running` and `tokenLink` removed | changed (step 6)                                                                                                                                                                |
| `editor_open`                                                                               | `{ project: string }`                 | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept; no longer prepares a VM nor writes `.level`                                                                                                                               |
| `pty_sessions`, event `pty-opened`                                                          | —                                     | `PtyInfo` with new fields `level: Level`, `label: LevelLabel`, `sandbox: string \| null`                                                                                                                                                                                                                                                                                                         | changed                                                                                                                                                                         |
| `isolation_status`                                                                          | `{ force: boolean }`                  | `IsolationStatus`                                                                                                                                                                                                                                                                                                                                                                                | kept (field `level` renamed `availability` at step 10)                                                                                                                          |
| `agent_token_link`                                                                          | —                                     | —                                                                                                                                                                                                                                                                                                                                                                                                | **removed** (runs `claude setup-token` on the PC: level 3 on the PC, I3)                                                                                                        |
| `agent_token_save`                                                                          | —                                     | —                                                                                                                                                                                                                                                                                                                                                                                                | **removed** in step 6: a token nothing uses (no agent on the PC, none of the kit's secrets in a VM, I5) is not saved; a token saved before stays as kit data, «Esci» deletes it |
| `agent_sign_out`                                                                            | `{ agent }`                           | `void`                                                                                                                                                                                                                                                                                                                                                                                           | kept for the kit's data only (offered when `kitLogin`); `codex logout` is no longer run (it starts the agent's binary on the PC); the files are removed directly (step 6)       |
| `components_list`                                                                           | as today                              | `ComponentInfo[]` with new fields `agent: { kind: "provided" } \| { kind: "needsVm", reason } \| null` and `leftover: boolean` (an agent installed for the PC by an earlier Atollo, removed by the next «Applica»)                                                                                                                                                                               | changed (step 6)                                                                                                                                                                |
| `components_apply`, `components_install`, `versions_install`, `updates_list`, `setup_start` | as today                              | as today                                                                                                                                                                                                                                                                                                                                                                                         | agents planned only for the VM's Linux while the VM is on (`levels::installable`, section 7); an install asked anyway is refused with `refused_install`'s text                  |
| `vms_list`, `vm_remove`, `isolation_run`                                                    | as today                              | as today                                                                                                                                                                                                                                                                                                                                                                                         | unchanged                                                                                                                                                                       |

Every refused command returns `CmdError` with the new optional fields `refusal` and `fix` (`ipc/error.rs`), so the UI can show the right button without parsing text. On the wire (done in step 3): a `CmdError` without a refusal stays the plain string it always was, so every other error reaches the UI unchanged; a refusal is the object `RefusedError` (`{ message, refusal, fix }`). The UI reads both with `errorText(e)`/`fixOf(e)` (`ui/src/ipc/levels.ts`); `useAction` keeps the `fix` beside the error and `ErrorLine` shows the action of the fixes the app can act on: «Apri l'avviso VM» for `open_vm_notice` and `restart_daemon` (the notice holds «Ricontrolla»); the other fixes are said by the refusal's text. The Rust side: `ipc/levels.rs` (`admitted`, `pc_env`, `placed`, `place_now`), with one request builder per entry point (`kit_terminal_request`, `project_terminal_request`, `action_request`, `pool_request`, `scaffold_request`). A `Vm` placement whose `sbx` is not in the kit is an error ("Docker Sandboxes (sbx) non è nel kit."), never a start on the PC (I7).

### 4.2 Events

| Event                                 | Payload                                      | When                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------- | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `levels-changed`                      | `LevelsState`                                | **new** (step 6): the agents' availability flipped (`levels::agents::watch`, a thread started at the app's start that asks `isolation::status` then and every 60 s, the status's own cadence; and `isolation_status`, every check of the notice, forced or not); emitted only on a flip (`Told`), the first look included |
| `pty-opened`, `pty-exit`, `pty-moved` | `PtyInfo` (with `level`, `label`, `sandbox`) | changed payload                                                                                                                                                                                                                                                                                                           |
| `job`                                 | `Job` (§8.6)                                 | **new**: an install, an update or a VM's preparation changed (queued, a step, its bytes, its end); with the commands `jobs_list` and `job_cancel`                                                                                                                                                                         |

### 4.3 Request files (editor and forwarders)

Processes outside the app (VSCodium's profiles, the forwarders typed in a terminal) cannot call Tauri commands. They use files in `data/app/levels/requests/` (replaces `data/app/sandboxes/sessions/`), written and read by the same user.

Request, written by the client as `<id>.json` (`<id>`: 32 hex digits, random per request), atomically (write `<id>.tmp`, rename):

```json
{
  "v": 1,
  "origin": { "kind": "editorTask", "command": "npm test" },
  "cwd": "D:\\kit\\projects\\shop\\web",
  "args": []
}
```

`origin.kind` is one of `editorTask`, `editorAgentProfile` (with `agent`), `forwarded` (with `agent` and `from`), `editorExtension` (with `extension` and `agent`). Level-1 origins never send a request: they run on the PC without asking (R3/R4). `args`: the arguments the user typed after the agent's command (forwarders, extension wrapper).

The app (`levels::requests::serve`, started by `ipc::levels::serve_requests`, polling every 250 ms, one thread per request) takes the request by renaming it to `<id>.taken` (so it is taken once, and a client can take back a request nobody took), reads and deletes it, resolves `place_of(cwd)`, reads the facts, calls `decide`, `environment`, `prepare` (`levels::requests::reply_to`), and writes the reply twice, each atomically: `<id>.reply.json`, then `<id>.reply.sh` (the same reply as POSIX sh: its fields as quoted variables, the environment's changes in a function `atollo_env`, the arguments with `set --`, because `launch.sh` has no JSON reader). Both clients wait for `<id>.reply.sh`, the one written last, read theirs and delete both; a reply nobody read and a half-written file go after 15 minutes. A request that cannot be read, of another `v`, or of an origin that never asks (level 1, the app's own buttons) is removed and answered with `placement: "error"` and a `text`; a file named like no request is removed unanswered. The reply in JSON:

```json
{
  "v": 1,
  "placement": "vm",
  "level": "agent",
  "label": "Agente · VM",
  "program": "D:\\kit\\data\\apps\\win32-x64\\sandboxes\\bin\\sbx.exe",
  "argv": [
    "exec",
    "-it",
    "-w",
    "/d/kit/projects/shop/web",
    "--env-file",
    "…",
    "atollo-p-shop-…",
    "sh",
    "-c",
    "…; exec \"$0\" \"$@\"",
    "/d/kit/data/apps/linux-x64/pi/pi",
    "--continue"
  ],
  "env": { "set": { "TEMP": "…", "SBX_NO_TELEMETRY": "1" }, "unset": [] },
  "banner": "[Atollo] Agente · VM: pi in atollo-p-shop-…",
  "marker": "shop.pi"
}
```

In a VM `env.set` holds what `sbx` needs from `sbx_env` (the PC's `TEMP`/`TMP`/`TMPDIR`, `daemon::ENV`), as `vm.ps1` set before; for Pi `marker` is `<project>.pi`: the client writes `<project>.pi.<its pid>` in the requests' folder while Pi runs (the PC-side liveness of D-178, read by `isolation::editor_pis_alive` with `isolation::client_keys`: the kit's PowerShell, Git's `sh`/`bash` of the kit on Windows, `/bin/sh` and the shells macOS's `sh` hands over to elsewhere). The words of `args` follow the agent's own arguments: Pi's after its program (`exec "$0" "$@"`), Claude Code's and Codex's after `--` of `sbx run` **[unverified, Q4]**; any other program takes none (an error reply). On Windows the sh reply gives the program with `/` and, for a VM, `MSYS_NO_PATHCONV=1`/`MSYS2_ARG_CONV_EXCL=*` so that Git Bash leaves the VM's paths alone **[unverified on a real Git Bash with `sbx`]**.

or, refused:

```json
{ "v": 1, "placement": "refused", "level": "agent", "refusal": { "code": "agents_need_vm", "blockers": [ … ] }, "fix": "open_vm_notice", "text": "Gli agenti partono solo nella VM … (motivo, cosa fare)" }
```

or, for an `editorTask` with the VM off, `placement: "pc"` with `program` = the kit's shell (`levels::requests::task_on_pc`: the kit's PowerShell `-Command <line>` on Windows, Git Bash without it, `/bin/bash -c <line>` on macOS and Linux), `banner` «[Atollo] PC · senza VM», and `env.set`/`env.unset` = the difference between the editor server's environment (`environment(Pc{Manual, None})`) and `environment(Pc{Project})` (the level and the project, the project's lines and its MySQL login, the user's SSH agent out; the editor server's own environment is already level-1, 6.2).

The client (`runtime/launch.ps1`, PowerShell 7, the required component; `runtime/launch.sh`, POSIX sh, for a shell without the kit's PowerShell: Git Bash on Windows, the system's `/bin/sh` on macOS and Linux) writes the request, waits for the reply at most 10 minutes (the first VM start pulls its image; after 3 s it says «[Atollo] In preparazione...»), prints `banner` to stderr, applies `env`, and runs `program argv…` in the same terminal with stdio inherited, exiting with its code (`launch.sh` with `exec`, but for Pi, whose marker it removes when Pi ends). For a refusal or an error it prints `text` and exits 1. A request still in the folder after 15 s was taken by no app: the client takes it back (deletes it; failing that, the app took it meanwhile and the client waits on), prints "Atollo non è aperto: apri il terminale da Atollo." and exits 1 (`ATOLLO_LAUNCH_WAIT`, seconds, changes the 15 for the tests). This replaces the check of the `data/app/instance` lock, which a sh client cannot read. The client never decides anything: its only logic is write, wait, run. `launch.ps1` reads its words itself instead of binding PowerShell parameters, so the words typed after an agent's command reach it as they are, even one like `-c`: after `--`, or right after `-Agent <id>` of a forwarder; VSCodium passes a task's line after `-Command` on Windows and `-c` elsewhere.

Who may write requests: any process of the user can, as before (`<project>.<agent>.request`). A request can only cause what the user could do from the app: start a session that `decide` allows. A process inside a VM cannot write one: `data/app` is never mounted writable (D-168); of it a sandbox sees only its own `guest` folder, read-only (§8.4).

### 4.4 CLI

| Command                                  | Output                                                                         | Exit                                                                     |
| ---------------------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------ |
| `atollo vm <project> [--agent <id>] [--check] [--restart] [--recreate [--yes]] [--refresh] [--json]` (`app/vmcli.rs`) | the VM report (8.1) as text or JSON; `--agent claude\|codex` that agent's sandbox (its report only, its action); `--check` «Verifica strumenti» (8.2); one action at most (8.5): `--recreate` says what is lost and what stays and asks `[s/N]` unless `--yes` | 0 all good; 1 VM off, the sandbox asked missing, a tool check not `ok` (or `notInVm`), an action refused, failed or not confirmed; 2 usage |
| `atollo env --project <p>`               | today's output, now `environment(Pc{Project, p})`, plus `ATOLLO_LEVEL=project` | unchanged                                                                |

The forwarders do not go through the CLI: `Atollo.exe` is a Windows GUI-subsystem program (`main.rs`, `windows_subsystem = "windows"`), and running an interactive `sbx exec -it` as its child from a console is not something the code does today **[unverified]**; the PowerShell/sh clients run `sbx` themselves, as `vm.ps1` does today (verified, D-175).

## 5. Entry-point inventory

Every place in today's code that starts a terminal, task, action, site, pool, scaffold or agent (grep of `open_console`, `open_launch`, `pty::shell|command|program`, `kit_command`, `spawn_in`, `Command::new`, `sbx exec|run`, `vm_for`, `seal`, `without_agents`, `project_env`, `choice_env`, and the editor's profiles), on 26/10/2026, branch `feature/levels` = `main` at `c9aa2561`.

### 5.1 Levels 1–3

| #   | Entry point (file: function)                                                                                                                                                                               | Starts                                                                                 | Level                     | Today                                                                                                                                                            | With `levels`                                                                                                                                                       |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| E1  | `ipc/terminals.rs: terminal_launch` → `editor/terminal.rs: open_console`                                                                                                                                   | kit terminal (pty)                                                                     | 1                         | PC; `terminals::env` (choice, `without_agents`, stand-ins); `with_agents_notice`; **no project versions** (`ATOLLO_TERMINAL` makes profile.ps1/bashrc skip them) | `decide(KitTerminal)` → R3/R4 → `environment(Pc, terminal)` → `start_terminal`                                                                                      |
| E2  | `editor/cfg.rs: kit_profiles` ("Atollo", "Atollo - Git Bash", "Atollo - cmd", "Atollo - Bash", "Atollo - zsh") → `runtime/vm.ps1 -Shell`                                                                   | editor terminal                                                                        | 1                         | VM only (D-178), fail closed; on the PC nothing                                                                                                                  | direct kit shell (`pwsh_args`/`bash_args`), no request; `decide(EditorTerminal)` is R3/R4 by construction; the editor server's environment is already level 1 (E13) |
| E3  | `ipc/editing.rs: terminal_open`                                                                                                                                                                            | project terminal                                                                       | 2                         | `vm_for` → `project_vm_ready` → `kit_terminal_args`; else `open_console` with `project_env`                                                                      | `decide(ProjectTerminal)` → R14 or R5                                                                                                                               |
| E4  | `ipc/runs.rs: run_start` (VM branch) → `projects/run/start.rs: Runner::start_in` → `in_vm` (`exec_args`, `web_line`, `publish`)                                                                            | action / site in VM                                                                    | 2                         | `vm_for` → `project_vm_ready`                                                                                                                                    | `decide(Action                                                                                                                                                      | Site)`→ R16/R17 →`prepare`→`start_action` (step 4) |
| E5  | `ipc/runs.rs: run_start` (PC branch) → `Runner::start_in` (`Atollo.exe serve`, `projects/apache.rs: command`, `projects/caddy/command.rs`, `projects/fastcgi.rs: spawn_fastcgi_if_present`, plain command) | action / site on PC                                                                    | 2                         | `project_env`                                                                                                                                                    | R5 → `environment(Pc{Project})`                                                                                                                                     |
| E6  | `ipc/runs.rs: run_start` (pool branch) → `servers.attach` → `servers/process.rs` (pool start, `without_agents`)                                                                                            | site in a pool                                                                         | 2                         | PC; with VM the VM branch wins first                                                                                                                             | `decide(Site)` first; R5 only then pool                                                                                                                             |
| E7  | `ipc/servers.rs: server_start`, `server_reload` (`refused_with_vm`) → `servers/process.rs`                                                                                                                 | pool                                                                                   | 2                         | refused with VM                                                                                                                                                  | `decide(Pool)` → R2 or R6                                                                                                                                           |
| E8  | `app/mod.rs` setup: `servers.autostart` (skipped with VM)                                                                                                                                                  | pools at app start                                                                     | 2                         | same check inline                                                                                                                                                | `decide(Pool)` per pool                                                                                                                                             |
| E9  | `ipc/frameworks.rs: frameworks_create` → `projects/frameworks/runner.rs: run_line` (pwsh)                                                                                                                  | scaffold                                                                               | 2                         | **always PC**, `seal(None)`                                                                                                                                      | `decide(Scaffold)` → R15 (new project's sandbox) or R5                                                                                                              |
| E10 | `ipc/agents.rs: agent_launch` → `agent_vm_ready` → `Vm::launch` → `open_launch`                                                                                                                            | agent from the app                                                                     | 3                         | VM only; Antigravity refused                                                                                                                                     | `decide(AgentButton)` → R1/R8–R13                                                                                                                                   |
| E11 | `ipc/agents.rs: watch_agent_requests` ← `vm.ps1 -Agent` (`<project>.<agent>.request`) ← `editor/cfg.rs: kit_profiles` (Claude Code, Codex, Antigravity, Pi)                                                | agent from the editor                                                                  | 3                         | VM only, needs an editor session of that project                                                                                                                 | `levels::requests::serve` → `decide(EditorAgentProfile)`                                                                                                            |
| E12 | `editor/cfg.rs: merge` (`automationProfile`) → `vm.ps1 -Command`                                                                                                                                           | editor task                                                                            | 2                         | VM only; without VM nothing                                                                                                                                      | request `editorTask` → R15 or R5/R6                                                                                                                                 |
| E13 | `editor/mod.rs: Editor::start_on` (`kit_command` with `self.env()` = the full kit environment)                                                                                                             | editor server and its extension host                                                   | kit, but hosts extensions | **agents on `PATH`** (`env/path.rs` adds `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi`)                                                     | server environment = `environment(Pc{Manual, None})` (no agents, forwarders first)                                                                                  |
| E14 | `tauri/catalog/extensions.json` (`anthropic.claude-code`, general) → `editor/ext.rs` install                                                                                                               | Claude Code extension, its bundled CLI, on the PC                                      | 3                         | installed when `claude` is in the kit; runs on the PC                                                                                                            | section 10                                                                                                                                                          |
| E15 | `setup/kit_files.rs: no_agent_cmd/no_agent_sh` (`data/kit/no-agents`)                                                                                                                                      | stand-ins                                                                              | 3 (refusal only)          | print a fixed text, exit 1                                                                                                                                       | forwarders (6.8)                                                                                                                                                    |
| E16 | `agents/claude_token.rs: link` (`pty::command` of `claude setup-token`, macOS) via `ipc/agents.rs: agent_token_link`                                                                                       | agent binary on the PC                                                                 | 3                         | PC                                                                                                                                                               | removed in step 6 (I3), with `pty::command`                                                                                                                         |
| E17 | `agents/mod.rs: codex_logout` (`kit_command(codex, …) logout`) via `agent_sign_out`                                                                                                                        | agent binary on the PC                                                                 | 3                         | PC                                                                                                                                                               | removed in step 6; `auth.json` deleted directly                                                                                                                     |
| E18 | `editor/cfg.rs: kit_profiles` and `merge` when the kit's PowerShell is missing                                                                                                                             | agent profiles run `claude.exe`/`codex.exe`/`agy`/`pi.exe` directly; tasks `/bin/bash` | 3 / 2                     | **on the PC**                                                                                                                                                    | no agent profile and no task profile without the kit's PowerShell or `launch.sh` (I3)                                                                               |
| E19 | the editor's debugger adapters and extensions that run project interpreters (`ms-python.python`, `xdebug.php-debug`…)                                                                                      | project code on the PC                                                                 | 2                         | PC (declared limit in D-175)                                                                                                                                     | declared limit kept, open question Q14                                                                                                                              |
| E20 | `main.rs: Cli::Env { project }`                                                                                                                                                                            | prints an environment (no process)                                                     | 2                         | `project_env`                                                                                                                                                    | `environment(Pc{Project})`                                                                                                                                          |

Count: level 1: 2 (E1, E2); level 2: 12 (E3–E9, E12, E19, E20, plus the pool reload and autostart counted in E7/E8 as separate call sites: `server_start`, `server_reload`, `autostart`); level 3: 6 (E10, E11, E14, E15, E16, E17) plus the PC fallback E18 (levels 2 and 3); E13 is the kit process that hosts E14 and E19.

### 5.2 The kit's own programs (not levels; exempt from I1 by name)

| Program                                  | Where started                                                                                                                |
| ---------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| database servers and clients             | `db/services/{process,access,data,tools}.rs`                                                                                 |
| phpMyAdmin, mongo-express                | `db/phpmyadmin.rs`, `db/mongo_express.rs`                                                                                    |
| llama-server                             | `llm/service.rs`                                                                                                             |
| Git (clone, `ls-files`, identity), gh    | `github/mod.rs`, `github/repos.rs`, `projects/mod.rs`, `projects/frameworks/mine.rs` (with `-c core.fsmonitor=false`, D-167) |
| installers                               | `install/**`, `setup/checkup.rs`                                                                                             |
| editor server, extension installs, theme | `editor/mod.rs`, `editor/ext.rs`, `editor/theme.rs`                                                                          |
| `sbx` (status, login, ls, daemon)        | `isolation/**`, `ipc/isolation.rs: isolation_run`                                                                            |

The rule test (12.3, `no_process_outside_levels`, on since step 4) scans `tauri/src` (comments left out) twice. The gates: `open_launch`, `open_launch_then`, `pty::shell`, `pty::command`, `pty::program`, `.start_in`, `line_command`, `scaffold_args`, `kit_terminal_args`, `pi_session_args`, `agy_session_args`, `agent_session_args`, `exec_args`, `web_line`, allowed only in `levels/`, their own definitions (`editor/terminal.rs`, `editor/pty.rs`, `isolation/vm.rs`, `isolation/mod.rs`, `projects/run/start.rs`, `projects/frameworks/runner.rs`) and named exceptions: `ipc/isolation.rs` (`isolation_run`, the kit's `sbx`); `agents/claude_token.rs` (E16) was one until step 6; `isolation/session.rs` (E11/E12) was one until step 5 removed the file, and the test checks that `levels/requests.rs` goes through `decide`, `environment`, `prepare` and `start::vm_session_args`. The spawns: `Command::new`, `kit_command`, `spawn_in`, `shell_command`, allowed only in `levels/`, the system's layer (`platform/`, `os/`), the table above by file, and the level-2 programs that only a decided entry point starts (`projects/run/start.rs`, `projects/apache.rs`, `projects/caddy/command.rs`, `projects/fastcgi.rs`, `projects/frameworks/runner.rs`, `servers/process.rs`); `agents/mod.rs` (E17) was one until step 6. Each exception is written in the test with its reason.

## 6. Flows

Notation: `UI` the app's window, `IPC` a Tauri command, `L` the `levels` module, `I` `isolation`, `PTY` the app's terminal, `ED` the editor (VSCodium server and its window), `C` `launch.ps1`/`launch.sh`.

### 6.1 Kit terminal (level 1)

```
UI terminal/page.tsx ──terminal_launch{id,dir}──▶ IPC ipc/terminals.rs
  place  = L::place_of(kit, dir)                       Project{shop} | Folder
  facts  = L::facts(...)                               (vm read, not used by R3/R4)
  p      = L::decide({KitTerminal{id}, place}, facts)  ─▶ Pc{Manual, project?, dir}   (R3/R4)
  env    = L::environment(kit, base, p, Some(terminal))  versions of terminal, then project; sealed; no agents; forward/ first
  launch = L::start_terminal(...)                      pty::shell + PtyInfo{level:manual,label:"PC"}
  PTY open ─▶ profile.ps1 / bashrc: banner "PC · <terminal> · <project>"
```

### 6.2 Editor terminal (level 1)

```
UI ──editor_open{project}──▶ IPC: ensure; Editor::ensure_running (server env = environment(Pc{Manual,None}))
ED user opens a terminal (profile "Atollo")
  ED spawns pwsh -NoLogo -NoProfile -NoExit -File data/kit/profile.ps1     (no request, R3/R4)
  profile.ps1: ATOLLO_PROJECT not set → project from the folder (same rule as place_of)
              → project's swaps (versions.json) → banner "PC · editor · <project>"
  `claude` typed → data/kit/forward/claude.ps1 → 6.8
```

### 6.3 Editor task (level 2)

```
ED runs task "npm test" with automationProfile = pwsh -File data/kit/launch.ps1 -Origin editorTask -Command "npm test"
C writes requests/<id>.json {origin:{kind:editorTask,command}, cwd}
L requests::serve: place_of(cwd) ─ facts ─ decide ─▶ Pc{Project} (R2, R3)   reply {placement:pc, program: kit shell, env diff}
C prints banner, runs program argv (stdio inherited), exits with its code
```

### 6.4 Action (level 2)

```
UI projects/explorer.tsx ──run_start{project,action}──▶ IPC ipc/runs.rs
  detect action a; origin = a.static_dir|apache_dir|caddy_dir ? Site{a} : Action{a, web: a.kind == Web}
  p = decide({origin, Project{project}}, facts) = Pc (R2)
     ─ site? pool_of → servers.attach (pool) | own server ; else command
     ─ environment(Pc{Project}) ─ start::start_action ─ Runner::start_in(…)
```

### 6.5 Site through a pool (level 2)

```
UI servers/page.tsx ──server_start{id}──▶ IPC ipc/servers.rs
  p = decide({Pool{id}, Folder{data/servers/<id>}}, facts) = Pc{Project, None} (R3) ─ environment ─ servers.start
```

### 6.6 Agent from the project page (level 3)

```
UI AgentButtons ──agent_launch{agent,project}──▶ IPC ipc/agents.rs
  p = decide({AgentButton{agent}, Project{project}}, facts)
   ├ VM off ─▶ Refused{AgentsNeedVm} (R1)            UI shows reason + "Apri l'avviso VM"
   ├ claude|codex ─▶ Vm{Agent, Sandbox::Agent, SbxAgent} (R12)
   │     prepare: Vm::for_agent, tools, db.env, ports, start; args = agent_session_args
   ├ pi|antigravity ─▶ Vm{Agent, Sandbox::Project, AgentBinary} (R13)
   │     prepare: ensure Linux build (D-173), tools, db.env (+ Pi local models, pi.env;
   │              + Antigravity: Vm::with_gemini, the Gemini key through sbx's proxy), start
   │     args = pi_session_args / agy_session_args (step 7)
   └ start::open_terminal: PTY open_launch_then(sbx, args) with PtyInfo{level:agent, label:"Agente · VM", sandbox}
```

### 6.7 Agent from the editor's profile (level 3)

```
ED profile "Claude Code" = pwsh -File data/kit/launch.ps1 -Origin editorAgentProfile -Agent claude
   (profile present only while agentsEnabled, section 7)
C ─ request ─▶ L serve ─ decide(EditorAgentProfile) ─ same as 6.6 ─ reply {vm, sbx, argv}
C runs sbx argv in the editor's terminal
```

Difference from today: no editor session of the project is required (`isolation::session::take_requests` only answered projects with a session dir); `cwd` decides the project.

### 6.8 `claude` typed in a terminal on the PC (forwarder)

Files in `data/kit/forward/` (written by `setup::kit_files`, replacing `data/kit/no-agents`), for each command of `Agent::commands()` (`claude`, `codex`, `pi`, `agy`, `antigravity`):

- `<cmd>.ps1`: `& $pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot '..' 'launch.ps1') -Origin forwarded -Agent <id> -- @args`, `$pwsh` the terminal's own PowerShell: in a PowerShell of its own, because the client sets variables and exits (PowerShell finds `.ps1` on `PATH` before `.cmd` **[unverified for pwsh 7 command precedence with both present; to test]**);
- `<cmd>.cmd`: `pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%~dp0..\launch.ps1" -Origin forwarded -Agent <id> -- %*`, after `where pwsh` (without it: «Atollo: senza il PowerShell del kit gli agenti non si inoltrano alla VM…», exit 1) (cmd's `%*` keeps quotes as typed; arguments with `"`, `%` or `^` may be altered: declared limit, Q16);
- `<cmd>` (sh): `exec /bin/sh "$(dirname "$0")/../launch.sh" forwarded <id> "$@"`.

Each finds the client beside its own folder (`%~dp0`, `$PSScriptRoot`, `dirname "$0"`) rather than through `CP_DATA`, which Git Bash would give with `\`; the client reads the level the request is from (`from`) in `ATOLLO_LEVEL` itself (`manual` when unset).

```
user in a PC terminal, folder D:\kit\projects\shop\web, types: claude --resume
 forward/claude.ps1 ─▶ C writes requests/<id>.json {origin:{kind:forwarded, agent:claude, from:manual}, cwd, args:["--resume"]}
 L serve: place_of(cwd) = Project{shop, below:"web"}; decide
   ├ VM on  ─▶ R12 Vm{Agent, atollo-a-claude-shop-…} ─ prepare ─ reply argv (+ args, Q4)
   ├ VM off ─▶ R1 Refused{AgentsNeedVm}: text "Gli agenti partono solo nella VM e la VM non è attiva: <motivo>. <cosa fare>"
   └ Folder ─▶ R8 Refused{AgentOutsideProject}: "Apri il terminale nella cartella di un progetto…"
 C: same terminal, stdio passed through; exit code of sbx, or 1 on refusal
```

From a level-2 session inside the VM there is no forwarder (`data/kit` is not mounted and the agents' Linux folders are stripped): typing `claude` there gives the VM's "command not found" **[unverified: whether `sbx`'s `shell` image ships an agent CLI]**.

### 6.9 Editor extension (level 3)

```
ED extension anthropic.claude-code (only once the Q2 spike passes; section 10)
  setting claudeCode.claudeProcessWrapper = data/kit/forward/claude-extension(.cmd|sh)
  the extension runs: <wrapper> <bundled-binary-path> <its args…>
  wrapper = a forwarder with origin editorExtension: every word to the app
  L requests::agent_args drops the first word when it is not a flag (the bundled binary)
  → decide(EditorExtension) → R12 → sbx argv + the flags; the extension speaks to the agent over stdio
```

Feasibility is **unverified** (Q2): path arguments of the PC inside the VM, the extension's own local connection, a non-TTY `sbx run`. Until a spike proves it, the extension is **not installed and is uninstalled at every editor start** in Atollo's editor (section 10).

Built in step 9 (the spike as far as it goes without a VM): `data/kit/forward/claude-extension.cmd` (`launch.ps1 -Origin editorExtension -Extension anthropic.claude-code -Agent claude -- %*`) and `data/kit/forward/claude-extension` (`launch.sh editorExtension anthropic.claude-code claude "$@"`), written by `setup::kit_files` (`CLAUDE_EXTENSION`, `claude_extension_cmd`, `claude_extension_sh`); the clients accept the origin `editorExtension`; the app drops the bundled binary (`levels::requests::agent_args`, used by `reply_to`), so the agent is the sandbox's own `claude` and the extension's flags follow it. **Not wired**: Atollo never writes `claudeCode.claudeProcessWrapper` and never installs the extension; there is no setting to turn it on (the simplest honest state: nothing can start it by accident). Proven by tests: the wrapper's request (origin, every word, through `launch.sh` and `launch.ps1` for real), the dropping of the bundled path with the flags kept, the reply's `sbx` arguments with the flags right after the agent (with the fake effects of `prepare`'s tests, on Pi's placement: Claude's own sandbox asks the real `sbx` for its session). What only a real VM can tell is listed in Q2 (section 14).

### 6.10 An agent typed in the project's VM terminal (decided by the user, 26/10/2026)

A VM cannot ask the PC to open an agent's sandbox, so there is no forwarder inside a VM. The project's guest PATH has stand-ins `claude`, `codex`, `pi`, `agy` (in the read-only guest folder of §8.4) that print, translated: «Gli agenti si avviano dalla pagina del progetto, dall'editor, o scrivendo il loro nome in un terminale del PC nella cartella del progetto» and exit 1. They are absent from an agent's own placement (Pi and Antigravity running their build). Test: `an_agent_typed_in_the_vm_says_where_to_start_it`.

Done in step 8 as follows: one stand-in per command of `Agent::commands()` (`claude`, `codex`, `pi`, `agy`, `antigravity`), POSIX sh, LF and executable (`setup::kit_files::{vm_stand_in, write_guest}`), in the sandbox's `guest/agents/<agent id>/`, one folder per agent; `prepare` puts on the session's guest `PATH`, after `guest/bin` (the read-only `atollo`), the folder of every agent but the one the session runs (`prepare::guest_path_entries`): a project terminal has all four, Pi's session no `pi`, Claude Code's sandbox no `claude` (its image's runs). The text is in the kit's language at the preparation. Tests: `an_agent_typed_in_the_vm_says_where_to_start_it`, `a_stand_in_in_the_vm_refuses` (Unix), `the_guest_files_are_lf_and_executable`.

## 7. Agents disabled without the VM

`agentsEnabled = (isolation availability == Vm)`. One function decides it: `levels::agents_enabled(&VmState) -> bool` (`levels/agents.rs`, step 6); everything below reads it. Where the kit keeps an agent (Q11, decided): only for the VM, `levels::agents::kept_on`: Pi and Antigravity as their builds for the VM's Linux (every VM is Linux), Claude Code and Codex for no system (the `sbx` agent images bring them). `levels::installable(component, target, &VmState)` = any component but an agent; an agent only where kept and while the VM is on. `levels::agents::obsolete(component, target)` = an agent the kit never keeps on that target (an install of an earlier Atollo for the PC): never decided by the VM's state, so Pi's Linux build stays while the VM is off for a moment.

| Place                                                                                                                                                            | With the VM                                                                                                                                                                                                                                                                                                                                        | Without the VM                                                                                                                                                                                                                                       |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tools page, group "Agenti" (`ui/src/components/rows.tsx`, `tool.tsx`, `status.tsx`; `ComponentInfo.agent`)                                                       | Pi and Antigravity installed and updated for the VM's Linux (D-173; their state is read on the VM's Linux, `ipc::setup::agent_kit`); Claude Code and Codex «fornito dalla VM» (`AgentRow::Provided`: nothing to tick, install or update)                                                                                                           | Pi and Antigravity «serve la VM» (`AgentRow::NeedsVm`), the reason (`off_reason`: «Gli agenti girano solo nella VM, che non è attiva: <the blockers' texts>») as the title; nothing to tick, install or update; Claude Code and Codex as with the VM |
| The quick panel's tiles (`ui/src/projects/tools.tsx`)                                                                                                            | as the Tools page                                                                                                                                                                                                                                                                                                                                  | «serve la VM» / «fornito dalla VM», no button, the reason in the tooltip                                                                                                                                                                             |
| An agent installed for the PC by an earlier Atollo (`ComponentInfo.leftover`)                                                                                    | «da rimuovere» with why; removed by the next «Applica» (`runner::plan` and `remove_elsewhere` take `obsolete` agents out, chosen or not)                                                                                                                                                                                                           | the same                                                                                                                                                                                                                                             |
| Runner plans (`setup::runner::plan`: first start, «Aggiorna tutto», `components_apply`, `components_install`, `versions_install`, `setup_start`, the VM's tools) | `levels::installable` with `Options::vm` (the VM as the caller found it: `Some(On)` for the tools of the VM's Linux, `then_tools_for`, `ensure_guest_tools`, an install routed to the VM)                                                                                                                                                          | `Options::vm` off or not asked: no agent planned                                                                                                                                                                                                     |
| Updates (`updates_list`, update summary D-161)                                                                                                                   | `setup::updates::pending(kit, vm)` lists an agent only while installable; `watched` never asks for an obsolete one                                                                                                                                                                                                                                 | agents not listed                                                                                                                                                                                                                                    |
| The VM's tools (`editor::terminals::chosen_tools`)                                                                                                               | never an agent: Pi's or Antigravity's build is asked by its own session (`prepare::wanted_tools`); Claude Code and Codex never downloaded                                                                                                                                                                                                          | —                                                                                                                                                                                                                                                    |
| Agents page (`ui/src/agents/page.tsx`)                                                                                                                           | the sign-in the VM's agents use, as far as the PC can tell (§13 item 12): Antigravity's Gemini key through `sbx`'s proxy, Pi on the kit's local models; otherwise «l'accesso si fa nella VM, al primo avvio dell'agente». A sign-in of the PC saved in the kit by an earlier Atollo (`kitLogin`) is told as unused, «Esci» removes it (files only) | only the notice with the reason; MCP and instructions editing kept (kit data, the shared row)                                                                                                                                                        |
| `AgentButtons` (project page, side bar row, Home card) and palette entries (`ui/src/shell/commands.tsx`, `PaletteItem.disabled`)                                 | shown                                                                                                                                                                                                                                                                                                                                              | shown disabled with the reason as title (not hidden: the user sees why); Enter and a click do nothing                                                                                                                                                |
| `startsOnProject` (`ui/src/ipc/agents.ts`)                                                                                                                       | removed: Antigravity starts on a project (section 9); availability comes from `AgentInfo.available`                                                                                                                                                                                                                                                | —                                                                                                                                                                                                                                                    |
| IPC `agent_launch`                                                                                                                                               | R12/R13                                                                                                                                                                                                                                                                                                                                            | R1 `Refused{AgentsNeedVm}`                                                                                                                                                                                                                           |
| Editor settings (`editor/cfg.rs: kit_profiles(kit, system_root, agents)`)                                                                                        | agent profiles written: Claude Code and Codex, and Pi and Antigravity once their Linux build is in the kit (never one installed for the PC)                                                                                                                                                                                                        | agent profiles **absent**; a `defaultProfile` pointing to one falls back to "Atollo"                                                                                                                                                                 |
| Editor extensions with `"level": "agent"` in `extensions.json` (section 10)                                                                                      | per section 10                                                                                                                                                                                                                                                                                                                                     | not installed; installed ones uninstalled at every editor start (§10)                                                                                                                                                                                |
| Forwarders                                                                                                                                                       | forward (6.8)                                                                                                                                                                                                                                                                                                                                      | print the reason (R1) and exit 1                                                                                                                                                                                                                     |
| Kit data (`data/home/.claude`, `.codex`, Pi's dir, Antigravity's saved login, `data/agents/claude/oauth-token`)                                                  | kept                                                                                                                                                                                                                                                                                                                                               | kept, untouched                                                                                                                                                                                                                                      |

Re-enabling: `levels::agents::watch` (started by `ipc::levels::watch_levels` at the app's start) asks the status then and every minute; it and every check of the notice (`isolation_status`) go through `levels::agents::TOLD`, which tells only a flip of the availability, as `levels-changed` with `LevelsState`. On it the interface reads again `agents_status` (`useAgents`), the agents' rows of the Tools page (keeping the choice not yet applied) and the quick panel's tiles. The commands that answer at once (`components_list`, `updates_list`, an install) read the VM as the last status saw it (`levels::agents::known_vm`), without asking `sbx`. The editor's agent profiles follow **at the editor's next start** (`cfg::write`, written at every start with `known_vm`): VSCodium's reload of a changed Machine `settings.json` is unverified (Q9), so nothing rewrites it while the editor runs; until then a profile written with the VM on asks the app, which refuses with R1's reason, and none appears with the VM off (decided in step 6, replacing "on it the app rewrites the editor settings").

## 8. Trust features

### 8.1 The VM report (data model)

```rust
// levels/report.rs
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct VmReport {
    pub project: String,
    pub vm: VmOnOff,                     // "on" | "off"
    pub reason: Option<String>,          // why off, the notice's text
    pub sandboxes: Vec<SandboxReport>,   // the project's atollo-p-…, then atollo-a-claude-…, atollo-a-codex-… (§8.5)
    pub published: Vec<PublishedPort>,   // the runner's VM runs with a port (`Runner::in_sandbox`), address 127.0.0.1 or 0.0.0.0
    pub github: bool,                    // project setting github_in_vm (D-172)
}
pub struct SandboxReport {
    pub name: String,
    pub agent: Option<Agent>,            // None for the project's
    pub state: SandboxRun,               // "running" | "stopped" | "missing" (sbx ls --json; without sbx's answer: "stopped" once made, else "missing")
    pub last_used: Option<String>,
    pub mounts: Vec<MountInfo>,          // from kit::mounts (.mounts): the workspace rw, the rest ro
    pub host_ports: Vec<HostPortInfo>,   // from kit::host_ports (.ports): which PC ports it reaches
    pub info: Option<SandboxInfo>,       // its info.json (§8.4)
    pub sessions: Vec<String>,           // the app's sessions running in it (terminals, actions, agents)
    pub tools: Option<Vec<ToolCheck>>,   // Some only after "Verifica strumenti"
    pub note: Option<String>,            // why the check found nothing (not running)
}
pub struct MountInfo { pub host: String, pub guest: String, pub mode: Mode /* "rw" | "ro" */ }
pub struct HostPortInfo { pub port: u16, pub what: String /* "mysql 8.4", "mongodb 8.0", "llama-server" */ }
pub struct PublishedPort { pub port: u16, pub address: String, pub action: String }
pub struct ToolCheck {
    pub id: String, pub name: String,
    pub chosen: Option<String>,   // the project's line, else the kit's choice; None for unversioned
    pub command: String,          // what ran, shown verbatim
    pub path: Option<String>,     // `command -v` in the VM
    pub found: Option<String>,    // first version number in the output
    pub verdict: Verdict,         // ok | otherLine | missing | notFromKit | notInVm
    pub output: String,           // first 3 lines, for the detail
}
```

The project page gets a panel «VM del progetto» (`ui/src/projects/vm.tsx`, section «VM» of the project's page): the project's sandbox, then each agent's, each with its state on the VM pill's colours (`SandboxPill`, `ui/src/shell/isolation.tsx`: green «accesa», grey «spenta» / «non c'è», blue while an action works on it), its name, its mounts with rw/ro, the PC ports it reaches, its running sessions, the tools table after «Verifica strumenti» (`project_vm_check`), and the four actions of §8.5; the published ports and GitHub in the VM below; without the VM it shows the reason and «Apri l'avviso VM», the actions disabled. Step 8 changed the data model: one `SandboxReport` per sandbox (§8.5 «Every sandbox, one per agent»), where the draft had one set of mounts and ports for the project.

### 8.2 «Verifica strumenti»: what really runs

Only with the sandbox running (else `verdict: missing` for all and the text "Avvia il terminale o un'azione del progetto"; the check never starts a sandbox by itself **[assumption]**). One `sbx exec` per sandbox, with the session's tools (`tools_options`: `ATOLLO_PATH`, variables), so the check sees what a session sees:

```
sbx exec -e ATOLLO_PATH=<…> [-e K=V…] <vm> sh -c '
  PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH"
  for each tool:  printf "@@%s\n" <id>; command -v <prog> || true; <prog> <args> 2>&1 | head -n 3
'
```

`<prog> <args>` come from the catalog's `check` (`tauri/catalog/components.json`), with the path translated for Linux (`setup::catalog::translate_rel`) and reduced to its file name (so `PATH` resolution is what is checked); `notFromKit` when `command -v` is not under the mounted `data/apps/linux-…`:

| Tool                                                                    | Catalog `check`                            | Command in the VM                                                                |
| ----------------------------------------------------------------------- | ------------------------------------------ | -------------------------------------------------------------------------------- |
| git                                                                     | `git/cmd/git.exe --version`                | `git --version`                                                                  |
| python                                                                  | `python/python.exe --version`              | `python3 --version`                                                              |
| node                                                                    | `node/node.exe --version`                  | `node --version`                                                                 |
| uv                                                                      | `uv/uv.exe --version`                      | `uv --version`                                                                   |
| php                                                                     | `php/php.exe --version`                    | `php --version`                                                                  |
| apache                                                                  | `apache/bin/httpd.exe -v`                  | not in VM (`notInVm`: sites use `httpd:2.4`/`php:<line>-apache` images)          |
| caddy                                                                   | `caddy/caddy.exe version`                  | not in VM (`notInVm`: a Caddy site becomes Apache, D-175)                        |
| rust                                                                    | `rust/cargo/bin/rustc.exe --version`       | `rustc --version` (built in the tools sandbox, D-180)                            |
| go                                                                      | `go/bin/go.exe version`                    | `go version`                                                                     |
| java                                                                    | `java/bin/java.exe -version`               | `java -version` (stderr captured)                                                |
| kotlin                                                                  | `kotlin/bin/kotlinc.bat -version`          | `kotlinc -version`                                                               |
| android                                                                 | `android/platform-tools/adb.exe version`   | `adb version` (built in the tools sandbox, D-180)                                |
| dotnet                                                                  | `dotnet/dotnet.exe --version`              | `dotnet --version`                                                               |
| dart                                                                    | `dart/bin/dart.exe --version`              | `dart --version`                                                                 |
| zig                                                                     | `zig/zig.exe version`                      | `zig version`                                                                    |
| julia                                                                   | `julia/bin/julia.exe --version`            | `julia --version`                                                                |
| bun                                                                     | `bun/bun.exe --version`                    | `bun --version`                                                                  |
| deno                                                                    | `deno/deno.exe --version`                  | `deno --version`                                                                 |
| typst                                                                   | `typst/typst.exe --version`                | `typst --version`                                                                |
| tinytex                                                                 | `tinytex/bin/windows/pdftex.exe --version` | `pdftex --version`                                                               |
| pnpm                                                                    | `@data/npm-global/pnpm.cmd --version`      | `pnpm --version` (`data/npm-global` mounted read-only, D-180)                    |
| cmake                                                                   | `cmake/bin/cmake.exe --version`            | `cmake --version`                                                                |
| cc                                                                      | `cc/bin/clang.exe --version`               | `clang --version`                                                                |
| ruby                                                                    | `ruby/bin/ruby.exe --version`              | `ruby --version`                                                                 |
| r                                                                       | `r/bin/Rscript.cmd --version`              | `Rscript --version` (built in the tools sandbox, D-180)                          |
| mysql                                                                   | `mysql/bin/mysql.exe --version`            | `mysql --version` (the client; the server stays on the PC)                       |
| mongodb                                                                 | `mongodb/bin/mongod.exe --version`         | `mongod --version` **[assumption: only the client tools matter in the VM; Q17]** |
| sqlite                                                                  | `sqlite/sqlite3.exe --version`             | `sqlite3 --version`                                                              |
| llama                                                                   | `llama/llama-server.exe --version`         | not in VM (`notInVm`: the server runs on the PC, D-178)                          |
| pi                                                                      | `pi/pi.exe --version`                      | `pi --version` (with Pi's folder on `PATH` for this check only)                  |
| antigravity                                                             | `antigravity/agy.exe --version`            | `agy --version` (idem)                                                           |
| claude                                                                  | `claude/claude.exe --version`              | in `atollo-a-claude-…`: `sbx exec <it> claude --version` (the image's)           |
| codex                                                                   | `codex/bin/codex.exe --version`            | in `atollo-a-codex-…`: `sbx exec <it> codex --version`                           |
| gh, pwsh, sandboxes, vscodium-server, conpty, phpmyadmin, mongo-express | —                                          | not checked (kit on the PC)                                                      |

Which tools run in the VM is the catalog's `withVm` (D-180): `vm` (the default) and `both` are checked there, `pc` and `neither` are `notInVm`; system components and the parts of the app are never checked. The tools that run Linux programs to install (rustup, micromamba, npm, sdkmanager, tlmgr, `tar` for an archive of links) are built in the kit's tools sandbox `atollo-k-tools` (`install::guest`), never staged with the VM on.

Done in step 8 as follows (`levels::report::{check_specs, check_script, parse_check, check}`): the tools checked are those of the sandbox's last preparation (its `info.json`, else `prepare::wanted_tools`), Pi and Antigravity in the project's sandbox when their Linux build is there, and an agent's own sandbox adds its agent from the image (`from_image`: any path is fine); the `sbx exec` carries the session's `ATOLLO_PATH` and variables but never the databases' `--env-file` (`vm::check_args`: the check needs no password); each tool runs in a subshell, `command -v` printed on its own line (empty when missing) and the version command only when found. The check goes through `decide` with the sandbox's request (the project terminal's origin, or the agent's button for Claude Code's and Codex's sandboxes, §8.5); in the app it is a job «Verifica strumenti · <project>» (`JobKind::VmCheck`, `vm-check`), a step per sandbox. A sandbox not running is never started: its tools are `missing` with the note «La sandbox non è accesa: avvia il terminale o un'azione del progetto, poi verifica di nuovo.».

Comparison: `chosen` = `versions::project_choice(project)[id]`, else `versions::kit_choice(guest)[id]`, else the line of the tool's default folder for the VM's Linux (the line the VM's `PATH` resolves when the project chose nothing). `found` = the first match of `[0-9]+(\.[0-9]+)+` in the output (`go version go1.23.2` → `1.23.2`, `openjdk version "21.0.5"` → `21.0.5`). Verdict: `ok` when `found == chosen` or `found` starts with `chosen + "."`, and the path is under the mounted tools; `otherLine` when found but another line; `notFromKit` when the path is outside the mount; `missing` when `command -v` finds nothing; unversioned tools are `ok` when found from the kit.

### 8.3 Labels and banners

| Level / placement | Label (it)      | Label (en)   | Where                                                                                                                                                                          |
| ----------------- | --------------- | ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1, `Pc`           | «PC»            | "PC"         | terminal tab and detached window title (`PtyInfo.label`, `ui/src/terminal/panel.tsx`; the window's title is set by `editor/terminal.rs: detach`); banner of profile.ps1/bashrc |
| 2, `Pc`           | «PC · senza VM» | "PC · no VM" | same, plus the status bar's "No VM" (unchanged)                                                                                                                                |
| 2, `Vm`           | «VM progetto»   | "Project VM" | tab; `launch` banner; action log first line                                                                                                                                    |
| 3, `Vm`           | «Agente · VM»   | "Agent · VM" | tab; `launch` banner                                                                                                                                                           |

On the tab (step 3): the label follows the session's title in the description colour and is the tab's accessible description (`aria-describedby`), the name staying the title; the sandbox's name is the tab's tooltip («Gira in <sandbox>»). Sessions of the kit's own programs (`isolation_run`'s `sbx`; macOS `claude setup-token` until step 6 removed it) are labelled «PC», level 1. The shells' banner below comes with step 8: profile.ps1/bashrc compose it from `ATOLLO_LEVEL`/`ATOLLO_PROJECT`/`ATOLLO_TERMINAL`, which `mark` already sets; no variable carries a ready line (`labels::banner_line` is `None` on the PC).

Banner text (stderr, one line, before the program):

- profile.ps1 / bashrc, from `ATOLLO_LEVEL`: `PC · <terminal or "editor"> · <project or "nessun progetto"> · agenti: inoltrati alla VM del progetto se attiva`. The shell does not know the VM's state at every moment, so the banner does not say whether it is on; the forwarder says it when used **[assumption]**.
- VM sessions (`start_terminal`, `launch` reply `banner`): `[Atollo] VM progetto: <sandbox> · <cartella nella VM>` and `[Atollo] Agente · VM: <agent> in <sandbox>`. Printed by the PC side before `sbx` starts, so it does not depend on the VM's shell; a prompt prefix inside the VM is not set (the image's `/etc/profile` may override `PS1`) **[unverified]**.

Done in step 8 as follows. profile.ps1 and bashrc print one line on stderr when `ATOLLO_LEVEL` is `manual` or `project`, after the block-letter banner and even with `ATOLLO_QUIET` (it is a trust line, not decoration): `<PC | PC · senza VM> · <ATOLLO_TERMINAL, or «editor» at level 1, «terminale del progetto» at level 2> · <ATOLLO_PROJECT or «nessun progetto»> · agenti: inoltrati alla VM del progetto se attiva`, in Italian when `ATOLLO_LANG` is `it`, in English otherwise (the shells have no catalog: French and German read the English line). The project's terminal in the app's terminal prints its `[Atollo] VM progetto: …` line from the VM's shell, before `bash -l` (`vm::kit_terminal_args(…, Some(banner))`: the PC's pty has no stderr of its own before `sbx`); a request's client prints the reply's `banner` itself (`kit_terminal_args(…, None)`); an agent's session in the app's terminal says its level by its tab's label. Test: `the_shells_print_their_level_line` (bash, and PowerShell with `ATOLLO_TEST_PWSH`).

### 8.4 `atollo` inside the VM: read only (decided by the user, 26/10/2026)

Inside a sandbox, `atollo` tells where you are and what you have; it never commands the kit (the kit's command line installs, removes, changes versions, moves sign-ins and prints secrets such as `atollo secret github`: none of it is ever reachable from a VM).

- **What it shows:** the level and the sandbox, the project, the tools with their versions (the same lines as the VM report, §8.1), the project's databases with their address in the VM (`host.docker.internal:<port>`) and user, never a password, and the PC's ports opened for this sandbox.
- **How:** Atollo writes `info.json` into the sandbox's kit folder (`data/app/sandboxes/<sandbox>/`, never the project's folder) at every `prepare` (done in step 4: `levels::prepare::SandboxInfo`: `v`, `level`, `sandbox`, `project`, `agent`, `workdir`, `tools` (id → line chosen), `databases` (`engine`, `database`, `user`, `address`), `hostPorts`); that folder is mounted read-only in the VM (step 8). A small POSIX script, `atollo`, first on the guest PATH, prints it (`atollo`, `atollo --json`); any other argument says that the kit is managed from the PC and exits 1.
- **Never:** a call to the PC, a write, a secret. Tests: `the_vm_info_holds_no_secret` (no password, key or token field; database passwords absent), `the_vm_atollo_is_read_only` (the script has no network or write command and refuses other arguments), `the_vm_info_follows_the_report`.

Done in step 8 as follows; the draft mounted the whole kit folder of the sandbox, which also holds `db.env` and `pi.env` (the databases' password, the local models' key): read-only or not, they would be readable in the VM by a file path, beyond the sessions' `--env-file`. So only a subfolder is mounted: `data/app/sandboxes/<sandbox>/guest/` (`isolation::kit::guest_dir`), read-only (`Vm::with_read_only`, mounted at the sandbox's making or with `sbx mount` into one made before, `Vm::mount_missing`), with `info.json` and `info.txt` (what `atollo` prints without words: `SandboxInfo::text`, in the kit's language at the preparation), `bin/atollo` and the stand-ins of §6.10 (`agents/<id>/<cmd>`); `prepare` writes them at every preparation (`setup::kit_files::write_guest`, LF, executable) and puts `guest/bin` first on the session's guest `PATH`. The script is `runtime/vm/atollo` (POSIX sh; its two texts put in by `kit_files::vm_atollo(lang)`): `atollo` prints `info.txt`, `atollo --json` prints `info.json`, any other word prints «Il kit si gestisce dal PC: qui atollo mostra soltanto dove sei e cosa hai (atollo, atollo --json).» and exits 1; its only commands are `dirname` and `cat`, its only redirection `>&2`. Tests also: `the_vm_info_is_mounted_read_only`, `every_sandbox_of_a_project_has_its_info`, `the_guest_files_are_lf_and_executable`.

### 8.5 The project's VM: restart, make again, refresh (decided by the user, 26/10/2026)

**Every sandbox, one per agent (decided by the user, 26/10/2026).** §8.1–§8.5 and §6.10 hold for each sandbox of a project, not only `atollo-p-`: Claude Code and Codex each have their own (`atollo-a-<agent>-<project>-<hash>`), and each gets its `info.json` and read-only `atollo` (level «Agente · VM», the agent's name, its sandbox), its own «Verifica strumenti», and the four actions. The panel lists the project's sandbox, then each agent's sandbox with its state; the CLI takes `--agent <id>` (`atollo vm <project> --agent claude --restart`), without it the project's sandbox. The agents' stand-ins (§6.10) are in every sandbox except where that agent itself runs. Tests: `every_sandbox_of_a_project_has_its_info`, `the_actions_take_the_agents_sandbox`.

The «VM del progetto» panel (§8.1) has four actions, and the CLI on the PC the same ones:

| Action                 | What it does                                                                                                                                                                                                                                                   | CLI                                                    |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ |
| **Riavvia**            | stops and starts the project's sandbox (`sbx stop`, then the start of §3.6)                                                                                                                                                                                    | `atollo vm <project> --restart`                        |
| **Ricrea**             | removes the sandbox and makes it again with the kit's current tools (`Vm::make_again`), after a confirmation that says what is lost (the VM's home, the images it pulled) and what stays (the project, its databases, the tools, the sign-ins kept by the kit) | `atollo vm <project> --recreate` (asks unless `--yes`) |
| **Aggiorna strumenti** | runs `prepare` again: tools for the guest, project versions, databases, ports, `info.json`; no restart unless the kit of the sandbox changed                                                                                                                   | `atollo vm <project> --refresh`                        |
| **Verifica strumenti** | §8.2                                                                                                                                                                                                                                                           | `atollo vm <project> --check`                          |

Every action goes through `decide` with the project terminal's origin (a refusal says why and how to fix it) and takes the sandbox's lock. Running sessions in the sandbox (terminals, actions, agents) are listed before «Riavvia» and «Ricrea», which stop them.

Done in step 8 as follows (`levels::report::{act, VmEffects, sandbox_request}`, the app's `ipc::levels::AppVm`, the command line's `app::vmcli::CliVm`):

- **The request.** The project's sandbox: `Origin::ProjectTerminal` in the project's folder. An agent's own sandbox: `Origin::AgentButton { agent }` (R12 places it there; the project terminal's origin would place the project's sandbox), so without the VM it gets R1's refusal with its fix. A `Pc` placement (the VM off) is the error «La VM non è attiva: non c'è una sandbox su cui agire.»; nothing is touched.
- **The lock.** `act` holds the sandbox's action lock (`report::action_lock`) from the sessions' list to the end of `prepare`: two actions on one sandbox never interleave. `sbx stop` and `sbx rm -f` take the sandbox's own lock of `isolation` (`isolation::{stop_sandbox, remove_to_make_again}`), the one `Vm::start` takes, so a session starting meanwhile waits.
- **The steps.** The sessions listed (`ipc::levels::sessions_in`: the app's terminals whose `PtyInfo.sandbox` is it, the actions whose `VmRun` is in it, `Runner::in_sandbox`); «Riavvia» and «Ricrea» end them (the terminals closed as «Chiudi», the actions stopped); «Riavvia» `sbx stop` **[unverified: the verb of sbx]**; «Ricrea» GitHub and the Gemini key taken back, `sbx rm -f`, `kit::forget` (its kit, `db.env`, `info.json` stay); then `prepare` again for every action (through `ipc::levels::ready`, so its progress is the job «Preparazione VM · …» of §8.6). The command returns the sessions it stopped. In the panel, «Ricrea» always asks in the page (what is lost, what stays, the sessions that stop), «Riavvia» asks only when sessions run.
- **The command line.** The app holds the sandboxes' locks and sessions, and a second process would race it: with Atollo open, `atollo vm` reports and checks but refuses the actions («Atollo è aperto: usa il pannello «VM del progetto» nella pagina del progetto, o chiudi Atollo e riprova.»). With Atollo closed it holds the kit's instance while it runs, and the effects need no app: no session to stop; `sbx stop` / `sbx rm -f`; of `prepare` only the tools for the VM's Linux (`prepare::guest_plans`, the runs of `ensure_guest_tools`, printed as `prepare` prints) — the databases, ports, `info.json` and the start come with the next session in Atollo, as the command says. A kit daemon of `sbx` it started (the report asks `isolation::status`) is stopped at its end: no VM runs without Atollo. `--recreate` prints what is lost and what stays, then asks `[s/N]` on standard input (nothing read: no) unless `--yes` **[unverified: reading a Windows console from the GUI-subsystem exe; without it, `--yes` is the way]**.

### 8.6 Progress in sight and the VM's pill (decided by the user, 26/10/2026)

The user (26/10/2026): «tutto il processo di download e preparazione della VM e strumenti deve essere evidente che si sta svolgendo»; «per il pulsante della VM, metti un'icona dedicata con un colore di fondo o pill che fa capire lo stato»; «per aggiornamento e download degli strumenti sia la pagina sia il resto deve essere più fluido e meno macchinoso».

**One model for long work**: `common::jobs::Job` (`tauri/src/common/jobs.rs`, `ui/src/ipc/jobs.ts`): `{ id, kind: install | update | vm-prepare | vm-check (step 8: «Verifica strumenti», §8.2), title, project, items, step: { index, total, label, state, error }, steps, bytes: { done, total }, note, state: queued | running | done | error | cancelled, error, created, automatic }`. Every change is the event `job`; `jobs_list` returns those kept (running, queued and the last 20 that ended), `job_cancel(id)` cancels one. The registry is the setup task's (`SetupTask::jobs`); the app sets its emitter at start.

- **Installs and updates**: each `SetupTask::start` is a job, titled by `setup::task::describe` («Installo Node.js», «Aggiorno 3 strumenti», «Scarico Node.js 22.11», «… · VM» for the VM's Linux). Asked while another run runs, it **waits in the queue** (`Snapshot.pending`, state `queued`) instead of being refused («C'è già un'installazione in corso» is gone from installs, updates and Apply); the queue runs in the run's thread, one after another, `running` until it is empty (still one run per kit: the runner's lock is unchanged). `job_cancel` takes a queued run out, or stops the running one (the downloads' cancel flag, as before); the others go on; `setup_cancel` cancels all. The runner's events become the job's: `Step` → step i/n with the component's name, `Bytes` → bytes, `Note` → note, `Done` → the step's state and error. Verification, rollback and outcomes are unchanged. `ensure_guest_tools` uses `start_and_wait` (its own job, waited by id).
- **A VM's preparation**: `levels::prepare` tells `Effects::progress(PrepareProgress)` (the trait's new method, so the tests' fake records it without `sbx`): `Begin { project, sandbox, agent, total }`, then `Step { index, total, label }` **before** each step («Strumenti per il Linux della VM»; «Database del progetto» when it reaches them; «Modelli locali» for Pi; «Informazioni della VM»; «Avvio la VM»), then `Done` or `Failed { error }`; nothing for the PC. The app's `AppEffects` makes it the job «Preparazione VM · <project>[ · <agent>]» (`ipc::levels::VmJob`): the guest tools' download is followed in its step (the tool as note, its bytes, `VmJob::follow`); `sbx` prints no progress while it pulls its image the first time, so `Vm::start(env, lang, making)` calls `making` just before `sbx create` and the step becomes «Scarico l'immagine della VM…» (indeterminate). A preparation is not cancelled (its step ends). The editor's and the forwarders' requests (`levels::requests::reply_to`, served by `ipc::levels::serve_requests`) prepare with the same `AppEffects`: a VM they prepare is a job too, in the status bar and the pill (test `a_request_in_the_vm_reports_its_progress`).
- **Status bar** (`ui/src/shell/statusbar.tsx`, `JobsItem`): while a job runs or waits, its title · step i/n and label · a small bar with the percent (the download's when its size is known, else the steps'; a part going across when unknown); with several, «3 lavori in corso:» and the running one first. A click opens Activity on that job (`JobDetails`: each step with its error, «Annulla» for an install, «Riprova» for a failed one). The Activity panel lists the jobs going on and the last five ended.
- **The session waiting for its VM** (a terminal, an action, an agent): the project's page shows «Preparazione della VM…» with the job's step and bar under its buttons (`VmPreparing`); on the other pages the same card is above the status bar, with «Dettagli».
- **The VM's pill** (`vmPill`, `VmPillBadge` in `ui/src/shell/isolation.tsx`, `VmItem`): the codicon `vm` and the state's text on a pill: green «VM» ready; blue, pulsing «VM… n%» while a VM is prepared; orange «Senza VM» when the user can fix it (`hypervisor_off`, `kvm_denied`, `apparmor`, `sbx_missing`, `sbx_signed_out`, `foreign_daemon`); grey «Senza VM» when this PC cannot have it (`old_windows`, `old_macos`, `distribution`, `kvm_missing`: firmware, `kit_disk`); red «VM ⚠» when the last preparation failed (none ended well after it), red «Senza VM» on `sbx_failed`. The colours are the tokens `--vsc-pill-*` (aliases of the workbench's success, info, orange, description and error colours, light and dark), the text in the editor's background; the button's name and tooltip say the state and what to do, never the colour alone. A click opens the VM dialog as before.
- **Fluid tools** (Tools page, quick panel, Updates pane, tool page, first start): each row or tile shows its own install (`in coda`, running with its bar, failed with «Riprova»), «Annulla» per item (`itemJob` → `job_cancel`); only the item worked on waits (`itemBusy`), the others can be asked meanwhile and queue; a step that ends reads that row only (`components_list { ids }`) and merges it, keeping the choice not applied yet, nothing remounted; the four systems' grid is read when the queue ends; «Applica» asks first only when it removes something.

Tests: `test/rust/common/jobs.rs` (steps, end, emit, kept), `test/rust/setup/task.rs` (`runs_asked_meanwhile_wait_in_the_queue`, `cancelling_the_running_job_stops_its_run`, `the_runs_events_are_its_jobs_progress`, `a_runs_title_says_what_it_does`), `test/rust/levels/prepare.rs` (`prepare_tells_its_steps_through_the_effects`, `prepare_tells_the_agent_and_the_error`, `nothing_is_told_for_the_pc`), `test/rust/ipc/levels.rs` (`a_vms_preparation_is_a_job`); `test/ui/shell/jobs.test.tsx` (status bar item, pill per state, details, waiting card, events), `test/ui/projects/hometools.test.tsx` and `test/ui/components/setup.test.tsx` (inline progress, queue, cancel, no remount).

## 9. Antigravity

What changes from D-178: Antigravity's `agy` is a terminal CLI like `claude` and `codex` (the user, 26/10/2026: «la cli antigravity non ha il medesimo utilizzo della cli codex oppure claude code?»), not a desktop editor. It becomes a level-3 agent with `AgentRunner::ProjectVm`, like Pi.

- **Linux build.** `install/agents.rs: Antigravity::latest` reads `…/manifests/{go_os}_{go_arch}.json`; for `linux_amd64`/`linux_arm64` the `url` is a `.tar.gz` with one binary `antigravity`, installed as `data/apps/linux-x64/antigravity/agy` (`antigravity_download`, verified by the existing tests in `test/rust/install/agents.rs`). Installed for the guest only (D-173 `Scope::Tools`), never for the PC any more (I6, Q11).
- **Session.** `vm::agy_session_args(vm, dir, agy, tools, extra)` = `pi_session_args` with `agy` (`sbx exec -it -w <dir> <tools> -e AGY_CLI_DISABLE_AUTO_UPDATE=true <vm> sh -c '<TOOLS_FIRST>; exec "$0" "$@"' <agy> <args>`; the two share `vm::binary_session_args`, `vm::AGY_VARS` holds the variable). No `CP_ANTIGRAVITY`, no key, no keyring (`Atollo.exe keyring restore/save` are PC-only and are not run). The program is `isolation::agent_in_vm(kit, guest, "antigravity")` (`pi_in_vm` generalised), `prepare` asks the build of the effects (`wanted_tools`: the agent's own id) and refuses with «Antigravity per il Linux della VM non è installato…» when it is still missing; the guest `PATH` gets `antigravity`'s folder back, last, for its own placement only (3.5).
- **Sign-in.** Two ways, neither puts a kit secret in the VM:
  1. **API key through the `sbx` proxy, when the settings hold a Gemini key (built in step 7) [assumption on `agy`'s side]**: the `google` service of `sbx secret` covers `GEMINI_API_KEY`/`GOOGLE_API_KEY` for `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com` (Docker docs, section 11). `prepare` marks the project's sandbox of an `AgentBinary { Antigravity }` placement (`Vm::with_gemini(true)`); its `Vm::start` (`sync_gemini`) then registers, as D-172 does for GitHub, `sbx secret set google --sandbox <atollo-p-…> --command '"<Atollo.exe>" secret gemini <nonce>'` (`isolation::secrets::{Secret::Gemini, set_args, command}`, `isolation::secret_for_sandbox`, the generalised body of `github_for_sandbox`; same guards: `asked_by_sbx`, the nonce noted in `secrets.json` under `google`, `nonce_known(kit, Secret::Gemini, nonce)`). `Atollo.exe secret gemini <nonce>` (`Cli::Secret`) prints the key of `data/app/secrets/gemini-api-key` to `sbx` only. Taken back: at an Antigravity start when the settings no longer hold a key, when the sandbox is removed (`remove_vm`) or made again (`make_again`); a start of the same sandbox for anything else (a terminal, an action, Pi) leaves it as it is, since an `agy` may be running there. Not verified on a real `sbx`: whether a `sbx exec` session of a `shell` sandbox sees `GEMINI_API_KEY` set to the proxy's placeholder (Atollo sets none: a value in the VM would be a kit secret if the proxy did not replace it), and whether `agy` sends the key in a header the proxy replaces, and to which host **[unverified]** (Q6).
  2. **Google account inside the VM** (no key in the settings, or the key not taken by `agy`): `agy`'s own login runs in the VM. The browser is off in `sbx` (`browser.enabled=false`, D-175), so what the user sees is what `agy` prints in the terminal: a link or device code to open on the PC if `agy` offers one **[unverified]**; its OAuth redirect to a loopback port inside the VM does not reach the PC's browser **[unverified]**, and then the Gemini key in the settings (Agents page, Antigravity) is the way in (Q6). A login done there stays in the sandbox's disk (the VM's home), lost if the sandbox is made again.
- **The kit's Antigravity data** (keyring blob, `data/home` settings, the Gemini key in settings) stays on disk; it is never mounted or passed.
- **Removed (step 7):** `vm.ps1`'s (step 5) and `ipc::agents::refused_on_project`'s refusal of Antigravity, in `agent_launch` and in `ipc::levels::serve_requests`; `isolation::vm::starts_in_vm`; `ui/src/ipc/agents.ts: startsOnProject` (the `AgentButtons` and the palette offer every installed agent); the agents page's "no Antigravity button" rule; the PC launchers `runtime/agents/{agy, agy.cmd}` with their keyring round trip (`data/kit/agents/agy*` in `kit_files::OBSOLETE`): nothing ran them any more (`strip_agents` keeps `data/kit/agents` off every level's `PATH`). Removed by step 6: the PC's Antigravity install (an agent's folder for the PC goes with the next «Applica», `levels::agents::obsolete`), `runtime/agents/claude`, the `keyring restore|save` command and the keyring round trip of `agents::keyring` (the eject window no longer saves anything: no agy ran on the PC); `agents::keyring` only reads the login an earlier Atollo saved in the kit (kit data). A login an agy run of an earlier Atollo left in the system's credential store without ending cleanly is no longer moved back: a declared trace (on Windows it was kept for the logon session only). The editor profile "Antigravity" was a level-3 profile through `launch` since step 5.

## 10. Editor extensions that run an agent

Rule: an extension that runs an agent (a model that decides commands and edits files) is level 3. In Atollo's editor it is allowed only if it can be pointed at the forwarder, so that the agent runs in the VM; otherwise it is disabled with a message. Extensions run in the editor server's extension host, on the PC (E13).

What was found (26/10/2026):

| Extension (Open VSX id)                                                                                                                     | Runs                                                                                                                         | Setting to point it elsewhere                                                                                                                                                                                                                                    | Source                                                                                                                                        | Verdict                                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Claude Code (`anthropic.claude-code`) — in Atollo's catalog today (`extensions.json`, general, `programs: ["claude"]`)                      | its **bundled** copy of the CLI ("The extension bundles its own copy of the CLI … for the chat panel")                       | `claudeCode.claudeProcessWrapper`: "Executable used to launch the Claude process. The bundled binary path is passed as an argument when present." Also `claudeCode.environmentVariables`, `claudeCode.useTerminal`                                               | https://code.claude.com/docs/en/vs-code (read 26/10/2026)                                                                                     | wrapper exists → candidate for the forwarder (6.9). **[unverified]**: that the panel works when the process is `sbx run …` in another file system (PC paths in arguments, its local IDE connection on `127.0.0.1` of the PC, no TTY). Until a spike passes: **disabled** |
| Codex (`openai.chatgpt`)                                                                                                                    | its bundled Codex CLI                                                                                                        | `chatgpt.cliExecutable`: "Development only: Set the path to the Codex CLI executable." The page warns "manually overriding the bundled executable can prevent parts of the extension from working." Also `chatgpt.runCodexInWindowsSubsystemForLinux` (WSL only) | https://learn.chatgpt.com/docs/developer-settings?surface=ide (redirect of https://developers.openai.com/codex/ide/settings, read 26/10/2026) | documented as development-only → **disabled**                                                                                                                                                                                                                            |
| Gemini Code Assist (`google.geminicodeassist`)                                                                                              | agent mode in the extension host **[unverified]**                                                                            | none found                                                                                                                                                                                                                                                       | not verified                                                                                                                                  | **disabled** **[assumption]**                                                                                                                                                                                                                                            |
| Gemini CLI companion (`google.gemini-cli-vscode-ide-companion`)                                                                             | nothing by itself: connects to a `gemini` CLI in a terminal                                                                  | —                                                                                                                                                                                                                                                                | not verified                                                                                                                                  | allowed (the kit has no `gemini` CLI; harmless) **[assumption]**                                                                                                                                                                                                         |
| Cline (`saoudrizwan.claude-dev`), Roo Code (`rooveterinaryinc.roo-cline`), Continue (`continue.continue`), Kilo Code (`kilocode.kilo-code`) | the agent loop inside the extension host on the PC, calling cloud APIs; runs commands in the editor's terminal (level 1, PC) | none (they are the agent)                                                                                                                                                                                                                                        | not verified                                                                                                                                  | **disabled**                                                                                                                                                                                                                                                             |

Mechanism (step 9):

- `tauri/catalog/extensions.json` has `"level": "agent"` for the known agentic extensions (above: `anthropic.claude-code`, `openai.chatgpt`, `google.geminicodeassist`, `saoudrizwan.claude-dev`, `rooveterinaryinc.roo-cline`, `continue.continue`, `kilocode.kilo-code`; the list of known ids is not complete **[assumption]**, Q15), and Claude Code's entry changed from "general, `programs: ["claude"]`" to `"level": "agent"` (§13 item 5). `editor::ext` (`Extension::runs_an_agent`, `is_agent`) never installs one: not as a general extension, not from the user's `extensions.txt`, not added by hand to a project (`clean_choice` drops it, `of_project` and `pending` skip one an older choice holds); the matrix of «Linguaggi e strumenti» does not show them.
- **At every start of the editor server**, before it starts, `ext::remove_agents` uninstalls those found in the server's extensions folder with the server's own `--uninstall-extension <id>` (`uninstall_args`); the user's choice is not kept. `--disable-extension <id>` was the first choice and does not work: the VSCodium 1.135 server prints «Ignoring option 'disable-extension': not supported for server» (run on 26/10/2026 against the kit's `vscodium-server`, Q10), and its `--help` lists only install, uninstall, list and update. `--uninstall-extension` was run on a copy of an extensions folder holding `anthropic.claude-code`: «Extension 'anthropic.claude-code' was successfully uninstalled!», no longer listed by `--list-extensions`, its folder marked in `.obsolete` (removed at the server's next start); `ext::installed` therefore leaves out the folders `.obsolete` names. A failed uninstall is logged and the editor starts: the windows still turn the extension off (next point).
- Every editor window also turns them off in its workspace (`ext::hidden`, the workbench storage of D-71), whatever the user's list says: an agentic extension installed from the editor's own view during a session runs on the PC until that window reloads or the editor restarts, a declared limit.
- The editor says why, once per start that removed one: the next editor window (or the open ones after «Riavvia l'editor») shows «Atollo ha tolto dall'editor <nomi>: esegue un agente, e in Atollo gli agenti girano solo nella VM. Avvialo dalla pagina del progetto, dal suo profilo nel terminale dell'editor o scrivendo il suo comando (per esempio claude) in un terminale nella cartella del progetto.» (`ext::agents_notice`, `Editor::take_agents_removed`, `window.__atolloAgents`).
- An extension not in the list cannot be recognised in general: declared limit (Q15).

## 11. Secrets: none from the kit enters a VM

Definition: a **kit secret** is a credential the kit holds for the user or for itself: the Claude OAuth token (D-115), Codex's and Pi's sign-ins in the kit's home, the Gemini API key of the settings, Antigravity's keyring, the GitHub token of the kit's `gh`, the root logins of the kit's database servers, the llama-server key, the editor's connection token. **Not** kit secrets: the project's own database user and password (D-171: they belong to the project and are in its `.env`).

How each agent signs in, with the VM:

| Agent                    | Sign-in                                                                                                            | Key through the `sbx` proxy                                                                                                                                                                                                                                                                                                                                                                                                                                                                                               | Source                                                     |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| Claude Code              | inside its `sbx` sandbox (`/login`); "the flow runs on the host, so the token is never exposed inside the sandbox" | `anthropic` (`ANTHROPIC_API_KEY`; `api.anthropic.com`, `console.anthropic.com`, `claude.ai`, `mcp-proxy.anthropic.com`)                                                                                                                                                                                                                                                                                                                                                                                                   | https://docs.docker.com/ai/sandboxes/security/credentials/ |
| Codex                    | `sbx run codex` prompts on the host; `sbx secret set openai --oauth` ahead of time                                 | `openai` (`OPENAI_API_KEY`; `api.openai.com`, `openai.com`, `chatgpt.com`)                                                                                                                                                                                                                                                                                                                                                                                                                                                | same                                                       |
| Antigravity              | section 9                                                                                                          | `google` (`GEMINI_API_KEY`, `GOOGLE_API_KEY`; `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com`)                                                                                                                                                                                                                                                                                                                                                       | same                                                       |
| Pi, cloud providers      | its own `/login` in the VM (its auth file in the VM's home), or a key through the proxy                            | `anthropic`, `openai`, `google`, `openrouter`, `groq`, `mistral`, `xai` cover Pi's common providers **[assumption: Pi reads these variables; Q7]**                                                                                                                                                                                                                                                                                                                                                                        | same                                                       |
| Pi, local models (D-178) | —                                                                                                                  | none: the llama-server key is not a `sbx` service. Today it goes into `pi.env` in the VM (**contradiction**, 13). Proposal: a key for that sandbox only (llama-server accepting several keys through `--api-key-file` **[unverified]**), revoked when the last Pi session ends; or `sbx secret set-custom --sandbox` with a placeholder replaced in the `Authorization` header ("Replacement applies only to HTTP request headers") toward `host.docker.internal` **[unverified that custom secrets apply to that host]** | same; Q7                                                   |

Facts from the Docker page (read 26/10/2026): service secrets are global by default and "Sandbox-scoped secrets take precedence over global secrets"; "Adding, updating, or removing a service secret takes effect in existing local sandboxes without a restart"; custom secrets are experimental. Atollo registers only sandbox-scoped secrets (D-172 rule). A key registered for the project's sandbox is usable (not readable) by the project's own code in that VM, as GitHub in D-172 and the local models in D-178, and the Gemini key once Antigravity has run there (§9; Antigravity shares the project's sandbox): declared.

## 12. Migration

### 12.1 Removed or renamed

| Symbol (file)                                                                                                                                                                                                                | Fate                                                                                                                                                                                                                                                                                        |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `isolation::Level` (`isolation/mod.rs`)                                                                                                                                                                                      | renamed `Availability { Vm, NoVm }`                                                                                                                                                                                                                                                         |
| `isolation::{SEALED, seal, agent_dirs, without_agents}`                                                                                                                                                                      | moved to `levels::env` (`without_agents` = `strip_agents` + `forward_first`; `seal` no longer strips the agents itself) — done in step 2                                                                                                                                                    |
| `versions::choice_env`, `editor::terminals::env`                                                                                                                                                                             | removed in step 2: `levels::env::pc_environment` (with `versions::swap_lines`, `editor::terminals::apply_tools`)                                                                                                                                                                            |
| `versions::project_env`                                                                                                                                                                                                      | a thin wrapper of `levels::env::pc_environment` since step 2; step 3 removed its callers in E1, E3, E5, E20, step 4 the last ones (E10, E11, `editor_vm_begin`, `editor_vm_session`) and the wrapper                                                                                        |
| `isolation::{agents_notice, with_agents_notice}`, `setup::kit_files::{AGENTS_NOTICE_VAR, AGENTS_NOTICE_FALLBACK, NO_AGENTS_DIR, AGENT_COMMANDS, no_agent_cmd, no_agent_sh}`                                                  | removed; `FORWARD_DIR` and forwarder generators instead; `data/kit/no-agents` added to `OBSOLETE` (removed at start)                                                                                                                                                                        |
| `isolation::vm_for`                                                                                                                                                                                                          | `isolation::project_vm`, called only by `levels::prepare` (step 4)                                                                                                                                                                                                                          |
| `isolation/session.rs` (all), `test/rust/isolation/session.rs`                                                                                                                                                               | replaced by `levels/requests.rs`, `test/rust/levels/requests.rs`                                                                                                                                                                                                                            |
| `isolation::vm::{starts_in_vm, sbx_agent}`                                                                                                                                                                                   | `Agent::runner()`                                                                                                                                                                                                                                                                           |
| `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}`                                                                                                                                   | `levels::prepare`                                                                                                                                                                                                                                                                           |
| `ipc::agents::{agent_vm_ready, refused_on_project, no_vm_for_agents, pi_local_models, watch_agent_requests}`                                                                                                                 | `levels::{prepare, labels, requests}`                                                                                                                                                                                                                                                       |
| `ipc::editing::{editor_vm_begin, editor_vm_session}`                                                                                                                                                                         | removed                                                                                                                                                                                                                                                                                     |
| `ipc::servers::refused_with_vm`                                                                                                                                                                                              | removed (R2)                                                                                                                                                                                                                                                                                |
| `ipc::agents::agent_token_link`, `agents::claude_token::link`                                                                                                                                                                | removed                                                                                                                                                                                                                                                                                     |
| `agents::codex_logout`                                                                                                                                                                                                       | removed                                                                                                                                                                                                                                                                                     |
| `agents::launch_env`                                                                                                                                                                                                         | removed in step 6                                                                                                                                                                                                                                                                           |
| `agents::{running, watch, codex_logout}`, `pty::command`, `claude_token::{link, apply_in, save, offered, plain_text, find_token, TokenScan}`, `ipc::agents::{agent_token_link, agent_token_save}`, `ui/src/agents/token.tsx` | removed in step 6                                                                                                                                                                                                                                                                           |
| `agents::keyring` (`moves`, `lock`, `portable`, the stores `os/{windows,unix}/agy_store.rs`, `os/{macos,linux}/agy_keyring.rs`), `Cli::Keyring`, the crates `keyring`, `dbus-secret-service`                                 | removed in step 6; `agents/keyring.rs` only reads the saved login (`saved_secrets`, `has_login`)                                                                                                                                                                                            |
| `env/mod.rs: CP_ANTIGRAVITY`; `platform::files::{write_secret, mode_after_600}`, `platform::host::machine_id`                                                                                                                | removed in step 6 (their only users were the PC's launchers and the keyring round trip)                                                                                                                                                                                                     |
| `runtime/vm.ps1`                                                                                                                                                                                                             | replaced by `runtime/launch.ps1`, `runtime/launch.sh`                                                                                                                                                                                                                                       |
| `runtime/agents/{claude, agy, agy.cmd}` (`data/kit/agents`)                                                                                                                                                                  | removed: they inject kit secrets into agents on the PC; no agent runs on the PC. `agy`, `agy.cmd` (with their keyring round trip) in step 7, `claude` in step 6                                                                                                                             |
| `isolation::pi_in_vm`                                                                                                                                                                                                        | `isolation::agent_in_vm(kit, guest, id)` (step 7)                                                                                                                                                                                                                                           |
| `isolation::secrets::{github_prefix, github_command}`, `nonce_known(kit, nonce)`, `isolation::mine_github`, `Cli::SecretGithub`                                                                                              | `isolation::secrets::{Secret, prefix, command, set_args, rm_args}`, `nonce_known(kit, secret, nonce)`, `isolation::{secret_for_sandbox, mine_noted}`, `Cli::Secret(secret, nonce)` (step 7; `github_for_sandbox` stays as its GitHub case)                                                  |
| `env/path.rs` entries `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi`                                                                                                                                     | removed from the kit's base `PATH`                                                                                                                                                                                                                                                          |
| `env/mod.rs: agent_vars` (`CP_ANTIGRAVITY`, `CLAUDE_CONFIG_DIR`, …)                                                                                                                                                          | kept only for the kit's own reading of agent data (stats, sync); never reaches a process of levels 1–3 (stripped by `strip_agents`)                                                                                                                                                         |
| `editor/cfg.rs: left_by_the_kit` and the keeping of user profiles                                                                                                                                                            | removed in step 5: only Atollo's profiles (D-179); a file with comments or trailing commas is read without them (`cfg::strip_jsonc`), its other keys kept; one still no JSON object is moved aside to `settings.json.bak-<seconds>` and written anew (`Outcome::Replaced`) **[assumption]** |
| `profile.ps1`/`bashrc` `ATOLLO_TERMINAL` skip of project versions                                                                                                                                                            | removed: `environment` applies them (3.5)                                                                                                                                                                                                                                                   |
| `ui/src/ipc/agents.ts: startsOnProject`                                                                                                                                                                                      | removed                                                                                                                                                                                                                                                                                     |

### 12.2 Order (each step compiles and passes `cargo test` and `npm test`)

1. **Types and decide.** `levels/mod.rs`, `place.rs`, `facts.rs` (reading only), `labels.rs`; tests R1–R17 and totality. No caller changes.
2. **Environment.** `levels/env.rs`: move `seal`, `SEALED`, `agent_dirs`, `strip_agents`; `environment()` for `Pc` and `Vm`; `versions::{project_env, choice_env}` and `editor::terminals::env` become calls to it (behaviour unchanged except: project versions in kit terminals (Q1), every kit terminal sealed the level-1 way, keeping only the user's SSH agent, `ATOLLO_LEVEL`/`ATOLLO_PROJECT`). The folder first on `PATH` stays the stand-ins' `data/kit/no-agents` (`forward_dir`) until step 5 makes the forwarders. The pools kept their unsealed environment without agents until step 3.
3. **PC entry points.** E1, E3 (PC branch), E5, E6, E7, E8, E9 (PC branch), E20 through `decide` + `environment`; `PtyInfo.level/label/sandbox`; `CmdError.refusal/fix`. Done as follows: each entry point builds its `Request` (place from `place_of`), reads `facts`, and `decide` answers; `Refused` is the command's error with its reason and fix; `Pc` starts with `environment`; `Vm` keeps today's VM path (E3, E4) with the refusals R9/R10 already applied. E6 decides the site first (`Site`), then the pool it joins is decided as a pool (`Pool`, R6). E8 decides per pool (`WebServers::autostart` takes the environment per pool). Every pool start (page, app start, a project's site, a reload) runs with `pc_environment(Project, None)`: sealed, no agent, the forwarders' folder first (`servers/process.rs`, again on the environment the command passes: it changes nothing then). E9 makes the new project's folder (empty) before `place_of` and removes it again before the creation (as an empty project's); with the VM on it still runs on the PC, sealed for no project, until step 4. E20 starts nothing, so it asks no `decide`: `environment(Pc{Project, p})` whatever the VM.
4. **Prepare and VM entry points.** `levels/prepare.rs` with `Effects`; E3, E4, E10 through it; E9 in the VM (new: scaffold in the new project's sandbox); rule test `no_process_outside_levels` on. Done as follows: `ipc::levels::ready` is `admitted` → `environment` → `prepare` with the app's `AppEffects`; E3 and E10 open their session with `levels::start::open_terminal` (E1 too, its `Ready` on the PC), E4/E5 start with `start::start_action`, E9 builds each line with `start::scaffold_command` (3.6). The VM's sandbox is started by `prepare` only (no longer by `Runner::start_in`). The editor's sessions (E11 `watch_agent_requests`, `editor_vm_begin`/`editor_vm_session` for E12) go through the same `decide` → `prepare` until step 5 removes them; Antigravity stays refused before `decide` (`refused_on_project`) until step 7. `sbx` runs with `environment`'s `sbx_env` (the kit's environment with the daemon's variables, 3.5), no longer with the project's sealed one. Removed: `isolation::vm_for`, `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs}`, `ipc::agents::{agent_vm_ready, pi_local_models, no_vm_for_agents}`, `ipc::editing::no_sbx`, `versions::project_env`, `Runner::start`, `frameworks::run_line`, `editor::terminal::open_console`, `ipc::levels::Placed::Vm`.
5. **Requests.** `levels/requests.rs`, `runtime/launch.ps1`, `runtime/launch.sh`; editor profiles: terminals direct (E2 level 1), tasks and agents through `launch` (E11, E12); forwarder files in `data/kit/forward` (`FORWARD_DIR`, replacing `data/kit/no-agents` in `forward_dir`), forwarding (6.8); editor server environment = `environment(Pc{Manual})` (with its profiles: until then its terminals still go through `vm.ps1` on the editor's environment); remove `session.rs`, `vm.ps1`, `editor_vm_*`; editor settings: only Atollo's profiles. Done as follows: `levels::requests` (4.3) is served by `ipc::levels::serve_requests`, started at the app's start after `requests::clear_all` (it replaces `isolation::session::clear_all` and `ipc::agents::watch_agent_requests`); Antigravity is still refused before `decide` (`refused_on_project`) until step 7. `start::vm_session_args` takes the user's words and builds an editor task's `Line` too (`vm::exec_args` in the request's folder); `vm::pi_session_args` runs Pi as `exec "$0" "$@"` so the words follow. The editor's profiles (`editor/cfg.rs`): the terminals the kit's shells as they are (`Atollo` = `pwsh_args`, Git Bash and zsh/bash with the kit's rc, cmd), the agents' profiles and `automationProfile` through `cfg::client`: `launch.ps1` with the kit's PowerShell, else `launch.sh` with the kit's Git Bash on Windows or `/bin/sh` elsewhere; on Windows with neither, no agent profile and a task profile of Windows PowerShell that says why and exits 1, the task's line after `#` (E18). An agent's profile is offered while the agent is installed for the PC or for the VM's Linux (step 6 gates it on the VM). `Editor::start_on` starts the server with `pc_environment(Manual, None)` (E13: no agent folder, the forwarders first). `kit_files` writes `launch.ps1`, `launch.sh` and the forwarders, and removes `vm.ps1` and the folder `no-agents` (`OBSOLETE`). The editor's window no longer prepares a VM (`editor_vm_begin`/`editor_vm_session` gone). The shells' notice of the agents (`ATOLLO_AGENTS_NOTICE`) is gone with `isolation::{agents_notice, with_agents_notice}`: a forwarder says why when it cannot forward; the shells' level banner stays with step 8 (8.3).
6. **Agents disabled without the VM.** `levels::agents_enabled`, `levels::installable`, runner filter, `AgentInfo.available`, `levels-changed` and `levels::watch`, editor profile gating, UI (Tools, Agents page, buttons, palette). Remove `agent_token_link`, `codex_logout`, `runtime/agents/*`, agent entries of `env/path.rs` (E16, E17, E18). Done as follows (section 7): `levels/agents.rs`; `setup::runner::Options::vm` and the filter of `plan` (agents never planned for the PC; `obsolete` ones removed); `levels_state` and `levels-changed` (`ipc::levels::{levels_state, observed, watch_levels}`); `ComponentInfo.{agent, leftover}`; `AgentInfo` the VM's (`provided`, `kitLogin`, `available`, `unavailableReason`, `runsIn`; `running`, `tokenLink` gone); `editor::cfg::{kit_profiles, merge, write}(…, agents)`; `chosen_tools` without agents. Q11: no agent installed for the PC; Claude Code and Codex never downloaded («fornito dalla VM»); an earlier install for the PC is left on disk (never on a `PATH`) and listed «da rimuovere» in Tools, removed by the next «Applica» (the codebase's own removal of components, not a sweep at start). Removed (the PC's agent code): `agent_token_link`, `agent_token_save` and the token's page (E16), `claude_token::{link, apply_in, save, …}` (a saved token is read and deleted only), `pty::command`, `agents::{launch_env, codex_logout, running, watch}` (E17; «Esci» removes the files), `runtime/agents/claude` and the folder `data/kit/agents` (`OBSOLETE`), the `keyring restore|save` command, the keyring round trip of `agents::keyring` (its stores in `os/`, `moves`, `portable`, the crates `keyring` and `dbus-secret-service`; the eject window and the app's start save nothing), `CP_ANTIGRAVITY`, the agents' folders of `env/path.rs`, `platform::{files::write_secret, files::mode_after_600, host::machine_id}`. The rule test `no_agent_binary_is_started_by_the_app_on_the_pc` (12.3).
7. **Antigravity in the VM.** `vm::agy_session_args`, `Agent::Antigravity.runner() = ProjectVm`, `secret gemini`, UI. Done as follows (§9): `Agent::Antigravity.runner()` was `ProjectVm` since step 1, so R11/R13 already placed it; the refusals before `decide` went (`ipc::agents::refused_on_project` in `agent_launch` and `ipc::levels::serve_requests`, `vm::starts_in_vm`), and Antigravity from the project's page, its editor profile and the `agy`/`antigravity` forwarders reaches `prepare` like Pi: its Linux build asked of the effects (D-173 `ensure_guest_tools`), refused with its name when still missing, its folder back on its own guest `PATH`, `start::vm_session_args` → `vm::agy_session_args`. Outside a project a forwarder gets R8's refusal; without the VM, R1's. Sign-in: the Gemini key of the settings through `sbx`'s `google` service for that sandbox only (`Vm::with_gemini`, `isolation::secret_for_sandbox`, `Atollo.exe secret gemini <nonce>`), never in the VM; without a key, `agy`'s own login in the VM. The PC launchers `runtime/agents/{agy, agy.cmd}` and their keyring round trip went (`OBSOLETE`). UI: `startsOnProject` removed, the agent buttons and the palette offer Antigravity when installed.
8. **Trust.** `levels/report.rs`, `project_vm_report`, `atollo vm`, the project panel, labels and banners. Done as follows: `levels::report` (§8.1, §8.2, §8.5) with the commands `project_vm_report`, `project_vm_check`, `project_vm_action` (`ipc/levels.rs`) and `atollo vm` (`app/vmcli.rs`, §4.4); the panel «VM del progetto» (`ui/src/projects/vm.tsx`, the section «VM» of the project's page; `SandboxPill` of `ui/src/shell/isolation.tsx`); every sandbox's `guest` folder mounted read-only with `info.json`, `info.txt`, the read-only `atollo` (`runtime/vm/atollo`) and the stand-ins (§8.4, §6.10); the shells' level line and the project terminal's line from the VM's shell (§8.3); `JobKind::VmCheck`. `ipc::terminals::ensure_guest_tools` plans with `levels::prepare::guest_plans`, shared with the command line; `app::cli::prepare` runs through `run_plan`. Agents' Tools rows, buttons and editor profiles are step 6's.
9. **Extensions.** `extensions.json` `level: agent`, disable at start, message; spike for `claudeProcessWrapper` (Q2) and, if it passes, `forward/claude-extension`. Done as follows (§10, §6.9): the seven known agentic extensions marked `level: agent`, Claude Code no longer installed by default; at every editor start they are uninstalled (`ext::remove_agents`: the server ignores `--disable-extension`, Q10) and turned off in every window; the editor's message says why and where the agents run. The spike stopped where a VM is needed: the wrapper `forward/claude-extension(.cmd|sh)` and the origin `editorExtension` of the clients are written, the bundled binary is dropped by `requests::agent_args`; not wired (no setting, the extension not installed) until the checks of Q2 pass by hand. §13 item 16 stays a declared limit (Q14).
10. **Docs.** ARCHITECTURE.md (Isolation section → points here), the Guide, DECISIONS D-179 "Prove", `isolation::Level` serialized rename in the UI. Done as follows: ARCHITECTURE.md/.it.md (the section "Isolation and the three levels" summarises this file and links it; `levels\`, `ipc\levels.rs`, `data\kit\forward`, `launch.ps1`/`launch.sh`, `guest\`, `data\app\levels\requests\`; `vm.ps1`, the editor's sessions, the agents' launchers and the keyring round trip gone), TESTING.md/.it.md (the levels' tests; the hand checklist "Docker Sandboxes and the three levels": the forwarders, the editor and the levels, the «VM del progetto» panel, `atollo` in the VM, `atollo vm`), PRIVACY.md and its three translations (agents only in the VMs, no kit secret in a VM but the local models' key of §13 item 8, the request files), the Guide (the topic "vm": the three levels, the labels and lines, the panel and its four actions, `atollo` in the VM; the agents, terminals, editor, tools and FAQ topics), DECISIONS D-179 «Prove (fase 2)» (what the automated tests prove, what remains for the hand tests of §14), CHANGELOG. **Not done:** the rename of `isolation::Level { Vm, Env }` to `Availability { Vm, NoVm }` (3.1, 12.1): a Rust change, left for a step of its own; the serialized `"vm"`/`"env"` stays as the UI reads it.

### 12.3 Test plan

Unit (Rust, `test/rust/levels/`):

| File          | Tests                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `decide.rs`   | `the_level_comes_from_the_origin`; `every_request_has_exactly_one_placement` (all origins × {Project, project subfolder, Folder} × {On, Off} × `restart_pending` × `guest_dir` × `linux_build` × `guest_tools`); one test per row: `r1_agents_need_the_vm`, `r2_pools_only_without_the_vm`, `r3_manual_in_a_project_is_on_the_pc`, `r4_manual_elsewhere_is_on_the_pc`, `r5_project_code_on_the_pc_without_the_vm`, `r6_project_code_outside_a_project_without_the_vm`, `r7_project_code_needs_a_project_with_the_vm`, `r8_an_agent_needs_a_project`, `r9_a_pending_restart_starts_nothing`, `r10_a_network_share_never_gets_a_sandbox`, `r11_no_linux_build_no_agent`, `r12_claude_and_codex_get_their_own_sandbox`, `r13_pi_and_antigravity_run_in_the_project_vm`, `r14_the_project_terminal_in_the_project_vm`, `r15_a_task_or_a_scaffold_in_the_project_vm`, `r16_an_action_in_the_project_vm`, `r17_a_site_in_a_container_of_the_vm`; invariants `an_agent_is_never_on_the_pc` (I3), `a_refusal_always_has_a_reason_and_a_fix` (I7); `placements_serialize_as_the_ipc_says` |
| `facts.rs`    | `the_guest_dir_is_the_projects_folder_in_the_vm`, `the_guest_tools_are_the_kits_linux_ones`, `a_linux_build_is_asked_for_project_vm_agents_only` (the pure parts; `facts()` itself asks `sbx`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   |
| `labels.rs`   | `every_placement_has_the_label_of_its_level`, `the_banner_names_the_sandbox`, `a_refusal_says_why_and_what_to_do`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| `env.rs`      | on each of `win32-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`: `a_pc_environment_has_no_agent_folder` (I4), `the_forwarders_come_first`, `the_project_versions_win_in_a_project_folder`, `a_user_terminal_keeps_its_other_lines`, `level_1_keeps_only_the_users_ssh_agent`, `sealed_variables_never_reach_level_2`, `a_vm_environment_has_only_the_kits_linux_tools`, `no_kit_secret_reaches_a_vm_environment` (I5: Claude token, Gemini key, llama key, root `mysql.cnf`, `GH_TOKEN`, editor token), `pi_and_agy_get_their_own_folder_back_only_for_themselves`, `refused_has_no_environment`, `the_environment_is_idempotent`, `the_old_callers_get_the_same_environment` (golden values of `project_env` and of a kit terminal); `a_project_gets_a_sealed_environment` (`seal` alone)                                                                                                                                                                                                                                                                                    |
| `agents.rs`   | step 6: `agents_are_enabled_only_with_the_vm`, `installable_on_every_target_and_vm_state`, `the_rows_of_the_agents`, `a_refused_install_says_why`, `the_levels_state_as_the_interface_reads_it`, `only_a_flip_is_told`; elsewhere `setup/runner.rs` (`agents_are_never_planned_without_the_vm`, `agents_only_for_the_vms_linux`, `an_agent_for_the_pc_is_removed`), `editor/cfg.rs` (`no_agent_profile_without_the_vm`), `agents/mod.rs` (`agents_status_follows_the_vm`, `the_sign_in_shown_is_the_vms`), `ipc/setup.rs` (`the_agents_rows_of_the_tools_page`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| `place.rs`    | `the_project_of_a_subfolder`, `a_link_is_never_a_project`, `projects_itself_is_a_folder`, `case_rules_follow_the_file_system`                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| `requests.rs` | `a_request_round_trips`, `a_refused_reply_carries_reason_fix_and_text`, `a_pc_reply_carries_only_the_environment_difference`, `a_request_is_taken_once`, `args_are_appended_after_the_agent`, `a_malformed_request_is_ignored_and_removed`, `the_reply_for_sh_sets_its_words`, `pi_sessions_are_the_clients_markers`; the clients for real in `test/rust/setup/kit_files.rs` (`a_forwarder_asks_the_app`, `a_forwarder_outside_a_project_says_why`, `launch_sh_runs_a_task_and_marks_pi`, `launch_sh_says_when_atollo_is_not_open` with a bash; `launch_ps1_forwards_and_refuses` with a `pwsh` on `PATH` or `ATOLLO_TEST_PWSH`)                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| `report.rs` | `the_check_command_of_every_catalog_tool` (the table of 8.2 from `components.json`), `versions_are_compared_by_line`, `a_tool_outside_the_mount_is_not_from_the_kit`, `java_version_on_stderr_is_read`, `the_report_reads_every_sandbox` (fake `sbx ls`: running, stopped, missing; mounts rw/ro; ports), `the_vm_info_follows_the_report`, `the_check_reads_match_mismatch_and_missing`, `the_check_script_only_reads`, `the_check_runs_in_a_shell` (Unix), `a_check_never_starts_a_sandbox`, `the_actions_stop_the_sessions_then_prepare`, `the_actions_take_the_agents_sandbox`, `an_action_goes_through_decide`, `an_action_waits_for_the_sandboxs_lock`, `a_recreation_says_what_is_lost_and_what_stays`, `a_project_has_its_sandbox_and_one_per_agent_of_sbx`; the command line in `test/rust/app/vmcli.rs` (`vm_parses_its_words`, `recreate_asks_unless_yes`, `recreate_without_a_yes_changes_nothing`, `the_exit_code_says_whether_all_is_good`, `the_report_prints_states_mounts_and_tools`) |
| `prepare.rs`  | with a fake `Effects`: `a_project_vm_gets_tools_databases_and_ports`, `pi_gets_the_local_models_port`, `an_agent_sandbox_mirrors_the_projects`, `the_vm_info_holds_no_secret` (§8.4: the writing part), `a_scaffold_gets_the_kits_powershell_and_the_plans_tools`, `no_sandbox_without_the_kits_sbx`, `nothing_is_prepared_for_the_pc` (named `…_but_ensure` before step 4: the PC's `ensure` is the caller's, before `environment`)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| `rule.rs`     | `no_process_outside_levels` (source scan, I1; on since step 4, section 5.2), `no_agent_binary_is_started_by_the_app_on_the_pc` (no `kit_command`/`pty::command` with an `agents::exe_of` path; with step 6)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |

Existing tests that change: `test/rust/isolation/mod.rs` (`the_pcs_shells_have_no_agent_on_any_system` and `a_project_gets_a_sealed_environment` moved to `levels/env.rs` in step 2, the first merged into `a_pc_environment_has_no_agent_folder`), `test/rust/isolation/session.rs` (removed), `test/rust/isolation/vm.rs` (`agy_session_args`), `test/rust/ipc/agents.rs` (`antigravity_never_starts_on_a_project` replaced by `r13_…`; `no_agent_starts_without_the_vm` stays as an integration of R1), `test/rust/setup/kit_files.rs` (`a_stand_in_says_where_the_agents_run` → `a_forwarder_asks_the_app`), `test/rust/editor/cfg.rs` (`only_atollos_profiles`, `no_agent_profile_without_the_vm`, `terminals_are_level_one`, `tasks_and_agents_go_through_launch`), `test/rust/editor/terminals.rs` (project versions), `test/rust/setup/runner.rs` (`agents_are_never_planned_without_the_vm`, `agents_only_for_the_vms_linux`), `test/rust/editor/ext.rs` (`extensions_that_run_an_agent_are_marked`, `an_agent_extension_is_never_installed`, `an_agent_extension_is_off_in_every_window`, `agents_are_uninstalled_at_the_editors_start`), `test/rust/editor/window.rs` (`the_editor_says_why_an_agent_extension_is_gone`), `test/rust/levels/requests.rs` (`the_extensions_bundled_binary_is_dropped`), `test/rust/setup/kit_files.rs` (`the_claude_extension_wrapper_asks_for_the_vms_claude`, `launch_ps1_asks_for_an_extensions_agent`).

Integration per entry point (Rust with fakes where possible; the rest by hand on Windows 11 + sbx, and on one Linux, recorded in DECISIONS "Prove"): E1 kit terminal in and out of a project (versions, forwarders, banner); E2 editor terminal on the PC; E3 project terminal VM/PC; E4/E5 an action and a site VM/PC; E6–E8 pools refused/started; E9 scaffold in the VM (new); E10 each agent from the page, VM on/off; E11 each agent profile; E12 a task in a subfolder, VM on/off, outside a project; 6.8 `claude --version` typed in a kit terminal in a project, outside, and without VM; E14 Claude Code extension disabled; `atollo vm shop --check`.

UI (`test/ui/`): `terminal/panel.test.tsx` (label per level, sandbox name), `agents/page.test.tsx` (disabled with the reason, re-enabled on `levels-changed`, Antigravity button present), `projects/vm.test.tsx` (panel, mounts rw/ro, ports, «Verifica strumenti» verdicts), `components/table.test.tsx` (agents' install disabled without VM), `shell/commands.test.tsx` (palette entries disabled with the reason), `servers/page.test.tsx` (pool refusal text from `refusal`).

## 13. Today's code that contradicts this document

1. ~~**Kit terminals ignore the project's versions** in a project folder: `editor/terminals.rs: env` sets `ATOLLO_TERMINAL` for every kit terminal, and `profile.ps1`/`bashrc` skip the project's swaps when it is set.~~ Fixed by step 2: `terminal_launch` gives `pc_environment` the project of its folder (`place_of`); `ATOLLO_TERMINAL` now only keeps the profile from applying them twice.
2. ~~**Editor terminals are VM-only** (D-178, `vm.ps1` fail-closed); D-179 makes them level 1 on the PC.~~ Fixed by step 5: the kit's shells on the PC, no request.
3. ~~**Project creation always runs on the PC**: `ipc/frameworks.rs: frameworks_create` uses `frameworks::run_line` even with the VM on.~~ Fixed by step 4: decided (`Scaffold`, R5/R9/R10/R15), and with the VM on it runs in the new project's sandbox (3.6).
4. ~~**The editor server, hence every extension, has the agents on `PATH`**: `env/path.rs` puts `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi` on the kit's `PATH`, and `editor/mod.rs: Editor::start_on` starts the server with the full kit environment.~~ Fixed by step 5 for the editor: `Editor::start_on` uses `pc_environment(Manual, None)`, which strips them; step 6 removed them from the kit's base `PATH` in `env/path.rs`.
5. ~~**The kit installs the Claude Code extension** (`extensions.json`, general, `programs: ["claude"]`), which runs its bundled CLI on the PC (level 3 on the PC).~~ Fixed by step 9: `level: agent`, never installed, uninstalled at every editor start (§10).
6. ~~**Agent binaries started on the PC by the app**: `agents/claude_token.rs: link` (`claude setup-token`, macOS) and `agents/mod.rs: codex_logout`.~~ Fixed by step 6: both removed (E16, E17).
7. ~~**Without the kit's PowerShell** the editor's agent profiles run `claude.exe`/`codex.exe`/`agy`/`pi.exe` directly on the PC and the task profile is `/bin/bash` on the PC (`editor/cfg.rs: kit_profiles`, `merge`).~~ Fixed by step 5: `launch.sh` takes over (Git Bash, `/bin/sh`); with no client, no agent profile and a task profile that refuses.
8. **A kit secret enters a VM**: Pi's local-models key goes into `pi.env` (`ipc/agents.rs: pi_local_models`), passed with `--env-file` (D-178).
9. ~~**Antigravity is refused on projects** (`vm.ps1`, `ipc/agents.rs: refused_on_project`, `ui/src/ipc/agents.ts: startsOnProject`).~~ Fixed by step 7: it runs in the project's VM (§9).
10. ~~**User terminal profiles are kept** and an unparsable settings file is left alone (`editor/cfg.rs: merge`, `write`): D-179 wants only Atollo's profiles, rewritten at every start.~~ Fixed by step 5 (12.1). The user's own settings of the editor (its `User` scope) can still add profiles: not written by Atollo, declared limit.
11. ~~**Agents are installed for the PC** (and Claude Code/Codex also for the VM's Linux, `terminals::chosen_tools` includes the `agents` group) although no agent runs on the PC and `sbx`'s agent sandboxes bring their own Claude Code and Codex.~~ Fixed by step 6 (Q11): Pi and Antigravity only for the VM's Linux, Claude Code and Codex never downloaded.
12. ~~**The agents page shows the PC-side sign-in** (`agents::status` reads the kit's home), not the one the VM's agents use.~~ Fixed by step 6: what the PC can tell of the VM's sign-in; a sign-in of the PC in the kit's data is only told as unused (`kitLogin`).
13. ~~**The editor's agent requests need an editor session of that project** (`session::take_requests` checks the session folder): a forwarder could not use them as they are.~~ Fixed by step 5: `cwd` decides the project.
14. ~~**The editor's level is noted once, when the editor opens** (`.level`): a VM that goes on or off later is not seen by the editor's terminals.~~ Fixed by step 5: each request reads the facts when it comes.
15. ~~**`agents::launch_env` has no caller** and the wrapper `runtime/agents/claude` injects a kit secret (the Claude token) into Claude Code on the PC; `agy`, `agy.cmd` (the Gemini key) went with step 7.~~ Fixed by step 6: both removed.
16. **Debugger and language extensions run project code on the PC** with the VM on (declared limit of D-175; violates I1 in spirit). **Kept as a declared limit by step 9** (Q14): step 9 covers only the extensions that run an agent. A debug adapter (`debugpy`, CodeLLDB, PHP Debug, a `launch.json` configuration) and a language extension that runs the project's code or tools (rust-analyzer's build scripts and proc macros, a test explorer, ESLint's plugins, scripts run by an extension) run in the editor server's extension host on the PC, in level 1's environment (sealed, no agent on `PATH`, the forwarders first), not in the VM, also with the VM on. Moving them into the VM needs the extension host in the VM (VSCodium's server inside the sandbox), a later decision; DECISIONS D-179 records the limit.

## 14. Open questions and unverified points

| #   | Question                                                                                                                                                                                                                                                                                                                                                        | Default in this document                                                                                                                                                                                       |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | A user terminal with explicit lines opened in a project folder: project's lines or terminal's?                                                                                                                                                                                                                                                                  | project's lines for the tools the project chose, the terminal's for the others **decided by the user, 26/10/2026**                                                                                             |
| Q2  | Can the Claude Code extension run through `claudeCode.claudeProcessWrapper` → forwarder → `sbx run` (PC paths in its arguments, its local connection to the editor, no TTY)?                                                                                                                                                                                    | **decided by the user, 26/10/2026**: disabled now; a separate spike tries the wrapper and, if it holds, re-enables it through the VM. Step 9: the extension uninstalled at every editor start; the wrapper written and not wired (§6.9). Proven here: the wrapper's request and words (both clients, for real), the bundled path dropped and the flags kept, the reply's `sbx` arguments. To try by hand with a real VM before wiring it: (a) the extension really calls the wrapper with its bundled path first and only flags after it (`--output-format stream-json --input-format stream-json …`); (b) on Windows Node starts a `.cmd` wrapper at all (Node refuses `.cmd`/`.bat` without a shell since 2024 **[unverified]** for the extension's spawn); (c) `sbx run` passes the flags after `--` (Q4) and runs with pipes, no TTY (Q5); (d) the stdio stream-JSON protocol survives the `sbx` proxying unchanged; (e) the extension's local connection (its IDE server on the PC's `127.0.0.1`, `CLAUDE_CODE_SSE_PORT`, the lock files in `~/.claude/ide`) is not reachable from the VM: which panel features break without it; (f) the PC paths the extension puts in its flags or messages (files, the workspace folder) mean nothing in the VM. If (a)–(d) hold, wiring = writing `claudeCode.claudeProcessWrapper` in the editor's settings and no longer marking the extension `level: agent` |
| Q3  | Codex extension: `chatgpt.cliExecutable` is "Development only"                                                                                                                                                                                                                                                                                                  | disabled                                                                                                                                                                                                       |
| Q4  | Does `sbx run --name <sandbox>` pass extra arguments to the agent (`-- --resume`)?                                                                                                                                                                                                                                                                              | [unverified]; if not, forwarded args are refused for Claude/Codex with a message                                                                                                                               |
| Q5  | Does `sbx run` work without a TTY (an extension's pipe)?                                                                                                                                                                                                                                                                                                        | [unverified]                                                                                                                                                                                                   |
| Q6  | Antigravity in the VM: does `agy` accept `GEMINI_API_KEY` through the `google` proxy service (header vs query parameter, host)? Does a `sbx exec` session of a `shell` sandbox see the variable's placeholder, or must Atollo pass one (never the key)? Does its Google login complete inside the VM with the browser disabled (a link or device code printed)? | built in step 7: the key through the proxy for that sandbox when the settings hold one [assumption]; otherwise `agy`'s login inside the VM, and if it cannot complete there the key is the way in [unverified] |
| Q7  | Pi: which variables it reads for cloud providers; a per-sandbox llama-server key (`--api-key-file`) or a custom secret toward `host.docker.internal`                                                                                                                                                                                                            | per-sandbox key, revoked at the last session [proposal]                                                                                                                                                        |
| Q8  | Does `sbx`'s `shell` image contain any agent CLI?                                                                                                                                                                                                                                                                                                               | [unverified]; if yes, `environment(Vm{Project})` must hide it (a `PATH` without the image's agent folder)                                                                                                      |
| Q9  | VSCodium: `automationProfile` also used for debug terminals; can a trusted workspace add terminal profiles; is Machine `settings.json` reloaded live?                                                                                                                                                                                                           | [unverified]                                                                                                                                                                                                   |
| Q10 | VSCodium web server `--disable-extension`                                                                                                                                                                                                                                                                                                                       | **verified, 26/10/2026**: not supported («Ignoring option 'disable-extension': not supported for server», VSCodium 1.135 server); uninstalled at every start instead (`--uninstall-extension`, verified on a copy of the extensions folder), step 9 |
| Q11 | Stop installing agents for the PC entirely, and Claude Code/Codex for the VM's Linux (the `sbx` images have them)?                                                                                                                                                                                                                                              | yes: agents are installed only for the VM (Pi and Antigravity Linux builds; Claude Code and Codex from the `sbx` images) — **decided by the user, 26/10/2026**                                                 |
| Q12 | A prompt label inside the VM's `bash -l`                                                                                                                                                                                                                                                                                                                        | banner only [unverified]                                                                                                                                                                                       |
| Q13 | The kit's MCP servers for agents in the VM (D-175 "Da fare")                                                                                                                                                                                                                                                                                                    | unchanged, out of D-179's scope                                                                                                                                                                                |
| Q14 | Debug adapters and language extensions executing project code on the PC with the VM on                                                                                                                                                                                                                                                                          | declared limit kept by step 9 (§13 item 16, D-179); candidate for a later decision                                                                                                                             |
| Q15 | Agentic extensions not in the list                                                                                                                                                                                                                                                                                                                              | cannot be detected in general: declared limit                                                                                                                                                                  |
| Q16 | Windows `.cmd` forwarder: arguments with `"`, `%`, `^` through `%*`                                                                                                                                                                                                                                                                                             | declared limit; `.ps1` and sh forwarders pass them intact                                                                                                                                                      |
| Q17 | MongoDB in the VM: is `mongod` the right check, or `mongosh`                                                                                                                                                                                                                                                                                                    | `mongod --version` [assumption]                                                                                                                                                                                |
| Q18 | Can PowerShell capture the stdout of `Atollo.exe` (GUI subsystem)? Relevant only if a client ever calls the CLI                                                                                                                                                                                                                                                 | not used: clients use files                                                                                                                                                                                    |
| Q19 | A creation in the VM: do the catalog's PowerShell lines written for Windows (`vendor\bin\wp`, `Expand-Archive`, `Invoke-WebRequest`) run with PowerShell for Linux; does «Annulla» (ending `sbx exec` on the PC) stop the line inside the VM?                                                                                                                   | [unverified]; a line that fails says so in the wizard, as on the PC; a cancelled line may go on in the VM until it ends (as an action before `vm::tracked`)                                                    |
