//! The folders in transit (`platform::temp`) on Windows.

use std::fs::{self, File};
use std::io;
use std::path::Path;

/// Windows gives each account its own `%TEMP%`.
pub fn private_base(_base: &Path) -> bool {
    true
}

/// Shared for reading and writing but not for delete: while this handle
/// lives, `sweep` cannot remove the lock, which marks the folder as in use.
pub fn open_lock(path: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_SHARE_READ_WRITE: u32 = 0x1 | 0x2;
    fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .share_mode(FILE_SHARE_READ_WRITE)
        .open(path)
}

/// The removal itself tells a lock in use (sharing violation).
pub fn held(_path: &Path) -> bool {
    false
}
