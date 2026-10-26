//! How shut in a project's programs are (D-167), worked out by itself:
//! nothing to turn on in the settings. The VM (a Docker Sandboxes microVM
//! per project, `sbx`) when the PC has what it needs; otherwise the kit's
//! environment only, with the reasons the VM is off and, for each one, what
//! the user does about it (the interface's notice in the status bar).
//!
//! `sbx` is the kit's (the required `sandboxes` component, in `data`),
//! never one of the PC (D-175): it runs with the kit's environment, so its
//! sandboxes, settings and state travel with the kit. Its sign-in to
//! Docker stays in the system's keyring, a declared trace.
//!
//! On every level a project's programs get a sealed environment
//! ([`seal`]): less of what reaches them by itself. It is no wall (they run
//! as the user and can read what the user can): the VM is.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::platform::virt::{self, Gap};

pub mod daemon;
pub mod kit;
pub mod secrets;
pub mod session;
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
    /// The kit's Docker Sandboxes is not installed (one of the PC is
    /// never used, D-175); `failed`: why its last installation failed.
    SbxMissing {
        #[serde(skip_serializing_if = "Option::is_none")]
        failed: Option<String>,
    },
    /// `sbx` is there but not signed in to Docker.
    SbxSignedOut,
    /// `sbx` answered with an error (its text).
    SbxFailed {
        message: String,
    },
    /// The daemon running is another `sbx`'s (the PC's, another kit's):
    /// the kit's VMs would go into its state, out of the kit
    /// ([`daemon::Daemon::Other`]).
    ForeignDaemon,
    /// The kit's disk has no sparse files nor permissions (FAT32, exFAT):
    /// each VM's disks (20 + 10 GB, sparse on NTFS) would not fit or would
    /// take their whole size.
    KitDisk {
        fs: String,
    },
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

/// The last status and when it was worked out.
static LAST: Mutex<Option<(Instant, Status)>> = Mutex::new(None);
/// When the VM was last found ready ([`status`]'s grace).
static GOOD: Mutex<Option<Instant>> = Mutex::new(None);
/// One [`compute`] at a time.
static COMPUTE: Mutex<()> = Mutex::new(());

/// How long a VM found ready outlives an `sbx` that does not answer for a
/// moment (another VM pulling its image, starting).
const GRACE: Duration = Duration::from_secs(180);

fn last() -> std::sync::MutexGuard<'static, Option<(Instant, Status)>> {
    LAST.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The status, asked again when older than a minute or when `force` (the
/// notice's "Ricontrolla", an installation of Docker Sandboxes).
///
/// One check at a time, and nobody waits behind it while an older answer
/// exists: a check can take minutes (a daemon started, restarted). An
/// `sbx` that fails for a moment ([`Blocker::SbxFailed`] alone) within
/// [`GRACE`] of a VM found ready keeps the VM, asked again at the next
/// call: the programs stay in their VM, whose start then says what fails,
/// never on the PC without a word.
pub fn status(kit: &Kit, host: &Host, env: &KitEnv, force: bool) -> Status {
    let fresh = |force: bool| {
        last()
            .as_ref()
            .filter(|(at, _)| !force && at.elapsed() < FRESH)
            .map(|(_, st)| st.clone())
    };
    if let Some(st) = fresh(force) {
        return st;
    }
    let _one = match COMPUTE.try_lock() {
        Ok(guard) => guard,
        Err(std::sync::TryLockError::Poisoned(p)) => p.into_inner(),
        Err(std::sync::TryLockError::WouldBlock) => {
            if let Some((_, st)) = last().as_ref().filter(|_| !force) {
                return st.clone();
            }
            COMPUTE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        }
    };
    // Worked out meanwhile by the check this one waited for.
    if let Some(st) = fresh(force) {
        return st;
    }
    let st = compute(kit, host, env);
    let mut good = GOOD
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let transient = matches!(st.blockers.as_slice(), [Blocker::SbxFailed { .. }]);
    if st.level == Level::Vm {
        *good = Some(Instant::now());
    } else if transient && good.is_some_and(|at| at.elapsed() < GRACE) {
        crate::common::log::warn(&format!("sbx: {:?}: the VM kept", st.blockers));
        let kept = Status::of(Vec::new(), st.sbx.clone());
        // Not noted as fresh: the next call asks again.
        return kept;
    }
    *last() = Some((Instant::now(), st.clone()));
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
        None => blockers.push(Blocker::SbxMissing {
            failed: install_failure(kit),
        }),
        Some(sbx) => {
            if let Err(b) = kit_daemon(kit, sbx, host, env)
                .and_then(|()| ask(sbx, host, env))
                .and_then(|()| policy(sbx, host, env))
            {
                blockers.push(b);
            } else {
                daemon_settings(kit, sbx, host, env);
                legacy_global_github(kit, sbx, host, env);
            }
        }
    }
    Status::of(blockers, sbx.map(|s| s.path().display().to_string()))
}

/// Where the kit's `sbx` keeps its VMs (D-169): in the kit, so on a disk
/// with sparse files, and with a daemon of its own. The PC's `sbx` keeps
/// its own where the user installed it: nothing to check.
fn kit_daemon(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    let exe = sbx.path();
    if let Some(volume) = kit.volume().filter(|v| !v.fs.has_permissions()) {
        return Err(Blocker::KitDisk {
            fs: volume.fs.as_str().to_owned(),
        });
    }
    match daemon::current(kit, exe) {
        daemon::Daemon::Other => Err(Blocker::ForeignDaemon),
        daemon::Daemon::Kit => Ok(()),
        daemon::Daemon::None => {
            // Started here, its output to nothing: a daemon started by a
            // command whose output is read keeps that pipe open, and the
            // reader waits for it until its limit.
            // A daemon started now reads the settings as they are: a
            // restart owed to the one before is not owed any more.
            if let Some(temp) = host.temp() {
                daemon::unlink_foreign(Path::new(temp), &kit.home());
            }
            if daemon_call(sbx, host, env, &["daemon", "start"]) {
                let _ = std::fs::remove_file(kit::root(kit).join(".restart-pending"));
            }
            Ok(())
        }
    }
}

/// `sbx <args>` of the daemon (`daemon start`, `daemon restart`), its
/// output to nothing, waited for at most a minute.
fn daemon_call(sbx: &Sbx, host: &Host, env: &KitEnv, args: &[&str]) -> bool {
    let mut cmd = sbx.command(env, host);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let child = match crate::platform::jobs::spawn(&mut cmd) {
        Ok(child) => child,
        Err(e) => {
            crate::common::log::warn(&format!("sbx {}: {e}", args.join(" ")));
            return false;
        }
    };
    let (tx, rx) = mpsc::channel();
    let mut child = child;
    std::thread::spawn(move || {
        let _ = tx.send(child.wait());
    });
    match rx.recv_timeout(Duration::from_secs(60)) {
        Ok(Ok(status)) if status.success() => true,
        Ok(Ok(status)) => {
            crate::common::log::warn(&format!("sbx {}: {status}", args.join(" ")));
            false
        }
        Ok(Err(e)) => {
            crate::common::log::warn(&format!("sbx {}: {e}", args.join(" ")));
            false
        }
        Err(_) => {
            crate::common::log::warn(&format!("sbx {}: no end in 60 s", args.join(" ")));
            false
        }
    }
}

/// The kit daemon's settings ([`daemon::SETTINGS`]): set when they differ,
/// then the daemon restarted for them when no sandbox runs (a running one
/// would stop; the next status tries again). Failures only go to the log:
/// the VMs work without them.
fn daemon_settings(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) {
    // A restart still owed (a setting changed while a sandbox ran) stays
    // owed across checks and starts of the app: `settings get` already
    // answers the new value, the daemon running does not have it yet.
    let pending = kit::root(kit).join(".restart-pending");
    for (key, value) in daemon::SETTINGS {
        let now = run(sbx, host, env, &["settings", "get", key], ASK_LIMIT)
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
        if now.as_deref() == Ok(value) {
            continue;
        }
        match run(sbx, host, env, &["settings", "set", key, value], ASK_LIMIT) {
            Ok(out) if out.status.success() => {
                let _ = std::fs::create_dir_all(kit::root(kit));
                if let Err(e) = std::fs::write(&pending, key) {
                    crate::common::log::warn(&format!("{}: {e}", pending.display()));
                }
            }
            Ok(out) => crate::common::log::warn(&format!(
                "sbx settings set {key}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => crate::common::log::warn(&format!("sbx settings set {key}: {e}")),
        }
    }
    if !pending.exists() {
        return;
    }
    let running = run(sbx, host, env, &["ls", "--json"], ASK_LIMIT)
        .map(|o| sandboxes_running(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(true);
    if running {
        crate::common::log::info("sbx settings changed: restart when no sandbox runs");
        return;
    }
    if daemon_call(sbx, host, env, &["daemon", "restart"]) {
        let _ = std::fs::remove_file(&pending);
    }
}

/// True while the daemon still has to restart for a setting of
/// [`daemon::SETTINGS`] (a sandbox ran when it changed): no sandbox starts
/// until then, so none runs without it.
pub fn restart_pending(kit: &Kit) -> bool {
    kit::root(kit).join(".restart-pending").exists()
}

/// The PC's name, to note per PC what Atollo registered in `sbx`:
/// `HOSTNAME` is seldom exported on macOS and Linux, so the system's own
/// name next; "pc" only when nothing answers (then every such PC shares
/// the notes).
fn computer(host: &Host) -> String {
    host.get("COMPUTERNAME")
        .or(host.get("HOSTNAME"))
        .map(str::to_owned)
        .or_else(|| Some(crate::platform::host::host_name()).filter(|n| !n.is_empty()))
        .unwrap_or_else(|| LEGACY_PC.to_owned())
}

/// The name [`computer`] gave on macOS and Linux before it read the
/// system's own.
const LEGACY_PC: &str = "pc";

/// What Atollo noted for GitHub on `pc` for `sandbox`, a note under
/// [`LEGACY_PC`] moved to `pc` first: left there it would never be found,
/// and its registration never taken back. Moved only when `listed_here`
/// (this PC's `sbx` has that secret): a kit carried between PCs without
/// a name shared that key, and another PC's note stays its own.
fn mine_github(
    kit: &Kit,
    pc: &str,
    sandbox: &str,
    listed_here: impl FnOnce() -> bool,
) -> Option<secrets::Noted> {
    if let Some(n) = secrets::mine(kit, "github", pc, sandbox) {
        return Some(n);
    }
    if pc == LEGACY_PC {
        return None;
    }
    let old = secrets::mine(kit, "github", LEGACY_PC, sandbox)?;
    if !listed_here() {
        return None;
    }
    let moved = secrets::note(kit, "github", pc, sandbox, Some(&old.command))
        .and_then(|()| secrets::note(kit, "github", LEGACY_PC, sandbox, None));
    if let Err(e) = moved {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
    }
    Some(old)
}

/// The kit's GitHub access for the sandbox `sandbox` ([`secrets`], D-172),
/// that one only (`--sandbox`), never global: given when `wanted` (the
/// project allowed it) and the kit is signed in to GitHub; taken back when
/// not wanted any more and Atollo had given it. A secret Atollo did not
/// register (the user's own) is never touched. Failures only go to the
/// log: the VM works without it.
pub(crate) fn github_for_sandbox(
    kit: &Kit,
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    sandbox: &str,
    wanted: bool,
) {
    let Some(exe) = crate::kit::app_exe() else {
        return;
    };
    let prefix = secrets::github_prefix(&exe, kit.target().is_windows());
    let pc = computer(host);
    let listed_here = || {
        run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
            o.status.success()
                && secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", sandbox)
        })
    };
    let mine = mine_github(kit, &pc, sandbox, listed_here);
    let warn = |what: &str, out: Result<std::process::Output, String>| match out {
        Ok(o) if o.status.success() => true,
        Ok(o) => {
            crate::common::log::warn(&format!(
                "sbx secret {what} github {sandbox}: {}",
                String::from_utf8_lossy(&o.stderr).trim()
            ));
            false
        }
        Err(e) => {
            crate::common::log::warn(&format!("sbx secret {what} github {sandbox}: {e}"));
            false
        }
    };
    let remove = || {
        let args = ["secret", "rm", "github", "--sandbox", sandbox, "--force"];
        warn("rm", run(sbx, host, env, &args, ASK_LIMIT))
    };
    if !wanted {
        // A note `sbx` no longer lists (removed by hand, the sandbox gone)
        // goes too: its nonce must not open `secret github` any more.
        let gone = || {
            run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
                o.status.success()
                    && !secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", sandbox)
            })
        };
        if mine.is_some() && (gone() || remove()) {
            if let Err(e) = secrets::note(kit, "github", &pc, sandbox, None) {
                crate::common::log::warn(&format!("secrets.json: {e:#}"));
            }
        }
        return;
    }
    let Ok(ls) = run(sbx, host, env, &["secret", "ls"], ASK_LIMIT) else {
        return;
    };
    let listed = secrets::listed(&String::from_utf8_lossy(&ls.stdout), "github", sandbox);
    let step = secrets::step(listed, mine.as_ref(), &prefix);
    if step == secrets::Step::Keep || crate::github::token(kit, env).is_none() {
        return;
    }
    if step == secrets::Step::Replace && !remove() {
        return;
    }
    // A nonce of its own: `secret github` answers only a command that
    // carries one Atollo noted.
    let Ok(nonce) = secrets::new_nonce() else {
        return;
    };
    let command = secrets::github_command(&exe, kit.target().is_windows(), &nonce);
    // Noted first: `sbx` runs the command already to verify it.
    if let Err(e) = secrets::note(kit, "github", &pc, sandbox, Some(&command)) {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
        return;
    }
    let args = [
        "secret",
        "set",
        "github",
        "--sandbox",
        sandbox,
        "--command",
        command.as_str(),
    ];
    if !warn("set", run(sbx, host, env, &args, ASK_LIMIT)) {
        // Not registered: its nonce opens nothing.
        if let Err(e) = secrets::note(kit, "github", &pc, sandbox, None) {
            crate::common::log::warn(&format!("secrets.json: {e:#}"));
        }
    }
}

/// A global GitHub secret a first version of Atollo may have registered on
/// this PC (a note without a sandbox) is taken back: GitHub goes only to
/// the projects that chose it (D-172).
fn legacy_global_github(kit: &Kit, sbx: &Sbx, host: &Host, env: &KitEnv) {
    let pc = computer(host);
    let listed_here = || {
        run(sbx, host, env, &["secret", "ls"], ASK_LIMIT).is_ok_and(|o| {
            o.status.success()
                && secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", "(global)")
        })
    };
    if mine_github(kit, &pc, "", listed_here).is_none() {
        return;
    }
    let listed = run(sbx, host, env, &["secret", "ls"], ASK_LIMIT)
        .map(|o| secrets::listed(&String::from_utf8_lossy(&o.stdout), "github", "(global)"))
        .unwrap_or(false);
    if listed {
        let args = ["secret", "rm", "github", "--force"];
        match run(sbx, host, env, &args, ASK_LIMIT) {
            Ok(o) if o.status.success() => {}
            _ => {
                crate::common::log::warn(
                    "sbx secret rm github (global): failed, tried again later",
                );
                return;
            }
        }
    }
    if let Err(e) = secrets::note(kit, "github", &pc, "", None) {
        crate::common::log::warn(&format!("secrets.json: {e:#}"));
    }
}

/// True when `sbx ls --json` lists a running sandbox.
pub(crate) fn sandboxes_running(json: &str) -> bool {
    serde_json::from_str::<Vec<serde_json::Value>>(json)
        .map(|list| list.iter().any(|s| s["status"] == "running"))
        .unwrap_or(true)
}

/// The `sbx` the projects' microVMs run with: the kit's own (the
/// `sandboxes` component), never one of the PC (D-175: nothing of a
/// project's box leaves the kit). It runs with the kit's environment, so
/// its sandboxes, settings and state are in the kit and travel with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sbx(PathBuf);

impl Sbx {
    /// The kit's, when installed.
    pub fn find(kit: &Kit, _host: &Host) -> Option<Sbx> {
        kit_sbx(kit).map(Sbx)
    }

    /// The kit's `sbx` at `path` (the tests).
    #[cfg(test)]
    pub(crate) fn at(path: PathBuf) -> Sbx {
        Sbx(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// The microVM's shim beside the kit's `sbx` (`libexec`), which
    /// AppArmor must let create user namespaces (Linux).
    fn shim(&self) -> Option<PathBuf> {
        Some(
            self.0
                .parent()?
                .parent()?
                .join("libexec")
                .join("containerd-shim-nerdbox-v1"),
        )
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
        if let Some(temp) = host.temp() {
            for name in ["TEMP", "TMP", "TMPDIR"] {
                out.set(name, temp);
            }
        }
        for (name, value) in daemon::ENV {
            out.set(name, value);
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
}

impl Fix {
    pub fn args(self) -> Vec<String> {
        let args: &[&str] = match self {
            Fix::Login => &["login"],
            Fix::Ls => &["ls"],
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
    let mut env = sbx.session_env(env, host);
    if fix == Fix::Login {
        // The sign-in page is not opened by `sbx`, whose environment is the
        // kit's (its browser would make its profile in the kit): its link
        // is in the terminal, and a click opens it with the PC's (D-169).
        env.set("SBX_NO_BROWSER_OPEN", "1");
    }
    Some((launch, env))
}

/// The component of the kit's `sbx`: required, but the editor and the
/// consoles never wait for it (the notice says it is missing).
pub const COMPONENT: &str = "sandboxes";

fn install_failure_file(kit: &Kit) -> PathBuf {
    kit::root(kit).join("install-failed.txt")
}

/// Notes the outcome of an installation of [`COMPONENT`] (`setup::runner`):
/// a failure's message, for the notice to say why the VM is still missing;
/// taken away once it is installed.
pub fn note_install(kit: &Kit, outcome: &crate::setup::runner::Outcome) {
    use crate::setup::runner::Outcome;
    let file = install_failure_file(kit);
    let done = match outcome {
        Outcome::Failed { message, .. } => {
            std::fs::create_dir_all(kit::root(kit)).and_then(|()| std::fs::write(&file, message))
        }
        Outcome::Installed { .. } | Outcome::UpToDate { .. } => match std::fs::remove_file(&file) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
            _ => Ok(()),
        },
        _ => Ok(()),
    };
    if let Err(e) = done {
        crate::common::log::warn(&format!("{}: {e}", file.display()));
    }
}

/// Why the last installation of [`COMPONENT`] failed, if it did.
pub fn install_failure(kit: &Kit) -> Option<String> {
    std::fs::read_to_string(install_failure_file(kit))
        .ok()
        .map(|t| t.trim().to_owned())
        .filter(|t| !t.is_empty())
}

/// The kit's `sbx` (`data/apps/<target>/sandboxes/bin/sbx`), when the
/// component is installed.
pub fn kit_sbx(kit: &Kit) -> Option<PathBuf> {
    let c = crate::setup::Catalog::get().find(COMPONENT)?;
    c.is_installed(kit)
        .then(|| c.key_paths(kit).into_iter().next())
        .flatten()
}

/// Runs `sbx` with `args`, its output once it ends; after `limit` it is
/// ended, never left half way (an `sbx run -d` still making a sandbox).
fn run(
    sbx: &Sbx,
    host: &Host,
    env: &KitEnv,
    args: &[&str],
    limit: Duration,
) -> Result<std::process::Output, String> {
    use std::io::Read;
    let mut cmd = sbx.command(env, host);
    cmd.args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = crate::platform::jobs::spawn(&mut cmd).map_err(|e| e.to_string())?;
    // Both pipes read while it runs: a full pipe would stop it.
    let read = |pipe: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            if let Some(mut p) = pipe {
                let _ = p.read_to_end(&mut buf);
            }
            buf
        })
    };
    let stdout = read(
        child
            .stdout
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let stderr = read(
        child
            .stderr
            .take()
            .map(|p| Box::new(p) as Box<dyn Read + Send>),
    );
    let until = Instant::now() + limit;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if Instant::now() < until => std::thread::sleep(Duration::from_millis(25)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("sbx: no answer in {} s", limit.as_secs()));
            }
            Err(e) => return Err(e.to_string()),
        }
    };
    Ok(std::process::Output {
        status,
        stdout: stdout.join().unwrap_or_default(),
        stderr: stderr.join().unwrap_or_default(),
    })
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
/// `balanced` (already set is fine).
fn policy(sbx: &Sbx, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    match run(sbx, host, env, &vm::POLICY_ARGS, ASK_LIMIT) {
        Ok(out) => policy_answer(
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

/// What `sbx policy init` says: done, or set already, is fine.
pub(crate) fn policy_answer(success: bool, text: &str) -> Result<(), Blocker> {
    if success || text.to_lowercase().contains("already initialized") {
        return Ok(());
    }
    answer(false, text)
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
    /// The kit the sandbox's own kit is written in (`data\app\sandboxes`).
    kit: Kit,
    /// Whose sandbox it is, written in its kit.
    owner: kit::Owner,
    /// The kit's tools a session gets ([`Vm::with_tools`]).
    tools: GuestTools,
    /// The PC's ports it may reach ([`Vm::with_host_ports`]).
    host_ports: Vec<u16>,
    /// What `sbx` runs in it: `shell`, or an agent of `sbx`
    /// ([`Vm::for_agent`]).
    agent: String,
}

/// A sandbox of Atollo, for the list of the VMs (D-169).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmInfo {
    pub name: String,
    /// Whose it is, from its kit; `None` for an agent's sandbox (`sbx`'s
    /// own kit) or one made before the kits.
    pub owner: Option<kit::Owner>,
    /// `running`, `stopped`… as `sbx` says it.
    pub status: String,
    pub last_used: Option<String>,
}

/// The sandboxes of Atollo and the room their state takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VmList {
    pub vms: Vec<VmInfo>,
    /// Bytes the kit daemon's state takes on the disk (images, the VMs'
    /// disks, sparse): `None` before the kit's daemon ever ran.
    pub disk: Option<u64>,
}

/// Atollo's sandboxes in `sbx ls --json` (`atollo-…`), with their owner
/// from their kits.
pub(crate) fn vms_of(kit: &Kit, json: &str) -> Vec<VmInfo> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    list.iter()
        .filter_map(|s| {
            let name = s["name"].as_str()?;
            name.starts_with(vm::PREFIX).then(|| VmInfo {
                name: name.to_owned(),
                owner: kit::owner(kit, name),
                status: s["status"].as_str().unwrap_or_default().to_owned(),
                last_used: s["last_used_at"].as_str().map(str::to_owned),
            })
        })
        .collect()
}

/// The sandboxes of Atollo `sbx` has, and the room the kit's take.
pub fn vm_list(kit: &Kit, host: &Host, env: &KitEnv) -> anyhow::Result<VmList> {
    let Some(sbx) = Sbx::find(kit, host) else {
        return Ok(VmList {
            vms: Vec::new(),
            disk: None,
        });
    };
    let out = run(&sbx, host, env, &["ls", "--json"], ASK_LIMIT).map_err(anyhow::Error::msg)?;
    anyhow::ensure!(
        out.status.success(),
        "sbx ls: {}",
        String::from_utf8_lossy(&out.stderr).trim()
    );
    let disk = daemon::state_dir(kit).map(|d| crate::platform::files::allocated_size(&d));
    Ok(VmList {
        vms: vms_of(kit, &String::from_utf8_lossy(&out.stdout)),
        disk,
    })
}

/// The workspace of sandbox `name` in `sbx ls --json`: the first of its
/// `workspaces`.
pub(crate) fn workspace_of(json: &str, name: &str) -> Option<String> {
    let list: Vec<serde_json::Value> = serde_json::from_str(json).ok()?;
    list.iter()
        .find(|s| s["name"] == name)?
        .get("workspaces")?
        .get(0)?
        .as_str()
        .map(str::to_owned)
}

/// A name of one of Atollo's sandboxes: `atollo-` and `[a-z0-9.-]`.
pub fn is_atollos(name: &str) -> bool {
    name.strip_prefix(vm::PREFIX).is_some_and(|rest| {
        !rest.is_empty()
            && rest
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-')
    })
}

/// Removes the sandbox `name` of Atollo (`sbx rm -f`: its disk with it),
/// then its kit.
pub fn remove_vm(kit: &Kit, host: &Host, env: &KitEnv, name: &str) -> anyhow::Result<()> {
    anyhow::ensure!(is_atollos(name), "not a sandbox of Atollo: {name}");
    if let Some(sbx) = Sbx::find(kit, host) {
        // GitHub taken back first, if Atollo had given it (D-172).
        github_for_sandbox(kit, &sbx, host, env, name, false);
        let out =
            run(&sbx, host, env, &["rm", "-f", name], START_LIMIT).map_err(anyhow::Error::msg)?;
        let text = String::from_utf8_lossy(&out.stderr).to_lowercase();
        anyhow::ensure!(
            out.status.success() || text.contains("not found") || text.contains("no such"),
            "sbx rm: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    kit::remove(kit, name)
}

/// The sandboxes of `project`: its own and its agents'.
pub fn project_sandboxes(project: &str) -> Vec<String> {
    let mut names = vec![vm::name(project)];
    names.extend(["claude", "codex"].map(|a| vm::agent_name(project, a)));
    names
}

/// The sandboxes of the kit terminal `terminal`, one per folder, from
/// their kits.
pub fn terminal_sandboxes(kit: &Kit, terminal: &str) -> Vec<String> {
    kit::all(kit)
        .into_iter()
        .filter(
            |(_, owner)| matches!(owner, kit::Owner::Terminal { terminal: t, .. } if t == terminal),
        )
        .map(|(name, _)| name)
        .collect()
}

/// Removes `names` with their owner gone (a project deleted, a terminal
/// removed): a failure only goes to the log, the list of the VMs still
/// shows what is left.
pub fn remove_owned(kit: &Kit, host: &Host, env: &KitEnv, names: &[String]) {
    for name in names {
        if let Err(e) = remove_vm(kit, host, env, name) {
            crate::common::log::warn(&format!("sandbox {name}: {e:#}"));
        }
    }
}

/// The microVM of `project` (folder `dir`) when every project runs in one
/// (the level of the last status); `None` on the kit's environment.
pub fn vm_for(kit: &Kit, host: &Host, env: &KitEnv, project: &str, dir: &Path) -> Option<Vm> {
    if status(kit, host, env, false).level != Level::Vm {
        return None;
    }
    // The kit's tools for the VM's Linux, read-only (D-170).
    let read_only = vm::guest_target(kit.target())
        .map(|t| vec![kit.for_target(t).apps()])
        .unwrap_or_default();
    Some(Vm {
        sbx: Sbx::find(kit, host)?,
        host: host.clone(),
        name: vm::name(project),
        dir: dir.to_path_buf(),
        read_only,
        kit: kit.clone(),
        owner: kit::Owner::Project {
            project: project.to_owned(),
        },
        tools: GuestTools::default(),
        host_ports: Vec::new(),
        agent: "shell".into(),
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
        kit: kit.clone(),
        owner: kit::Owner::Terminal {
            terminal: terminal.to_owned(),
            dir: dir.display().to_string(),
        },
        tools: GuestTools::default(),
        host_ports: Vec::new(),
        agent: "shell".into(),
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
    // A project, or a folder in one; never the folder of every project,
    // which would put them all, writable, in one VM.
    if canon(&kit.projects()).is_some_and(|p| dir.starts_with(&p) && dir != p) {
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

/// What a session or an action in a VM gets: the kit's tools' `PATH`
/// entries (first, before the VM's own) and variables (`JAVA_HOME`…), as
/// the VM sees them, and a file of variables on the PC with what must not
/// be on a command line (the project's database login, D-171).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuestTools {
    pub path: Vec<String>,
    pub vars: Vec<(String, String)>,
    pub env_file: Option<PathBuf>,
}

/// `db` (a project's database on the kit's server, at `port`) as a
/// program in the VM reaches it: through [`vm::HOST`].
pub fn db_in_vm(db: &crate::db::auth::ProjectDb, port: u16) -> crate::db::auth::ProjectDb {
    let engine = crate::db::DbEngine::from_id(&db.engine).unwrap_or(crate::db::DbEngine::MySql);
    let url = engine
        .project_url(&db.database, &db.password, port)
        .replacen("@127.0.0.1:", &format!("@{}:", vm::HOST), 1);
    crate::db::auth::ProjectDb {
        host: vm::HOST.to_owned(),
        port,
        url,
        ..db.clone()
    }
}

/// The kit's tools for the VM's Linux in the kit's terminal `terminal`
/// ([`guest_view`]), with `ATOLLO_TERMINAL`.
pub fn guest_tools(
    kit: &Kit,
    guest: &Kit,
    terminal: &crate::editor::terminals::Terminal,
    base: KitEnv,
) -> GuestTools {
    let env = crate::editor::terminals::env(guest, terminal, base);
    let mut tools = guest_view(kit, guest, &env);
    tools
        .vars
        .push(("ATOLLO_TERMINAL".to_owned(), terminal.name.clone()));
    tools
}

/// The kit's tools for the VM's Linux in `project`, at the lines it chose
/// (D-170), as on the PC (`versions::project_env`).
pub fn project_guest_tools(kit: &Kit, guest: &Kit, project: &str, base: KitEnv) -> GuestTools {
    let env = crate::versions::project_env(guest, project, base);
    guest_view(kit, guest, &env)
}

/// The `PATH` entries and the variables of `env` (an environment of
/// `guest`, the kit scoped to the VM's target, [`vm::guest_target`]) that
/// lie in its `data/apps`, as the VM sees them. The rest of the kit's
/// environment (its home, temp, agents) is the PC's business: the VM has
/// its own.
pub fn guest_view(kit: &Kit, guest: &Kit, env: &KitEnv) -> GuestTools {
    let windows = kit.target().is_windows();
    let apps = guest.apps();
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
    let vars = env
        .iter()
        .filter(|(name, _)| !name.eq_ignore_ascii_case("PATH"))
        .filter_map(|(name, value)| Some((name.to_owned(), in_vm(value)?)))
        .collect();
    GuestTools {
        path,
        vars,
        env_file: None,
    }
}

impl Vm {
    /// The VM of `project` in `dir` with an `sbx` named `sbx` (the tests).
    #[cfg(test)]
    pub(crate) fn for_tests(kit: &Kit, project: &str, dir: &Path) -> Vm {
        Vm {
            sbx: Sbx(PathBuf::from("sbx")),
            host: Host::default(),
            name: vm::name(project),
            dir: dir.to_path_buf(),
            read_only: Vec::new(),
            kit: kit.clone(),
            owner: kit::Owner::Project {
                project: project.to_owned(),
            },
            tools: GuestTools::default(),
            host_ports: Vec::new(),
            agent: "shell".into(),
        }
    }

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
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::ensure!(
            out.status.success(),
            "sbx {}: {}",
            args.iter().take(2).copied().collect::<Vec<_>>().join(" "),
            stderr.trim()
        );
        Ok(())
    }

    /// Creates the sandbox the first time, starts it again when stopped.
    /// Writes the sandbox's kit (D-169), then: the first time the sandbox
    /// is made with it (`--kit`); after, a changed kit goes in with
    /// `sbx kit add` (its disk kept); then it starts again when stopped.
    ///
    /// A read-only folder an existing sandbox lacks (one made before the
    /// kit's tools were mounted, D-170) is mounted with `sbx mount`, its
    /// disk kept ([`Vm::mount_missing`]).
    pub fn start(&self, env: &KitEnv, lang: &str) -> anyhow::Result<()> {
        self.ready(lang)?;
        let (kit_dir, print) = kit::write(&self.kit, &self.name, &self.owner)?;
        let mounts = self.mounts();
        // One start per sandbox at a time: a terminal and an action of the
        // same project would both find it missing and both make it.
        let lock = sandbox_lock(&self.name);
        let _one = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut exists = self.exists(env)?;
        if exists && self.moved(env)? {
            self.make_again(env)?;
            exists = false;
        }
        if exists {
            self.mount_missing(env, lang)?;
            if kit::applied(&self.kit, &self.name).as_deref() != Some(print.as_str()) {
                let args = vec![
                    "kit".to_owned(),
                    "add".to_owned(),
                    self.name.clone(),
                    kit_dir.display().to_string(),
                ];
                self.call(env, &args, START_LIMIT)?;
                kit::mark_applied(&self.kit, &self.name, &print)?;
            }
            self.call(
                env,
                &vm::start_args(&self.name, &self.dir, &self.read_only, None, &self.agent),
                START_LIMIT,
            )?;
            self.sync_host_ports(env, false)?;
            self.sync_github(env);
            return Ok(());
        }
        self.call(
            env,
            &vm::start_args(
                &self.name,
                &self.dir,
                &self.read_only,
                Some(&kit_dir),
                &self.agent,
            ),
            START_LIMIT,
        )?;
        kit::mark_mounts(&self.kit, &self.name, &mounts)?;
        kit::mark_applied(&self.kit, &self.name, &print)?;
        self.sync_host_ports(env, true)?;
        self.sync_github(env);
        Ok(())
    }

    /// True when the sandbox of a project works in another folder than the
    /// project's: the kit moved (another drive letter, a folder renamed).
    /// `sbx` never changes a sandbox's workspace and refuses to start one
    /// whose folder is gone (tried with sbx 0.49.2). A kit terminal's
    /// sandbox is named after its folder: never moved.
    fn moved(&self, env: &KitEnv) -> anyhow::Result<bool> {
        if !matches!(self.owner, kit::Owner::Project { .. }) {
            return Ok(false);
        }
        let noted =
            kit::mounts(&self.kit, &self.name).and_then(|m| m.lines().next().map(str::to_owned));
        let workspace = match noted {
            Some(w) => w,
            None => match self.workspace(env)? {
                Some(w) => w,
                None => return Ok(false),
            },
        };
        Ok(crate::platform::processes::path_key(&workspace)
            != crate::platform::processes::path_key(&self.dir.display().to_string()))
    }

    /// Removes the sandbox of a project whose kit moved, to make it again
    /// in the project's folder (D-175): its disk holds nothing that is not
    /// in the kit (the project's files, its database, the tools and the
    /// sign-ins are all there), only the VM's home and the images it
    /// pulled. GitHub is taken back first (D-172), the PC's ports go with
    /// the sandbox (D-171).
    fn make_again(&self, env: &KitEnv) -> anyhow::Result<()> {
        crate::common::log::info(&format!(
            "{}: the kit moved, the sandbox is made again in {}",
            self.name,
            self.dir.display()
        ));
        github_for_sandbox(&self.kit, &self.sbx, &self.host, env, &self.name, false);
        self.call(
            env,
            &["rm".to_owned(), "-f".to_owned(), self.name.clone()],
            START_LIMIT,
        )?;
        kit::forget(&self.kit, &self.name)
    }

    /// GitHub for this sandbox as its project chose (`github_in_vm`, off
    /// unless turned on; never for a kit terminal), D-172.
    fn sync_github(&self, env: &KitEnv) {
        let wanted = match &self.owner {
            kit::Owner::Project { project } => {
                crate::projects::settings::load(&self.kit, project).github_in_vm
            }
            kit::Owner::Terminal { .. } => false,
        };
        github_for_sandbox(&self.kit, &self.sbx, &self.host, env, &self.name, wanted);
    }

    /// Mounts the read-only folders an existing sandbox lacks, after those
    /// noted at its making (`.mounts`; none noted: a sandbox made before,
    /// every one tried, one mounted already is fine), then notes them. The
    /// workspace never changes (the sandbox's name comes from it): another
    /// one noted is an error, never a sandbox made again, which would delete
    /// its disk.
    fn mount_missing(&self, env: &KitEnv, lang: &str) -> anyhow::Result<()> {
        let noted = kit::mounts(&self.kit, &self.name).unwrap_or_default();
        let lines: Vec<&str> = noted.lines().collect();
        // Nothing noted is nothing known: the workspace `sbx` has, checked,
        // never taken for this one.
        let workspace = match lines.first() {
            Some(w) => (*w).to_owned(),
            None => self.workspace(env)?.unwrap_or_default(),
        };
        let same = crate::platform::processes::path_key(&workspace)
            == crate::platform::processes::path_key(&self.dir.display().to_string());
        if !same {
            anyhow::bail!(crate::common::i18n::t(
                lang,
                "La sandbox {0} lavora in un'altra cartella: rimuovila in «Sandbox delle VM» (si cancella il suo disco, non i file del progetto) e riapri.",
                &[&self.name],
            ));
        }
        let windows = self.kit.target().is_windows();
        let mut changed = noted.is_empty();
        for folder in &self.read_only {
            let shown = folder.display().to_string();
            if lines.iter().skip(1).any(|l| *l == shown) {
                continue;
            }
            let guest = vm::guest_path(windows, folder)
                .ok_or_else(|| anyhow::anyhow!("{shown}: no path in the VM"))?;
            let args = vm::mount_args(&self.name, folder, &guest);
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            let out =
                run(&self.sbx, &self.host, env, &args, START_LIMIT).map_err(anyhow::Error::msg)?;
            let text = String::from_utf8_lossy(&out.stderr);
            anyhow::ensure!(
                out.status.success() || text.to_lowercase().contains("already"),
                "sbx mount: {}",
                text.trim()
            );
            changed = true;
        }
        if changed {
            kit::mark_mounts(&self.kit, &self.name, &self.mounts())?;
        }
        Ok(())
    }

    /// The workspace `sbx` has for this sandbox (`sbx ls --json`, the first
    /// of its `workspaces`), if any.
    fn workspace(&self, env: &KitEnv) -> anyhow::Result<Option<String>> {
        let out = run(&self.sbx, &self.host, env, &["ls", "--json"], ASK_LIMIT)
            .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx ls: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        Ok(workspace_of(
            &String::from_utf8_lossy(&out.stdout),
            &self.name,
        ))
    }

    /// The folders this sandbox mounts, one per line: the workspace, then
    /// the read-only ones.
    fn mounts(&self) -> String {
        std::iter::once(&self.dir)
            .chain(&self.read_only)
            .map(|p| format!("{}\n", p.display()))
            .collect()
    }

    /// The sandbox of `agent` (one of `sbx`'s, [`vm::sbx_agent`]) for this
    /// project's VM: its own (`atollo-a-`), the same folders, kit, tools,
    /// databases and ports as the project's (D-175); `None` for an agent
    /// `sbx` has none for.
    pub fn for_agent(mut self, project: &str, agent: &str) -> Option<Vm> {
        let id = vm::sbx_agent(agent)?;
        self.name = vm::agent_name(project, id);
        self.agent = id.to_owned();
        Some(self)
    }

    /// The session of the agent in this sandbox, started ([`Vm::start`]):
    /// `sbx run --name` with the database's variables and the kit's tools
    /// first on the `PATH` before the image's own, read in the sandbox.
    pub fn agent_session_args(&self, env: &KitEnv) -> anyhow::Result<Vec<String>> {
        let args = [
            "exec".to_owned(),
            self.name.clone(),
            "sh".to_owned(),
            "-c".to_owned(),
            r#"printf %s "$PATH""#.to_owned(),
        ];
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, ASK_LIMIT).map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx exec {}: {}",
            self.name,
            String::from_utf8_lossy(&out.stderr).trim()
        );
        let image_path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        Ok(vm::agent_session_args(&self.name, &image_path, &self.tools))
    }

    /// The kit's tools its sessions and actions get (D-170).
    pub fn with_tools(mut self, tools: GuestTools) -> Vm {
        self.tools = tools;
        self
    }

    pub fn tools(&self) -> &GuestTools {
        &self.tools
    }

    /// True when `sbx` has this sandbox.
    fn exists(&self, env: &KitEnv) -> anyhow::Result<bool> {
        let out = run(&self.sbx, &self.host, env, &["ls", "-q"], ASK_LIMIT)
            .map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            out.status.success(),
            "sbx ls: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
        Ok(String::from_utf8_lossy(&out.stdout)
            .lines()
            .any(|l| l.trim() == self.name))
    }

    /// Publishes `port` of the VM on the PC: on `127.0.0.1`, or on the
    /// local network when `lan` (Settings). The other address, left by a
    /// start with the other setting, is taken back first, and failing that
    /// nothing starts: with the local network turned off, the site must not
    /// stay on it (`--unpublish` of nothing published succeeds, so a
    /// failure is a real one). A port published already on the same
    /// address (an action that ended by itself) is fine: `sbx` answers
    /// `409 … port <address>:<port>/tcp4 is already published` (tried with
    /// sbx 0.49.2).
    pub fn publish(&self, env: &KitEnv, port: u16, lan: bool) -> anyhow::Result<()> {
        self.call(env, &vm::unpublish_args(&self.name, port, !lan), ASK_LIMIT)?;
        let args = vm::publish_args(&self.name, port, lan);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = run(&self.sbx, &self.host, env, &args, ASK_LIMIT).map_err(anyhow::Error::msg)?;
        let text = String::from_utf8_lossy(&out.stderr);
        anyhow::ensure!(
            out.status.success() || vm::published_already(&text, port, lan),
            "sbx ports: {}",
            text.trim()
        );
        Ok(())
    }

    /// Stops what the action noted as `tag` runs in the VM ([`vm::stop_args`]:
    /// ending its `sbx exec` on the PC does not), its web `container` if
    /// any, and takes its `port` back from the PC. Failures only go to the
    /// log: the daemon's stop at the app's exit ends the VM anyway.
    pub fn stop_run(
        &self,
        env: &KitEnv,
        tag: &str,
        container: Option<&str>,
        port: Option<u16>,
        lan: bool,
    ) {
        let mut steps = vec![vm::stop_args(&self.name, tag, container)];
        if let Some(p) = port {
            steps.push(vm::unpublish_args(&self.name, p, lan));
        }
        for args in steps {
            if let Err(e) = self.call(env, &args, ASK_LIMIT) {
                crate::common::log::warn(&format!("{}: {e:#}", self.name));
            }
        }
    }

    /// The session of the app's terminal for `args` (`vm::kit_terminal_args`,
    /// `vm::agent_args`), titled `what`; refused while [`Vm::ready`] is not
    /// (an agent's `sbx run` makes its sandbox without [`Vm::start`]).
    pub fn launch(
        &self,
        kit: &Kit,
        args: Vec<String>,
        what: &str,
        lang: &str,
    ) -> anyhow::Result<crate::editor::pty::Launch> {
        self.ready(lang)?;
        Ok(crate::editor::pty::program(
            kit,
            self.sbx.path(),
            args,
            &self.dir,
            what,
        ))
    }

    /// The one gate of every way into a sandbox (start, sessions, agents):
    /// a daemon that has not restarted with the kit's settings yet (no SSH
    /// agent of the PC in the VMs) starts and opens nothing, and never
    /// falls back to the PC.
    fn ready(&self, lang: &str) -> anyhow::Result<()> {
        if restart_pending(&self.kit) {
            anyhow::bail!(crate::common::i18n::t(
                lang,
                "Docker Sandboxes deve ripartire per un'impostazione di sicurezza: chiudi le sessioni nella VM, poi «Ricontrolla» nella finestra della VM e riprova.",
                &[],
            ));
        }
        Ok(())
    }

    /// The environment of that session ([`Sbx::session_env`]).
    pub fn session_env(&self, env: &KitEnv) -> KitEnv {
        self.sbx.session_env(env, &self.host)
    }

    /// The PC's ports this sandbox may reach (its databases on the kit's
    /// servers, D-171), opened by [`Vm::start`].
    pub fn with_host_ports(mut self, ports: Vec<u16>) -> Vm {
        self.host_ports = ports;
        self
    }

    /// Opens the PC's ports of this sandbox, closes those it had and has
    /// no more (a database moved to another version's server), and notes
    /// them; `made`: the sandbox was just made, nothing open before.
    fn sync_host_ports(&self, env: &KitEnv, made: bool) -> anyhow::Result<()> {
        let before = if made {
            Vec::new()
        } else {
            kit::host_ports(&self.kit, &self.name)
        };
        // Noted before any rule is opened: a rule opened and then lost by
        // a failure or the app's end would never be closed.
        let mut noted = self.host_ports.clone();
        noted.extend(before.iter().filter(|p| !self.host_ports.contains(p)));
        kit::mark_host_ports(&self.kit, &self.name, &noted)?;
        for port in &self.host_ports {
            self.call(env, &vm::host_port_allow(&self.name, *port), ASK_LIMIT)?;
        }
        // A port leaves the list only once its rule is gone: one that
        // could not be closed stays noted, and the next start tries again.
        let mut noted = self.host_ports.clone();
        for port in before.iter().filter(|p| !self.host_ports.contains(p)) {
            if let Err(e) = self.call(env, &vm::host_port_close(&self.name, *port), ASK_LIMIT) {
                crate::common::log::warn(&format!("{}: port {port}: {e:#}", self.name));
                noted.push(*port);
            }
        }
        kit::mark_host_ports(&self.kit, &self.name, &noted)
    }
}

/// The lock of the sandbox `name`, the same for every caller.
fn sandbox_lock(name: &str) -> std::sync::Arc<Mutex<()>> {
    static LOCKS: Mutex<BTreeMap<String, std::sync::Arc<Mutex<()>>>> = Mutex::new(BTreeMap::new());
    let mut all = LOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    std::sync::Arc::clone(all.entry(name.to_owned()).or_default())
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

/// Stops the daemon of the kit's `sbx` and the microVMs it runs, at the
/// app's exit and before the drive is removed: the PC's own daemon is the
/// user's, never stopped.
pub fn stop_daemon(kit: &Kit, host: &Host, env: &KitEnv) {
    let Some(exe) = kit_sbx(kit) else {
        return;
    };
    // The daemon is one per user: with another `sbx`'s running, the kit's
    // `daemon stop` would stop that one and its VMs.
    if !matches!(daemon::current(kit, &exe), daemon::Daemon::Kit) {
        return;
    }
    let sbx = Sbx(exe);
    match run(&sbx, host, env, &["daemon", "stop"], ASK_LIMIT) {
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
