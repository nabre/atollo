//! The kit's locks across its processes (the app, another window, the
//! command line beside it): one preparation at a time (`setup/runner.rs`),
//! one change of `db.json` at a time (`db/auth/lock.rs`), one move of
//! Antigravity's login at a time (`agents/keyring/lock.rs`).
//!
//! A lock has a name, for the named mutex `Local\<name>` of Windows, and a
//! file, for the exclusive `flock` of macOS and Linux; the system drops it
//! when the process ends, however it ends. A lock is always taken the same
//! way, with [`try_take`] or with [`wait`]: on Windows the first keeps the
//! mutex in existence, the second owns it.

use std::path::Path;
use std::time::Duration;

use anyhow::Result;

use crate::os::current::lock as imp;

/// A lock held for as long as it lives.
pub struct KitLock(#[allow(dead_code)] imp::Held);

impl std::fmt::Debug for KitLock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("KitLock")
    }
}

/// Takes the lock without waiting: `None` when another holder has it, in
/// another process or in this one. Any other error (a file system without
/// `flock`, some network shares) is returned: nothing goes on unguarded.
pub fn try_take(name: &str, file: &Path) -> Result<Option<KitLock>> {
    Ok(imp::try_take(name, file)?.map(KitLock))
}

/// Takes the lock, waiting at most `timeout` for another holder to let it
/// go: `None` when it stays busy.
pub fn wait(name: &str, file: &Path, timeout: Duration) -> Result<Option<KitLock>> {
    Ok(imp::wait(name, file, timeout)?.map(KitLock))
}

#[cfg(test)]
#[path = "../../../test/rust/platform/lock.rs"]
mod tests;
