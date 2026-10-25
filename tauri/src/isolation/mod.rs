//! How shut in a project's programs are (D-167), worked out by itself:
//! nothing to turn on in the settings. The VM (a Docker Sandboxes microVM
//! per project, `sbx`) when the PC has what it needs; otherwise the kit's
//! environment only, with the reasons the VM is off and, for each one, what
//! the user does about it (the interface's notice in the status bar).
//!
//! `sbx` is a program of the PC, not of the kit: it is found in the PC's
//! `PATH` (and where its installers put it) and runs with the PC's
//! environment, where its sign-in lives (D-167, declared trace).

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::env::{Host, KitEnv};
use crate::kit::Os;
use crate::platform::virt::{self, Gap};

/// How long a status stays good: `sbx` is asked at most once a minute.
const FRESH: Duration = Duration::from_secs(60);
/// How long `sbx` may take to answer.
const ASK_LIMIT: Duration = Duration::from_secs(15);

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
    /// `sbx` is not on the PC.
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
/// notice's "Ricontrolla").
pub fn status(host: &Host, env: &KitEnv, force: bool) -> Status {
    let mut last = LAST
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, st)) = last.as_ref() {
        if !force && at.elapsed() < FRESH {
            return st.clone();
        }
    }
    let st = compute(host, env);
    *last = Some((Instant::now(), st.clone()));
    st
}

fn compute(host: &Host, env: &KitEnv) -> Status {
    let mut blockers: Vec<Blocker> = virt::gaps().into_iter().map(Blocker::from).collect();
    if blockers.iter().any(Blocker::unsupported) {
        return Status::of(blockers, None);
    }
    let sbx = find_sbx(host);
    match &sbx {
        None => blockers.push(Blocker::SbxMissing),
        Some(exe) => {
            if let Err(b) = ask(exe, host, env) {
                blockers.push(b);
            }
        }
    }
    if blockers.is_empty() {
        blockers.push(Blocker::Pending);
    }
    Status::of(blockers, sbx.map(|p| p.display().to_string()))
}

/// The folders `sbx` is looked for in: the PC's `PATH`, then where its
/// installers put it (WinGet's links for the user, Homebrew, the Ubuntu
/// package).
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

fn find_sbx(host: &Host) -> Option<PathBuf> {
    let os = crate::platform::OS;
    let name = format!("sbx{}", os.exe_suffix());
    sbx_dirs(os, host)
        .into_iter()
        .map(|d| d.join(&name))
        .find(|p| p.is_file())
}

/// `sbx ls --json`: answers only when the daemon runs and the user is
/// signed in. With the PC's environment, where `sbx` keeps its sign-in.
fn ask(exe: &Path, host: &Host, env: &KitEnv) -> Result<(), Blocker> {
    let mut cmd = crate::platform::jobs::kit_command(exe, env);
    cmd.env_clear()
        .envs(host.iter())
        .args(["ls", "--json"])
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = crate::platform::jobs::spawn(&mut cmd).map_err(|e| Blocker::SbxFailed {
        message: e.to_string(),
    })?;
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });
    match rx.recv_timeout(ASK_LIMIT) {
        Ok(Ok(out)) => answer(out.status.success(), &String::from_utf8_lossy(&out.stderr)),
        Ok(Err(e)) => Err(Blocker::SbxFailed {
            message: e.to_string(),
        }),
        Err(_) => Err(Blocker::SbxFailed {
            message: format!("sbx: no answer in {} s", ASK_LIMIT.as_secs()),
        }),
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
