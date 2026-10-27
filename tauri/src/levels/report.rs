//! What the user can check of a project's VM (LEVELS.md §8.1, §8.2, §8.5):
//! the report of every sandbox of the project (the project's `atollo-p-…`,
//! then each agent's `atollo-a-…`) with its state, its mounts, the PC's
//! ports it reaches and its `info.json`; «Verifica strumenti», one
//! `sbx exec` per sandbox running each chosen tool's version command, the
//! lines compared with the project's; and the actions on a sandbox
//! (restart, make again, refresh), each through `decide` with the
//! sandbox's lock.
//!
//! What touches `sbx`, the app's sessions and `prepare` goes through
//! [`VmEffects`] (the app's: `ipc::levels`), so that the order can be
//! tested without Tauri and without `sbx`.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex, PoisonError};

use serde::{Deserialize, Serialize};

use crate::common::i18n::t;
use crate::isolation::{self, vm};
use crate::kit::Kit;

use super::prepare::{info_file, SandboxInfo};
use super::{decide, Agent, Facts, Fix, Placement, Refusal, Request, VmProgram};

/// The report of a project's VM (§8.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmReport {
    pub project: String,
    /// `on` | `off`: whether this PC has the VM now.
    pub vm: VmOnOff,
    /// Why it is off (the notice's text); `None` when on.
    pub reason: Option<String>,
    /// The project's sandbox, then each agent's with a sandbox of its own.
    pub sandboxes: Vec<SandboxReport>,
    /// The project's actions running in its VM with a port published on
    /// the PC.
    pub published: Vec<PublishedPort>,
    /// «GitHub nella VM» of the project (D-172).
    pub github: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VmOnOff {
    On,
    Off,
}

/// One sandbox of the project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SandboxReport {
    pub name: String,
    /// The agent of an agent's sandbox; `None` for the project's.
    pub agent: Option<Agent>,
    pub state: SandboxRun,
    /// When `sbx` last used it.
    pub last_used: Option<String>,
    /// The folders it mounts: the project rw, the rest ro.
    pub mounts: Vec<MountInfo>,
    /// The PC's ports it may reach.
    pub host_ports: Vec<HostPortInfo>,
    /// What its read-only `atollo` prints (§8.4), from its last preparation.
    pub info: Option<SandboxInfo>,
    /// The app's sessions running in it (terminals, actions, agents):
    /// «Riavvia» and «Ricrea» stop them.
    pub sessions: Vec<String>,
    /// «Verifica strumenti», after it ran.
    pub tools: Option<Vec<ToolCheck>>,
    /// Why the check found nothing (the sandbox is not running).
    pub note: Option<String>,
}

/// A sandbox's state as `sbx` says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SandboxRun {
    Running,
    Stopped,
    /// `sbx` has no such sandbox (never made, or removed).
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Rw,
    Ro,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountInfo {
    /// The folder of the PC.
    pub host: String,
    /// Where the VM sees it.
    pub guest: String,
    pub mode: Mode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostPortInfo {
    pub port: u16,
    /// What listens there: a database (`mysql shop`) or `llama-server`.
    pub what: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedPort {
    pub port: u16,
    /// `127.0.0.1`, or `0.0.0.0` with «Rete locale».
    pub address: String,
    pub action: String,
}

/// A tool as «Verifica strumenti» found it in a sandbox (§8.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCheck {
    pub id: String,
    pub name: String,
    /// The project's line, else the kit's; `None` for an unversioned tool.
    pub chosen: Option<String>,
    /// What ran, shown verbatim (empty for a tool not in the VM).
    pub command: String,
    /// `command -v` in the VM.
    pub path: Option<String>,
    /// The first version number of its output.
    pub found: Option<String>,
    pub verdict: Verdict,
    /// The first 3 lines of its output.
    pub output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    /// The chosen line, from the kit's tools.
    Ok,
    /// Found, another line.
    OtherLine,
    /// Not found in the VM.
    Missing,
    /// Found outside the kit's tools mounted in the VM.
    NotFromKit,
    /// Not meant to run in the VM (it runs on the PC or in a container).
    NotInVm,
}

/// The sandboxes of `project`: its own, then each agent's of `sbx`
/// (Claude Code, Codex); Pi and Antigravity run in the project's.
pub fn sandboxes_of(project: &str) -> Vec<(String, Option<Agent>)> {
    let mut out = vec![(vm::name(project), None)];
    for agent in Agent::ALL {
        if agent.runner() == super::AgentRunner::SbxAgent {
            out.push((vm::agent_name(project, agent.id()), Some(agent)));
        }
    }
    out
}

/// The state of `name` in `sbx ls --json` (`ls`) and when it was last
/// used; without an answer of `sbx` (the VM off), `stopped` when Atollo
/// made it once (its kit says so), else `missing`.
pub fn sandbox_state(kit: &Kit, ls: Option<&str>, name: &str) -> (SandboxRun, Option<String>) {
    let Some(ls) = ls else {
        let made = isolation::kit::applied(kit, name).is_some();
        return (
            if made {
                SandboxRun::Stopped
            } else {
                SandboxRun::Missing
            },
            None,
        );
    };
    let list: Vec<serde_json::Value> = serde_json::from_str(ls).unwrap_or_default();
    match list.iter().find(|s| s["name"] == name) {
        None => (SandboxRun::Missing, None),
        Some(s) => (
            if s["status"].as_str() == Some("running") {
                SandboxRun::Running
            } else {
                SandboxRun::Stopped
            },
            s["last_used_at"].as_str().map(str::to_owned),
        ),
    }
}

/// The folders `sandbox` mounts, as Atollo noted them at its making
/// (`.mounts`: the workspace rw, then the read-only ones).
pub fn mounts_of(kit: &Kit, sandbox: &str) -> Vec<MountInfo> {
    let windows = kit.target().is_windows();
    isolation::kit::mounts(kit, sandbox)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.trim().is_empty())
        .enumerate()
        .map(|(i, line)| MountInfo {
            host: line.to_owned(),
            guest: vm::guest_path(windows, Path::new(line)).unwrap_or_else(|| line.to_owned()),
            mode: if i == 0 { Mode::Rw } else { Mode::Ro },
        })
        .collect()
}

/// The PC's ports `sandbox` may reach (`.ports`), each with what listens
/// there by its `info`: a database of the project, else the local models.
pub fn host_ports_of(kit: &Kit, sandbox: &str, info: Option<&SandboxInfo>) -> Vec<HostPortInfo> {
    isolation::kit::host_ports(kit, sandbox)
        .into_iter()
        .map(|port| {
            let db = info.and_then(|i| {
                i.databases
                    .iter()
                    .find(|d| d.address.rsplit(':').next() == Some(port.to_string().as_str()))
            });
            HostPortInfo {
                port,
                what: db.map_or_else(
                    || "llama-server".to_owned(),
                    |d| format!("{} {}", d.engine, d.database),
                ),
            }
        })
        .collect()
}

/// The `info.json` of `sandbox`, if a preparation wrote one.
pub fn read_info(kit: &Kit, sandbox: &str) -> Option<SandboxInfo> {
    serde_json::from_str(&std::fs::read_to_string(info_file(kit, sandbox)).ok()?).ok()
}

/// What the report needs besides the kit's files.
#[derive(Debug, Clone, Default)]
pub struct Seen {
    /// The VM is on.
    pub on: bool,
    /// Why it is off.
    pub reason: Option<String>,
    /// `sbx ls --json`, when `sbx` answered.
    pub ls: Option<String>,
    /// The project's actions with a port published from its VM.
    pub published: Vec<PublishedPort>,
    /// «GitHub nella VM».
    pub github: bool,
}

/// The report of `project` (§8.1): every sandbox of it with its state,
/// mounts, PC's ports, `info.json`, and the app's sessions in it
/// (`sessions`).
pub fn report(
    kit: &Kit,
    project: &str,
    seen: Seen,
    sessions: &dyn Fn(&str) -> Vec<String>,
) -> VmReport {
    let sandboxes = sandboxes_of(project)
        .into_iter()
        .map(|(name, agent)| {
            let (state, last_used) = sandbox_state(kit, seen.ls.as_deref(), &name);
            let info = read_info(kit, &name);
            SandboxReport {
                mounts: mounts_of(kit, &name),
                host_ports: host_ports_of(kit, &name, info.as_ref()),
                sessions: sessions(&name),
                info,
                name,
                agent,
                state,
                last_used,
                tools: None,
                note: None,
            }
        })
        .collect();
    VmReport {
        project: project.to_owned(),
        vm: if seen.on { VmOnOff::On } else { VmOnOff::Off },
        reason: seen.reason,
        sandboxes,
        published: seen.published,
        github: seen.github,
    }
}

/// What «Verifica strumenti» runs for one tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckSpec {
    pub id: String,
    pub name: String,
    pub chosen: Option<String>,
    /// The program's file name, as `PATH` resolves it; `None`: `notInVm`.
    pub program: Option<String>,
    pub args: Vec<String>,
    /// A folder on `PATH` for this check only (Pi's, Antigravity's build).
    pub extra_path: Option<String>,
    /// From the sandbox's image (Claude Code, Codex in their own): any
    /// path is fine.
    pub from_image: bool,
}

impl CheckSpec {
    /// The command shown: the program and its words.
    pub fn command(&self) -> String {
        match &self.program {
            None => String::new(),
            Some(p) => std::iter::once(p.as_str())
                .chain(self.args.iter().map(String::as_str))
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

/// The check of the tool `id` (its catalog's `check`, the program reduced
/// to its file name for the VM's Linux, `guest`); `chosen` its line.
/// `None` for a component of the kit on the PC (a system component, one
/// without a program to ask, phpMyAdmin's and mongo-express's pages); no
/// program (`notInVm`) for a tool that does not run in the VM
/// (`Component::in_vm`, D-180: llama.cpp, the sites' servers).
pub fn check_spec(kit: &Kit, guest: &Kit, id: &str, chosen: Option<String>) -> Option<CheckSpec> {
    let c = crate::setup::Catalog::get().find(id)?;
    if c.group.always() || c.check.is_empty() || c.check.iter().any(|a| a.starts_with('<')) {
        return None;
    }
    let program = c.in_vm().then(|| {
        let rel = crate::setup::catalog::translate_rel(guest.target(), &c.check[0]);
        rel.rsplit('/').next().unwrap_or(&rel).to_owned()
    });
    let extra_path = matches!(id, "pi" | "antigravity")
        .then(|| vm::guest_path(kit.target().is_windows(), &guest.apps().join(id)))
        .flatten();
    Some(CheckSpec {
        id: id.to_owned(),
        name: c.name.clone(),
        chosen,
        program,
        args: c.check[1..].to_vec(),
        extra_path,
        from_image: false,
    })
}

/// The checks of the sandbox `sandbox` of `project` (`agent`: an agent's
/// own): the tools of its last preparation (`info.json`), else the kit's
/// and the project's; each at the project's line, else the kit's choice,
/// else the line of its default folder; the agent of the sandbox from its
/// image.
pub fn check_specs(
    kit: &Kit,
    project: &str,
    sandbox: &str,
    agent: Option<Agent>,
) -> Vec<CheckSpec> {
    let Some(guest) = vm::guest_target(kit.target()).map(|t| kit.for_target(t)) else {
        return Vec::new();
    };
    let tools = read_info(kit, sandbox).map_or_else(
        || super::prepare::wanted_tools(kit, project, &VmProgram::Shell, &[]),
        |i| i.tools,
    );
    let kit_lines = crate::versions::kit_choice(&guest);
    let catalog = crate::setup::Catalog::get();
    let mut out: Vec<CheckSpec> = tools
        .into_iter()
        .filter(|(id, _)| !matches!(id.as_str(), "claude" | "codex"))
        .filter_map(|(id, line)| {
            let c = catalog.find(&id)?;
            let chosen = line.or_else(|| kit_lines.get(&id).cloned()).or_else(|| {
                c.versioned
                    .then(|| crate::versions::default_line(&guest, c))
                    .flatten()
            });
            check_spec(kit, &guest, &id, chosen)
        })
        .collect();
    // Pi and Antigravity in the project's VM, when their build is there.
    if agent.is_none() {
        for id in ["pi", "antigravity"] {
            let listed = out.iter().any(|s| s.id == id);
            if !listed && isolation::agent_in_vm(kit, &guest, id).is_some() {
                out.extend(check_spec(kit, &guest, id, None));
            }
        }
    }
    if let Some(agent) = agent {
        if let Some(mut spec) = check_spec(kit, &guest, agent.id(), None) {
            spec.from_image = true;
            out.push(spec);
        }
    }
    out
}

/// The script of «Verifica strumenti» (§8.2): the kit's tools first on
/// `PATH`, then for each tool its id, `command -v`, and the first three
/// lines of its version command. It only reads.
pub fn check_script(specs: &[CheckSpec]) -> String {
    use crate::platform::sh_quote as q;
    let mut script = String::from(vm::TOOLS_FIRST);
    script.push('\n');
    for spec in specs {
        let Some(program) = &spec.program else {
            continue;
        };
        let words: Vec<String> = std::iter::once(program)
            .chain(&spec.args)
            .map(|w| q(w))
            .collect();
        let extra = spec
            .extra_path
            .as_ref()
            .map(|dir| format!("PATH=\"$PATH\":{}; ", q(dir)))
            .unwrap_or_default();
        script.push_str(&format!(
            "printf '@@%s\\n' {}\n({extra}p=$(command -v {}); printf '%s\\n' \"$p\"; if [ -n \"$p\" ]; then {} 2>&1 | head -n 3; fi)\n",
            q(&spec.id),
            q(program),
            words.join(" ")
        ));
    }
    script
}

/// The first version number of `text` (`go version go1.23.2` → `1.23.2`,
/// `openjdk version "21.0.5"` → `21.0.5`).
pub fn found_version(text: &str) -> Option<String> {
    static RE: std::sync::OnceLock<Option<regex::Regex>> = std::sync::OnceLock::new();
    RE.get_or_init(|| regex::Regex::new(r"[0-9]+(\.[0-9]+)+").ok())
        .as_ref()?
        .find(text)
        .map(|m| m.as_str().to_owned())
}

/// The verdict of `spec` found at `path` (none: not found) with the
/// version `found`; `mount`: the kit's tools in the VM.
pub fn verdict(spec: &CheckSpec, path: Option<&str>, found: Option<&str>, mount: &str) -> Verdict {
    if spec.program.is_none() {
        return Verdict::NotInVm;
    }
    let Some(path) = path.filter(|p| !p.trim().is_empty()) else {
        return Verdict::Missing;
    };
    let from_kit = path.starts_with(&format!("{}/", mount.trim_end_matches('/')));
    if !spec.from_image && !from_kit {
        return Verdict::NotFromKit;
    }
    match (&spec.chosen, found) {
        (None, _) => Verdict::Ok,
        (Some(chosen), Some(found))
            if found == chosen || found.starts_with(&format!("{chosen}.")) =>
        {
            Verdict::Ok
        }
        (Some(_), _) => Verdict::OtherLine,
    }
}

/// The checks of `specs` read from the output `out` of [`check_script`],
/// `mount` the kit's tools in the VM.
pub fn parse_check(out: &str, specs: &[CheckSpec], mount: &str) -> Vec<ToolCheck> {
    let mut blocks: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut current: Option<&str> = None;
    for line in out.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(id) = line.strip_prefix("@@") {
            current = Some(id);
            blocks.entry(id).or_default();
        } else if let Some(id) = current {
            blocks.entry(id).or_default().push(line);
        }
    }
    specs
        .iter()
        .map(|spec| {
            let block = blocks.get(spec.id.as_str());
            let path = block
                .and_then(|b| b.first())
                .map(|p| p.trim().to_owned())
                .filter(|p| !p.is_empty());
            let output: Vec<&str> = block
                .map(|b| b.iter().skip(1).take(3).copied().collect())
                .unwrap_or_default();
            let output = output.join("\n");
            let found = path.as_ref().and_then(|_| found_version(&output));
            ToolCheck {
                id: spec.id.clone(),
                name: spec.name.clone(),
                chosen: spec.chosen.clone(),
                command: spec.command(),
                verdict: verdict(spec, path.as_deref(), found.as_deref(), mount),
                path,
                found,
                output,
            }
        })
        .collect()
}

/// The checks of `specs` in a sandbox that is not running: `missing` (the
/// check never starts one), `notInVm` as they are.
pub fn not_running(specs: &[CheckSpec]) -> Vec<ToolCheck> {
    parse_check("", specs, "")
}

/// The text of a check that could not run: the sandbox is not running.
pub fn not_running_note(lang: &str) -> String {
    t(
        lang,
        "La sandbox non è accesa: avvia il terminale o un'azione del progetto, poi verifica di nuovo.",
        &[],
    )
}

/// «Verifica strumenti» in the sandbox `s` of `project` (§8.2): `req`
/// decided with `facts` first (the project terminal's origin, or the
/// agent's for its own sandbox: a refusal says why); a sandbox not running
/// is never started, its tools `missing` with a note; else one `sbx exec`
/// with the session's tools (`base` the kit's environment, `sbx`'s built
/// from it), the output compared line by line.
#[allow(clippy::too_many_arguments)]
pub fn check(
    kit: &Kit,
    host: &crate::env::Host,
    base: &crate::env::KitEnv,
    req: &Request,
    facts: &Facts,
    project: &str,
    s: &mut SandboxReport,
    lang: &str,
) -> Result<(), ActError> {
    let (level, program) = match decide(req, facts) {
        Placement::Refused { reason, fix, .. } => return Err(ActError::Refused { reason, fix }),
        Placement::Pc { .. } => return Err(ActError::NoVm),
        Placement::Vm {
            level,
            program,
            sandbox,
            ..
        } => {
            if sandbox.name() != s.name {
                return Err(ActError::Failed(anyhow::anyhow!(
                    "{} is not {}",
                    sandbox.name(),
                    s.name
                )));
            }
            (level, program)
        }
    };
    let specs = check_specs(kit, project, &s.name, s.agent);
    if s.state != SandboxRun::Running {
        s.tools = Some(not_running(&specs));
        s.note = Some(not_running_note(lang));
        return Ok(());
    }
    let mut tools = super::env::guest_tools(kit, level, project, &program, lang, host);
    let first = super::prepare::guest_path_entries(kit, &s.name, program.agent());
    tools.path.splice(0..0, first);
    let args = vm::check_args(&s.name, &tools, &check_script(&specs));
    let sbx_env = isolation::sbx_session_env(base, host);
    let out = isolation::sbx_output(kit, host, &sbx_env, &args).map_err(ActError::Failed)?;
    s.tools = Some(parse_check(&out, &specs, &tools_mount(kit)));
    s.note = None;
    Ok(())
}

/// The kit's tools as the VM sees them (`data/apps/linux-…`).
pub fn tools_mount(kit: &Kit) -> String {
    vm::guest_target(kit.target())
        .and_then(|t| vm::guest_path(kit.target().is_windows(), &kit.for_target(t).apps()))
        .unwrap_or_default()
}

/// An action on a sandbox of a project (§8.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VmAction {
    /// «Riavvia»: stopped, then started as a session would.
    Restart,
    /// «Ricrea»: removed and made again with the kit's tools of now.
    Recreate,
    /// «Aggiorna strumenti»: `prepare` again; no restart.
    Refresh,
}

impl VmAction {
    /// True for the actions that stop the sessions running in the sandbox.
    pub fn stops_sessions(self) -> bool {
        matches!(self, VmAction::Restart | VmAction::Recreate)
    }
}

/// What an action needs from the app.
pub trait VmEffects {
    /// The app's sessions running in `sandbox` (terminals, actions, agents).
    fn sessions(&self, sandbox: &str) -> Vec<String>;
    /// Ends them.
    fn end_sessions(&self, sandbox: &str);
    /// `sbx stop`, under the sandbox's lock.
    fn stop_sandbox(&self, sandbox: &str) -> anyhow::Result<()>;
    /// `sbx rm -f`, under the sandbox's lock (`isolation::remove_to_make_again`).
    fn remove_sandbox(&self, sandbox: &str) -> anyhow::Result<()>;
    /// `req` made ready with `facts` (`prepare`: tools, databases, ports,
    /// `info.json`, the start).
    fn prepare(&self, req: &Request, facts: &Facts) -> anyhow::Result<()>;
}

/// Why an action did not run.
#[derive(Debug)]
pub enum ActError {
    /// `decide` refused the request.
    Refused { reason: Refusal, fix: Fix },
    /// The VM is off: nothing to act on.
    NoVm,
    /// A step failed.
    Failed(anyhow::Error),
}

/// The lock of the actions on the sandbox `name`: two actions on it never
/// run together (each step of `sbx` takes the sandbox's own lock too).
pub fn action_lock(name: &str) -> Arc<Mutex<()>> {
    static LOCKS: Mutex<BTreeMap<String, Arc<Mutex<()>>>> = Mutex::new(BTreeMap::new());
    let mut all = LOCKS.lock().unwrap_or_else(PoisonError::into_inner);
    Arc::clone(all.entry(name.to_owned()).or_default())
}

/// `action` on the sandbox `req` is placed in (§8.5): `decide` first (a
/// refusal says why and how to fix it; the PC: no VM), then, under the
/// sandbox's action lock, its sessions listed and, for «Riavvia» and
/// «Ricrea», ended; `sbx stop` or `sbx rm -f`; then `prepare` again. The
/// sessions that were running.
pub fn act(
    fx: &dyn VmEffects,
    req: &Request,
    facts: &Facts,
    action: VmAction,
) -> Result<Vec<String>, ActError> {
    let name = match decide(req, facts) {
        Placement::Refused { reason, fix, .. } => return Err(ActError::Refused { reason, fix }),
        Placement::Pc { .. } => return Err(ActError::NoVm),
        Placement::Vm { sandbox, .. } => sandbox.name().to_owned(),
    };
    let lock = action_lock(&name);
    let _one = lock.lock().unwrap_or_else(PoisonError::into_inner);
    let sessions = fx.sessions(&name);
    if action.stops_sessions() && !sessions.is_empty() {
        fx.end_sessions(&name);
    }
    match action {
        VmAction::Restart => fx.stop_sandbox(&name).map_err(ActError::Failed)?,
        VmAction::Recreate => fx.remove_sandbox(&name).map_err(ActError::Failed)?,
        VmAction::Refresh => {}
    }
    fx.prepare(req, facts).map_err(ActError::Failed)?;
    Ok(sessions)
}

/// The request of an action or a check on `project`'s sandbox (`dir` its
/// folder): the project terminal's (the project's sandbox), or the agent's
/// own button for Claude Code's and Codex's sandboxes (R12).
pub fn sandbox_request(kit: &Kit, dir: &Path, agent: Option<Agent>) -> Request {
    Request {
        origin: match agent {
            Some(agent) => super::Origin::AgentButton { agent },
            None => super::Origin::ProjectTerminal,
        },
        place: super::place::place_of(kit, dir),
    }
}

/// What a refused or failed action says, in `lang`.
pub fn act_error_text(e: &ActError, lang: &str) -> String {
    match e {
        ActError::Refused { reason, .. } => super::labels::refusal_text(reason, lang),
        ActError::NoVm => t(
            lang,
            "La VM non è attiva: non c'è una sandbox su cui agire.",
            &[],
        ),
        ActError::Failed(e) => format!("{e:#}"),
    }
}

/// What a recreation removes and what it keeps (§8.5), in `lang`: said
/// before it runs.
pub fn recreate_warning(lang: &str) -> (String, String) {
    (
        t(
            lang,
            "Si perde: la home della VM e le immagini che ha scaricato.",
            &[],
        ),
        t(
            lang,
            "Resta: il progetto, i suoi database, gli strumenti del kit e gli accessi che il kit conserva.",
            &[],
        ),
    )
}

#[cfg(test)]
#[path = "../../../test/rust/levels/report.rs"]
mod tests;
