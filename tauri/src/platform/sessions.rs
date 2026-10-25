//! Programs of the kit that are open, other copies of the kit, and "Rimuovi
//! l'unità".
//!
//! A process belongs to the kit when its program lives under the kit root;
//! only the tops of process trees are listed (a terminal, not every shell
//! inside it). Another kit: a process under `<other root>\data\apps\<os>-*`
//! (`win32`, `darwin`, `linux`) or another `Atollo.exe` (`Atollo` on
//! macOS and Linux) with a `data\app` folder next to it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;

use super::processes::{self, Proc};
use crate::kit::Kit;
use crate::os::current::sessions as imp;

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
        .filter(|p| Some(p.pid) != super::jobs::guard_pid())
        .collect();
    let roots = kit_roots(kit);
    let ours: Vec<&Proc> = {
        // As the PC compares paths (D-123): on Linux a kit whose root
        // differs only in case is another kit, its programs not ours.
        let prefixes: Vec<String> = roots.iter().map(|r| processes::prefix_of(r)).collect();
        all.iter()
            .filter(|p| prefixes.iter().any(|prefix| processes::is_under(p, prefix)))
            .collect()
    };
    // The app is not a parent that hides: what it started is listed.
    let pids: BTreeSet<u32> = ours
        .iter()
        .map(|p| p.pid)
        .filter(|&pid| pid != me)
        .collect();
    // Compared inside the kit, whichever spelling of its root a path has.
    let inside = |path: &Path| -> Option<PathBuf> {
        roots
            .iter()
            .find_map(|r| path.strip_prefix(r).ok())
            .map(Path::to_path_buf)
    };
    let server = inside(&crate::editor::server_dir(kit));
    let webview = inside(&super::webview::fixed_folder(kit));
    let under = |rel: &Option<PathBuf>, dir: &Option<PathBuf>| matches!((rel, dir), (Some(r), Some(d)) if r.starts_with(d));
    ours.iter()
        .filter(|p| p.pid != me && !pids.contains(&p.parent))
        .map(|p| (p, inside(&p.path)))
        .filter(|(_, rel)| !under(rel, &server) && !under(rel, &webview))
        .map(|(p, rel)| KitProcess {
            pid: p.pid,
            name: p.name.clone(),
            program: rel.map_or_else(|| p.path.display().to_string(), |r| r.display().to_string()),
        })
        .collect()
}

/// The kit root as the app names it and, when a link leads there, its real
/// path: the system names a process by the real one (on macOS
/// `/var/folders/…` is `/private/var/folders/…`).
fn kit_roots(kit: &Kit) -> Vec<PathBuf> {
    let mut roots = vec![kit.root().to_path_buf()];
    if let Ok(real) = std::fs::canonicalize(kit.root()) {
        let real = crate::kit::strip_verbatim(&real);
        if !roots.contains(&real) {
            roots.push(real);
        }
    }
    roots
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
    let webview = super::webview::fixed_folder(kit);
    let all = processes::all();
    // Descendants, not only children: the programs a terminal's shell or
    // an agent started.
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
    let guard = super::jobs::guard_pid();
    // Under each spelling of the root: on macOS a kit reached through a
    // link has children named by the real path (BUG-V7-19).
    let roots = kit_roots(kit);
    let inside: Vec<String> = roots.iter().map(|r| processes::prefix_of(r)).collect();
    let webviews: Vec<String> = match webview.strip_prefix(kit.root()) {
        Ok(rel) => roots
            .iter()
            .map(|r| processes::prefix_of(&r.join(rel)))
            .collect(),
        Err(_) => vec![processes::prefix_of(&webview)],
    };
    let pids: Vec<u32> = all
        .iter()
        .filter(|p| p.pid != me && Some(p.pid) != guard && ours.contains(&p.pid))
        .filter(|p| {
            inside.iter().any(|r| processes::is_under(p, r))
                && !webviews.iter().any(|w| processes::is_under(p, w))
        })
        .map(|p| p.pid)
        .collect();
    if pids.is_empty() {
        return;
    }
    stop(&pids);
    std::thread::sleep(std::time::Duration::from_millis(300));
}

/// Ends `pids` and everything they started: at once on Windows; on macOS
/// and Linux `SIGTERM`, up to 5 s for them to end, then `SIGKILL` (D-105).
pub fn stop(pids: &[u32]) {
    imp::stop_tree(&tree_of(pids, &processes::all(), std::process::id()));
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

/// The root of the kit a program belongs to: one under
/// `<root>/data/apps/<os>-<arch>/` (Windows, macOS or Linux folders, either
/// separator), or the app itself (`Atollo.exe`, `Atollo` elsewhere, or a
/// release name such as `Atollo-0.3.0-win32-x64.exe`) when `is_kit` says
/// its folder holds `data/app`.
fn kit_root_of(path: &Path, name: &str, is_kit: impl Fn(&Path) -> bool) -> Option<PathBuf> {
    let text = path.display().to_string();
    // ASCII lower case and one separator: byte offsets stay those of `text`.
    let lower = text.to_ascii_lowercase().replace('\\', "/");
    for os in ["win32", "darwin", "linux"] {
        if let Some(i) = lower.find(&format!("/data/apps/{os}-")) {
            return Some(PathBuf::from(&text[..i]));
        }
    }
    let app = name.eq_ignore_ascii_case("Atollo.exe")
        || name == "Atollo"
        || crate::setup::self_update::is_release_name(name);
    app.then(|| path.parent())
        .flatten()
        .filter(|d| is_kit(d))
        .map(Path::to_path_buf)
}

/// Roots of other kits with programs running now.
pub fn other_kits(kit: &Kit) -> Vec<PathBuf> {
    let key = |r: &Path| processes::path_key(&r.display().to_string());
    let mine: Vec<String> = kit_roots(kit).iter().map(|r| key(r)).collect();
    let mut out = BTreeSet::new();
    for p in processes::all() {
        let root = kit_root_of(&p.path, &p.name, |d| d.join("data").join("app").is_dir());
        if let Some(root) = root {
            if !mine.contains(&key(&root)) {
                out.insert(root);
            }
        }
    }
    out.into_iter().collect()
}

#[cfg(test)]
#[path = "../../../test/rust/platform/sessions.rs"]
mod tests;
