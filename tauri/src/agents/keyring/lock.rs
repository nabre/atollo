//! The kit's lock around the moves: a named mutex on Windows, an exclusive `flock` elsewhere.

#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::time::Duration;

use anyhow::{Context, Result};
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, WAIT_ABANDONED, WAIT_OBJECT_0,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

use crate::kit::Kit;

/// The kit's lock: an exclusive `flock` on `data/antigravity/keyring.lock`,
/// waited for at most 30 s like on Windows.
#[cfg(unix)]
pub(super) struct Lock(#[allow(dead_code)] fs::File);

#[cfg(unix)]
impl Lock {
    pub(super) fn take(kit: &Kit) -> Result<Lock> {
        use std::os::fd::AsRawFd;
        let dir = kit.data().join("antigravity");
        fs::create_dir_all(&dir)?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("keyring.lock"))
            .context("keyring.lock")?;
        let start = std::time::Instant::now();
        // SAFETY: a valid descriptor owned by `file`.
        while unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            if start.elapsed() > Duration::from_secs(30) {
                crate::log::warn("antigravity keyring: lock busy for 30 s, going on");
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        Ok(Lock(file))
    }
}

/// The kit's lock: `Local\Atollo-agy-keyring-<kit id>` (same as the kit).
#[cfg(windows)]
pub(super) struct Lock(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl Lock {
    pub(super) fn take(kit: &Kit) -> Result<Lock> {
        let name = HSTRING::from(format!(r"Local\Atollo-agy-keyring-{}", kit.id()));
        // SAFETY: valid name; waits at most 30 s like the kit.
        unsafe {
            let h = CreateMutexW(None, false, &name).context("CreateMutexW")?;
            let _ = GetLastError() == ERROR_ALREADY_EXISTS;
            let r = WaitForSingleObject(h, 30_000);
            if r != WAIT_OBJECT_0 && r != WAIT_ABANDONED {
                crate::log::warn("antigravity keyring: lock busy for 30 s, going on");
            }
            Ok(Lock(h))
        }
    }
}

#[cfg(windows)]
impl Drop for Lock {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW; released and closed once.
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}
