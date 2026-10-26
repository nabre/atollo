//! A placement made ready to start (LEVELS.md §3.6): for a `Vm` one, the
//! kit's tools for the VM's Linux (D-170, D-173), the project's databases
//! on the PC's servers and the file of their variables (D-171), Pi's local
//! models (D-178), the sandbox (`isolation::Vm`, the project's or the
//! agent's) built from the placement and started, and `info.json` beside
//! the sandbox's kit (§8.4). For a `Pc` one nothing: its caller's `ensure`
//! ran before `levels::env::environment`, which is built from what it
//! returns.
//!
//! What touches the app (installs, servers, `sbx`) goes through
//! [`Effects`], so that the order and the files can be tested without
//! Tauri and without `sbx`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::common::i18n::t;
use crate::db::auth::ProjectDb;
use crate::env::{Host, KitEnv};
use crate::isolation::{self, vm, Vm};
use crate::kit::Kit;

use super::env::Environment;
use super::{Agent, Level, Placement, Sandbox, VmProgram};

/// What `prepare` needs from the app, so that it can be tested without
/// Tauri and without `sbx`.
pub trait Effects {
    /// Installs for the VM's Linux (`guest`, the kit scoped to its target)
    /// what `wanted` lacks (id → the line chosen; `None`: the newest),
    /// waiting for the downloads. A failure only goes to the log: the
    /// session opens with what is there.
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>);
    /// Starts the servers of `project`'s databases on the PC (the kit's
    /// environment, never the sealed one: the servers are the kit's own
    /// programs); each database with its server's port.
    fn start_databases(&self, project: &str) -> Result<Vec<(ProjectDb, u16)>>;
    /// Starts llama-server on the PC when Pi uses a local model (D-178);
    /// its port and key, `None` when Pi uses none.
    fn start_local_models(&self, lang: &str) -> Option<LocalModels>;
    /// Starts the sandbox (`Vm::start`: made the first time, its kit, its
    /// mounts, its PC ports and GitHub synced) with `sbx`'s environment.
    fn start_sandbox(&self, vm: &Vm, env: &KitEnv, lang: &str) -> Result<()>;
    /// Closes the local models' `port` of `vm` once no Pi started from the
    /// PC runs there (`isolation::watch_pi_port`); never one of the
    /// databases' (`db_ports`).
    fn watch_local_models(&self, vm: &Vm, env: &KitEnv, project: &str, port: u16, db_ports: &[u16]);
}

/// The kit's local models as Pi in a VM reaches them (D-178).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalModels {
    /// llama-server's port on the PC's `127.0.0.1`.
    pub port: u16,
    /// Its key: a kit secret, only in `pi.env` (LEVELS.md §13.8, Q7).
    pub key: String,
}

/// A placement made ready to start.
#[derive(Debug)]
pub struct Ready {
    /// Never `Refused`.
    pub placement: Placement,
    /// For `Vm`: `guest` as the sandbox got it (with its `env_file`).
    pub env: Environment,
    /// For `Vm`: started, its ports and GitHub synced.
    pub vm: Option<Vm>,
}

impl Ready {
    /// A `Pc` placement with its environment: nothing to prepare.
    pub fn on_the_pc(placement: Placement, env: KitEnv) -> Ready {
        Ready {
            placement,
            env: Environment::Pc(env),
            vm: None,
        }
    }
}

/// The file of the project's database variables, beside its sandbox's kit.
pub const DB_ENV: &str = "db.env";
/// The file of Pi's variables (the databases' and the local models'),
/// beside the project's sandbox's kit (D-178).
pub const PI_ENV: &str = "pi.env";
/// What the read-only `atollo` of a sandbox prints (§8.4), beside its kit.
pub const INFO: &str = "info.json";

/// For a `Vm` placement: installs the tools (D-170), starts the databases
/// and writes `db.env` (D-171), adds Pi's local models (D-178), builds the
/// `isolation::Vm` (the project's or the agent's sandbox; for Antigravity
/// with the kit's Gemini key through the proxy of `sbx`), writes its
/// `info.json` (§8.4) and starts it. `needs`: the tools the program needs
/// besides the project's (a creation's plan). For `Pc`: nothing (the
/// caller's `ensure` ran before its environment). `Refused` is an error:
/// nothing is ever prepared for it (I7).
pub fn prepare(
    fx: &dyn Effects,
    kit: &Kit,
    host: &Host,
    placement: Placement,
    env: Environment,
    needs: &[String],
    lang: &str,
) -> Result<Ready> {
    let (
        Placement::Vm {
            level,
            project,
            sandbox,
            mounts,
            host: access,
            program,
            ..
        },
        Environment::Vm { sbx_env, .. },
    ) = (&placement, &env)
    else {
        return match (placement, env) {
            (placement @ Placement::Pc { .. }, Environment::Pc(env)) => {
                Ok(Ready::on_the_pc(placement, env))
            }
            (placement, _) => Err(anyhow!("levels: nothing is prepared for {placement:?}")),
        };
    };
    // 1. The kit's tools for the VM's Linux, at the project's lines (D-170),
    //    installed when missing; the agent's own build for its session.
    let wanted = wanted_tools(kit, project, program, needs);
    let guest = vm::guest_target(kit.target()).map(|t| kit.for_target(t));
    if let Some(guest) = &guest {
        fx.ensure_guest_tools(guest, &wanted);
        std::fs::create_dir_all(guest.apps())
            .with_context(|| guest.apps().display().to_string())?;
    }
    if let VmProgram::AgentBinary { agent } = program {
        guest
            .as_ref()
            .and_then(|guest| isolation::agent_in_vm(kit, guest, agent.id()))
            .ok_or_else(|| {
                anyhow!(t(
                    lang,
                    "{0} per il Linux della VM non è installato e non si è potuto installare: guarda la barra di stato o la pagina Strumenti.",
                    &[agent.name()],
                ))
            })?;
    }
    // Read again after the installs: a tool installed now is on its PATH.
    let mut tools = super::env::guest_tools(kit, *level, project, program, lang, host);
    // 2. The databases on the PC's servers, their ports, `db.env` (D-171).
    let mut dbs = Vec::new();
    if access.databases {
        dbs = fx.start_databases(project)?;
        let file = isolation::kit::dir(kit, &vm::name(project)).join(DB_ENV);
        tools.env_file = write_db_env(&file, &mounts.workspace_rw, &dbs)?;
    }
    let db_ports: Vec<u16> = dbs.iter().map(|(_, port)| *port).collect();
    // 3. The sandbox of the placement.
    let vm = isolation::project_vm(
        kit,
        host,
        project,
        &mounts.workspace_rw,
        mounts.tools_ro.clone(),
    )
    .ok_or_else(|| anyhow!(t(lang, "Docker Sandboxes (sbx) non è nel kit.", &[])))?;
    let vm = match sandbox {
        Sandbox::Project { .. } => vm,
        Sandbox::Agent { agent, .. } => vm
            .for_agent(project, agent.id())
            .ok_or_else(|| anyhow!("levels: {} has no sandbox of sbx", agent.id()))?,
    };
    anyhow::ensure!(
        vm.name == sandbox.name(),
        "levels: {} is not {}",
        vm.name,
        sandbox.name()
    );
    let mut vm = vm.with_host_ports(db_ports.clone());
    // 4. Pi's local models (D-178): their port for this sandbox only, their
    //    address and key in `pi.env` beside the databases' variables.
    let mut watched = None;
    if access.local_models {
        if let Some(models) = fx.start_local_models(lang) {
            // Not a database's: a watcher of the port waits again, it never
            // closes it while this Pi is being started.
            if !db_ports.contains(&models.port) {
                isolation::pi_port_touch(&vm.name, models.port);
                watched = Some(models.port);
            }
            let file = isolation::kit::dir(kit, &vm.name).join(PI_ENV);
            tools.env_file = Some(write_pi_env(&file, tools.env_file.as_deref(), &models)?);
            vm = vm.with_host_port(models.port);
        }
    }
    // 5. Antigravity: the kit's Gemini key, when the settings hold one,
    //    through the proxy of `sbx` for this sandbox only (`secret
    //    gemini`, §9, §11), never in the VM; without it `agy` signs in
    //    inside the VM.
    let antigravity = matches!(
        program,
        VmProgram::AgentBinary {
            agent: Agent::Antigravity
        }
    );
    let vm = vm.with_tools(tools).with_gemini(antigravity);
    // 6. What `atollo` in the sandbox tells (§8.4), then the start.
    let info = SandboxInfo::of(&placement, &wanted, &dbs, vm.host_ports());
    write_info(kit, &vm.name, &info)?;
    fx.start_sandbox(&vm, sbx_env, lang)?;
    if let Some(port) = watched {
        fx.watch_local_models(&vm, sbx_env, project, port, &db_ports);
    }
    let env = Environment::Vm {
        sbx_env: sbx_env.clone(),
        guest: vm.tools().clone(),
    };
    Ok(Ready {
        placement,
        env,
        vm: Some(vm),
    })
}

/// The kit's tools for `project`'s VM running `program`: those the kit
/// chose (`editor::terminals::chosen_tools`), what the program needs
/// (`needs`; the kit's PowerShell for a creation, whose lines are
/// PowerShell's; the agent's own build for `AgentBinary`), at the lines the
/// project chose (`versions::project_choice`).
pub fn wanted_tools(
    kit: &Kit,
    project: &str,
    program: &VmProgram,
    needs: &[String],
) -> BTreeMap<String, Option<String>> {
    let mut wanted = crate::editor::terminals::chosen_tools(kit);
    let own = match program {
        VmProgram::Scaffold { .. } => Some("pwsh"),
        VmProgram::AgentBinary { agent } => Some(agent.id()),
        _ => None,
    };
    for id in needs.iter().map(String::as_str).chain(own) {
        wanted.entry(id.to_owned()).or_insert(None);
    }
    for (id, line) in crate::versions::project_choice(kit, project) {
        wanted.insert(id, Some(line));
    }
    wanted
}

/// The file of the variables that point the programs of the VM to the
/// project's databases (`dbs`, each with its server's port) as the
/// project's user (D-171): the keys the kit writes in a `.env` like the
/// project's (`db::auth::env_wanted`, read from `dir`'s `.env`), with the
/// VM's way to the PC, plus the MySQL client's. Readable by its owner only,
/// given with `--env-file`: the password is never on a command line. No
/// database: the file goes (a database removed takes its password out of
/// the VM's file too).
pub fn write_db_env(file: &Path, dir: &Path, dbs: &[(ProjectDb, u16)]) -> Result<Option<PathBuf>> {
    if dbs.is_empty() {
        remove(file)?;
        return Ok(None);
    }
    let text = crate::platform::safe_path::read_text(dir, &dir.join(".env")).unwrap_or_default();
    let mut lines = String::new();
    for (db, port) in dbs {
        let in_vm = isolation::db_in_vm(db, *port);
        for (k, v) in crate::db::auth::env_wanted(&text, &in_vm) {
            lines.push_str(&format!("{k}={v}\n"));
        }
        if crate::db::DbEngine::from_id(&db.engine) == Some(crate::db::DbEngine::MySql) {
            lines.push_str(&format!("MYSQL_HOST={}\nMYSQL_TCP_PORT={port}\n", vm::HOST));
        }
    }
    write_private(file, &lines)?;
    Ok(Some(file.to_path_buf()))
}

/// `pi.env`: the databases' variables (`db_file`, if any) and the local
/// models' address and key (`isolation::pi_local_models_env`), readable by
/// its owner only; deleted when Pi ends (`isolation::watch_pi_port`).
fn write_pi_env(file: &Path, db_file: Option<&Path>, models: &LocalModels) -> Result<PathBuf> {
    let mut text = db_file
        .and_then(|f| std::fs::read_to_string(f).ok())
        .unwrap_or_default();
    for (k, v) in isolation::pi_local_models_env(models.port, &models.key) {
        text.push_str(&format!("{k}={v}\n"));
    }
    write_private(file, &text)?;
    Ok(file.to_path_buf())
}

fn write_private(file: &Path, text: &str) -> Result<()> {
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }
    crate::platform::files::replace_private(file, text)
}

fn remove(file: &Path) -> Result<()> {
    match std::fs::remove_file(file) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| file.display().to_string())
        }
        _ => Ok(()),
    }
}

/// What a sandbox's read-only `atollo` prints (§8.4): where it is and what
/// it has; never a secret (no password, key or token).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxInfo {
    pub v: u32,
    pub level: Level,
    pub sandbox: String,
    pub project: String,
    /// The agent of an agent's placement.
    pub agent: Option<Agent>,
    /// The working folder in the VM.
    pub workdir: String,
    /// The kit's tools, each at the line chosen (`None`: the newest).
    pub tools: BTreeMap<String, Option<String>>,
    pub databases: Vec<DbInfo>,
    /// The PC's ports this sandbox may reach (its databases, Pi's models).
    pub host_ports: Vec<u16>,
}

/// A database as the VM reaches it: never its password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DbInfo {
    pub engine: String,
    pub database: String,
    pub user: String,
    /// `host.docker.internal:<port>`.
    pub address: String,
}

impl SandboxInfo {
    /// The info of the `Vm` `placement` with its tools (`tools`), databases
    /// (`dbs`, with their ports) and PC ports (`host_ports`).
    pub fn of(
        placement: &Placement,
        tools: &BTreeMap<String, Option<String>>,
        dbs: &[(ProjectDb, u16)],
        host_ports: &[u16],
    ) -> SandboxInfo {
        let (project, sandbox, workdir, agent) = match placement {
            Placement::Vm {
                project,
                sandbox,
                workdir,
                program,
                ..
            } => (
                project.clone(),
                sandbox.name().to_owned(),
                workdir.clone(),
                match program {
                    VmProgram::SbxAgent { agent } | VmProgram::AgentBinary { agent } => {
                        Some(*agent)
                    }
                    _ => None,
                },
            ),
            _ => Default::default(),
        };
        SandboxInfo {
            v: 1,
            level: placement.level(),
            sandbox,
            project,
            agent,
            workdir,
            tools: tools.clone(),
            databases: dbs
                .iter()
                .map(|(db, port)| DbInfo {
                    engine: db.engine.clone(),
                    database: db.database.clone(),
                    user: db.user.clone(),
                    address: format!("{}:{port}", vm::HOST),
                })
                .collect(),
            host_ports: host_ports.to_vec(),
        }
    }
}

/// The `info.json` of `sandbox`: beside its kit (`data/app/sandboxes/<it>`),
/// never in the project's folder.
pub fn info_file(kit: &Kit, sandbox: &str) -> PathBuf {
    isolation::kit::dir(kit, sandbox).join(INFO)
}

fn write_info(kit: &Kit, sandbox: &str, info: &SandboxInfo) -> Result<()> {
    let file = info_file(kit, sandbox);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
    }
    let text = serde_json::to_string_pretty(info)?;
    std::fs::write(&file, text + "\n").with_context(|| file.display().to_string())
}

#[cfg(test)]
#[path = "../../../test/rust/levels/prepare.rs"]
pub(crate) mod tests;
