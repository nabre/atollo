//! The disk a folder is on: its file system (FAT32 has a 4 GiB limit per
//! file and, like exFAT, no hard links and no permissions), its kind (fixed,
//! removable, network) and its free space. Read when the app starts for the
//! kit's root (D-90) and before an installation copies
//! between two disks.
//!
//! On macOS and Linux also how the disk is mounted (D-105): a `noexec`
//! mount, or a FAT32 one with `showexec` (Linux), runs none of the kit's
//! programs: [`disk_notice`] says so, and `main.rs` stops the app at
//! start when the notice is blocking (FAT32 and exFAT are only logged).

use std::io;
use std::path::{Path, PathBuf};

use crate::kit::{Kit, Os};
use crate::os::current::volume as imp;

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
    /// or Linux's `linux_name` derives from the magic number.
    pub(crate) fn from_name(name: &str) -> FsKind {
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
    /// Mounted `noexec` (macOS, Linux): no program on it can start.
    /// Always false on Windows.
    pub noexec: bool,
}

/// How a disk is mounted (Linux `/proc/self/mountinfo`, macOS `statfs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mount {
    /// The folder the disk is mounted on.
    pub point: PathBuf,
    /// The file system's name (`vfat`, `exfat`, `ext4`, `msdos`...).
    pub fs_type: String,
    /// The options of the mount and of the file system, together.
    pub options: Vec<String>,
}

impl Mount {
    fn has(&self, option: &str) -> bool {
        self.options.iter().any(|o| o == option)
    }

    /// Mounted without the right to run programs.
    pub fn noexec(&self) -> bool {
        self.has("noexec")
    }

    /// FAT on Linux with `showexec`: only `.exe`, `.com` and `.bat` files
    /// are executable, so none of the kit's programs is.
    pub fn showexec(&self) -> bool {
        matches!(self.fs_type.as_str(), "vfat" | "msdos" | "fat") && self.has("showexec")
    }
}

/// The file system, kind, free space and `noexec` flag of the disk of
/// `path` (an existing folder).
pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    imp::info(path)
}

/// The mount of the disk of `path` (an existing folder): `None` on
/// Windows, which mounts nothing `noexec`.
pub fn mount_of(path: &Path) -> Option<Mount> {
    imp::mount_of(path)
}

/// Bytes free for this user on the disk of `path` (an existing folder).
pub fn free_space(path: &Path) -> io::Result<u64> {
    imp::free_space(path)
}

/// `a` and `b` (existing) are on the same volume: a `rename` between them
/// works; otherwise the files must be copied.
pub fn same_volume(a: &Path, b: &Path) -> bool {
    imp::same_volume(a, b)
}

/// What the kit's disk means for the app (D-105).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiskNotice {
    /// The kit cannot work there: the app stops at start with the text.
    pub blocking: bool,
    pub text: String,
}

/// The notice for a kit in `folder` on `fs`, on `os`. `noexec` is the
/// disk's flag as `statvfs`/`statfs` read it, `mount` its mount when it was
/// found (on btrfs subvolumes `/proc/self/mountinfo` has no line with the
/// folder's device, so it may be missing). Windows has none (nothing
/// changes there). `noexec` (the flag or the mount option), or FAT32 with
/// `showexec` on Linux: no program of the kit could start, so the app
/// stops with the command that fixes it, on the mount point or, unknown,
/// on the folder. FAT32 and exFAT otherwise: a warning, since links,
/// sockets and permissions are missing (installations do without them,
/// D-105).
pub fn notice_for(
    os: Os,
    fs: FsKind,
    noexec: bool,
    mount: Option<&Mount>,
    folder: &Path,
    lang: &str,
) -> Option<DiskNotice> {
    use crate::common::i18n::t;
    if os.is_windows() {
        return None;
    }
    let point = mount
        .map_or(folder, |m| m.point.as_path())
        .display()
        .to_string();
    let quoted = super::sh_quote(&point);
    let blocking = |text: String| {
        Some(DiskNotice {
            blocking: true,
            text,
        })
    };
    if noexec || mount.is_some_and(Mount::noexec) {
        let remount = if os.is_macos() {
            format!("sudo mount -u -o exec {quoted}")
        } else {
            format!("sudo mount -o remount,exec {quoted}")
        };
        return blocking(format!(
            "{}\n\n{}",
            t(lang, "Il disco del kit ({0}) è montato con l'opzione noexec: da lì non può partire nessun programma, e Atollo non installa nulla.", &[&point]),
            t(lang, "Rimontalo permettendo i programmi, per esempio con: {0}. Poi riavvia Atollo.", &[&remount]),
        ));
    }
    if os.is_linux() && mount.is_some_and(Mount::showexec) {
        return blocking(format!(
            "{}\n\n{}",
            t(lang, "Il disco del kit ({0}) è FAT32 montato con l'opzione showexec: Linux lascia partire solo i file .exe, .com e .bat, non i programmi del kit.", &[&point]),
            t(lang, "Formatta la chiavetta in exFAT (il kit funziona così anche su Windows e macOS), oppure rimontala senza showexec. Poi riavvia Atollo.", &[]),
        ));
    }
    let warning = match fs {
        FsKind::Fat32 => t(lang, "Il disco del kit è FAT32: file fino a 4 GB, niente link simbolici, socket e permessi. Le installazioni ne fanno a meno; per un kit su chiavetta è meglio exFAT.", &[]),
        FsKind::ExFat => t(lang, "Il disco del kit è exFAT: niente link simbolici, socket e permessi. Le installazioni ne fanno a meno, ma alcune occupano più spazio.", &[]),
        _ => return None,
    };
    Some(DiskNotice {
        blocking: false,
        text: warning,
    })
}

/// Where the kit's programs run: `data` may be on another disk than the
/// app (a link), and a `noexec` root would not have started the app at all.
fn exec_folder(kit: &Kit) -> PathBuf {
    let data = kit.data();
    if data.is_dir() {
        data
    } else {
        kit.root().to_path_buf()
    }
}

/// The disk the kit's programs run from is mounted `noexec` (always
/// `false` on Windows): for the Settings row (D-105).
pub fn kit_noexec(kit: &Kit) -> bool {
    !Os::current().is_windows() && info(&exec_folder(kit)).is_ok_and(|v| v.noexec)
}

/// The notice for the disk of `kit`, read now (`None` on Windows): the
/// file system of the root; the `noexec` flag and the mount of the folder
/// the programs run from.
pub fn disk_notice(kit: &Kit, lang: &str) -> Option<DiskNotice> {
    let os = Os::current();
    if os.is_windows() {
        return None;
    }
    let fs = kit.volume().map_or(FsKind::Other, |v| v.fs);
    let folder = exec_folder(kit);
    // The flag decides `noexec`; the mount adds `showexec` and the mount
    // point for the command. Both failing is said, not hidden.
    let noexec = match info(&folder) {
        Ok(v) => v.noexec,
        Err(e) => {
            crate::common::log::warn(&format!("{}: statvfs: {e}", folder.display()));
            false
        }
    };
    let mount = mount_of(&folder);
    if mount.is_none() {
        crate::common::log::warn(&format!(
            "{}: mount not found, showexec not checked",
            folder.display()
        ));
    }
    notice_for(os, fs, noexec, mount.as_ref(), &folder, lang)
}

#[cfg(test)]
#[path = "../../../test/rust/platform/volume.rs"]
mod tests;
