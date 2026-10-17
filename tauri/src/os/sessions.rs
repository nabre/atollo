//! Programs of the kit that are open, other copies of the kit, and "Rimuovi
//! l'unità" (port of nabre/dev-kit `Eject.ps1` and `Test-Sessions.ps1`).
//!
//! A process belongs to the kit when its program lives under the kit root;
//! only the tops of process trees are listed (a terminal, not every shell
//! inside it). Another kit: a process under `<other root>\data\apps\<os>-*`
//! (`win32`, `darwin`, `linux`) or another `Dev-Kit.exe` (`Dev-Kit` on
//! macOS and Linux) with a `data\app` folder next to it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;
#[cfg(windows)]
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

use super::processes::{self, Proc};
use crate::kit::Kit;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct KitProcess {
    pub pid: u32,
    pub name: String,
    /// Path relative to the kit root.
    pub program: String,
}

/// The kit's programs open now, without this app and its own children
/// that end with it anyway (WebView2, editor server): tops of trees only.
pub fn kit_processes(kit: &Kit) -> Vec<KitProcess> {
    let me = std::process::id();
    // The guard (macOS, Linux) is a copy of the app: not one of its programs.
    let all: Vec<Proc> = processes::all()
        .into_iter()
        .filter(|p| Some(p.pid) != super::guard::pid())
        .collect();
    let ours: Vec<&Proc> = {
        let prefix = format!(
            "{}{}",
            kit.root().display().to_string().to_lowercase(),
            std::path::MAIN_SEPARATOR
        );
        all.iter()
            .filter(|p| {
                p.path
                    .display()
                    .to_string()
                    .to_lowercase()
                    .starts_with(&prefix)
            })
            .collect()
    };
    // The app is not a parent that hides: what it started is listed.
    let pids: BTreeSet<u32> = ours
        .iter()
        .map(|p| p.pid)
        .filter(|&pid| pid != me)
        .collect();
    let server = crate::editor::server_dir(kit);
    let webview = crate::os::webview::fixed_folder(kit);
    ours.iter()
        .filter(|p| p.pid != me && !pids.contains(&p.parent))
        .filter(|p| !p.path.starts_with(&server) && !p.path.starts_with(&webview))
        .map(|p| KitProcess {
            pid: p.pid,
            name: p.name.clone(),
            program: p.path.strip_prefix(kit.root()).map_or_else(
                |_| p.path.display().to_string(),
                |r| r.display().to_string(),
            ),
        })
        .collect()
}

/// The kit's programs not started by this app (from another tool, or left
/// by an earlier session): what the app started is listed by kind (editor,
/// consoles, projects) and ends with its job anyway.
pub fn foreign_processes(kit: &Kit) -> Vec<KitProcess> {
    let me = std::process::id();
    let parents: BTreeMap<u32, u32> = processes::all().iter().map(|p| (p.pid, p.parent)).collect();
    let from_app = |pid: u32| {
        let mut cur = pid;
        // Bounded walk: parent ids can be reused and form a loop.
        for _ in 0..64 {
            match parents.get(&cur) {
                Some(&p) if p == me => return true,
                Some(&p) if p != 0 && p != cur => cur = p,
                _ => return false,
            }
        }
        false
    };
    kit_processes(kit)
        .into_iter()
        .filter(|p| !from_app(p.pid))
        .collect()
}

/// At exit: ends the kit's programs this app started (editor server,
/// consoles, agents), which the job would end only after the process is
/// gone, and waits a moment for them to let go of their files. WebView2
/// is left to close on its own.
pub fn stop_children(kit: &Kit) {
    let me = std::process::id();
    let webview = crate::os::webview::fixed_folder(kit);
    let all = processes::all();
    // Descendants, not only children: a console's shell is under conhost.exe.
    let mut ours = BTreeSet::from([me]);
    loop {
        let before = ours.len();
        for p in &all {
            if ours.contains(&p.parent) && p.pid != p.parent {
                ours.insert(p.pid);
            }
        }
        if ours.len() == before {
            break;
        }
    }
    // The guard stays until the app is gone: it then ends what is left.
    let guard = super::guard::pid();
    let pids: Vec<u32> = all
        .iter()
        .filter(|p| p.pid != me && Some(p.pid) != guard && ours.contains(&p.pid))
        .filter(|p| p.path.starts_with(kit.root()) && !p.path.starts_with(&webview))
        .map(|p| p.pid)
        .collect();
    if pids.is_empty() {
        return;
    }
    stop(&pids);
    std::thread::sleep(std::time::Duration::from_millis(300));
}

/// Ends `pids` and everything they started.
pub fn stop(pids: &[u32]) {
    let tree = tree_of(pids, &processes::all(), std::process::id());
    #[cfg(windows)]
    for pid in tree {
        terminate(pid);
    }
    #[cfg(unix)]
    stop_gently(
        &tree,
        std::time::Duration::from_secs(5),
        std::time::Duration::from_millis(100),
        signal,
        alive,
    );
}

/// `pids` and their descendants in `all`, without `me`, in the order the
/// processes are ended (each before its children).
pub(crate) fn tree_of(pids: &[u32], all: &[Proc], me: u32) -> Vec<u32> {
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for p in all {
        children.entry(p.parent).or_default().push(p.pid);
    }
    let mut todo: Vec<u32> = pids.to_vec();
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    while let Some(pid) = todo.pop() {
        if pid == me || !seen.insert(pid) {
            continue;
        }
        todo.extend(children.get(&pid).cloned().unwrap_or_default());
        out.push(pid);
    }
    out
}

/// A signal of [`stop_gently`].
#[cfg(any(unix, test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Signal {
    Term,
    Kill,
}

/// `SIGTERM` to every process, a wait of up to `grace` (checked every
/// `step`) for them to end, then `SIGKILL` to those left (T1.5.3): the
/// databases write their files to disk on `SIGTERM`.
#[cfg(any(unix, test))]
pub(crate) fn stop_gently(
    pids: &[u32],
    grace: std::time::Duration,
    step: std::time::Duration,
    signal: impl Fn(u32, Signal),
    alive: impl Fn(u32) -> bool,
) {
    for &pid in pids {
        signal(pid, Signal::Term);
    }
    let start = std::time::Instant::now();
    let mut left: Vec<u32> = pids.to_vec();
    loop {
        left.retain(|&pid| alive(pid));
        if left.is_empty() || start.elapsed() >= grace {
            break;
        }
        std::thread::sleep(step);
    }
    for pid in left {
        signal(pid, Signal::Kill);
    }
}

#[cfg(windows)]
fn terminate(pid: u32) {
    // SAFETY: terminate-only handle, closed right after.
    unsafe {
        if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(h, 1);
            let _ = CloseHandle(h);
        }
    }
}

#[cfg(unix)]
fn signal(pid: u32, which: Signal) {
    // 0 and negative numbers would signal whole process groups.
    if let Ok(pid @ 1..) = libc::pid_t::try_from(pid) {
        let sig = match which {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
        };
        // SAFETY: a plain signal to one process; a pid that is gone just fails.
        unsafe { libc::kill(pid, sig) };
    }
}

/// The process still runs: it exists and is not a child of the app that
/// has ended and waits to be reaped (a zombie still answers `kill`).
#[cfg(unix)]
fn alive(pid: u32) -> bool {
    let Ok(id @ 1..) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // SAFETY: signal 0 only checks that the process exists.
    if unsafe { libc::kill(id, 0) } != 0
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    {
        return false;
    }
    // SAFETY: a zeroed siginfo_t is valid; WNOWAIT leaves the zombie to
    // whoever waits for it.
    let ended = unsafe {
        let mut info: libc::siginfo_t = std::mem::zeroed();
        libc::waitid(
            libc::P_PID,
            pid,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        ) == 0
            && info.si_pid() == id
    };
    !ended
}

/// The root of the kit a program belongs to: one under
/// `<root>/data/apps/<os>-<arch>/` (Windows, macOS or Linux folders, either
/// separator), or the app itself (`Atollo.exe`, `Atollo` elsewhere) when
/// `is_kit` says its folder holds `data/app`.
fn kit_root_of(path: &Path, name: &str, is_kit: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let text = path.display().to_string();
    // ASCII lower case and one separator: byte offsets stay those of `text`.
    let lower = text.to_ascii_lowercase().replace('\\', "/");
    for os in ["win32", "darwin", "linux"] {
        if let Some(i) = lower.find(&format!("/data/apps/{os}-")) {
            return Some(PathBuf::from(&text[..i]));
        }
    }
    let app = name.eq_ignore_ascii_case("Atollo.exe") || name == "Atollo";
    app.then(|| path.parent())
        .flatten()
        .filter(|d| is_kit(d))
        .map(Path::to_path_buf)
}

/// Roots of other kits with programs running now.
pub fn other_kits(kit: &Kit) -> Vec<PathBuf> {
    let mine = kit.root().display().to_string().to_lowercase();
    let mut out = BTreeSet::new();
    for p in processes::all() {
        let root = kit_root_of(&p.path, &p.name, |d| d.join("data").join("app").is_dir());
        if let Some(root) = root {
            if root.display().to_string().to_lowercase() != mine {
                out.insert(root);
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
#[path = "../../../test/rust/os/sessions.rs"]
mod tests;
