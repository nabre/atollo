//! The disk a folder is on (`platform::volume`): what macOS and Linux share,
//! `statfs`, `statvfs` and the device number.

use std::io;
use std::path::Path;

/// The device of `path` (`st_dev`): the same number means the same volume.
pub(crate) fn device(path: &Path) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata(path)?.dev())
}

fn c_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)
}

pub(crate) fn statfs(path: &Path) -> io::Result<libc::statfs> {
    let c = c_path(path)?;
    // SAFETY: zeroed is a valid `statfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}

pub(crate) fn statvfs(path: &Path) -> io::Result<libc::statvfs> {
    let c = c_path(path)?;
    // SAFETY: zeroed is a valid `statvfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}

pub(crate) fn free_of(s: &libc::statvfs) -> u64 {
    #[allow(clippy::useless_conversion)]
    u64::from(s.f_bavail).saturating_mul(u64::from(s.f_frsize))
}

/// Bytes free for this user on the disk of `path` (an existing folder).
pub fn free_space(path: &Path) -> io::Result<u64> {
    Ok(free_of(&statvfs(path)?))
}

/// `a` and `b` (existing) are on the same device.
pub fn same_volume(a: &Path, b: &Path) -> bool {
    matches!((device(a), device(b)), (Ok(a), Ok(b)) if a == b)
}
