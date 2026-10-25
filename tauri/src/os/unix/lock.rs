//! `flock`, the lock of a file on macOS and Linux: exclusive, held while the
//! file stays open and dropped by the system when the process dies.

use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;

/// Takes the exclusive lock of `file` without waiting: `WouldBlock` when
/// another open file holds it.
pub fn try_lock(file: &File) -> io::Result<()> {
    // SAFETY: a valid descriptor owned by `file`.
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
