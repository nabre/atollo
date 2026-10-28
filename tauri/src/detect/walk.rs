//! One walk of a project's folder (D-186): every file at any depth, the
//! folders that hold no sources left out ([`super::IGNORED`], and every
//! folder whose name starts with `.`: `.git`, `.venv`, `.venv-linux`,
//! `.idea`…), links never entered (D-77). Breadth first, so that when the
//! budget ([`MAX_ENTRIES`], [`MAX_TIME`]) stops it, what is near the root
//! (the manifests, the pin files) has been read.

use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// Entries (files and folders) read at most in one walk.
pub const MAX_ENTRIES: usize = 50_000;

/// Time a walk takes at most.
pub const MAX_TIME: Duration = Duration::from_secs(1);

/// What a walk found.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Walk {
    /// The files, relative to the folder walked, `/`-separated, shallowest
    /// first.
    pub files: Vec<String>,
    /// The budget stopped it: what was found counts, the rest was not read.
    pub cut: bool,
}

impl Walk {
    /// The file names, one per file.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.files.iter().map(|f| name_of(f))
    }

    /// The files named `name`, shallowest first (in walk order for one
    /// depth).
    pub fn named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> {
        self.files
            .iter()
            .map(String::as_str)
            .filter(move |f| name_of(f) == name)
    }
}

/// The name of the file `rel` (`web/package.json` → `package.json`).
pub fn name_of(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

/// How deep `rel` is: `package.json` 0, `web/package.json` 1.
pub fn depth(rel: &str) -> usize {
    rel.matches('/').count()
}

/// The walk of `dir` with the budget of every detection.
pub fn walk(dir: &Path) -> Walk {
    let walk = walk_with(dir, MAX_ENTRIES, MAX_TIME);
    if walk.cut {
        told_once(dir);
    }
    walk
}

/// The log says once per folder and run that a walk was cut.
fn told_once(dir: &Path) {
    static TOLD: Mutex<BTreeSet<PathBuf>> = Mutex::new(BTreeSet::new());
    let mut told = TOLD.lock().unwrap_or_else(PoisonError::into_inner);
    if told.insert(dir.to_path_buf()) {
        crate::common::log::info(&format!(
            "detect: {} has more than {MAX_ENTRIES} entries or takes over {} ms: the tools of the files read are counted",
            dir.display(),
            MAX_TIME.as_millis()
        ));
    }
}

/// The walk of `dir`, stopped after `max_entries` entries or `max_time`.
pub fn walk_with(dir: &Path, max_entries: usize, max_time: Duration) -> Walk {
    let mut out = Walk::default();
    if crate::platform::safe_path::is_link(dir) {
        return out;
    }
    let start = Instant::now();
    let mut seen = 0usize;
    let mut queue: VecDeque<(PathBuf, String)> =
        VecDeque::from([(dir.to_path_buf(), String::new())]);
    while let Some((abs, rel)) = queue.pop_front() {
        let Ok(entries) = fs::read_dir(&abs) else {
            continue;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(fs::DirEntry::file_name);
        for e in entries {
            seen += 1;
            if seen > max_entries || start.elapsed() > max_time {
                out.cut = true;
                return out;
            }
            let name = e.file_name().to_string_lossy().into_owned();
            let path = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            // A DirEntry's type is the link's own: a link to a folder is
            // neither entered nor counted as a file.
            let Ok(t) = e.file_type() else { continue };
            if t.is_dir() {
                if !name.starts_with('.') && !super::IGNORED.contains(&name.as_str()) {
                    queue.push_back((e.path(), path));
                }
            } else if t.is_file() {
                out.files.push(path);
            }
        }
    }
    out
}
