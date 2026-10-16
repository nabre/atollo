//! The disk a folder is on: its file system (FAT32 has a 4 GiB limit per
//! file and, like exFAT, no hard links and no permissions), its kind (fixed,
//! removable, network) and its free space. Read when the app starts for the
//! kit's root (PLAN-DATA.md §4.2.3) and before an installation copies
//! between two disks.

use std::io;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsKind {
    Ntfs,
    Refs,
    Fat32,
    ExFat,
    /// A file system of macOS or Linux with owners, modes and hard links
    /// (APFS, ext4, btrfs...).
    Posix,
    Other,
}

impl FsKind {
    /// From the name Windows gives, or the one macOS gives (`msdos`, `apfs`)
    /// or `linux_name` derives from the magic number.
    fn from_name(name: &str) -> FsKind {
        match name.to_ascii_uppercase().as_str() {
            "NTFS" => FsKind::Ntfs,
            "REFS" => FsKind::Refs,
            "FAT32" | "FAT" | "MSDOS" | "VFAT" => FsKind::Fat32,
            "EXFAT" => FsKind::ExFat,
            "APFS" | "HFS" | "EXT2" | "EXT3" | "EXT4" | "BTRFS" | "XFS" | "ZFS" | "F2FS"
            | "TMPFS" => FsKind::Posix,
            _ => FsKind::Other,
        }
    }

    /// The largest file the file system holds, when it has a limit below
    /// what an installation may need (FAT32: 4 GiB - 1).
    pub fn max_file_size(self) -> Option<u64> {
        match self {
            FsKind::Fat32 => Some(u32::MAX as u64),
            _ => None,
        }
    }

    /// Permissions (DACL, or owner and mode) and hard links: NTFS, ReFS and
    /// the Unix file systems.
    pub fn has_permissions(self) -> bool {
        matches!(self, FsKind::Ntfs | FsKind::Refs | FsKind::Posix)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FsKind::Ntfs => "NTFS",
            FsKind::Refs => "ReFS",
            FsKind::Fat32 => "FAT32",
            FsKind::ExFat => "exFAT",
            FsKind::Posix => "POSIX",
            FsKind::Other => "other",
        }
    }
}

/// The name of a Linux file system from its `statfs` magic number; network
/// file systems are told apart for `DriveKind::Remote`.
#[cfg(any(target_os = "linux", test))]
fn linux_name(magic: u32) -> (&'static str, bool) {
    match magic {
        0x4d44 => ("FAT", false),
        0x2011_bab0 => ("EXFAT", false),
        0x5346_544e => ("NTFS", false),
        0xef53 => ("EXT4", false),
        0x9123_683e => ("BTRFS", false),
        0x5846_5342 => ("XFS", false),
        0x2fc1_2fc1 => ("ZFS", false),
        0xf2f5_2010 => ("F2FS", false),
        0x0102_1994 => ("TMPFS", false),
        0x6969 | 0xff53_4d42 | 0xfe53_4d42 | 0x517b => ("", true),
        _ => ("", false),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveKind {
    Fixed,
    Removable,
    Remote,
    Other,
}

impl DriveKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DriveKind::Fixed => "fixed",
            DriveKind::Removable => "removable",
            DriveKind::Remote => "remote",
            DriveKind::Other => "other",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeInfo {
    pub fs: FsKind,
    pub drive: DriveKind,
    /// Bytes free for this user.
    pub free: u64,
}

/// The device of `path` (`st_dev`): the same number means the same volume.
#[cfg(unix)]
fn device(path: &Path) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata(path)?.dev())
}

/// The mount point of the volume `path` is on: the highest folder above it
/// that is still on the same device.
#[cfg(unix)]
pub fn root_of(path: &Path) -> io::Result<PathBuf> {
    let path = std::fs::canonicalize(path)?;
    let dev = device(&path)?;
    let mut root = path.clone();
    for p in path.ancestors().skip(1) {
        if device(p).ok() != Some(dev) {
            break;
        }
        root = p.to_path_buf();
    }
    Ok(root)
}

#[cfg(unix)]
fn statfs(path: &Path) -> io::Result<libc::statfs> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)?;
    // SAFETY: zeroed is a valid `statfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}

#[cfg(target_os = "linux")]
pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    let s = statfs(path)?;
    #[allow(clippy::unnecessary_cast)]
    let (name, remote) = linux_name(s.f_type as u32);
    Ok(VolumeInfo {
        fs: FsKind::from_name(name),
        drive: if remote {
            DriveKind::Remote
        } else {
            DriveKind::Other
        },
        free: free_space(path)?,
    })
}

#[cfg(all(unix, not(target_os = "linux")))]
pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    /// `<sys/mount.h>`: a removable medium (not in the `libc` crate).
    const MNT_REMOVABLE: u32 = 0x0000_0200;
    let s = statfs(path)?;
    // SAFETY: `f_fstypename` is a NUL-terminated name filled by statfs.
    let name = unsafe { std::ffi::CStr::from_ptr(s.f_fstypename.as_ptr()) }.to_string_lossy();
    let remote = matches!(name.as_ref(), "nfs" | "smbfs" | "afpfs" | "webdav");
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
    })
}

/// Bytes free for this user on the disk of `path` (an existing folder).
#[cfg(unix)]
pub fn free_space(path: &Path) -> io::Result<u64> {
    use std::os::unix::ffi::OsStrExt;
    let c = std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)?;
    // SAFETY: zeroed is a valid `statvfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    #[allow(clippy::useless_conversion)]
    Ok(u64::from(s.f_bavail).saturating_mul(u64::from(s.f_frsize)))
}

/// `a` and `b` (existing) are on the same device: a `rename` between them
/// works; otherwise the files must be copied.
#[cfg(unix)]
pub fn same_volume(a: &Path, b: &Path) -> bool {
    matches!((device(a), device(b)), (Ok(a), Ok(b)) if a == b)
}

/// The root of the volume `path` is on (`D:\`, or a mounted folder).
#[cfg(windows)]
pub fn root_of(path: &Path) -> io::Result<PathBuf> {
    let mut buf = [0u16; 1024];
    // SAFETY: the buffer outlives the call, which writes at most its length.
    unsafe { GetVolumePathNameW(&HSTRING::from(path.as_os_str()), &mut buf) }
        .map_err(io::Error::other)?;
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Ok(PathBuf::from(String::from_utf16_lossy(&buf[..len])))
}

#[cfg(windows)]
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
    })
}

/// Bytes free for this user on the disk of `path` (an existing folder).
#[cfg(windows)]
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

/// `a` and `b` (existing) are on the same volume: a `rename` between them
/// works; otherwise the files must be copied.
#[cfg(windows)]
pub fn same_volume(a: &Path, b: &Path) -> bool {
    match (root_of(a), root_of(b)) {
        (Ok(a), Ok(b)) => a.as_os_str().eq_ignore_ascii_case(b.as_os_str()),
        _ => false,
    }
}

#[cfg(test)]
#[path = "../../../test/rust/os/volume.rs"]
mod tests;
