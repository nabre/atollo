//! The disk a folder is on: its file system (FAT32 has a 4 GiB limit per
//! file and, like exFAT, no hard links and no permissions), its kind (fixed,
//! removable, network) and its free space. Read when the app starts for the
//! kit's root (PLAN-DATA.md §4.2.3) and before an installation copies
//! between two disks.

use std::io;
use std::path::{Path, PathBuf};

use windows::core::HSTRING;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsKind {
    Ntfs,
    Refs,
    Fat32,
    ExFat,
    Other,
}

impl FsKind {
    fn from_name(name: &str) -> FsKind {
        match name.to_ascii_uppercase().as_str() {
            "NTFS" => FsKind::Ntfs,
            "REFS" => FsKind::Refs,
            "FAT32" | "FAT" => FsKind::Fat32,
            "EXFAT" => FsKind::ExFat,
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

    /// Permissions (DACL) and hard links: NTFS and ReFS only.
    pub fn has_permissions(self) -> bool {
        matches!(self, FsKind::Ntfs | FsKind::Refs)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            FsKind::Ntfs => "NTFS",
            FsKind::Refs => "ReFS",
            FsKind::Fat32 => "FAT32",
            FsKind::ExFat => "exFAT",
            FsKind::Other => "other",
        }
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
    })
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

/// `a` and `b` (existing) are on the same volume: a `rename` between them
/// works; otherwise the files must be copied.
pub fn same_volume(a: &Path, b: &Path) -> bool {
    match (root_of(a), root_of(b)) {
        (Ok(a), Ok(b)) => a.as_os_str().eq_ignore_ascii_case(b.as_os_str()),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_disk() {
        let windir = PathBuf::from(std::env::var("SystemRoot").unwrap_or(r"C:\Windows".into()));
        let v = info(&windir).unwrap();
        assert_eq!(v.fs, FsKind::Ntfs);
        assert_eq!(v.drive, DriveKind::Fixed);
        assert!(v.free > 0);
        assert!(same_volume(&windir, &windir.join("System32")));
        assert!(root_of(&windir).unwrap().to_string_lossy().ends_with('\\'));
    }

    #[test]
    fn limits() {
        assert_eq!(
            FsKind::from_name("FAT32").max_file_size(),
            Some(4_294_967_295)
        );
        assert_eq!(FsKind::from_name("exFAT").max_file_size(), None);
        assert_eq!(FsKind::from_name("NTFS").max_file_size(), None);
        assert!(!FsKind::ExFat.has_permissions() && FsKind::Ntfs.has_permissions());
        assert_eq!(FsKind::from_name("udf"), FsKind::Other);
    }
}
