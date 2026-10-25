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

use serde::Serialize;

use crate::env::{Host, KitEnv};
use crate::kit::{Kit, Os};
use crate::platform::virt::{self, Gap};

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
    /// All is there, but Atollo does not run the projects in it yet.
    Pending,
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
            if let Err(b) = ask(sbx, host, env) {
                blockers.push(b);
            }
        }
    }
    if blockers.is_empty() {
        blockers.push(Blocker::Pending);
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

    /// A command of this `sbx`, with its environment: the kit's for the
    /// kit's, the PC's for the PC's.
    pub fn command(&self, env: &KitEnv, host: &Host) -> std::process::Command {
        let mut cmd = crate::platform::jobs::kit_command(self.path(), env);
        if let Sbx::Pc(_) = self {
            cmd.env_clear().envs(host.iter());
        }
        cmd
    }
}

/// The kit's `sbx` (`data/apps/<target>/sandboxes/bin/sbx`), when the
/// component is installed.
pub fn kit_sbx(kit: &Kit) -> Option<PathBuf> {
    let c = crate::setup::Catalog::get().find("sandboxes")?;
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
