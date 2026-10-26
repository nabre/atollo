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

/// The daemon's settings Atollo keeps on the kit's `sbx` (D-175: nothing
/// of the box reaches the PC but its published ports): the VMs never sign
/// with the user's SSH keys (the PC's agent), as the sealed environment of
/// D-167 keeps them from a project's programs; never open the PC's browser
/// (the app's terminal shows a link, a click opens it); and `sbx` never
/// uploads a diagnostics bundle of the kit (tried with sbx 0.49.2).
pub const SETTINGS: [(&str, &str); 3] = [
    ("ssh.agentForwardingEnabled", "false"),
    ("browser.enabled", "false"),
    ("diagnostics.autoUpload", "no"),
];

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

/// The links of the PC's temporary folder (`temp`) that the daemon's
/// sockets go through (`sboxd-<id>`, a junction on Windows) leading out of
/// the kit's home (`home`), or to nothing. There is one per user of the
/// PC, made by the first `sbx` that started a daemon and kept by every
/// later one wherever it leads (tried with sbx 0.49.2): one left by another
/// home puts the kit daemon's containerd there (networks, images, the
/// VMs' disks, out of the kit), and a network left over in that state
/// fails every `sbx run` ("failed to create network: already exists").
pub fn foreign_links(temp: &Path, home: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(temp) else {
        return Vec::new();
    };
    let home = std::fs::canonicalize(home).ok();
    entries
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().starts_with("sboxd-"))
        .map(|e| e.path())
        .filter(|p| crate::platform::safe_path::is_link(p))
        .filter(|p| {
            let to = std::fs::canonicalize(p).ok();
            !matches!((&to, &home), (Some(to), Some(home)) if to.starts_with(home))
        })
        .collect()
}

/// Removes the [`foreign_links`] before the kit's `sbx` starts its daemon,
/// so it makes the link again to the kit's state. Only the link goes (a
/// junction is removed as a folder, never what it leads to).
pub fn unlink_foreign(temp: &Path, home: &Path) {
    for link in foreign_links(temp, home) {
        let gone = std::fs::remove_file(&link).or_else(|_| std::fs::remove_dir(&link));
        match gone {
            Ok(()) => crate::common::log::info(&format!(
                "sbx: {} led out of the kit, removed",
                link.display()
            )),
            Err(e) => crate::common::log::warn(&format!("sbx: {}: {e}", link.display())),
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/daemon.rs"]
mod tests;
