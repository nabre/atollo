//! The disk a folder is on (`platform::volume`) on macOS: `statfs` gives
//! the file system's name, its flags and the mount point.

use std::io;
use std::path::{Path, PathBuf};

use crate::os::unix::volume::statfs;
pub use crate::os::unix::volume::{free_space, same_volume};
use crate::platform::volume::{DriveKind, FsKind, Mount, VolumeInfo};

/// `f_fstypename` or `f_mntonname` of a macOS `statfs`.
fn c_text(field: &[libc::c_char]) -> String {
    // SAFETY: statfs fills these fields with NUL-terminated strings.
    unsafe { std::ffi::CStr::from_ptr(field.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    /// `<sys/mount.h>`: a removable medium (not in the `libc` crate).
    const MNT_REMOVABLE: u32 = 0x0000_0200;
    let s = statfs(path)?;
    let name = c_text(&s.f_fstypename);
    let remote = matches!(name.as_str(), "nfs" | "smbfs" | "afpfs" | "webdav");
    let local = s.f_flags & libc::MNT_LOCAL as u32 != 0;
    Ok(VolumeInfo {
        fs: FsKind::from_name(&name),
        drive: match (remote, local) {
            (true, _) => DriveKind::Remote,
            (false, true) if s.f_flags & MNT_REMOVABLE != 0 => DriveKind::Removable,
            (false, true) => DriveKind::Fixed,
            _ => DriveKind::Other,
        },
        free: free_space(path)?,
        noexec: s.f_flags & libc::MNT_NOEXEC as u32 != 0,
    })
}

/// The mount of the disk of `path`: macOS has no `showexec`, so only the
/// mount point, the name and `noexec` matter.
pub fn mount_of(path: &Path) -> Option<Mount> {
    let s = statfs(path).ok()?;
    let noexec = s.f_flags & libc::MNT_NOEXEC as u32 != 0;
    Some(Mount {
        point: PathBuf::from(c_text(&s.f_mntonname)),
        fs_type: c_text(&s.f_fstypename),
        options: if noexec {
            vec!["noexec".into()]
        } else {
            Vec::new()
        },
    })
}
