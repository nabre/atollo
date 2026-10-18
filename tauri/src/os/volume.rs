//! The disk a folder is on: its file system (FAT32 has a 4 GiB limit per
//! file and, like exFAT, no hard links and no permissions), its kind (fixed,
//! removable, network) and its free space. Read when the app starts for the
//! kit's root (D-90) and before an installation copies
//! between two disks.
//!
//! On macOS and Linux also how the disk is mounted (D-105): a `noexec`
//! mount, or a FAT32 one with `showexec` (Linux), runs none of the kit's
//! programs, and the app says so at start ([`startup_problem`]).

use std::io;
use std::path::{Path, PathBuf};

#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetVolumeInformationW, GetVolumePathNameW,
};

use crate::kit::{Kit, Os};

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
    /// Mounted `noexec` (macOS, Linux): no program on it can start.
    /// Always false on Windows.
    pub noexec: bool,
}

/// The kind of a Linux block device from its folder in `/sys/devices`
/// (`/sys/dev/block/<major>:<minor>` resolved): a partition's `removable`
/// flag is its disk's, the folder above. A disk on USB is removable even
/// when it says otherwise (USB hard disks and many card readers report 0),
/// and so is an SD card in a built-in reader (`mmcblk*`, removable 0, its
/// card's `device/type` is `SD`; soldered eMMC says `MMC` and stays fixed).
#[cfg(any(target_os = "linux", test))]
fn block_drive_kind(device: &Path) -> DriveKind {
    let disk = if device.join("partition").is_file() {
        device.parent().unwrap_or(device)
    } else {
        device
    };
    let usb = device
        .components()
        .any(|c| c.as_os_str().to_str().is_some_and(|s| s.starts_with("usb")));
    let sd_card =
        std::fs::read_to_string(disk.join("device").join("type")).is_ok_and(|t| t.trim() == "SD");
    match std::fs::read_to_string(disk.join("removable")).map(|s| s.trim() == "1") {
        Ok(true) => DriveKind::Removable,
        _ if usb || sd_card => DriveKind::Removable,
        Ok(false) => DriveKind::Fixed,
        Err(_) => DriveKind::Other,
    }
}

/// The kind of the Linux disk `major:minor`, from the `sys` tree (`/sys`):
/// `Other` for a device without one (btrfs subvolumes, overlays, tmpfs).
#[cfg(target_os = "linux")]
fn linux_drive_kind(sys: &Path, major: u32, minor: u32) -> DriveKind {
    let link = sys.join("dev/block").join(format!("{major}:{minor}"));
    match std::fs::canonicalize(link) {
        Ok(device) => block_drive_kind(&device),
        Err(_) => DriveKind::Other,
    }
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

/// `\040` (space), `\011`, `\012`, `\134` in a field of `mountinfo`.
#[cfg(any(target_os = "linux", test))]
fn unescape_mount(field: &str) -> String {
    let bytes = field.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        // Three octal digits of one byte (at most \377).
        let octal = bytes.get(i + 1..i + 4).filter(|d| {
            (b'0'..=b'3').contains(&d[0]) && d.iter().all(|b| (b'0'..=b'7').contains(b))
        });
        match (bytes[i], octal) {
            (b'\\', Some(d)) => {
                out.push((d[0] - b'0') * 64 + (d[1] - b'0') * 8 + (d[2] - b'0'));
                i += 4;
            }
            (b, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The mount of the device `major:minor` holding `path`, from the text of
/// `/proc/self/mountinfo`: of the lines of that device, the one whose
/// mount point is the longest prefix of `path` (a device can be mounted
/// more than once).
#[cfg(any(target_os = "linux", test))]
fn mount_from_info(text: &str, major: u32, minor: u32, path: &Path) -> Option<Mount> {
    let device = format!("{major}:{minor}");
    text.lines()
        .filter_map(|line| {
            // id parent major:minor root point options [optional...] - type source super
            let (left, right) = line.split_once(" - ")?;
            let left: Vec<&str> = left.split(' ').collect();
            let right: Vec<&str> = right.split(' ').collect();
            if left.get(2) != Some(&device.as_str()) {
                return None;
            }
            let point = PathBuf::from(unescape_mount(left.get(4)?));
            let options = left
                .get(5)?
                .split(',')
                .chain(right.get(2).map_or("", |s| s).split(','))
                .filter(|o| !o.is_empty())
                .map(str::to_owned)
                .collect();
            Some(Mount {
                point,
                fs_type: right.first()?.to_string(),
                options,
            })
        })
        .filter(|m| path.starts_with(&m.point))
        .max_by_key(|m| m.point.as_os_str().len())
}

/// The device of `path` (`st_dev`): the same number means the same volume.
#[cfg(unix)]
fn device(path: &Path) -> io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    Ok(std::fs::metadata(path)?.dev())
}

#[cfg(unix)]
fn c_path(path: &Path) -> io::Result<std::ffi::CString> {
    use std::os::unix::ffi::OsStrExt;
    std::ffi::CString::new(path.as_os_str().as_bytes()).map_err(io::Error::other)
}

#[cfg(unix)]
fn statfs(path: &Path) -> io::Result<libc::statfs> {
    let c = c_path(path)?;
    // SAFETY: zeroed is a valid `statfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}

#[cfg(unix)]
fn statvfs(path: &Path) -> io::Result<libc::statvfs> {
    let c = c_path(path)?;
    // SAFETY: zeroed is a valid `statvfs`; `c` is NUL-terminated and alive.
    let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(c.as_ptr(), &mut s) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(s)
}

#[cfg(target_os = "linux")]
pub fn info(path: &Path) -> io::Result<VolumeInfo> {
    let s = statfs(path)?;
    #[allow(clippy::unnecessary_cast)]
    let (name, remote) = linux_name(s.f_type as u32);
    let vfs = statvfs(path)?;
    let dev = device(path)?;
    Ok(VolumeInfo {
        fs: FsKind::from_name(name),
        drive: if remote {
            DriveKind::Remote
        } else {
            linux_drive_kind(Path::new("/sys"), libc::major(dev), libc::minor(dev))
        },
        free: free_of(&vfs),
        noexec: vfs.f_flag & libc::ST_NOEXEC != 0,
    })
}

/// The mount of the disk of `path` (an existing folder).
#[cfg(target_os = "linux")]
pub fn mount_of(path: &Path) -> Option<Mount> {
    let dev = device(path).ok()?;
    let text = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    mount_from_info(&text, libc::major(dev), libc::minor(dev), &path)
}

/// `f_fstypename` or `f_mntonname` of a macOS `statfs`.
#[cfg(all(unix, not(target_os = "linux")))]
fn c_text(field: &[libc::c_char]) -> String {
    // SAFETY: statfs fills these fields with NUL-terminated strings.
    unsafe { std::ffi::CStr::from_ptr(field.as_ptr()) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(all(unix, not(target_os = "linux")))]
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
#[cfg(all(unix, not(target_os = "linux")))]
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

#[cfg(unix)]
fn free_of(s: &libc::statvfs) -> u64 {
    #[allow(clippy::useless_conversion)]
    u64::from(s.f_bavail).saturating_mul(u64::from(s.f_frsize))
}

/// Bytes free for this user on the disk of `path` (an existing folder).
#[cfg(unix)]
pub fn free_space(path: &Path) -> io::Result<u64> {
    Ok(free_of(&statvfs(path)?))
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
        noexec: false,
    })
}

/// Windows mounts nothing `noexec`: no mount to tell about.
#[cfg(windows)]
pub fn mount_of(_path: &Path) -> Option<Mount> {
    None
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
    use crate::i18n::t;
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
            crate::log::warn(&format!("{}: statvfs: {e}", folder.display()));
            false
        }
    };
    let mount = mount_of(&folder);
    if mount.is_none() {
        crate::log::warn(&format!(
            "{}: mount not found, showexec not checked",
            folder.display()
        ));
    }
    notice_for(os, fs, noexec, mount.as_ref(), &folder, lang)
}

/// The message that stops the app at start (D-105): the kit's disk runs
/// no program. `None`: the app can start.
#[allow(dead_code)] // called by main.rs at start (D-105)
pub fn startup_problem(kit: &Kit, lang: &str) -> Option<String> {
    disk_notice(kit, lang)
        .filter(|n| n.blocking)
        .map(|n| n.text)
}

#[cfg(test)]
#[path = "../../../test/rust/os/volume.rs"]
mod tests;
