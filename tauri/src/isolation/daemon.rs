//! The daemon of Docker Sandboxes (`sandboxd`, D-169): one per user of the
//! PC, behind a pipe of fixed name, with its state (the images, the VMs'
//! disks, their `/tmp`) where whoever started it keeps its own. The kit's
//! `sbx` starts it with the kit's environment, so that state is in the
//! kit's home and travels with it; a daemon another `sbx` started (the
//! PC's, another kit's) would get the kit's VMs in its own state, out of the
//! kit: never used, the notice says so.

use std::path::{Path, PathBuf};

use crate::kit::Kit;
use crate::platform::processes::{path_key, Proc};

/// The daemon's pipe on Windows: the same name for every `sbx` of the user.
pub const PIPE: &str = r"\\.\pipe\docker_kaname_sandboxd";

/// What every run of the kit's `sbx` gets besides the kit's environment
/// (tried with sbx 0.49.2): `llmman`, the local models of `sbx` that Atollo
/// does not use, looks for `llama-server` on the PATH instead of
/// downloading llama.cpp (713 MB in the kit's home); Docker's usage
/// reports off.
pub const ENV: [(&str, &str); 2] = [("LLMMAN_RUNTIME", "path"), ("SBX_NO_TELEMETRY", "1")];

/// The daemon's settings Atollo keeps on the kit's `sbx`: the VMs never
/// sign with the user's SSH keys (the PC's agent), as the sealed
/// environment of D-167 keeps them from a project's programs.
pub const SETTINGS: [(&str, &str); 1] = [("ssh.agentForwardingEnabled", "false")];

/// Who runs the daemon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Daemon {
    /// None: the kit's `sbx` starts one, with its state in the kit.
    None,
    /// The kit's: started by its `sbx`, its `sandboxd.pid` in the kit.
    Kit,
    /// Another `sbx`'s.
    Other,
}

/// Where a daemon started by the kit's `sbx` writes its state: under the
/// kit's home (`%LOCALAPPDATA%\DockerSandboxes` on Windows; the other
/// systems' folders of the same home).
fn state_roots(kit: &Kit) -> Vec<PathBuf> {
    let home = kit.home();
    [
        "AppData/Local/DockerSandboxes",
        ".local/state",
        ".local/share",
        "Library/Application Support",
        ".sbx",
    ]
    .iter()
    .map(|rel| home.join(rel))
    .collect()
}

/// The daemon's state folder in the kit (`…/sandboxes/state/sandboxd`,
/// the one holding `sandboxd.pid`), when it has run from the kit.
pub fn state_dir(kit: &Kit) -> Option<PathBuf> {
    state_roots(kit)
        .into_iter()
        .find_map(|root| find_pid_dir(&root, 6))
}

fn find_pid_dir(dir: &Path, depth: usize) -> Option<PathBuf> {
    if dir.file_name().is_some_and(|n| n == "sandboxd") && dir.join("sandboxd.pid").is_file() {
        return Some(dir.to_path_buf());
    }
    if depth == 0 {
        return None;
    }
    std::fs::read_dir(dir)
        .ok()?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .find_map(|e| find_pid_dir(&e.path(), depth - 1))
}

/// The pid in `sandboxd.pid` of the kit's state, if any.
pub fn kit_pid(kit: &Kit) -> Option<u32> {
    let file = state_dir(kit)?.join("sandboxd.pid");
    std::fs::read_to_string(file).ok()?.trim().parse().ok()
}

/// Who runs the daemon, from the kit's `sandboxd.pid` (`pid`), the
/// processes of the PC (`procs`), the kit's `sbx` (`sbx`) and whether the
/// daemon's pipe is there (`pipe`, Windows): the kit's when its pid runs
/// the kit's `sbx`; another's when the pipe is there or another `sbx`
/// runs; none otherwise.
pub fn which(pid: Option<u32>, procs: &[Proc], sbx: &Path, pipe: bool) -> Daemon {
    let sbx_key = path_key(&sbx.display().to_string());
    let is_kits = |p: &Proc| path_key(&p.path.display().to_string()) == sbx_key;
    if let Some(pid) = pid {
        if procs.iter().any(|p| p.pid == pid && is_kits(p)) {
            return Daemon::Kit;
        }
    }
    let other_sbx = procs.iter().any(|p| {
        let name = p.name.to_lowercase();
        (name == "sbx" || name == "sbx.exe") && !is_kits(p)
    });
    if pipe || other_sbx {
        Daemon::Other
    } else {
        Daemon::None
    }
}

/// [`which`] on this PC.
pub fn current(kit: &Kit, sbx: &Path) -> Daemon {
    let pipe = kit.target().is_windows() && Path::new(PIPE).exists();
    which(kit_pid(kit), &crate::platform::processes::all(), sbx, pipe)
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/daemon.rs"]
mod tests;
