# The three levels: where every program of Atollo runs

English · [Italiano](LEVELS.it.md)

Normative document of decision D-179 (DECISIONS.md, 26/10/2026). It is the single source of truth for: which program runs where, with which `PATH`, with which agents, with which network and mounts; the code that decides it; the IPC that carries it; and how the user can check it. Code, tests, the Guide and ARCHITECTURE.md follow this file; where they differ, this file wins and the difference is a bug.

Status: **design, phase 1**. Nothing here is implemented yet unless the text says "today". Every statement that was not verified on a real system is marked **[unverified]**; every choice made by this document that the user has not confirmed is marked **[assumption]**.

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

| Threat | Level 1 (PC) | Level 2 with VM | Level 2 without VM | Level 3 (VM only) |
|---|---|---|---|---|
| Reading the user's files outside the project | possible (it is the user) | no: only the project folder (rw) and the kit's Linux tools (ro) are mounted | possible: same user; the sealed environment only removes the easy ways (SSH agent, root MySQL login, real profile path) | no, as level 2 with VM |
| Using the user's SSH keys, keyring, kit secrets | possible | no | reduced (`seal`), not prevented | no |
| Reaching the PC's network services | yes | only the ports Atollo opens for that sandbox (its databases, D-171; Pi's local models, D-178) | yes | same as level 2 with VM |
| Changing the kit's programs | possible | no (tools mounted read-only; the kit and the profile are never a workspace, D-168) | possible | no |
| Acting on GitHub as the user | possible | only if the project turned on "GitHub in the VM" (D-172), through the `sbx` proxy | possible | as level 2 with VM |

The boundary is the VM (D-167). Without it Atollo does not pretend a boundary exists: level 2 on the PC is "sealed", not "isolated", and the interface says so.

A fourth category is **not** a level: the **kit's own programs** (database servers, phpMyAdmin, mongo-express, llama.cpp, Git for cloning, the editor server process itself, `sbx`, the app). They never run project code by design and stay on the PC (D-175, "Kit on the PC"). Section 5 lists them so the inventory is complete.

## 2. The constraint table

The same table holds for the app and for the editor. "VM on" means `isolation::status(...).level == Vm` at the moment of the request (see 3.3).

| Who (origin) | Level | Where, VM on | Where, VM off | `PATH` | Agent commands (`claude`, `codex`, `pi`, `agy`, `antigravity`) | Network and ports | Mounts |
|---|---|---|---|---|---|---|---|
| Kit terminal ("All tools", user terminals), any folder | 1 | PC | PC | kit tools; in a project folder at the project's lines (3.5) | **forwarders** first on `PATH`: in a project folder they forward to the agent in the project's VM; elsewhere, or VM off, they say why and exit 1 | the PC's | none (PC) |
| Editor integrated terminal (every Atollo profile except agents) | 1 | PC | PC | as the kit terminal; the project of the folder the terminal opens in | forwarders, as above | the PC's | none (PC) |
| Project terminal (project page) | 2 | project VM `atollo-p-…` | PC, sealed | VM: kit Linux tools at the project's lines, then the image's; PC: kit tools at the project's lines | VM: none (the agents' Linux folders are stripped, `sbx`'s agents are not in the `shell` image **[unverified]**); PC: forwarders, which refuse (VM off) | VM: `sbx` policy `balanced`, its databases' ports, GitHub if chosen; PC: the PC's | VM: project rw, kit Linux tools ro |
| Action (`npm run dev`…) | 2 | project VM | PC, sealed | as project terminal | none / forwarders | VM: its web port published on `127.0.0.1` (or `0.0.0.0` with "Local network") | as project terminal |
| Site of an action (static, Apache, Caddy) | 2 | a container of the VM's Docker | PC: its own server or a pool | — | none | as action | as action, plus the project's own aliases |
| Web Server pool | 2 | **refused** (each site runs in its project's VM) | PC | kit tools, no agents | forwarders (refuse) | the PC's | none |
| Editor task (`tasks.json`) | 2 | project VM | PC, sealed | as project terminal | as project terminal | as project terminal | as project terminal |
| Project creation (framework scaffolding, "Le mie basi") | 2 | project VM (the new project's sandbox) **[new]** | PC, sealed | as project terminal | none / forwarders | VM: `balanced` (package registries) | VM: the new project folder rw, kit Linux tools ro |
| Claude Code, Codex (project page, palette, editor profile, forwarder, extension wrapper) | 3 | own `sbx` sandbox `atollo-a-<agent>-<project>-<hash>` | **refused** | kit Linux tools at the project's lines, then the image's | the agent itself only | as project VM (same database ports, same GitHub rule) | project rw, kit Linux tools ro |
| Pi, Antigravity (`agy`) | 3 | project VM `atollo-p-…`, their Linux build from the kit | **refused** | as project terminal | the agent itself only | as project VM; Pi: plus the local models' port when Pi uses them | as project terminal |
| Agentic editor extensions (section 10) | 3 | through the forwarder when the extension allows it; otherwise disabled | disabled | — | — | — | — |

Invariants that follow from the table (normative; each has a test, section 12.3):

- **I1** No code path starts a process for levels 1–3 without going through `levels::decide()`. The kit's own programs (section 5, table "Kit") are the only exception and are listed by name.
- **I2** The level is a function of the origin (`Origin::level()`), never a parameter a caller chooses.
- **I3** An `Agent` placement is never `Pc`.
- **I4** A `Pc` placement never has a folder of an agent program on `PATH` (`levels::env::agent_dirs` of the PC's target), and has the forwarders' folder first.
- **I5** A `Vm` placement never carries a kit secret: not in arguments, not in `-e`, not in `--env-file` (section 11 defines "kit secret").
- **I6** Without the VM, nothing of level 3 can be installed, updated, launched or configured in the editor; the kit's agent data stays on disk untouched.
- **I7** A refusal always carries a reason and a fix (`Refusal`, `Fix`); nothing silently falls back from the VM to the PC.
- **I8** `decide()` is pure and total: same inputs, same output; every input combination has exactly one row of 3.4.

## 3. The code model: `tauri/src/levels/`

### 3.1 Files

| File | Content | Comes from |
|---|---|---|
| `levels/mod.rs` | `Level`, `Agent`, `AgentRunner`, `Origin`, `Place`, `Request`, `VmState`, `Facts`, `Placement`, `Sandbox`, `VmProgram`, `HostAccess`, `Refusal`, `Fix`; `decide()` | new |
| `levels/place.rs` | `place_of(kit, dir) -> Place` (which project a folder belongs to) | the logic of `runtime/vm.ps1` (`$projects`, `$below`) and `ipc::project_dir` |
| `levels/facts.rs` | `facts(kit, host, env, request) -> Facts` (the only impure read before `decide`) | `isolation::status`, `vm::guest_path`, `isolation::restart_pending`, catalog availability |
| `levels/env.rs` | `environment()`, `SEALED`, `seal()`, `agent_dirs()`, `strip_agents()`, `FORWARD_DIR`, `LEVEL_VAR` | `isolation::{SEALED, seal, agent_dirs, without_agents, agents_notice, with_agents_notice}`, `versions::{project_env, choice_env}` (their sealing part), `editor::terminals::env` (its agent part) |
| `levels/prepare.rs` | `prepare()`: tools for the VM's Linux, databases and their ports, Pi's local models, `Vm::start` | `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}`, `ipc::agents::{agent_vm_ready, pi_local_models}`, `isolation::vm_for` |
| `levels/start.rs` | `start_terminal()`, `action_command()`, `scaffold_command()`: from `Ready` to a pty `Launch` or a `Command` | the tails of `terminal_launch`, `terminal_open`, `agent_launch`, `Runner::start_in`, `frameworks::run_line` |
| `levels/requests.rs` | the file protocol for processes outside the app (editor, forwarders) | replaces `isolation/session.rs` |
| `levels/report.rs` | `VmReport`, `ToolCheck`, `report()`, `check_tools()` | new |
| `levels/labels.rs` | `label(level, placement)` texts, `banner_line()` | new |

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
    /// The place's folder as the VM sees it (`vm::guest_path`); `None` for a
    /// path the VM cannot mount (a network share).
    pub guest_dir: Option<String>,
    /// `isolation::restart_pending`: the kit daemon must restart before any VM opens.
    pub restart_pending: bool,
    /// For `AgentRunner::ProjectVm` agents: the catalog has a build for the VM's
    /// Linux target (`Component::is_available_on(guest)`), installed or installable.
    pub linux_build: bool,
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
    Line { command: String },                // editor task, scaffold step: `sh -lc`
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
/// read without following links (`safe_path::is_dir`); `Folder` otherwise.
pub fn place_of(kit: &Kit, dir: &Path) -> Place;

// levels/facts.rs
/// Reads the facts for `req`: `isolation::status(kit, host, env, false)`
/// (cached at most 60 s, `FRESH`), `vm::guest_path`, `isolation::restart_pending`,
/// and for a ProjectVm agent the catalog's availability on `vm::guest_target`.
pub fn facts(kit: &Kit, host: &Host, env: &KitEnv, req: &Request) -> Facts;
```

The status used is the one at the moment of the request: no level is cached in files any more (today's `.level` of `isolation::session`, written when the editor opened, goes away).

### 3.4 `decide`: the decision table

```rust
/// The one decision (D-179): pure, total, no I/O.
pub fn decide(req: &Request, facts: &Facts) -> Placement;
```

`Origin::level()`:

| Origin | Level |
|---|---|
| `KitTerminal`, `EditorTerminal` | 1 Manual |
| `ProjectTerminal`, `EditorTask`, `Action`, `Site`, `Pool`, `Scaffold` | 2 Project |
| `AgentButton`, `EditorAgentProfile`, `Forwarded`, `EditorExtension` | 3 Agent |

Rules, evaluated top to bottom; the first that matches gives the placement. "Project" in the Place column means `Place::Project`, "Folder" means `Place::Folder`, "any" means either.

| # | Level | Origin | Place | `vm` | Other facts | Placement |
|---|---|---|---|---|---|---|
| R1 | 3 | any | any | Off | — | `Refused { AgentsNeedVm { blockers }, OpenVmNotice }` |
| R2 | 2 | `Pool` | any | On | — | `Refused { PoolsOnlyWithoutVm, StartFromProject }` |
| R3 | 1 | any | Project | any | — | `Pc { Manual, Some(name), dir+below }` |
| R4 | 1 | any | Folder | any | — | `Pc { Manual, None, dir }` |
| R5 | 2 | any | Project | Off | — | `Pc { Project, Some(name), dir+below }` |
| R6 | 2 | any | Folder | Off | — | `Pc { Project, None, dir }` (sealed for no project: a pool, a task outside a project) |
| R7 | 2 | any | Folder | On | — | `Refused { NotAProject, OpenFromProject }` |
| R8 | 3 | any | Folder | On | — | `Refused { AgentOutsideProject, OpenFromProject }` |
| R9 | 2, 3 | any | Project | On | `restart_pending` | `Refused { DaemonRestartPending, RestartDaemon }` |
| R10 | 2, 3 | any | Project | On | `guest_dir == None` | `Refused { NoPathInVm { dir }, UseLocalDisk }` |
| R11 | 3 | agent with `runner() == ProjectVm` | Project | On | `!linux_build` | `Refused { NoLinuxBuild { agent }, None }` |
| R12 | 3 | agent with `runner() == SbxAgent` | Project | On | — | `Vm { Agent, sandbox: Agent{agent_name(project, id)}, program: SbxAgent, host: {databases: true, local_models: false}, publish: false }` |
| R13 | 3 | agent with `runner() == ProjectVm` | Project | On | — | `Vm { Agent, sandbox: Project{name(project)}, program: AgentBinary, host: {databases: true, local_models: agent == Pi}, publish: false }` |
| R14 | 2 | `ProjectTerminal` | Project | On | — | `Vm { Project, Project sandbox, Shell, publish: false }` |
| R15 | 2 | `EditorTask { command }`, `Scaffold` | Project | On | — | `Vm { Project, Project sandbox, Line { command }, publish: false }` |
| R16 | 2 | `Action { action, web }` | Project | On | — | `Vm { Project, Project sandbox, Action, publish: web }` |
| R17 | 2 | `Site { action }` | Project | On | — | `Vm { Project, Project sandbox, Site, publish: true }` |

Totality: level 1 is covered by R3/R4 for every place and VM state; level 2 by R2, R5–R7, R9, R10, R14–R17 (every level-2 origin appears in R14–R17 except `Pool`, which is refused by R2 when On and placed by R5/R6 when Off); level 3 by R1, R8–R13. In every `Vm` placement `mounts` is `{ workspace_rw: project dir, tools_ro: [kit.for_target(guest).apps()] }` and `workdir` is `guest_dir` joined with `below`. The test `every_request_has_exactly_one_placement` enumerates all origins × both places × both VM states × every fact combination and checks that exactly one rule matches (12.3).

Note on `Forwarded { from }`: `from` is kept for the log and the label only; it never changes the row. A forwarder only exists on the PC (level 1, or level 2 without the VM); level-2 sessions in the VM have no forwarders, so an agent cannot be started from inside a project VM (section 6.8).

### 3.5 `environment`: the one `PATH`/environment builder

```rust
// levels/env.rs

/// The variable every level-1/2 process on the PC gets: "manual" or "project";
/// in the VM: "project" or "agent". Read by profile.ps1 and bashrc (banner).
pub const LEVEL_VAR: &str = "ATOLLO_LEVEL";
/// The project of a PC process, when it has one (profile.ps1/bashrc no longer guess it from the folder).
pub const PROJECT_VAR: &str = "ATOLLO_PROJECT";
/// `data/kit/forward`: the forwarders, first on every PC `PATH` (replaces `data/kit/no-agents`).
pub const FORWARD_DIR: &str = "forward";

/// What a placement runs with.
pub enum Environment {
    /// The environment of the PC process.
    Pc(KitEnv),
    /// The environment of `sbx` on the PC (`Sbx::session_env`: the kit's
    /// environment, the PC's temp, `daemon::ENV`) and what goes into the VM.
    Vm { sbx_env: KitEnv, guest: GuestTools },
}

/// Builds the environment of `placement` from the kit's `base` (built for
/// the PC's target). `terminal`: the kit terminal's tool choice, for
/// `Origin::KitTerminal` only. `None` exactly for `Placement::Refused`.
pub fn environment(
    kit: &Kit,
    base: &KitEnv,
    placement: &Placement,
    terminal: Option<&crate::editor::terminals::Terminal>,
    lang: &str,
) -> Option<Environment>;
```

`Pc { level, project, .. }`, in this order (each step is a named function, tested on its own):

1. `base` cloned.
2. If `terminal` is a user terminal: its tool choice (`versions::choice_env` without its sealing) and the left-out tools stripped (`editor::terminals::strip`), as today.
3. If `project` is `Some`: the project's version swaps (`versions::project_swaps`), applied **after** the terminal's, so a tool the project chose is at the project's line in any terminal opened in its folder **[assumption, open question Q1]**; tools the project did not choose keep the terminal's lines.
4. `seal(kit, project, env)`: the `SEALED` variables out; the project's own MySQL login in when `project` is `Some` (D-167). Unchanged rules.
5. `strip_agents(kit, env)`: every `agent_dirs(kit)` entry leaves `PATH`, with the variables pointing into them (`CP_ANTIGRAVITY`); `agent_dirs` = `data/apps/<pc-target>/{claude,codex,antigravity,pi}` and `data/kit/agents`.
6. `data/kit/forward` first on `PATH`.
7. `LEVEL_VAR` = `manual` or `project`; `PROJECT_VAR` = the project when `Some`, removed otherwise; `ATOLLO_TERMINAL` = the kit terminal's name for `KitTerminal`.

`Vm { project, sandbox, program, .. }`:

1. `guest = kit.for_target(vm::guest_target(kit.target()))`; `base_guest = KitEnv::build(&guest, settings, lang, host)`.
2. `isolation::project_guest_tools(kit, &guest, project, base_guest)`: the guest environment at the project's lines, sealed, agents stripped (step 5 above applied to `guest`), then `guest_view`: only `PATH` entries and variables inside `data/apps/linux-…`, as the VM sees them. For `AgentBinary { agent }` the agent's own Linux folder is added back to `guest.path` (it is the program being run, not a command offered to others).
3. `-e ATOLLO_LEVEL=project|agent` added to `guest.vars`.
4. `sbx_env = Sbx::session_env(base)`.
5. `guest.env_file` is set by `prepare` (database variables, D-171), never by `environment`.

`Refused` → `None`.

The old functions become thin calls and then disappear (12.2): `versions::project_env(kit, p, env)` = `environment(Pc{Project, Some(p)})`; `versions::choice_env` = steps 2 and 4–7 without a project; `editor::terminals::env` = `environment(Pc{Manual, …}, Some(terminal))`.

### 3.6 `prepare` and `start`

```rust
// levels/prepare.rs

/// What `prepare` needs from the app, so that it can be tested without Tauri.
pub trait Effects {
    /// `ensure_guest_tools` today: installs for the VM's Linux what `wanted` lacks.
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>);
    /// Starts the project's database servers on the PC; their ports.
    fn start_databases(&self, project: &str, env: &KitEnv) -> anyhow::Result<Vec<(crate::db::auth::ProjectDb, u16)>>;
    /// Starts llama-server on the PC (Pi's local models); its port.
    fn start_local_models(&self, lang: &str) -> Option<u16>;
}

/// A placement made ready to start.
pub struct Ready {
    pub placement: Placement,     // never Refused
    pub env: Environment,
    pub vm: Option<crate::isolation::Vm>,   // started, ports synced, GitHub synced
}

/// For `Vm`: installs the tools (D-170), starts the databases and writes
/// `db.env` (D-171), adds Pi's local models (D-178), builds the `isolation::Vm`
/// (project's or agent's sandbox) and starts it (`Vm::start`). For `Pc`: nothing
/// but the `ensure` of what the action needs (today's `ensure(&app, …, &wanted)`).
pub fn prepare(
    fx: &dyn Effects,
    kit: &Kit,
    host: &Host,
    placement: Placement,
    env: Environment,
    lang: &str,
) -> anyhow::Result<Ready>;
```

```rust
// levels/start.rs

/// The pty session of a terminal-like placement (Shell, SbxAgent, AgentBinary,
/// or a PC shell). Its `PtyInfo` carries `level` and `label` (section 8.3).
pub fn start_terminal(kit: &Kit, ready: &Ready, title: &str, lang: &str) -> anyhow::Result<crate::editor::pty::Launch>;

/// What `Runner::start_in` runs and keeps for an action or a site.
pub struct ActionCommand {
    pub cmd: std::process::Command,
    /// The line shown in the action's log.
    pub shown: String,
    /// In the VM: what stops it there (`run::start::VmRun`, D-175).
    pub vm_run: Option<crate::projects::run::start::VmRun>,
    /// Lines for the log (an alias not in the VM, a Caddy site served by Apache).
    pub notes: Vec<String>,
    /// On the PC: the FastCGI server of a Caddy/Apache site, kept with the action.
    pub fastcgi: Option<(crate::projects::fastcgi::FastCgi, std::process::Child)>,
}

/// The command of an action or a site (PC: today's branches of
/// `Runner::start_in`; VM: `vm::exec_args`/`vm::web_line` through `Vm::command`,
/// the port published with `Vm::publish`). `port`: the action's, for web ones.
#[allow(clippy::too_many_arguments)]
pub fn action_command(
    kit: &Kit,
    ready: &Ready,
    project: &str,
    dir: &Path,
    cwd: &Path,
    action: &crate::detect::Action,
    port: Option<u16>,
    lan: bool,
    job: &crate::platform::jobs::Job,
    lang: &str,
) -> anyhow::Result<ActionCommand>;

/// One step of a scaffold: PC: `frameworks::run_line` with the Pc environment;
/// VM: `vm::exec_args` in the new project's sandbox.
pub fn scaffold_command(ready: &Ready, line: &str, at: &Path, vars: &[(String, String)]) -> anyhow::Result<std::process::Command>;
```

What stays where:

| Today | Fate |
|---|---|
| `isolation::vm_for` | private to `levels::prepare` (`Vm` is built only from a `Placement::Vm`) |
| `isolation::Vm::for_agent`, `agent_session_args`, `launch`, `start`, `publish`, `stop_run` | stay in `isolation` (sandbox lifecycle); called only from `levels::prepare`/`start` and `projects::run` (stop) |
| `isolation::vm::*` builders | stay (pure `sbx` argument builders, D-175 rule) |
| `isolation::seal`, `SEALED`, `agent_dirs`, `without_agents` | move to `levels::env` (`without_agents` renamed `strip_agents`, and no longer prepends stand-ins: step 6 does) |
| `isolation::agents_notice`, `with_agents_notice`, `AGENTS_NOTICE_VAR` | removed; the forwarders get the reason from the reply (section 4.3) |
| `isolation::session` (whole file) | replaced by `levels::requests` |
| `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}` | move to `levels::prepare` (`ensure_guest_tools` behind `Effects`) |
| `ipc::agents::{agent_vm_ready, refused_on_project, no_vm_for_agents, pi_local_models, watch_agent_requests}` | `agent_vm_ready`, `pi_local_models` → `levels::prepare`; `refused_on_project`, `no_vm_for_agents` → `Refusal` texts in `levels::labels`; `watch_agent_requests` → `levels::requests::serve` |
| `ipc::editing::{editor_vm_begin, editor_vm_session}` | removed (no per-editor VM session: each task/agent request is decided when it comes) |
| `ipc::servers::refused_with_vm` | removed (R2) |
| `vm::starts_in_vm`, `vm::sbx_agent` | replaced by `Agent::runner()` |
| `setup::kit_files::{NO_AGENTS_DIR, AGENT_COMMANDS, no_agent_cmd, no_agent_sh, AGENTS_NOTICE_FALLBACK}` | replaced by `FORWARD_DIR` and the forwarder files (6.8) |
| `runtime/vm.ps1` | replaced by `runtime/launch.ps1` and `runtime/launch.sh` (request clients only, 4.3) |

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
  | "open_vm_notice" | "open_from_project" | "use_local_disk"
  | "start_from_project" | "restart_daemon" | "none";

/** The error of a refused command: `CmdError` with `refusal` set. */
export interface RefusedError { message: string; refusal: Refusal; fix: Fix }
```

| Command | Request | Response | Status |
|---|---|---|---|
| `levels_state` | `{}` | `LevelsState` | **new** |
| `project_vm_report` | `{ project: string; check: boolean }` | `VmReport` (8.1) | **new** |
| `terminal_launch` | `{ id: string; dir?: string }` | `void` | kept; goes through `decide(KitTerminal)` |
| `terminal_open` | `{ project: string }` | `void` | kept; `decide(ProjectTerminal)` |
| `run_start` | `{ project: string; action: string }` | `RunInfo` | kept; `decide(Action|Site)`, pools after R5 |
| `server_start`, `server_reload` | `{ id: string }` | `void` | kept; `decide(Pool)` (R2 replaces `refused_with_vm`) |
| `frameworks_create` | as today | `string` | kept; `decide(Scaffold)` (VM when on: new) |
| `agent_launch` | `{ agent: AgentId; project: string }` | `void` | kept; `decide(AgentButton)` |
| `agents_status` | `{}` | `AgentInfo[]` with new fields `available: boolean`, `unavailableReason: string \| null`, `runsIn: "sbxSandbox" \| "projectVm"` | changed |
| `editor_open` | `{ project: string }` | `void` | kept; no longer prepares a VM nor writes `.level` |
| `pty_sessions`, event `pty-opened` | — | `PtyInfo` with new fields `level: Level`, `label: LevelLabel`, `sandbox: string \| null` | changed |
| `isolation_status` | `{ force: boolean }` | `IsolationStatus` | kept (field `level` renamed `availability` at step 10) |
| `agent_token_link` | — | — | **removed** (runs `claude setup-token` on the PC: level 3 on the PC, I3) |
| `agent_token_save` | `{ token }` | `void` | kept as kit data only; the token is never given to a VM (I5); hidden in the UI **[assumption]** |
| `agent_sign_out` | `{ agent }` | `void` | kept for the kit's data only; `codex logout` is no longer run (it starts the agent's binary on the PC); `auth.json` is removed directly |
| `components_apply`, `components_install`, `versions_install`, `updates_list`, `setup_start` | as today | as today | agents filtered out when `agentsEnabled == false` (section 7) |
| `vms_list`, `vm_remove`, `isolation_run` | as today | as today | unchanged |

Every refused command returns `CmdError` with the new optional fields `refusal` and `fix` (`ipc/error.rs`), so the UI can show the right button without parsing text.

### 4.2 Events

| Event | Payload | When |
|---|---|---|
| `levels-changed` | `LevelsState` | **new**: the VM went on or off (`levels::watch`, a thread that asks `isolation::status` every 60 s and on every forced check, and emits only on change) |
| `pty-opened`, `pty-exit`, `pty-moved` | `PtyInfo` (with `level`, `label`, `sandbox`) | changed payload |

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

The app (`levels::requests::serve`, polling every 250 ms, one thread per request) deletes the request, resolves `place_of(cwd)`, reads the facts, calls `decide`, `environment`, `prepare`, and writes `<id>.reply.json`:

```json
{
  "v": 1,
  "placement": "vm",
  "level": "agent",
  "label": "Agente · VM",
  "program": "D:\\kit\\data\\apps\\win32-x64\\sandboxes\\bin\\sbx.exe",
  "argv": ["exec", "-it", "-w", "/d/kit/projects/shop/web", "--env-file", "…", "atollo-p-shop-…", "sh", "-c", "…; exec \"$0\" \"$@\"", "/d/kit/data/apps/linux-x64/pi/pi", "--continue"],
  "env": { "set": { "LOCALAPPDATA": "…" }, "unset": [] },
  "banner": "[Atollo] Agente · VM: pi in atollo-p-shop-…"
}
```

or, refused:

```json
{ "v": 1, "placement": "refused", "level": "agent", "refusal": { "code": "agents_need_vm", "blockers": [ … ] }, "fix": "open_vm_notice", "text": "Gli agenti partono solo nella VM … (motivo, cosa fare)" }
```

or, for an `editorTask` with the VM off, `placement: "pc"` with `program` = the kit's shell and `env.set`/`env.unset` = the difference between the editor server's environment and `environment(Pc{Project})` (the project's swaps and its MySQL login; the editor server's own environment is already level-1, 6.2).

The client (`runtime/launch.ps1`, PowerShell 7, the required component; `runtime/launch.sh`, POSIX sh, for bash without PowerShell) writes the request, waits for the reply at most 10 minutes (the first VM start pulls its image), prints `banner` to stderr, applies `env`, and runs `program argv…` in the same terminal with stdio inherited, exiting with its code. For a refusal it prints `text` and exits 1. Without a reply within 15 s and no app process running (`data/app/instance` lock not held) it prints "Atollo non è aperto: apri il terminale da Atollo." and exits 1. The client never decides anything: its only logic is write, wait, run.

Who may write requests: any process of the user can, as today (`<project>.<agent>.request`). A request can only cause what the user could do from the app: start a session that `decide` allows. A process inside a VM cannot write one: `data/app` is never mounted (D-168).

### 4.4 CLI

| Command | Output | Exit |
|---|---|---|
| `atollo vm <project> [--check] [--json]` | the VM report (8.2) | 0 all good; 1 VM off, sandbox missing, or a tool check not `ok`; 2 usage |
| `atollo env --project <p>` | today's output, now `environment(Pc{Project, p})`, plus `ATOLLO_LEVEL=project` | unchanged |

The forwarders do not go through the CLI: `Atollo.exe` is a Windows GUI-subsystem program (`main.rs`, `windows_subsystem = "windows"`), and running an interactive `sbx exec -it` as its child from a console is not something the code does today **[unverified]**; the PowerShell/sh clients run `sbx` themselves, as `vm.ps1` does today (verified, D-175).

## 5. Entry-point inventory

Every place in today's code that starts a terminal, task, action, site, pool, scaffold or agent (grep of `open_console`, `open_launch`, `pty::shell|command|program`, `kit_command`, `spawn_in`, `Command::new`, `sbx exec|run`, `vm_for`, `seal`, `without_agents`, `project_env`, `choice_env`, and the editor's profiles), on 26/10/2026, branch `feature/levels` = `main` at `c9aa2561`.

### 5.1 Levels 1–3

| # | Entry point (file: function) | Starts | Level | Today | With `levels` |
|---|---|---|---|---|---|
| E1 | `ipc/terminals.rs: terminal_launch` → `editor/terminal.rs: open_console` | kit terminal (pty) | 1 | PC; `terminals::env` (choice, `without_agents`, stand-ins); `with_agents_notice`; **no project versions** (`ATOLLO_TERMINAL` makes profile.ps1/bashrc skip them) | `decide(KitTerminal)` → R3/R4 → `environment(Pc, terminal)` → `start_terminal` |
| E2 | `editor/cfg.rs: kit_profiles` ("Atollo", "Atollo - Git Bash", "Atollo - cmd", "Atollo - Bash", "Atollo - zsh") → `runtime/vm.ps1 -Shell` | editor terminal | 1 | VM only (D-178), fail closed; on the PC nothing | direct kit shell (`pwsh_args`/`bash_args`), no request; `decide(EditorTerminal)` is R3/R4 by construction; the editor server's environment is already level 1 (E13) |
| E3 | `ipc/editing.rs: terminal_open` | project terminal | 2 | `vm_for` → `project_vm_ready` → `kit_terminal_args`; else `open_console` with `project_env` | `decide(ProjectTerminal)` → R14 or R5 |
| E4 | `ipc/runs.rs: run_start` (VM branch) → `projects/run/start.rs: Runner::start_in` → `in_vm` (`exec_args`, `web_line`, `publish`) | action / site in VM | 2 | `vm_for` → `project_vm_ready` | `decide(Action|Site)` → R16/R17 → `prepare` → `action_command` |
| E5 | `ipc/runs.rs: run_start` (PC branch) → `Runner::start_in` (`Atollo.exe serve`, `projects/apache.rs: command`, `projects/caddy/command.rs`, `projects/fastcgi.rs: spawn_fastcgi_if_present`, plain command) | action / site on PC | 2 | `project_env` | R5 → `environment(Pc{Project})` |
| E6 | `ipc/runs.rs: run_start` (pool branch) → `servers.attach` → `servers/process.rs` (pool start, `without_agents`) | site in a pool | 2 | PC; with VM the VM branch wins first | `decide(Site)` first; R5 only then pool |
| E7 | `ipc/servers.rs: server_start`, `server_reload` (`refused_with_vm`) → `servers/process.rs` | pool | 2 | refused with VM | `decide(Pool)` → R2 or R6 |
| E8 | `app/mod.rs` setup: `servers.autostart` (skipped with VM) | pools at app start | 2 | same check inline | `decide(Pool)` per pool |
| E9 | `ipc/frameworks.rs: frameworks_create` → `projects/frameworks/runner.rs: run_line` (pwsh) | scaffold | 2 | **always PC**, `seal(None)` | `decide(Scaffold)` → R15 (new project's sandbox) or R5 |
| E10 | `ipc/agents.rs: agent_launch` → `agent_vm_ready` → `Vm::launch` → `open_launch` | agent from the app | 3 | VM only; Antigravity refused | `decide(AgentButton)` → R1/R8–R13 |
| E11 | `ipc/agents.rs: watch_agent_requests` ← `vm.ps1 -Agent` (`<project>.<agent>.request`) ← `editor/cfg.rs: kit_profiles` (Claude Code, Codex, Antigravity, Pi) | agent from the editor | 3 | VM only, needs an editor session of that project | `levels::requests::serve` → `decide(EditorAgentProfile)` |
| E12 | `editor/cfg.rs: merge` (`automationProfile`) → `vm.ps1 -Command` | editor task | 2 | VM only; without VM nothing | request `editorTask` → R15 or R5/R6 |
| E13 | `editor/mod.rs: Editor::start_on` (`kit_command` with `self.env()` = the full kit environment) | editor server and its extension host | kit, but hosts extensions | **agents on `PATH`** (`env/path.rs` adds `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi`) | server environment = `environment(Pc{Manual, None})` (no agents, forwarders first) |
| E14 | `tauri/catalog/extensions.json` (`anthropic.claude-code`, general) → `editor/ext.rs` install | Claude Code extension, its bundled CLI, on the PC | 3 | installed when `claude` is in the kit; runs on the PC | section 10 |
| E15 | `setup/kit_files.rs: no_agent_cmd/no_agent_sh` (`data/kit/no-agents`) | stand-ins | 3 (refusal only) | print a fixed text, exit 1 | forwarders (6.8) |
| E16 | `agents/claude_token.rs: link` (`pty::command` of `claude setup-token`, macOS) via `ipc/agents.rs: agent_token_link` | agent binary on the PC | 3 | PC | removed (I3) |
| E17 | `agents/mod.rs: codex_logout` (`kit_command(codex, …) logout`) via `agent_sign_out` | agent binary on the PC | 3 | PC | removed; `auth.json` deleted directly |
| E18 | `editor/cfg.rs: kit_profiles` and `merge` when the kit's PowerShell is missing | agent profiles run `claude.exe`/`codex.exe`/`agy`/`pi.exe` directly; tasks `/bin/bash` | 3 / 2 | **on the PC** | no agent profile and no task profile without the kit's PowerShell or `launch.sh` (I3) |
| E19 | the editor's debugger adapters and extensions that run project interpreters (`ms-python.python`, `xdebug.php-debug`…) | project code on the PC | 2 | PC (declared limit in D-175) | declared limit kept, open question Q14 |
| E20 | `main.rs: Cli::Env { project }` | prints an environment (no process) | 2 | `project_env` | `environment(Pc{Project})` |

Count: level 1: 2 (E1, E2); level 2: 12 (E3–E9, E12, E19, E20, plus the pool reload and autostart counted in E7/E8 as separate call sites: `server_start`, `server_reload`, `autostart`); level 3: 6 (E10, E11, E14, E15, E16, E17) plus the PC fallback E18 (levels 2 and 3); E13 is the kit process that hosts E14 and E19.

### 5.2 The kit's own programs (not levels; exempt from I1 by name)

| Program | Where started |
|---|---|
| database servers and clients | `db/services/{process,access,data,tools}.rs` |
| phpMyAdmin, mongo-express | `db/phpmyadmin.rs`, `db/mongo_express.rs` |
| llama-server | `llm/service.rs` |
| Git (clone, `ls-files`, identity), gh | `github/mod.rs`, `github/repos.rs`, `projects/mod.rs`, `projects/frameworks/mine.rs` (with `-c core.fsmonitor=false`, D-167) |
| installers | `install/**`, `setup/checkup.rs` |
| editor server, extension installs, theme | `editor/mod.rs`, `editor/ext.rs`, `editor/theme.rs` |
| `sbx` (status, login, ls, daemon) | `isolation/**`, `ipc/isolation.rs: isolation_run` |
| Antigravity keyring helpers | `os/macos/agy_keyring.rs`, `agents/keyring` (no agent run) |

The rule test (12.3, `no_process_outside_levels`) scans the crate for `open_console`, `open_launch`, `pty::shell`, `pty::command`, `pty::program`, `Runner::start_in`, `frameworks::run_line` and `Vm::command` and fails for any caller outside `levels/` and the list above.

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
L requests::serve: place_of(cwd) ─ facts ─ decide
   VM on,  Project ─▶ Vm{Project, Line{command}} (R15) ─ environment ─ prepare (tools, dbs, Vm::start)
                      reply {placement:vm, program:sbx, argv: exec_args(vm, below, command, [], tools)}
   VM off, Project ─▶ Pc{Project} (R5)         reply {placement:pc, program: kit shell, env diff}
   VM on,  Folder  ─▶ Refused{NotAProject}     reply {placement:refused, text}
C prints banner, runs program argv (stdio inherited), exits with its code
```

### 6.4 Action (level 2)

```
UI projects/explorer.tsx ──run_start{project,action}──▶ IPC ipc/runs.rs
  detect action a; origin = a.static_dir|apache_dir|caddy_dir ? Site{a} : Action{a, web: a.kind == Web}
  p = decide({origin, Project{project}}, facts)
   ├ Vm (R16/R17) ─ environment ─ prepare (tools, db.env, ports, Vm::start)
   │     ─ Runner::start_in(…, ready) ─ vm.command(exec_args | web_line) ; publish port
   ├ Pc (R5) ─ site? pool_of → servers.attach (pool) | own server ; else command
   │     ─ environment(Pc{Project}) ─ Runner::start_in(…)
   └ Refused (R9/R10) ─ CmdError{refusal, fix}
```

### 6.5 Site through a pool (level 2)

```
UI servers/page.tsx ──server_start{id}──▶ IPC ipc/servers.rs
  p = decide({Pool{id}, Folder{data/servers/<id>}}, facts)
   ├ VM on  ─▶ Refused{PoolsOnlyWithoutVm, StartFromProject}   (R2)
   └ VM off ─▶ Pc{Project, None} (R6) ─ environment ─ servers.start
```

### 6.6 Agent from the project page (level 3)

```
UI AgentButtons ──agent_launch{agent,project}──▶ IPC ipc/agents.rs
  p = decide({AgentButton{agent}, Project{project}}, facts)
   ├ VM off ─▶ Refused{AgentsNeedVm} (R1)            UI shows reason + "Apri l'avviso VM"
   ├ claude|codex ─▶ Vm{Agent, Sandbox::Agent, SbxAgent} (R12)
   │     prepare: Vm::for_agent, tools, db.env, ports, start; args = agent_session_args
   ├ pi|antigravity ─▶ Vm{Agent, Sandbox::Project, AgentBinary} (R13)
   │     prepare: ensure Linux build (D-173), tools, db.env (+ Pi local models), start
   │     args = pi_session_args / agy_session_args
   └ PTY open_launch(sbx, args) with PtyInfo{level:agent, label:"Agente · VM", sandbox}
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

- `<cmd>.ps1`: `& "$env:CP_DATA/kit/launch.ps1" -Origin forwarded -Agent <id> -From $env:ATOLLO_LEVEL -- @args` (PowerShell finds `.ps1` on `PATH` before `.cmd` **[unverified for pwsh 7 command precedence with both present; to test]**);
- `<cmd>.cmd`: `pwsh -NoLogo -NoProfile -File "%CP_DATA%\kit\launch.ps1" -Origin forwarded -Agent <id> -From %ATOLLO_LEVEL% -- %*` (cmd's `%*` keeps quotes as typed; arguments with `"`, `%` or `^` may be altered: declared limit, Q16);
- `<cmd>` (sh): `exec "$CP_DATA/kit/launch.sh" forwarded <id> "$ATOLLO_LEVEL" "$@"`.

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
ED extension anthropic.claude-code (only while agentsEnabled; section 10)
  setting claudeCode.claudeProcessWrapper = data/kit/forward/claude-extension(.cmd|sh)
  the extension runs: <wrapper> <bundled-binary-path> <its args…>
  wrapper drops the first argument, then as 6.8 with origin editorExtension
  → decide(EditorExtension) → R12 → sbx argv; the extension speaks to the agent over stdio
```

Feasibility is **unverified** (Q2): path arguments of the PC inside the VM, the extension's own local connection, a non-TTY `sbx run`. Until a spike proves it, the extension is **disabled** in Atollo's editor (section 10).

## 7. Agents disabled without the VM

`agentsEnabled = (isolation availability == Vm)`. One function decides it: `levels::agents_enabled(&VmState) -> bool`; everything below reads it.

| Place | With the VM | Without the VM |
|---|---|---|
| Tools page, group "Agenti" (`ui/src/components/table.tsx`) | Install/Update offered: Pi and Antigravity for the VM's Linux (D-173 `Scope::Tools`); Claude Code and Codex marked "fornito da Docker Sandboxes" (the `sbx` agent images bring them) **[assumption, Q11]** | Install, Update, line choice disabled; tooltip: "Gli agenti girano solo nella VM, che non è attiva: <motivo>" with "Apri l'avviso VM" |
| Runner plans (`setup::runner`, `components_apply`, `components_install`, `versions_install`, `setup_start`, the hourly/first-start installs) | agents planned only for the guest target | `Group::Agents` filtered out of every plan (one filter, `levels::installable(component, &VmState)`) |
| Updates (`updates_list`, update summary D-161) | as above | agents not listed |
| Agents page (`ui/src/agents/page.tsx`) | as today | a banner with the reason; launch, sign-in and token actions disabled; MCP and instructions editing kept (kit data) |
| `AgentButtons` (project page, side bar row, Home card) and palette entries (`ui/src/shell/commands.tsx`) | shown | shown disabled with the reason as title (not hidden: the user sees why) |
| `startsOnProject` (`ui/src/ipc/agents.ts`) | removed: Antigravity starts on a project (section 9); availability comes from `AgentInfo.available` | — |
| IPC `agent_launch` | R12/R13 | R1 `Refused{AgentsNeedVm}` |
| Editor settings (`editor/cfg.rs: kit_profiles`) | agent profiles written | agent profiles **absent** (`kit_profiles(kit, system_root, agents_enabled)`); a `defaultProfile` pointing to one falls back to "Atollo" |
| Editor extensions with `"level": "agent"` in `extensions.json` (section 10) | per section 10 | not installed; installed ones disabled |
| Forwarders | forward (6.8) | print the reason (R1) and exit 1 |
| Kit data (`data/home/.claude`, `.codex`, Pi's dir, Antigravity's keyring, `data/apps/*/claude…`) | kept | kept, untouched |

Re-enabling: `levels::watch` emits `levels-changed`; on it the app rewrites the editor settings (`cfg::write`), runs the extension sync, and the UI reloads `agents_status` and the Tools page. VSCodium applies a changed Machine `settings.json` without a restart **[unverified]**; if it does not, the editor's status bar notice says "Riavvia l'editor per gli agenti".

## 8. Trust features

### 8.1 The VM report (data model)

```rust
// levels/report.rs
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct VmReport {
    pub project: String,
    pub vm: VmOnOff,                     // "on" | "off"
    pub reason: Option<String>,          // why off, the notice's text
    pub sandbox: Option<SandboxState>,   // the project's
    pub agents: Vec<SandboxState>,       // atollo-a-claude-…, atollo-a-codex-…
    pub mounts: Vec<MountInfo>,          // from kit::mounts (.mounts) and sbx ls --json
    pub host_ports: Vec<HostPortInfo>,   // from kit::host_ports (.ports): which PC ports the VM reaches
    pub published: Vec<PublishedPort>,   // the runner's VM runs with a port (RunInfo), address 127.0.0.1 or 0.0.0.0
    pub github: bool,                    // project setting github_in_vm (D-172)
    pub tools: Option<Vec<ToolCheck>>,   // Some only after "Verifica strumenti"
}
pub struct SandboxState { pub name: String, pub state: String /* sbx ls status, or "absent" */, pub last_used: Option<String> }
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

The project page gets a panel «VM del progetto» (`ui/src/projects/vm.tsx`, new): state, sandbox name, mounts with rw/ro, PC ports reached and published ports, GitHub in the VM, agent sandboxes; button «Verifica strumenti» (`project_vm_report{check:true}`); without the VM it shows the reason and "Apri l'avviso VM".

### 8.2 «Verifica strumenti»: what really runs

Only with the sandbox running (else `verdict: missing` for all and the text "Avvia il terminale o un'azione del progetto"; the check never starts a sandbox by itself **[assumption]**). One `sbx exec` per sandbox, with the session's tools (`tools_options`: `ATOLLO_PATH`, variables), so the check sees what a session sees:

```
sbx exec -e ATOLLO_PATH=<…> [-e K=V…] <vm> sh -c '
  PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH"
  for each tool:  printf "@@%s\n" <id>; command -v <prog> || true; <prog> <args> 2>&1 | head -n 3
'
```

`<prog> <args>` come from the catalog's `check` (`tauri/catalog/components.json`), with the path translated for Linux (`setup::catalog::translate_rel`) and reduced to its file name (so `PATH` resolution is what is checked); `notFromKit` when `command -v` is not under the mounted `data/apps/linux-…`:

| Tool | Catalog `check` | Command in the VM |
|---|---|---|
| git | `git/cmd/git.exe --version` | `git --version` |
| python | `python/python.exe --version` | `python3 --version` |
| node | `node/node.exe --version` | `node --version` |
| uv | `uv/uv.exe --version` | `uv --version` |
| php | `php/php.exe --version` | `php --version` |
| apache | `apache/bin/httpd.exe -v` | not in VM (`notInVm`: sites use `httpd:2.4`/`php:<line>-apache` images) |
| caddy | `caddy/caddy.exe version` | not in VM (`notInVm`: a Caddy site becomes Apache, D-175) |
| rust | `rust/cargo/bin/rustc.exe --version` | not in VM (`notInVm`: finishes on its own system, `router::bootstrap`) |
| go | `go/bin/go.exe version` | `go version` |
| java | `java/bin/java.exe -version` | `java -version` (stderr captured) |
| kotlin | `kotlin/bin/kotlinc.bat -version` | `kotlinc -version` |
| android | `android/platform-tools/adb.exe version` | not in VM (`notInVm`, `router::bootstrap`) |
| dotnet | `dotnet/dotnet.exe --version` | `dotnet --version` |
| dart | `dart/bin/dart.exe --version` | `dart --version` |
| zig | `zig/zig.exe version` | `zig version` |
| julia | `julia/bin/julia.exe --version` | `julia --version` |
| bun | `bun/bun.exe --version` | `bun --version` |
| deno | `deno/deno.exe --version` | `deno --version` |
| typst | `typst/typst.exe --version` | `typst --version` |
| tinytex | `tinytex/bin/windows/pdftex.exe --version` | `pdftex --version` |
| pnpm | `@data/npm-global/pnpm.cmd --version` | not in VM (`notInVm`: `router::bootstrap`, D-168) |
| cmake | `cmake/bin/cmake.exe --version` | `cmake --version` |
| cc | `cc/bin/clang.exe --version` | `clang --version` |
| ruby | `ruby/bin/ruby.exe --version` | `ruby --version` |
| r | `r/bin/Rscript.cmd --version` | not in VM (`notInVm`: micromamba, D-168) |
| mysql | `mysql/bin/mysql.exe --version` | `mysql --version` (the client; the server stays on the PC) |
| mongodb | `mongodb/bin/mongod.exe --version` | `mongod --version` **[assumption: only the client tools matter in the VM; Q17]** |
| sqlite | `sqlite/sqlite3.exe --version` | `sqlite3 --version` |
| llama | `llama/llama-server.exe --version` | not in VM (`notInVm`: the server runs on the PC, D-178) |
| pi | `pi/pi.exe --version` | `pi --version` (with Pi's folder on `PATH` for this check only) |
| antigravity | `antigravity/agy.exe --version` | `agy --version` (idem) |
| claude | `claude/claude.exe --version` | in `atollo-a-claude-…`: `sbx exec <it> claude --version` (the image's) |
| codex | `codex/bin/codex.exe --version` | in `atollo-a-codex-…`: `sbx exec <it> codex --version` |
| gh, pwsh, sandboxes, vscodium-server, conpty, phpmyadmin, mongo-express | — | not checked (kit on the PC) |

Comparison: `chosen` = `versions::project_choice(project)[id]`, else `versions::kit_choice(kit)[id]` (the line the VM's `PATH` resolves when the project chose nothing). `found` = the first match of `[0-9]+(\.[0-9]+)+` in the output (`go version go1.23.2` → `1.23.2`, `openjdk version "21.0.5"` → `21.0.5`). Verdict: `ok` when `found == chosen` or `found` starts with `chosen + "."`, and the path is under the mounted tools; `otherLine` when found but another line; `notFromKit` when the path is outside the mount; `missing` when `command -v` finds nothing; unversioned tools are `ok` when found from the kit.

### 8.3 Labels and banners

| Level / placement | Label (it) | Label (en) | Where |
|---|---|---|---|
| 1, `Pc` | «PC» | "PC" | terminal tab and detached window title (`PtyInfo.label`, `ui/src/terminal/panel.tsx`, `window.tsx`); banner of profile.ps1/bashrc |
| 2, `Pc` | «PC · senza VM» | "PC · no VM" | same, plus the status bar's "No VM" (unchanged) |
| 2, `Vm` | «VM progetto» | "Project VM" | tab; `launch` banner; action log first line |
| 3, `Vm` | «Agente · VM» | "Agent · VM" | tab; `launch` banner |

Banner text (stderr, one line, before the program):

- profile.ps1 / bashrc, from `ATOLLO_LEVEL`: `PC · <terminal or "editor"> · <project or "nessun progetto"> · agenti: inoltrati alla VM del progetto se attiva`. The shell does not know the VM's state at every moment, so the banner does not say whether it is on; the forwarder says it when used **[assumption]**.
- VM sessions (`start_terminal`, `launch` reply `banner`): `[Atollo] VM progetto: <sandbox> · <cartella nella VM>` and `[Atollo] Agente · VM: <agent> in <sandbox>`. Printed by the PC side before `sbx` starts, so it does not depend on the VM's shell; a prompt prefix inside the VM is not set (the image's `/etc/profile` may override `PS1`) **[unverified]**.

## 9. Antigravity

What changes from D-178: Antigravity's `agy` is a terminal CLI like `claude` and `codex` (the user, 26/10/2026: «la cli antigravity non ha il medesimo utilizzo della cli codex oppure claude code?»), not a desktop editor. It becomes a level-3 agent with `AgentRunner::ProjectVm`, like Pi.

- **Linux build.** `install/agents.rs: Antigravity::latest` reads `…/manifests/{go_os}_{go_arch}.json`; for `linux_amd64`/`linux_arm64` the `url` is a `.tar.gz` with one binary `antigravity`, installed as `data/apps/linux-x64/antigravity/agy` (`antigravity_download`, verified by the existing tests in `test/rust/install/agents.rs`). Installed for the guest only (D-173 `Scope::Tools`), never for the PC any more (I6, Q11).
- **Session.** `vm::agy_session_args(vm, dir, agy, tools)` = `pi_session_args` with `agy` (`sbx exec -it -w <dir> <tools> <vm> sh -c '<TOOLS_FIRST>; exec "$0" "$@"' <agy> <args>`). Variables: `AGY_CLI_DISABLE_AUTO_UPDATE=true` via `-e`; no `CP_ANTIGRAVITY`, no keyring (`Atollo.exe keyring restore/save` are PC-only and are not run).
- **Sign-in.** Two ways, neither puts a kit secret in the VM:
  1. **API key through the `sbx` proxy (default) [assumption]**: the `google` service of `sbx secret` covers `GEMINI_API_KEY`/`GOOGLE_API_KEY` for `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com` (Docker docs, section 11). Atollo registers it per sandbox with a command, as D-172 does for GitHub: `sbx secret set google --sandbox <atollo-p-…> --command '"<Atollo.exe>" secret gemini <nonce>'` (new internal command `secret gemini`, same guards: `asked_by_sbx`, nonce in `secrets.json`). The VM sees a placeholder. Whether `agy` sends the key in a header the proxy replaces, and to which host, is **[unverified]** (Q6).
  2. **Google account inside the VM**: `agy`'s own login run in the VM; `browser.enabled=false` (D-175) makes it print the link; the OAuth redirect to a loopback port inside the VM may not reach the PC's browser **[unverified]** (Q6). The login stays in the sandbox's disk (the VM's home), lost if the sandbox is made again.
- **The kit's Antigravity data** (keyring blob, `data/home` settings, the Gemini key in settings) stays on disk; it is never mounted or passed.
- **Removed:** `vm.ps1`'s and `ipc::agents::refused_on_project`'s refusal of Antigravity; `ui/src/ipc/agents.ts: startsOnProject`; the agents page's "no Antigravity button" rule; the editor profile "Antigravity" becomes a level-3 profile like the others.

## 10. Editor extensions that run an agent

Rule: an extension that runs an agent (a model that decides commands and edits files) is level 3. In Atollo's editor it is allowed only if it can be pointed at the forwarder, so that the agent runs in the VM; otherwise it is disabled with a message. Extensions run in the editor server's extension host, on the PC (E13).

What was found (26/10/2026):

| Extension (Open VSX id) | Runs | Setting to point it elsewhere | Source | Verdict |
|---|---|---|---|---|
| Claude Code (`anthropic.claude-code`) — in Atollo's catalog today (`extensions.json`, general, `programs: ["claude"]`) | its **bundled** copy of the CLI ("The extension bundles its own copy of the CLI … for the chat panel") | `claudeCode.claudeProcessWrapper`: "Executable used to launch the Claude process. The bundled binary path is passed as an argument when present." Also `claudeCode.environmentVariables`, `claudeCode.useTerminal` | https://code.claude.com/docs/en/vs-code (read 26/10/2026) | wrapper exists → candidate for the forwarder (6.9). **[unverified]**: that the panel works when the process is `sbx run …` in another file system (PC paths in arguments, its local IDE connection on `127.0.0.1` of the PC, no TTY). Until a spike passes: **disabled** |
| Codex (`openai.chatgpt`) | its bundled Codex CLI | `chatgpt.cliExecutable`: "Development only: Set the path to the Codex CLI executable." The page warns "manually overriding the bundled executable can prevent parts of the extension from working." Also `chatgpt.runCodexInWindowsSubsystemForLinux` (WSL only) | https://learn.chatgpt.com/docs/developer-settings?surface=ide (redirect of https://developers.openai.com/codex/ide/settings, read 26/10/2026) | documented as development-only → **disabled** |
| Gemini Code Assist (`google.geminicodeassist`) | agent mode in the extension host **[unverified]** | none found | not verified | **disabled** **[assumption]** |
| Gemini CLI companion (`google.gemini-cli-vscode-ide-companion`) | nothing by itself: connects to a `gemini` CLI in a terminal | — | not verified | allowed (the kit has no `gemini` CLI; harmless) **[assumption]** |
| Cline (`saoudrizwan.claude-dev`), Roo Code (`rooveterinaryinc.roo-cline`), Continue (`continue.continue`), Kilo Code (`kilocode.kilo-code`) | the agent loop inside the extension host on the PC, calling cloud APIs; runs commands in the editor's terminal (level 1, PC) | none (they are the agent) | not verified | **disabled** |

Mechanism:

- `tauri/catalog/extensions.json` gets `"level": "agent"` for the known agentic extensions (above), and Claude Code's entry changes from "general, `programs: ["claude"]`" to `"level": "agent"`. `editor/ext.rs` never installs a `level: agent` extension, and at every editor start disables those found installed. The disabling uses the server's `--disable-extension <id>` start option **[unverified for the VSCodium web server; Q10]**; failing that, they are uninstalled at start (`--uninstall-extension`), and the user's choice is not kept.
- The editor shows an informational message once per extension: "<name> esegue un agente: in Atollo gli agenti girano solo nella VM. Usa il profilo «<agente>» del terminale o la pagina del progetto."
- An extension not in the list cannot be recognised in general: declared limit (Q15).

## 11. Secrets: none from the kit enters a VM

Definition: a **kit secret** is a credential the kit holds for the user or for itself: the Claude OAuth token (D-115), Codex's and Pi's sign-ins in the kit's home, the Gemini API key of the settings, Antigravity's keyring, the GitHub token of the kit's `gh`, the root logins of the kit's database servers, the llama-server key, the editor's connection token. **Not** kit secrets: the project's own database user and password (D-171: they belong to the project and are in its `.env`).

How each agent signs in, with the VM:

| Agent | Sign-in | Key through the `sbx` proxy | Source |
|---|---|---|---|
| Claude Code | inside its `sbx` sandbox (`/login`); "the flow runs on the host, so the token is never exposed inside the sandbox" | `anthropic` (`ANTHROPIC_API_KEY`; `api.anthropic.com`, `console.anthropic.com`, `claude.ai`, `mcp-proxy.anthropic.com`) | https://docs.docker.com/ai/sandboxes/security/credentials/ |
| Codex | `sbx run codex` prompts on the host; `sbx secret set openai --oauth` ahead of time | `openai` (`OPENAI_API_KEY`; `api.openai.com`, `openai.com`, `chatgpt.com`) | same |
| Antigravity | section 9 | `google` (`GEMINI_API_KEY`, `GOOGLE_API_KEY`; `generativelanguage.googleapis.com`, `oauth2.googleapis.com`, `aiplatform.googleapis.com`, `vertexai.googleapis.com`) | same |
| Pi, cloud providers | its own `/login` in the VM (its auth file in the VM's home), or a key through the proxy | `anthropic`, `openai`, `google`, `openrouter`, `groq`, `mistral`, `xai` cover Pi's common providers **[assumption: Pi reads these variables; Q7]** | same |
| Pi, local models (D-178) | — | none: the llama-server key is not a `sbx` service. Today it goes into `pi.env` in the VM (**contradiction**, 13). Proposal: a key for that sandbox only (llama-server accepting several keys through `--api-key-file` **[unverified]**), revoked when the last Pi session ends; or `sbx secret set-custom --sandbox` with a placeholder replaced in the `Authorization` header ("Replacement applies only to HTTP request headers") toward `host.docker.internal` **[unverified that custom secrets apply to that host]** | same; Q7 |

Facts from the Docker page (read 26/10/2026): service secrets are global by default and "Sandbox-scoped secrets take precedence over global secrets"; "Adding, updating, or removing a service secret takes effect in existing local sandboxes without a restart"; custom secrets are experimental. Atollo registers only sandbox-scoped secrets (D-172 rule). A key registered for the project's sandbox is usable (not readable) by the project's own code in that VM, as GitHub in D-172 and the local models in D-178: declared.

## 12. Migration

### 12.1 Removed or renamed

| Symbol (file) | Fate |
|---|---|
| `isolation::Level` (`isolation/mod.rs`) | renamed `Availability { Vm, NoVm }` |
| `isolation::{SEALED, seal, agent_dirs, without_agents}` | moved to `levels::env` (`strip_agents`) |
| `isolation::{agents_notice, with_agents_notice}`, `setup::kit_files::{AGENTS_NOTICE_VAR, AGENTS_NOTICE_FALLBACK, NO_AGENTS_DIR, AGENT_COMMANDS, no_agent_cmd, no_agent_sh}` | removed; `FORWARD_DIR` and forwarder generators instead; `data/kit/no-agents` added to `OBSOLETE` (removed at start) |
| `isolation::vm_for` | private in `levels::prepare` |
| `isolation/session.rs` (all), `test/rust/isolation/session.rs` | replaced by `levels/requests.rs`, `test/rust/levels/requests.rs` |
| `isolation::vm::{starts_in_vm, sbx_agent}` | `Agent::runner()` |
| `ipc::terminals::{project_vm_ready, project_vm_tools, project_vm_dbs, ensure_guest_tools}` | `levels::prepare` |
| `ipc::agents::{agent_vm_ready, refused_on_project, no_vm_for_agents, pi_local_models, watch_agent_requests}` | `levels::{prepare, labels, requests}` |
| `ipc::editing::{editor_vm_begin, editor_vm_session}` | removed |
| `ipc::servers::refused_with_vm` | removed (R2) |
| `ipc::agents::agent_token_link`, `agents::claude_token::link` | removed |
| `agents::codex_logout` | removed |
| `agents::launch_env` | removed (no caller today) |
| `runtime/vm.ps1` | replaced by `runtime/launch.ps1`, `runtime/launch.sh` |
| `runtime/agents/{claude, agy, agy.cmd}` (`data/kit/agents`) | removed: they inject kit secrets into agents on the PC; no agent runs on the PC |
| `env/path.rs` entries `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi` | removed from the kit's base `PATH` |
| `env/mod.rs: agent_vars` (`CP_ANTIGRAVITY`, `CLAUDE_CONFIG_DIR`, …) | kept only for the kit's own reading of agent data (stats, sync); never reaches a process of levels 1–3 (stripped by `strip_agents`) |
| `editor/cfg.rs: left_by_the_kit` and the keeping of user profiles | removed: only Atollo's profiles (D-179); a file with comments/invalid JSON is moved aside to `settings.json.bak-<date>` and rewritten **[assumption]** |
| `profile.ps1`/`bashrc` `ATOLLO_TERMINAL` skip of project versions | removed: `environment` applies them (3.5) |
| `ui/src/ipc/agents.ts: startsOnProject` | removed |

### 12.2 Order (each step compiles and passes `cargo test` and `npm test`)

1. **Types and decide.** `levels/mod.rs`, `place.rs`, `facts.rs` (reading only), `labels.rs`; tests R1–R17 and totality. No caller changes.
2. **Environment.** `levels/env.rs`: move `seal`, `SEALED`, `agent_dirs`, `strip_agents`; `environment()` for `Pc` and `Vm`; `versions::{project_env, choice_env}` and `editor::terminals::env` become calls to it (behaviour unchanged except: forwarders dir instead of stand-ins, project versions in kit terminals, `ATOLLO_LEVEL`). Forwarder files written (refusing only, until step 5). Editor server environment = `environment(Pc{Manual})`.
3. **PC entry points.** E1, E3 (PC branch), E5, E6, E7, E8, E9 (PC branch), E20 through `decide` + `environment`; `PtyInfo.level/label/sandbox`; `CmdError.refusal/fix`.
4. **Prepare and VM entry points.** `levels/prepare.rs` with `Effects`; E3, E4, E10 through it; E9 in the VM (new: scaffold in the new project's sandbox); rule test `no_process_outside_levels` on.
5. **Requests.** `levels/requests.rs`, `runtime/launch.ps1`, `runtime/launch.sh`; editor profiles: terminals direct (E2 level 1), tasks and agents through `launch` (E11, E12); forwarders forward (6.8); remove `session.rs`, `vm.ps1`, `editor_vm_*`; editor settings: only Atollo's profiles.
6. **Agents disabled without the VM.** `levels::agents_enabled`, `levels::installable`, runner filter, `AgentInfo.available`, `levels-changed` and `levels::watch`, editor profile gating, UI (Tools, Agents page, buttons, palette). Remove `agent_token_link`, `codex_logout`, `runtime/agents/*`, agent entries of `env/path.rs` (E16, E17, E18).
7. **Antigravity in the VM.** `vm::agy_session_args`, `Agent::Antigravity.runner() = ProjectVm`, `secret gemini`, UI.
8. **Trust.** `levels/report.rs`, `project_vm_report`, `atollo vm`, the project panel, labels and banners.
9. **Extensions.** `extensions.json` `level: agent`, disable at start, message; spike for `claudeProcessWrapper` (Q2) and, if it passes, `forward/claude-extension`.
10. **Docs.** ARCHITECTURE.md (Isolation section → points here), the Guide, DECISIONS D-179 "Prove", `isolation::Level` serialized rename in the UI.

### 12.3 Test plan

Unit (Rust, `test/rust/levels/`):

| File | Tests |
|---|---|
| `decide.rs` | `the_level_comes_from_the_origin`; `every_request_has_exactly_one_placement` (all origins × {Project, Folder} × {On, Off} × `restart_pending` × `guest_dir` × `linux_build`); one test per row: `r1_agents_need_the_vm`, `r2_pools_only_without_the_vm`, `r3_r4_manual_is_always_on_the_pc`, `r5_r6_project_code_on_the_pc_without_the_vm`, `r7_project_code_needs_a_project_with_the_vm`, `r8_an_agent_needs_a_project`, `r9_a_pending_restart_starts_nothing`, `r10_a_network_share_never_gets_a_sandbox`, `r11_no_linux_build_no_agent`, `r12_claude_and_codex_get_their_own_sandbox`, `r13_pi_and_antigravity_run_in_the_project_vm`, `r14_r17_project_code_in_the_project_vm`; invariants `an_agent_is_never_on_the_pc` (I3), `a_refusal_always_has_a_reason_and_a_fix` (I7) |
| `env.rs` | on each of `win32-x64`, `darwin-arm64`, `linux-x64`, `linux-arm64`: `a_pc_environment_has_no_agent_folder` (I4), `the_forwarders_come_first`, `the_project_versions_win_in_a_project_folder`, `a_user_terminal_keeps_its_other_lines`, `sealed_variables_never_reach_levels_1_and_2`, `a_vm_environment_has_only_the_kits_linux_tools`, `no_kit_secret_reaches_a_vm_environment` (I5: Claude token, Gemini key, llama key, root `mysql.cnf`, `GH_TOKEN`, editor token), `pi_and_agy_get_their_own_folder_back_only_for_themselves`, `refused_has_no_environment` |
| `place.rs` | `the_project_of_a_subfolder`, `a_link_is_never_a_project`, `projects_itself_is_a_folder`, `case_rules_follow_the_file_system` |
| `requests.rs` | `a_request_round_trips`, `a_refused_reply_carries_reason_fix_and_text`, `a_pc_reply_carries_only_the_environment_difference`, `a_request_is_taken_once`, `args_are_appended_after_the_agent`, `a_malformed_request_is_ignored_and_removed` |
| `report.rs` | `the_check_command_of_every_catalog_tool` (the table of 8.2 from `components.json`), `versions_are_compared_by_line`, `a_tool_outside_the_mount_is_not_from_the_kit`, `java_version_on_stderr_is_read` |
| `prepare.rs` | with a fake `Effects`: `a_project_vm_gets_tools_databases_and_ports`, `pi_gets_the_local_models_port`, `an_agent_sandbox_mirrors_the_projects`, `nothing_is_prepared_for_the_pc_but_ensure` |
| `rule.rs` | `no_process_outside_levels` (source scan, I1), `no_agent_binary_is_started_by_the_app_on_the_pc` (no `kit_command`/`pty::command` with an `agents::exe_of` path) |

Existing tests that change: `test/rust/isolation/mod.rs` (`the_pcs_shells_have_no_agent_on_any_system` moves to `levels/env.rs`), `test/rust/isolation/session.rs` (removed), `test/rust/isolation/vm.rs` (`agy_session_args`), `test/rust/ipc/agents.rs` (`antigravity_never_starts_on_a_project` replaced by `r13_…`; `no_agent_starts_without_the_vm` stays as an integration of R1), `test/rust/setup/kit_files.rs` (`a_stand_in_says_where_the_agents_run` → `a_forwarder_asks_the_app`), `test/rust/editor/cfg.rs` (`only_atollos_profiles`, `no_agent_profile_without_the_vm`, `terminals_are_level_one`, `tasks_and_agents_go_through_launch`), `test/rust/editor/terminals.rs` (project versions), `test/rust/setup/runner.rs` (`agents_are_never_planned_without_the_vm`, `agents_only_for_the_vms_linux`), `test/rust/editor/ext.rs` (`an_agent_extension_is_never_installed`).

Integration per entry point (Rust with fakes where possible; the rest by hand on Windows 11 + sbx, and on one Linux, recorded in DECISIONS "Prove"): E1 kit terminal in and out of a project (versions, forwarders, banner); E2 editor terminal on the PC; E3 project terminal VM/PC; E4/E5 an action and a site VM/PC; E6–E8 pools refused/started; E9 scaffold in the VM (new); E10 each agent from the page, VM on/off; E11 each agent profile; E12 a task in a subfolder, VM on/off, outside a project; 6.8 `claude --version` typed in a kit terminal in a project, outside, and without VM; E14 Claude Code extension disabled; `atollo vm shop --check`.

UI (`test/ui/`): `terminal/panel.test.tsx` (label per level, sandbox name), `agents/page.test.tsx` (disabled with the reason, re-enabled on `levels-changed`, Antigravity button present), `projects/vm.test.tsx` (panel, mounts rw/ro, ports, «Verifica strumenti» verdicts), `components/table.test.tsx` (agents' install disabled without VM), `shell/commands.test.tsx` (palette entries disabled with the reason), `servers/page.test.tsx` (pool refusal text from `refusal`).

## 13. Today's code that contradicts this document

1. **Kit terminals ignore the project's versions** in a project folder: `editor/terminals.rs: env` sets `ATOLLO_TERMINAL` for every kit terminal, and `profile.ps1`/`bashrc` skip the project's swaps when it is set.
2. **Editor terminals are VM-only** (D-178, `vm.ps1` fail-closed); D-179 makes them level 1 on the PC.
3. **Project creation always runs on the PC**: `ipc/frameworks.rs: frameworks_create` uses `seal(&kit, None, …)` and `frameworks::run_line` even with the VM on; ARCHITECTURE.md already says it runs in the VM.
4. **The editor server, hence every extension, has the agents on `PATH`**: `env/path.rs` puts `data/kit/agents`, `claude`, `codex/bin`, `codex/codex-path`, `pi` on the kit's `PATH`, and `editor/mod.rs: Editor::start_on` starts the server with the full kit environment.
5. **The kit installs the Claude Code extension** (`extensions.json`, general, `programs: ["claude"]`), which runs its bundled CLI on the PC (level 3 on the PC).
6. **Agent binaries started on the PC by the app**: `agents/claude_token.rs: link` (`claude setup-token`, macOS) and `agents/mod.rs: codex_logout`.
7. **Without the kit's PowerShell** the editor's agent profiles run `claude.exe`/`codex.exe`/`agy`/`pi.exe` directly on the PC and the task profile is `/bin/bash` on the PC (`editor/cfg.rs: kit_profiles`, `merge`).
8. **A kit secret enters a VM**: Pi's local-models key goes into `pi.env` (`ipc/agents.rs: pi_local_models`), passed with `--env-file` (D-178).
9. **Antigravity is refused on projects** (`vm.ps1`, `ipc/agents.rs: refused_on_project`, `ui/src/ipc/agents.ts: startsOnProject`).
10. **User terminal profiles are kept** and an unparsable settings file is left alone (`editor/cfg.rs: merge`, `write`): D-179 wants only Atollo's profiles, rewritten at every start.
11. **Agents are installed for the PC** (and Claude Code/Codex also for the VM's Linux, `terminals::chosen_tools` includes the `agents` group) although no agent runs on the PC and `sbx`'s agent sandboxes bring their own Claude Code and Codex.
12. **The agents page shows the PC-side sign-in** (`agents::status` reads the kit's home), not the one the VM's agents use.
13. **The editor's agent requests need an editor session of that project** (`session::take_requests` checks the session folder): a forwarder could not use them as they are.
14. **The editor's level is noted once, when the editor opens** (`.level`): a VM that goes on or off later is not seen by the editor's terminals.
15. **`agents::launch_env` has no caller** and the wrappers `runtime/agents/{claude, agy, agy.cmd}` inject kit secrets (Claude token, Gemini key) into agents on the PC.
16. **Debugger and language extensions run project code on the PC** with the VM on (declared limit of D-175; violates I1 in spirit).

## 14. Open questions and unverified points

| # | Question | Default in this document |
|---|---|---|
| Q1 | A user terminal with explicit lines opened in a project folder: project's lines or terminal's? | project's lines for the tools the project chose, the terminal's for the others **decided by the user, 26/10/2026** |
| Q2 | Can the Claude Code extension run through `claudeCode.claudeProcessWrapper` → forwarder → `sbx run` (PC paths in its arguments, its local connection to the editor, no TTY)? | **decided by the user, 26/10/2026**: disabled now; a separate spike tries the wrapper and, if it holds, re-enables it through the VM |
| Q3 | Codex extension: `chatgpt.cliExecutable` is "Development only" | disabled |
| Q4 | Does `sbx run --name <sandbox>` pass extra arguments to the agent (`-- --resume`)? | [unverified]; if not, forwarded args are refused for Claude/Codex with a message |
| Q5 | Does `sbx run` work without a TTY (an extension's pipe)? | [unverified] |
| Q6 | Antigravity in the VM: does `agy` accept `GEMINI_API_KEY` through the `google` proxy service (header vs query parameter, host)? Does its Google login complete inside the VM with the browser disabled? | API key through the proxy [assumption]; login inside the VM as fallback |
| Q7 | Pi: which variables it reads for cloud providers; a per-sandbox llama-server key (`--api-key-file`) or a custom secret toward `host.docker.internal` | per-sandbox key, revoked at the last session [proposal] |
| Q8 | Does `sbx`'s `shell` image contain any agent CLI? | [unverified]; if yes, `environment(Vm{Project})` must hide it (a `PATH` without the image's agent folder) |
| Q9 | VSCodium: `automationProfile` also used for debug terminals; can a trusted workspace add terminal profiles; is Machine `settings.json` reloaded live? | [unverified] |
| Q10 | VSCodium web server `--disable-extension` | [unverified]; uninstall as fallback |
| Q11 | Stop installing agents for the PC entirely, and Claude Code/Codex for the VM's Linux (the `sbx` images have them)? | yes: agents are installed only for the VM (Pi and Antigravity Linux builds; Claude Code and Codex from the `sbx` images) — **decided by the user, 26/10/2026** |
| Q12 | A prompt label inside the VM's `bash -l` | banner only [unverified] |
| Q13 | The kit's MCP servers for agents in the VM (D-175 "Da fare") | unchanged, out of D-179's scope |
| Q14 | Debug adapters and language extensions executing project code on the PC with the VM on | declared limit kept; candidate for a later decision |
| Q15 | Agentic extensions not in the list | cannot be detected in general: declared limit |
| Q16 | Windows `.cmd` forwarder: arguments with `"`, `%`, `^` through `%*` | declared limit; `.ps1` and sh forwarders pass them intact |
| Q17 | MongoDB in the VM: is `mongod` the right check, or `mongosh` | `mongod --version` [assumption] |
| Q18 | Can PowerShell capture the stdout of `Atollo.exe` (GUI subsystem)? Relevant only if a client ever calls the CLI | not used: clients use files |
