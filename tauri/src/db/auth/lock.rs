//! The lock of `db.json` and `secrets.json`: a mutex in this process, the kit's lock across processes.

#[cfg(unix)]
use std::fs;

use anyhow::{bail, Result};

#[cfg(unix)]
use super::dir;
use crate::kit::Kit;

/// Held by every read-change-write of a project's `db.json` and of
/// `secrets.json` ([`update_secrets`]): two commands
/// on blocking threads (a version chosen while a database is created), or
/// the command line beside the app, would otherwise each write back what
/// they read, and one change would be lost. In this process a mutex; across
/// the processes of the kit the lock of the Antigravity keyring
/// (`agents/keyring.rs`): `Local\Atollo-db-json-<kit id>` on Windows, an
/// exclusive `flock` on `data/app/db/db-json.lock` elsewhere.
pub(crate) fn db_json_lock(kit: &Kit) -> Result<DbJsonLock> {
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let process = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(DbJsonLock {
        _kit: KitLock::take(kit)?,
        _process: process,
    })
}

/// How long another process may hold `db.json`: then the change is refused
/// (nothing written), never made without the lock.
const DB_JSON_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

pub(crate) struct DbJsonLock {
    // Released first: the other processes, then this one's threads.
    _kit: KitLock,
    _process: std::sync::MutexGuard<'static, ()>,
}

#[cfg(windows)]
pub(super) struct KitLock(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl KitLock {
    pub(super) fn take(kit: &Kit) -> Result<KitLock> {
        use windows::Win32::Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::{CreateMutexW, WaitForSingleObject};
        let name = windows::core::HSTRING::from(format!(r"Local\Atollo-db-json-{}", kit.id()));
        // SAFETY: valid name, default security; the handle is closed here
        // or by the guard's Drop.
        unsafe {
            let h = CreateMutexW(None, false, &name)?;
            let r = WaitForSingleObject(h, DB_JSON_WAIT.as_millis() as u32);
            // Abandoned: a process that held it ended; it is ours now.
            if r != WAIT_OBJECT_0 && r != WAIT_ABANDONED {
                let _ = CloseHandle(h);
                bail!("db.json: another Atollo process holds it, try again");
            }
            Ok(KitLock(h))
        }
    }
}

#[cfg(windows)]
impl Drop for KitLock {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::ReleaseMutex;
        // SAFETY: handle from CreateMutexW, owned since the wait; released
        // and closed once.
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

/// The lock file stays (it is empty); the system drops the `flock` when the
/// file closes, also when the process dies.
#[cfg(unix)]
pub(super) struct KitLock(#[allow(dead_code)] fs::File);

#[cfg(unix)]
impl KitLock {
    pub(super) fn take(kit: &Kit) -> Result<KitLock> {
        use std::os::fd::AsRawFd;
        let path = dir(kit).join("db-json.lock");
        fs::create_dir_all(dir(kit))?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        let start = std::time::Instant::now();
        // SAFETY: a valid descriptor owned by `file`.
        while unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            if start.elapsed() > DB_JSON_WAIT {
                bail!("db.json: another Atollo process holds it, try again");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Ok(KitLock(file))
    }
}
