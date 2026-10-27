//! Folders for files in transit (D-90): downloads, extractions, a framework
//! being created. They go in the host's temporary folder
//! (`<TEMP>\Atollo\<kit id>\`, `Kit::system_temp`), on the PC's own disk,
//! so a kit on FAT32/exFAT has neither the 4 GiB limit nor a slow USB disk
//! while they are written; `data\tmp` when the host gave no temporary folder
//! or it cannot be written. Each folder holds an open `.lock` while it is in
//! use and is removed when its `TempWork` is dropped.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::kit::Kit;
use crate::os::current::temp as imp;

/// One folder in transit, emptied when created and removed when dropped.
#[derive(Debug)]
pub struct TempWork {
    dir: PathBuf,
    /// Closed first in `drop`, or the folder could not be removed.
    lock: Option<File>,
}

impl TempWork {
    /// `rel` under the host's temporary folder of the kit (`setup/node`,
    /// `frameworks/1712345678`), or under `data\tmp` as a fallback.
    pub fn new(kit: &Kit, rel: &str) -> io::Result<TempWork> {
        TempWork::new_with_space(kit, rel, 0)
    }

    /// As `new`, but the host's temporary folder is used only when it has
    /// `bytes` free; otherwise `data\tmp` (the caller then checks the kit's
    /// own space and warns). The folder is not created to measure it: the
    /// space of its volume is read from the nearest folder that exists.
    pub fn new_with_space(kit: &Kit, rel: &str, bytes: u64) -> io::Result<TempWork> {
        if let Some(base) = kit.system_temp() {
            let dir = base.join(rel);
            match free_near(base) {
                Some(free) if free < bytes => crate::common::log::warn(&format!(
                    "{}: {free} bytes free, {bytes} needed; using data\\tmp",
                    base.display()
                )),
                _ => match TempWork::at(&dir) {
                    Ok(work) => return Ok(work),
                    Err(e) => crate::common::log::warn(&format!(
                        "{}: {e}; using data\\tmp",
                        dir.display()
                    )),
                },
            }
        }
        TempWork::at(&kit.data().join("tmp").join(rel))
    }

    /// `dir` itself, emptied: a work folder the tools sandbox sees
    /// (D-180, `install::guest::Guest::work_root`).
    pub fn at(dir: &Path) -> io::Result<TempWork> {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir)?;
        let lock = open_lock(&dir.join(".lock"))?;
        Ok(TempWork {
            dir: dir.to_path_buf(),
            lock: Some(lock),
        })
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for TempWork {
    fn drop(&mut self) {
        self.lock = None;
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// The host's temporary folder can hold the kit's `Atollo\<id>`: no other
/// account can make that folder (or a link in its place) before the app.
/// Windows gives each account its own `%TEMP%`; on macOS and Linux only a
/// folder this user owns and nobody else can write in.
pub fn private_base(base: &Path) -> bool {
    imp::private_base(base)
}

/// Opens the `.lock` that marks a folder in use while the file stays open.
fn open_lock(path: &Path) -> io::Result<File> {
    imp::open_lock(path)
}

/// The `.lock` at `path` is held by a running `TempWork` (here or in
/// another app). On Windows the removal itself tells it (sharing violation).
fn held(path: &Path) -> bool {
    imp::held(path)
}

/// Free bytes on the volume of `path`, asked to its nearest existing parent.
fn free_near(path: &Path) -> Option<u64> {
    path.ancestors()
        .find(|p| p.is_dir())
        .and_then(|p| super::volume::free_space(p).ok())
}

/// Folders older than this and not in use are leftovers.
const STALE: Duration = Duration::from_secs(24 * 3600);

/// At start: removes the folders in transit that no `TempWork` holds and
/// that are older than 24 h (an app killed in the middle of a download),
/// under the host's temporary folder and `data\tmp\{setup,frameworks}`.
/// A folder in use keeps its `.lock` open without delete sharing, so the
/// lock cannot be removed: that is how a live one is told from a dead one.
/// Never fails; problems are logged.
pub fn sweep(kit: &Kit) {
    if let Some(base) = kit.system_temp() {
        // `setup` and `frameworks` hold one folder per task; the others
        // (`system`, `claude`: the programs' temp) are one unit.
        for entry in children(base) {
            let name = entry.file_name().map(|n| n.to_ascii_lowercase());
            match name.as_deref().and_then(|n| n.to_str()) {
                Some("setup" | "frameworks") => children(&entry).for_each(|c| sweep_one(&c)),
                // Files added by hand: kept 30 days by `setup/downloads.rs` (D-82).
                Some("downloads") => {}
                _ => sweep_one(&entry),
            }
        }
    }
    let tmp = kit.data().join("tmp");
    for sub in ["setup", "frameworks"] {
        children(&tmp.join(sub)).for_each(|c| sweep_one(&c));
    }
}

fn children(dir: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
}

fn sweep_one(dir: &Path) {
    let lock = dir.join(".lock");
    let stamp = fs::metadata(&lock)
        .or_else(|_| fs::metadata(dir))
        .and_then(|m| m.modified());
    let old = stamp.is_ok_and(|t| t.elapsed().is_ok_and(|age| age > STALE));
    if !old || held(&lock) {
        return;
    }
    match fs::remove_file(&lock) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        // Held open by a running TempWork (sharing violation / access denied).
        Err(_) => return,
    }
    if let Err(e) = fs::remove_dir_all(dir) {
        crate::common::log::warn(&format!("{}: {e}", dir.display()));
    }
}

/// At exit, once the job has closed the programs: removes the programs' temp
/// in the host's temporary folder (`system`, and Claude's `claude`). The
/// folders of `TempWork` are not touched: each removes its own.
pub fn cleanup_at_exit(kit: &Kit) {
    if let Some(base) = kit.system_temp() {
        for name in ["system", "claude"] {
            let dir = base.join(name);
            if let Err(e) = fs::remove_dir_all(&dir) {
                if e.kind() != io::ErrorKind::NotFound {
                    crate::common::log::warn(&format!("{}: {e}", dir.display()));
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/platform/temp.rs"]
mod tests;
