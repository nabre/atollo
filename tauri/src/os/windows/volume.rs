//! The disk a folder is on (`platform::volume`), from the Win32 volume
//! functions.

use std::io;
use std::path::{Path, PathBuf};

use windows::core::HSTRING;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
};

use crate::platform::volume::{DriveKind, FsKind, Mount, VolumeInfo};

/// The root of the volume `path` is on (`D:\`, or a mounted folder).
pub fn root_of(path: &Path) -> io::Result<PathBuf> {
    let mut buf = [0u16; 1024];
    // SAFETY: the buffer outlives the call, which writes at most its length.
    unsafe { GetVolumePathNameW(&HSTRING::from(path.as_os_str()), &mut buf) }
        .map_err(io::Error::other)?;
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Ok(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    let root = root_of(path)?;
    let root_h = HSTRING::from(root.as_os_str());
    let mut fs_name = [0u16; 64];
    // SAFETY: only the file system name buffer is passed, alive during the call.
    unsafe { GetVolumeInformationW(&root_h, None, None, None, None, Some(&mut fs_name)) }
        .map_err(io::Error::other)?;
    let len = fs_name
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(fs_name.len());
    let fs = FsKind::from_name(&String::from_utf16_lossy(&fs_name[..len]));
    // SAFETY: a plain query on a valid string.
    let drive = match unsafe { GetDriveTypeW(&root_h) } {
        3 => DriveKind::Fixed,
        2 => DriveKind::Removable,
        4 => DriveKind::Remote,
        _ => DriveKind::Other,
    };
    Ok(VolumeInfo {
        fs,
        drive,
        free: free_space(path)?,
        noexec: false,
    })
}

/// Windows mounts nothing `noexec`: no mount to tell about.
pub fn mount_of(_path: &Path) -> Option<Mount> {
    None
}

/// Bytes free for this user on the disk of `path` (an existing folder).
pub fn free_space(path: &Path) -> io::Result<u64> {
    let mut free = 0u64;
    // SAFETY: `free` outlives the call.
    unsafe {
        GetDiskFreeSpaceExW(
            &HSTRING::from(path.as_os_str()),
            Some(&mut free),
            None,
            None,
        )
    }
    .map_err(io::Error::other)?;
    Ok(free)
}

/// `a` and `b` (existing) are on the same volume.
pub fn same_volume(a: &Path, b: &Path) -> bool {
    match (root_of(a), root_of(b)) {
        (Ok(a), Ok(b)) => a.as_os_str().eq_ignore_ascii_case(b.as_os_str()),
        _ => false,
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/volume.rs"]
mod tests;
