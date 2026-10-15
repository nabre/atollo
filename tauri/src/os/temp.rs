//! Folders for files in transit (D-90): downloads, extractions, a framework
//! being created. They go in the host's temporary folder
//! (`<TEMP>\Dev-Kit\<kit id>\`, `Kit::system_temp`), on the PC's own disk,
//! so a kit on FAT32/exFAT has neither the 4 GiB limit nor a slow USB disk
//! while they are written; `data\tmp` when the host gave no temporary folder
//! or it cannot be written. Each folder holds an open `.lock` while it is in
//! use and is removed when its `TempWork` is dropped.
// Contract of PLAN-DATA.md WP-0 (WP-8 adds the sweep at start); removed in WP-14.
#![allow(dead_code)]

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use crate::kit::Kit;

/// One folder in transit, emptied when created and removed when dropped.
#[derive(Debug)]
pub struct TempWork {
    dir: PathBuf,
    _lock: File,
}

impl TempWork {
    /// `rel` under the host's temporary folder of the kit (`setup/node`,
    /// `frameworks/1712345678`), or under `data\tmp` as a fallback.
    pub fn new(kit: &Kit, rel: &str) -> io::Result<TempWork> {
        if let Some(base) = kit.system_temp() {
            match TempWork::at(&base.join(rel)) {
                Ok(work) => return Ok(work),
                Err(e) => crate::log::warn(&format!(
                    "{}: {e}; using data\\tmp",
                    base.join(rel).display()
                )),
            }
        }
        TempWork::at(&kit.data().join("tmp").join(rel))
    }

    fn at(dir: &Path) -> io::Result<TempWork> {
        let _ = fs::remove_dir_all(dir);
        fs::create_dir_all(dir)?;
        let lock = File::create(dir.join(".lock"))?;
        Ok(TempWork {
            dir: dir.to_path_buf(),
            _lock: lock,
        })
    }

    pub fn path(&self) -> &Path {
        &self.dir
    }
}

impl Drop for TempWork {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

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
        assert!(work.path().starts_with(tmp.0.join(r"host\Dev-Kit")));
        // a second user of the same folder starts from empty
        fs::write(work.path().join("old"), "x").unwrap();
        drop(work);
        let again = TempWork::new(&host, "setup/x").unwrap();
        assert!(!again.path().join("old").exists());
    }
}
