//! The folders in transit (`platform::temp`) on macOS and Linux.

use std::fs::{self, File};
use std::io;
use std::path::Path;

/// On macOS `TMPDIR` is the account's own `/var/folders/.../T` (0700). On
/// Linux it may be the shared `/tmp`, where another account could create
/// `Atollo/<id>` first (or a link in its place), and `TempWork::at` would
/// then write the kit's downloads there: only a folder this user owns and
/// nobody else can write in.
pub fn private_base(base: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    let me = super::users::effective();
    fs::metadata(base).is_ok_and(|m| m.is_dir() && m.uid() == me && m.mode() & 0o022 == 0)
}

/// Unix removes open files: the lock is an exclusive `flock` instead, held
/// while this handle lives and released by the system when the app dies.
pub fn open_lock(path: &Path) -> io::Result<File> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    super::lock::try_lock(&file)?;
    Ok(file)
}

/// The lock is taken: the lock, when this check gets it, is released when
/// the file closes at the end of this function.
pub fn held(path: &Path) -> bool {
    let Ok(file) = fs::OpenOptions::new().read(true).open(path) else {
        return false;
    };
    super::lock::try_lock(&file).is_err()
}
