//! The disk a folder is on (`platform::volume`) on Linux: the file system
//! from the `statfs` magic number, the kind of disk from `/sys`, the mount
//! from `/proc/self/mountinfo`.

use std::io;
use std::path::{Path, PathBuf};

use crate::os::unix::volume::{device, free_of, statfs, statvfs};
pub use crate::os::unix::volume::{free_space, same_volume};
use crate::platform::volume::{DriveKind, FsKind, Mount, VolumeInfo};

/// The name of a Linux file system from its `statfs` magic number; network
/// file systems are told apart for `DriveKind::Remote`.
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

/// The kind of a Linux block device from its folder in `/sys/devices`
/// (`/sys/dev/block/<major>:<minor>` resolved): a partition's `removable`
/// flag is its disk's, the folder above. A disk on USB is removable even
/// when it says otherwise (USB hard disks and many card readers report 0),
/// and so is an SD card in a built-in reader (`mmcblk*`, removable 0, its
/// card's `device/type` is `SD`; soldered eMMC says `MMC` and stays fixed).
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
fn linux_drive_kind(sys: &Path, major: u32, minor: u32) -> DriveKind {
    let link = sys.join("dev/block").join(format!("{major}:{minor}"));
    match std::fs::canonicalize(link) {
        Ok(device) => block_drive_kind(&device),
        Err(_) => DriveKind::Other,
    }
}

/// `\040` (space), `\011`, `\012`, `\134` in a field of `mountinfo`.
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
pub fn mount_of(path: &Path) -> Option<Mount> {
    let dev = device(path).ok()?;
    let text = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    let path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    mount_from_info(&text, libc::major(dev), libc::minor(dev), &path)
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/volume.rs"]
mod tests;
