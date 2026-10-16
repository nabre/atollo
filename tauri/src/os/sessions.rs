//! Programs of the kit that are open, other copies of the kit, and "Rimuovi
//! l'unità" (port of nabre/dev-kit `Eject.ps1` and `Test-Sessions.ps1`).
//!
//! A process belongs to the kit when its program lives under the kit root;
//! only the tops of process trees are listed (a terminal, not every shell
//! inside it). Another kit: a process under `<other root>\data\apps\win32-*`
//! or another `Dev-Kit.exe` with a `data\app` folder next to it.

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
    let all = processes::all();
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
    let pids: Vec<u32> = all
        .iter()
        .filter(|p| p.pid != me && ours.contains(&p.pid))
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
    let all = processes::all();
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for p in &all {
        children.entry(p.parent).or_default().push(p.pid);
    }
    let mut todo: Vec<u32> = pids.to_vec();
    let mut seen = BTreeSet::new();
    let me = std::process::id();
    while let Some(pid) = todo.pop() {
        if pid == me || !seen.insert(pid) {
            continue;
        }
        todo.extend(children.get(&pid).cloned().unwrap_or_default());
        terminate(pid);
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
fn terminate(pid: u32) {
    // 0 and negative numbers would signal whole process groups.
    if let Ok(pid @ 1..) = libc::pid_t::try_from(pid) {
        // SAFETY: a plain signal to one process; a pid that is gone just fails.
        unsafe { libc::kill(pid, libc::SIGKILL) };
    }
}

/// Roots of other kits with programs running now.
pub fn other_kits(kit: &Kit) -> Vec<PathBuf> {
    let mine = kit.root().display().to_string().to_lowercase();
    let mut out = BTreeSet::new();
    for p in processes::all() {
        let path = p.path.display().to_string();
        let lower = path.to_lowercase();
        let root = if let Some(i) = lower.find("\\data\\apps\\win32-") {
            Some(PathBuf::from(&path[..i]))
        } else if p.name.eq_ignore_ascii_case("Dev-Kit.exe") {
            p.path
                .parent()
                .filter(|d| d.join(r"data\app").is_dir())
                .map(Path::to_path_buf)
        } else {
            None
        };
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
