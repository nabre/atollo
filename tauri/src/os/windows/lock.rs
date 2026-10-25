//! The kit's locks (`platform::lock`) on Windows: the named mutex
//! `Local\<name>`; the file is not used.

use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};
use windows::core::HSTRING;
use windows::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
};
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};

/// The mutex, and whether this process owns it (`wait`) or only keeps it
/// in existence (`try_take`).
pub struct Held {
    handle: HANDLE,
    owned: bool,
}

// SAFETY: a mutex handle is a kernel handle usable from any thread.
unsafe impl Send for Held {}
unsafe impl Sync for Held {}

impl Drop for Held {
    fn drop(&mut self) {
        // SAFETY: handle from CreateMutexW; released (when owned) and
        // closed once.
        unsafe {
            if self.owned {
                let _ = ReleaseMutex(self.handle);
            }
            let _ = CloseHandle(self.handle);
        }
    }
}

fn create(name: &str) -> Result<HANDLE> {
    let name = HSTRING::from(format!(r"Local\{name}"));
    // SAFETY: valid name, default security, not initially owned.
    unsafe { CreateMutexW(None, false, &name) }.context("CreateMutexW")
}

/// The mutex exists while a holder keeps its handle open: a second one
/// finds it already there.
pub fn try_take(name: &str, _file: &Path) -> Result<Option<Held>> {
    let handle = create(name)?;
    // SAFETY: last error of CreateMutexW on this thread.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        // SAFETY: handle from CreateMutexW.
        let _ = unsafe { CloseHandle(handle) };
        return Ok(None);
    }
    Ok(Some(Held {
        handle,
        owned: false,
    }))
}

/// Owns the mutex, waiting for it at most `timeout`. Abandoned: a process
/// that held it ended; it is this one's now.
pub fn wait(name: &str, _file: &Path, timeout: Duration) -> Result<Option<Held>> {
    let handle = create(name)?;
    let millis = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    // SAFETY: a valid handle from CreateMutexW.
    let r = unsafe { WaitForSingleObject(handle, millis) };
    if r != WAIT_OBJECT_0 && r != WAIT_ABANDONED {
        // SAFETY: handle from CreateMutexW.
        let _ = unsafe { CloseHandle(handle) };
        return Ok(None);
    }
    Ok(Some(Held {
        handle,
        owned: true,
    }))
}
