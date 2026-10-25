//! How shut in a project's programs are (D-167), worked out by itself:
//! nothing to turn on in the settings. The VM (a Docker Sandboxes microVM
//! per project, `sbx`) when the PC has what it needs; otherwise the kit's
//! environment only, with the reasons the VM is off and, for each one, what
//! the user does about it (the interface's notice in the status bar).
//!
//! `sbx` is a program of the PC, not of the kit: it is found in the PC's
//! `PATH` (and where its installers put it) and runs with the PC's
//! environment, where its sign-in lives (D-167, declared trace).
//!
//! On every level a project's programs get a sealed environment
//! ([`seal`]): less of what reaches them by itself. It is no wall (they run
//! as the user and can read what the user can): the VM is.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::env::{Host, KitEnv};
use crate::kit::{Kit, Os};
use crate::platform::virt::{self, Gap};

pub mod vm;

/// How long a status stays good: `sbx` is asked at most once a minute.
const FRESH: Duration = Duration::from_secs(60);
/// How long `sbx` may take to answer.
const ASK_LIMIT: Duration = Duration::from_secs(15);

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

/// Seals `env` for the programs of `project` (terminal, actions, agents,
/// `Atollo env --project`), or of a kit terminal when `None`: the
/// [`SEALED`] variables out, then the project's own MySQL login in
/// (`db::services::project_client_env`).
pub fn seal(kit: &Kit, project: Option<&str>, env: &mut KitEnv) {
    for name in SEALED {
        env.remove(name);
    }
    if let Some(project) = project {
        for (name, value) in crate::db::services::project_client_env(kit, project) {
            env.set(name, value.to_string_lossy());
        }
    }
}

/// What shuts a project's programs in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// A Docker Sandboxes microVM per project.
    Vm,
    /// The kit's environment only: the programs run as the user, on the PC.
    Env,
}

/// Why the VM is off: the interface says it and what to do, by `code`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Blocker {
    OldWindows {
        build: u32,
    },
    Processor,
    OldMacos {
        version: String,
    },
    Distribution {
        name: String,
    },
    HypervisorOff,
    KvmMissing,
    KvmDenied,
    /// AppArmor keeps the kit's microVM shim (its path) from starting.
    AppArmor {
        shim: String,
    },
    /// Docker Sandboxes is neither in the kit nor on the PC.
    SbxMissing,
    /// `sbx` is there but not signed in to Docker.
    SbxSignedOut,
    /// `sbx` answered with an error (its text).
    SbxFailed {
        message: String,
    },
    /// The PC's `sbx` has no global network policy yet: no VM starts.
    /// The kit's gets `balanced` by itself ([`vm::POLICY_ARGS`]).
    PolicyUnset,
}

impl From<Gap> for Blocker {
    fn from(gap: Gap) -> Blocker {
        match gap {
            Gap::OldWindows(build) => Blocker::OldWindows { build },
            Gap::Processor => Blocker::Processor,
            Gap::OldMacos(version) => Blocker::OldMacos { version },
            Gap::Distribution(name) => Blocker::Distribution { name },
            Gap::HypervisorOff => Blocker::HypervisorOff,
            Gap::KvmMissing => Blocker::KvmMissing,
            Gap::KvmDenied => Blocker::KvmDenied,
            Gap::AppArmor(shim) => Blocker::AppArmor {
                shim: shim.display().to_string(),
            },
        }
    }
}

impl Blocker {
    /// A system Docker does not support: nothing the user installs helps.
    pub fn unsupported(&self) -> bool {
        matches!(
            self,
            Blocker::OldWindows { .. }
                | Blocker::Processor
                | Blocker::OldMacos { .. }
                | Blocker::Distribution { .. }
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub level: Level,
    /// Empty when the level is the VM.
    pub blockers: Vec<Blocker>,
    /// The `sbx` found, for the notice.
    pub sbx: Option<String>,
}

impl Status {
    /// The level the blockers leave.
    pub fn of(blockers: Vec<Blocker>, sbx: Option<String>) -> Status {
        let level = if blockers.is_empty() {
            Level::Vm
        } else {
            Level::Env
        };
        Status {
            level,
            blockers,
            sbx,
        }
    }
}

static LAST: Mutex<Option<(Instant, Status)>> = Mutex::new(None);

/// The status, asked again when older than a minute or when `force` (the
/// notice's "Ricontrolla", an installation of Docker Sandboxes).
pub fn status(kit: &Kit, host: &Host, env: &KitEnv, force: bool) -> Status {
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, st)) = last.as_ref() {
        if !force && at.elapsed() < FRESH {
            return st.clone();
        }
    }
    let st = compute(kit, host, env);
    *last = Some((Instant::now(), st.clone()));
    st
}

fn compute(kit: &Kit, host: &Host, env: &KitEnv) -> Status {
    let sbx = Sbx::find(kit, host);
    let shim = sbx.as_ref().and_then(Sbx::shim);
    let mut blockers: Vec<Blocker> = virt::gaps(shim.as_deref())
        .into_iter()
        .map(Blocker::from)
        .collect();
    if blockers.iter().any(Blocker::unsupported) {
        return Status::of(blockers, None);
    }
    match &sbx {
        None => blockers.push(Blocker::SbxMissing),
        Some(sbx) => {
            if let Err(b) = ask(sbx, host, env).and_then(|()| policy(sbx, host, env)) {
                blockers.push(b);
            }
        }
    }
    Status::of(blockers, sbx.map(|s| s.path().display().to_string()))
}

/// The `sbx` the projects' microVMs run with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sbx {
    /// The kit's own (the `sandboxes` component): with the kit's
    /// environment, so its sandboxes, settings and sign-in are in the kit's
    /// home and travel with it.
    Kit(PathBuf),
    /// One of the PC (D-167, declared trace): with the PC's environment,
    /// where it keeps its own.
    Pc(PathBuf),
}

impl Sbx {
    /// The kit's, when installed; else the PC's.
    pub fn find(kit: &Kit, host: &Host) -> Option<Sbx> {
        kit_sbx(kit)
            .map(Sbx::Kit)
            .or_else(|| find_pc_sbx(host).map(Sbx::Pc))
    }

    pub fn path(&self) -> &Path {
        match self {
            Sbx::Kit(p) | Sbx::Pc(p) => p,
        }
    }

    /// The microVM's shim beside the kit's `sbx` (`libexec`), which
    /// AppArmor must let create user namespaces (Linux); none for the PC's,
    /// whose package installs its profile.
    fn shim(&self) -> Option<PathBuf> {
        match self {
            Sbx::Kit(p) => Some(
                p.parent()?
                    .parent()?
                    .join("libexec")
                    .join("containerd-shim-nerdbox-v1"),
            ),
            Sbx::Pc(_) => None,
        }
    }

    /// A command of this `sbx`, with its environment ([`Sbx::session_env`]).
    pub fn command(&self, env: &KitEnv, host: &Host) -> std::process::Command {
        crate::platform::jobs::kit_command(self.path(), &self.session_env(env, host))
    }

    /// The environment every run of this `sbx` gets, commands and sessions
    /// of the app's terminal alike, so its sign-in is the one the status
    /// asks: the PC's for the PC's; the kit's for the kit's, but with the
    /// PC's temporary folder. The daemon puts there the junction that
    /// shortens its sockets' paths (D-167), and the kit's (`data\tmp`, deep
    /// in a kit folder) would leave them over the limit of 107 bytes.
    pub fn session_env(&self, env: &KitEnv, host: &Host) -> KitEnv {
        let mut out = env.clone();
        match self {
            Sbx::Kit(_) => {
                if let Some(temp) = host.temp() {
                    for name in ["TEMP", "TMP", "TMPDIR"] {
                        out.set(name, temp);
                    }
                }
            }
            Sbx::Pc(_) => {
                let names: Vec<String> = out.iter().map(|(k, _)| k.to_owned()).collect();
                for name in names {
                    out.remove(&name);
                }
                for (name, value) in host.iter() {
                    out.set(name, value);
                }
            }
        }
        out
    }
}

/// What the notice's «Lancia» runs in the app's terminal: one of these,
/// never a text of the interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fix {
    /// `sbx login`: the browser opens for the Docker account.
    Login,
    /// `sbx ls`: shows the error `sbx` answers with.
    Ls,
    /// `sbx policy init balanced`, for the PC's `sbx` ([`Blocker::PolicyUnset`]).
    Policy,
}

impl Fix {
    pub fn args(self) -> Vec<String> {
        let args: &[&str] = match self {
            Fix::Login => &["login"],
            Fix::Ls => &["ls"],
            Fix::Policy => &vm::POLICY_ARGS,
        };
        args.iter().map(|a| (*a).to_owned()).collect()
    }
}

/// Runs `fix` with the `sbx` found in a new session of the app's terminal,
/// in `dir`; `None` without one.
pub fn fix_launch(
    kit: &Kit,
    host: &Host,
    env: &KitEnv,
    fix: Fix,
    dir: &Path,
) -> Option<(crate::editor::pty::Launch, KitEnv)> {
    let sbx = Sbx::find(kit, host)?;
    let launch = crate::editor::pty::program(kit, sbx.path(), fix.args(), dir, "sbx");
    Some((launch, sbx.session_env(env, host)))
}

/// The component of the kit's `sbx`: required, but the editor and the
/// consoles never wait for it (the notice says it is missing).
pub const COMPONENT: &str = "sandboxes";

/// The kit's `sbx` (`data/apps/<target>/sandboxes/bin/sbx`), when the
/// component is installed.
pub fn kit_sbx(kit: &Kit) -> Option<PathBuf> {
    let c = crate::setup::Catalog::get().find(COMPONENT)?;
    c.is_installed(kit)
        .then(|| c.key_paths(kit).into_iter().next())
        .flatten()
}

/// The folders a PC's `sbx` is looked for in: the PC's `PATH`, then where
/// its installers put it (WinGet's links for the user, Homebrew, the
/// Ubuntu package).
pub(crate) fn sbx_dirs(os: Os, host: &Host) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = host
        .get("PATH")
        .unwrap_or_default()
        .split(os.path_sep())
        .filter(|d| !d.trim().is_empty())
        .map(|d| PathBuf::from(d.trim().trim_matches('"')))
        .collect();
    let extra: Vec<PathBuf> = if os.is_windows() {
        let local = host.get("LOCALAPPDATA").map(PathBuf::from);
        let programs = host.get("ProgramFiles").map(PathBuf::from);
        [
            local.map(|l| l.join("Microsoft").join("WinGet").join("Links")),
            programs.map(|p| p.join("Docker").join("Sandboxes")),
        ]
        .into_iter()
        .flatten()
        .collect()
    } else {
        ["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"]
            .into_iter()
            .map(PathBuf::from)
            .collect()
    };
    for d in extra {
        if !dirs.contains(&d) {
            dirs.push(d);
        }
    }
    dirs
}

fn find_pc_sbx(host: &Host) -> Option<PathBuf> {
    let os = crate::platform::OS;
    let name = format!("sbx{}", os.exe_suffix());
    sbx_dirs(os, host)
        .into_iter()
        .map(|d| d.join(&name))
        .find(|p| p.is_file())
}

/// Runs `sbx` with `args`, its output once it ends or after `limit`.
fn run(
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    args: &[&str],
    limit: Duration,
) -> Result<std::process::Output, String> {
    let mut cmd = sbx.command(env, host);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = crate::platform::jobs::spawn(&mut cmd).map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    match rx.recv_timeout(limit) {
        Ok(Ok(out)) => Ok(out),
        Ok(Err(e)) => Err(e.to_string()),
        Err(_) => Err(format!("sbx: no answer in {} s", limit.as_secs())),
    }
}

/// `sbx ls --json`: answers only when the daemon runs and the user is
/// signed in.
fn ask(sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    match run(sbx, host, env, &["ls", "--json"], ASK_LIMIT) {
        Ok(out) => answer(out.status.success(), &String::from_utf8_lossy(&out.stderr)),
        Err(message) => Err(Blocker::SbxFailed { message }),
    }
}

/// The global network policy no VM starts without: the kit's `sbx` gets
/// `balanced` (already set is fine); the PC's is the user's, only told.
fn policy(sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    let args: &[&str] = match sbx {
        Sbx::Kit(_) => &vm::POLICY_ARGS,
        Sbx::Pc(_) => &["policy", "ls"],
    };
    match run(sbx, host, env, args, ASK_LIMIT) {
        Ok(out) => policy_answer(
            sbx,
            out.status.success(),
            &format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
        ),
        Err(message) => Err(Blocker::SbxFailed { message }),
    }
}

/// What `sbx policy init` (the kit's) or `sbx policy ls` (the PC's) says.
pub(crate) fn policy_answer(sbx: &Sbx, success: bool, text: &str) -> Result<(), Blocker> {
    let lower = text.to_lowercase();
    if lower.contains("already initialized") {
        return Ok(());
    }
    if lower.contains("not been initialized") || lower.contains("not initialized") {
        return Err(Blocker::PolicyUnset);
    }
    if success {
        return Ok(());
    }
    match sbx {
        // A list that cannot be read says nothing of the VM: `sbx ls` answered.
        Sbx::Pc(_) => Ok(()),
        Sbx::Kit(_) => answer(false, text),
    }
}

/// How long the start of a sandbox may take: the first one pulls its
/// image.
const START_LIMIT: Duration = Duration::from_secs(600);

/// A project's microVM, when the level is the VM (`vm_for`).
#[derive(Debug, Clone)]
pub struct Vm {
    sbx: Sbx,
    host: Host,
    /// The sandbox's name (`vm::name`).
    pub name: String,
    /// The project's folder, its workspace.
    pub dir: PathBuf,
    /// Folders of the PC mounted where the VM cannot change them (the
    /// kit's tools for Linux, for the kit's terminals).
    pub read_only: Vec<PathBuf>,
}

/// The microVM of `project` (folder `dir`) when every project runs in one
/// (the level of the last status); `None` on the kit's environment.
pub fn vm_for(kit: &Kit, host: &Host, env: &KitEnv, project: &str, dir: &Path) -> Option<Vm> {
    if status(kit, host, env, false).level != Level::Vm {
        return None;
    }
    Some(Vm {
        sbx: Sbx::find(kit, host)?,
        host: host.clone(),
        name: vm::name(project),
        dir: dir.to_path_buf(),
        read_only: Vec::new(),
    })
}

/// The microVM of the kit's terminal `terminal` in `dir` when the VM is
/// on (the level of the last status): `dir` as its workspace and the kit's
/// tools for Linux (`apps`, `data/apps/linux-…`) read-only; `None` on the
/// kit's environment.
pub fn kit_terminal_vm(
    kit: &Kit,
    host: &Host,
    env: &KitEnv,
    terminal: &str,
    dir: &Path,
    apps: &Path,
) -> Option<Vm> {
    if status(kit, host, env, false).level != Level::Vm {
        return None;
    }
    Some(Vm {
        sbx: Sbx::find(kit, host)?,
        host: host.clone(),
        name: vm::terminal_name(terminal, dir),
        dir: dir.to_path_buf(),
        read_only: vec![apps.to_path_buf()],
    })
}

/// True when the VM is on (the level of the last status).
pub fn vm_on(kit: &Kit, host: &Host, env: &KitEnv) -> bool {
    status(kit, host, env, false).level == Level::Vm
}

/// `dir` as a VM's writable workspace, when it may be one: its canonical
/// path, the one to mount, so the folder checked is the folder the VM gets.
/// What the VM writes there the PC must never run, so it is refused:
///
/// - when it or a folder above it is a link or a junction;
/// - a drive's root;
/// - anything in the kit or in the user's profile, or holding either (the
///   kit's programs and launchers, the profile's start files, PowerShell
///   profiles, `.ssh`, the agents' settings…, and the tools mounted
///   read-only, reachable again through it), except the kit's `projects`;
/// - the system's folders (Windows, Program Files, ProgramData; `/etc`,
///   `/usr`…), or one holding them.
///
/// Elsewhere (`D:\Siti\shop`) it may.
pub fn vm_workspace(kit: &Kit, host: &Host, dir: &Path) -> Option<PathBuf> {
    if !dir.is_absolute() || dir.ancestors().any(crate::platform::safe_path::is_link) {
        return None;
    }
    let canon = |p: &Path| std::fs::canonicalize(p).ok().map(|p| plain(&p));
    let dir = canon(dir)?;
    dir.parent()?;
    if canon(&kit.projects()).is_some_and(|p| dir.starts_with(p)) {
        return Some(dir);
    }
    let profile = host
        .get("CP_USER_PROFILE")
        .or(host.get("USERPROFILE"))
        .or(host.get("HOME"))
        .map(PathBuf::from);
    let mut guarded: Vec<PathBuf> = [Some(kit.root().to_path_buf()), profile]
        .into_iter()
        .flatten()
        .collect();
    if kit.target().is_windows() {
        guarded.extend(
            [
                "SystemRoot",
                "ProgramFiles",
                "ProgramFiles(x86)",
                "ProgramData",
            ]
            .iter()
            .filter_map(|v| host.get(v))
            .map(PathBuf::from),
        );
    } else {
        guarded.extend(
            [
                "/bin",
                "/boot",
                "/etc",
                "/lib",
                "/lib64",
                "/opt",
                "/sbin",
                "/usr",
                "/var",
                "/Applications",
                "/Library",
                "/System",
            ]
            .iter()
            .map(PathBuf::from),
        );
    }
    let near = guarded
        .iter()
        .filter_map(|p| canon(p))
        .any(|p| dir.starts_with(&p) || p.starts_with(&dir));
    (!near).then_some(dir)
}

/// A canonical Windows path without its `\\?\` (`D:\a`, as `sbx` and the
/// user write it); a network one (`\\?\UNC\…`) and the others as they are.
fn plain(path: &Path) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => path.to_path_buf(),
    }
}

/// The kit's tools for the VM's Linux in the kit's terminal `terminal`:
/// the `PATH` entries and the variables of `guest` (the kit scoped to the
/// VM's target, [`vm::guest_target`]) that lie in its `data/apps`, as the
/// VM sees them. The rest of the kit's environment (its home, temp,
/// agents) is the PC's business: the VM has its own.
pub fn guest_tools(
    kit: &Kit,
    guest: &Kit,
    terminal: &crate::editor::terminals::Terminal,
    base: KitEnv,
) -> (Vec<String>, Vec<(String, String)>) {
    let windows = kit.target().is_windows();
    let apps = guest.apps();
    let env = crate::editor::terminals::env(guest, terminal, base);
    let in_vm = |value: &str| -> Option<String> {
        let value = if windows {
            value.replace('/', "\\")
        } else {
            value.to_owned()
        };
        let path = PathBuf::from(value);
        path.starts_with(&apps)
            .then(|| vm::guest_path(windows, &path))
            .flatten()
    };
    let path = env.path_entries().into_iter().filter_map(in_vm).collect();
    let mut vars: Vec<(String, String)> = env
        .iter()
        .filter(|(name, _)| !name.eq_ignore_ascii_case("PATH"))
        .filter_map(|(name, value)| Some((name.to_owned(), in_vm(value)?)))
        .collect();
    vars.push(("ATOLLO_TERMINAL".to_owned(), terminal.name.clone()));
    (path, vars)
}

impl Vm {
    /// A command of `sbx` with `args`, with its environment.
    pub fn command(&self, env: &KitEnv, args: &[String]) -> std::process::Command {
        let mut cmd = self.sbx.command(env, &self.host);
        cmd.args(args);
        cmd
    }

    /// Runs `sbx` with `args` to its end; its error text when it fails.
    fn call(&self, env: &KitEnv, args: &[String], limit: Duration) -> anyhow::Result<()> {
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, limit).map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx {}: {}",
            args.first().copied().unwrap_or_default(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
        Ok(())
    }

    /// Creates the sandbox the first time, starts it again when stopped.
    pub fn start(&self, env: &KitEnv) -> anyhow::Result<()> {
        self.call(
            env,
            &vm::start_args(&self.name, &self.dir, &self.read_only),
            START_LIMIT,
        )
    }

    /// Publishes `port` of the VM on the PC's `127.0.0.1`.
    pub fn publish(&self, env: &KitEnv, port: u16) -> anyhow::Result<()> {
        self.call(env, &vm::publish_args(&self.name, port), ASK_LIMIT)
    }

    /// The session of the app's terminal for `args` (`vm::terminal_args`,
    /// `vm::agent_args`), titled `what`.
    pub fn launch(&self, kit: &Kit, args: Vec<String>, what: &str) -> crate::editor::pty::Launch {
        crate::editor::pty::program(kit, self.sbx.path(), args, &self.dir, what)
    }

    /// The environment of that session ([`Sbx::session_env`]).
    pub fn session_env(&self, env: &KitEnv) -> KitEnv {
        self.sbx.session_env(env, &self.host)
    }

    /// Runs the shell `line` in the VM to its end (a container started
    /// in the background).
    pub fn run_line(&self, env: &KitEnv, line: &str) -> anyhow::Result<()> {
        self.call(
            env,
            &vm::exec_args(&self.name, None, line, &[]),
            START_LIMIT,
        )
    }
}

/// The PHP line of `project` in its VM: its own choice, the kit's, or
/// `8.4` (the official images have every line the kit offers).
pub fn php_line(kit: &Kit, project: &str) -> String {
    crate::versions::project_choice(kit, project)
        .get("php")
        .cloned()
        .or_else(|| crate::versions::kit_choice(kit).get("php").cloned())
        .unwrap_or_else(|| "8.4".to_owned())
}

/// The line that starts the project's MySQL in its VM, when the project
/// has a MySQL user: its version (or `8.4`), its user and password, on the
/// port its `DB_PORT` names.
pub fn mysql_start_line(kit: &Kit, project: &str) -> Option<String> {
    let id = crate::db::DbEngine::MySql.id();
    let password = crate::db::auth::try_project_password(kit, project, id).ok()??;
    let user = crate::db::auth::project_db_name(kit, project, id).ok()?;
    let port = crate::db::services::project_db_port(kit, project)?;
    let line =
        crate::db::services::project_line(kit, project, id).unwrap_or_else(|| "8.4".to_owned());
    Some(vm::mysql_line(&line, &user, &password, port))
}

/// Stops the daemon of the kit's `sbx` and the microVMs it runs, at the
/// app's exit and before the drive is removed: the PC's own daemon is the
/// user's, never stopped.
pub fn stop_daemon(kit: &Kit, env: &KitEnv) {
    let Some(exe) = kit_sbx(kit) else {
        return;
    };
    let sbx = Sbx::Kit(exe);
    match run(&sbx, &Host::default(), env, &["daemon", "stop"], ASK_LIMIT) {
        Ok(out) if out.status.success() => crate::common::log::info("sbx: daemon stopped"),
        Ok(out) => crate::common::log::warn(&format!(
            "sbx daemon stop: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
        Err(e) => crate::common::log::warn(&format!("sbx daemon stop: {e}")),
    }
}

/// What `sbx`'s exit and error text say: signed out, another error (its
/// last line), or ready.
pub(crate) fn answer(success: bool, stderr: &str) -> Result<(), Blocker> {
    if success {
        return Ok(());
    }
    let text = stderr.to_lowercase();
    if [
        "sbx login",
        "not logged in",
        "log in",
        "sign in",
        "not authenticated",
        "unauthorized",
    ]
    .iter()
    .any(|w| text.contains(w))
    {
        return Err(Blocker::SbxSignedOut);
    }
    let message = stderr
        .lines()
        .map(str::trim)
        .rfind(|l| !l.is_empty())
        .unwrap_or("sbx")
        .chars()
        .take(300)
        .collect();
    Err(Blocker::SbxFailed { message })
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/mod.rs"]
mod tests;
