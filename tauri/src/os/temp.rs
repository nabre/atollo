//! Folders for files in transit (D-90): downloads, extractions, a framework
//! being created. They go in the host's temporary folder
//! (`<TEMP>\Atollo\<kit id>\`, `Kit::system_temp`), on the PC's own disk,
//! so a kit on FAT32/exFAT has neither the 4 GiB limit nor a slow USB disk
//! while they are written; `data\tmp` when the host gave no temporary folder
//! or it cannot be written. Each folder holds an open `.lock` while it is in
//! use and is removed when its `TempWork` is dropped.
// Contract of PLAN-DATA.md WP-0 (WP-8 adds the sweep at start); removed in WP-14.
#![allow(dead_code)]

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::kit::Kit;

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
                Some(free) if free < bytes => crate::log::warn(&format!(
                    "{}: {free} bytes free, {bytes} needed; using data\\tmp",
                    base.display()
                )),
                _ => match TempWork::at(&dir) {
                    Ok(work) => return Ok(work),
                    Err(e) => crate::log::warn(&format!("{}: {e}; using data\\tmp", dir.display())),
                },
            }
        }
        TempWork::at(&kit.data().join("tmp").join(rel))
    }

    fn at(dir: &Path) -> io::Result<TempWork> {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir)?;
        // Shared for reading and writing but not for delete: while this
        // handle lives, `sweep` cannot remove the lock, which marks the
        // folder as in use.
        let lock = {
            use std::os::windows::fs::OpenOptionsExt;
            const FILE_SHARE_READ_WRITE: u32 = 0x1 | 0x2;
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .share_mode(FILE_SHARE_READ_WRITE)
                .open(dir.join(".lock"))?
        };
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
    if !old {
        return;
    }
    match fs::remove_file(&lock) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        // Held open by a running TempWork (sharing violation / access denied).
        Err(_) => return,
    }
    if let Err(e) = fs::remove_dir_all(dir) {
        crate::log::warn(&format!("{}: {e}", dir.display()));
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
                    crate::log::warn(&format!("{}: {e}", dir.display()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use std::time::SystemTime;

    #[test]
    fn in_host_temp_or_data_tmp() {
        let tmp = TempDir::new("temp-work");
        let kit = Kit::at(tmp.0.join("kit"), Arch::X64);
        let work = TempWork::new(&kit, "setup/x").unwrap();
        assert_eq!(work.path(), tmp.0.join(r"kit\data\tmp\setup\x"));
        assert!(work.path().join(".lock").is_file());
        let dir = work.path().to_path_buf();
        drop(work);
        assert!(!dir.exists());

        let host = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&tmp.0.join("host")));
        let work = TempWork::new(&host, "setup/x").unwrap();
        assert!(work.path().starts_with(tmp.0.join(r"host\Atollo")));
        // a second user of the same folder starts from empty
        fs::write(work.path().join("old"), "x").unwrap();
        drop(work);
        let again = TempWork::new(&host, "setup/x").unwrap();
        assert!(!again.path().join("old").exists());
    }

    fn age(dir: &Path, hours: u64) {
        let t = SystemTime::now() - Duration::from_secs(hours * 3600);
        let f = fs::OpenOptions::new()
            .write(true)
            .open(dir.join(".lock"))
            .unwrap();
        f.set_modified(t).unwrap();
    }

    #[test]
    fn sweep_keeps_live_and_new_removes_old() {
        let tmp = TempDir::new("temp-sweep");
        let host = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&tmp.0.join("host")));
        let base = host.system_temp().unwrap().to_path_buf();

        // old and held by a live TempWork: kept
        let live = TempWork::new(&host, "setup/live").unwrap();
        age(live.path(), 48);
        // old and unlocked (a dead app left its lock closed): removed
        let dead = base.join(r"setup\dead");
        fs::create_dir_all(&dead).unwrap();
        fs::write(dead.join(".lock"), "").unwrap();
        fs::write(dead.join("file"), "x").unwrap();
        age(&dead, 48);
        // old, in data\tmp, unlocked: removed
        let old_data = host.data().join(r"tmp\frameworks\1");
        fs::create_dir_all(&old_data).unwrap();
        fs::write(old_data.join(".lock"), "").unwrap();
        age(&old_data, 48);
        // new and unlocked: kept
        let fresh = base.join(r"frameworks\fresh");
        fs::create_dir_all(&fresh).unwrap();
        fs::write(fresh.join(".lock"), "").unwrap();

        sweep(&host);
        assert!(live.path().join(".lock").is_file(), "live work kept");
        assert!(!dead.exists(), "old unlocked removed");
        assert!(!old_data.exists());
        assert!(fresh.is_dir(), "new kept");
        drop(live);
    }

    #[test]
    fn cleanup_at_exit_removes_programs_temp_only() {
        let tmp = TempDir::new("temp-exit");
        let host = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&tmp.0.join("host")));
        let base = host.system_temp().unwrap().to_path_buf();
        let work = TempWork::new(&host, "setup/x").unwrap();
        fs::create_dir_all(base.join(r"system\a")).unwrap();
        fs::create_dir_all(base.join("claude")).unwrap();
        cleanup_at_exit(&host);
        assert!(!base.join("system").exists() && !base.join("claude").exists());
        assert!(work.path().is_dir());
        cleanup_at_exit(&host); // nothing left: no error
    }

    #[test]
    fn falls_back_to_data_tmp_without_space() {
        let tmp = TempDir::new("temp-space");
        let host = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&tmp.0.join("host")));
        let work = TempWork::new_with_space(&host, "setup/big", u64::MAX).unwrap();
        assert_eq!(work.path(), host.data().join(r"tmp\setup\big"));
        let ok = TempWork::new_with_space(&host, "setup/small", 1).unwrap();
        assert!(ok.path().starts_with(host.system_temp().unwrap()));
    }
}
