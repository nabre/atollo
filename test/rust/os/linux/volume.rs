use super::*;

#[test]
fn magic_numbers() {
    assert_eq!(FsKind::from_name(linux_name(0x4d44).0), FsKind::Fat32);
    assert_eq!(FsKind::from_name(linux_name(0xef53).0), FsKind::Posix);
    assert_eq!(linux_name(0x6969), ("", true));
    assert_eq!(FsKind::from_name(linux_name(1).0), FsKind::Other);
}

/// D-105: a partition's `removable` is its disk's; a disk on USB is
/// removable whatever it says; no flag at all is `Other`.
#[test]
fn linux_removable_from_a_fake_sys_tree() {
    let tmp = crate::kit::tests::TempDir::new("volume-sys");
    let disk = |rel: &str, removable: Option<&str>| {
        let dir = tmp.0.join(rel);
        std::fs::create_dir_all(&dir).unwrap();
        if let Some(r) = removable {
            std::fs::write(dir.join("removable"), format!("{r}\n")).unwrap();
        }
        dir
    };
    let partition = |disk: &Path, name: &str| {
        let dir = disk.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("partition"), "1\n").unwrap();
        dir
    };
    let stick = disk("devices/pci0/a/host6/block/sdb", Some("1"));
    assert_eq!(
        block_drive_kind(&partition(&stick, "sdb1")),
        DriveKind::Removable
    );
    assert_eq!(block_drive_kind(&stick), DriveKind::Removable);
    let ssd = disk("devices/pci0/nvme/nvme0/block/nvme0n1", Some("0"));
    assert_eq!(
        block_drive_kind(&partition(&ssd, "nvme0n1p2")),
        DriveKind::Fixed
    );
    let usb_hdd = disk("devices/pci0/usb2/2-1/host7/block/sdc", Some("0"));
    assert_eq!(
        block_drive_kind(&partition(&usb_hdd, "sdc1")),
        DriveKind::Removable
    );
    // An SD card in a built-in reader says removable 0; soldered eMMC is
    // also `mmcblk` but its card type is MMC (`-` for the real `:`, which
    // Windows folders cannot hold).
    let card = |rel: &str, kind: &str| {
        let dir = disk(rel, Some("0"));
        std::fs::create_dir_all(dir.join("device")).unwrap();
        std::fs::write(dir.join("device/type"), format!("{kind}\n")).unwrap();
        dir
    };
    let sd = card("devices/pci0/mmc_host/mmc0/mmc0-aaaa/block/mmcblk0", "SD");
    assert_eq!(
        block_drive_kind(&partition(&sd, "mmcblk0p1")),
        DriveKind::Removable
    );
    let emmc = card("devices/pci0/mmc_host/mmc1/mmc1-0001/block/mmcblk1", "MMC");
    assert_eq!(
        block_drive_kind(&partition(&emmc, "mmcblk1p2")),
        DriveKind::Fixed
    );
    let odd = disk("devices/virtual/block/zram0", None);
    assert_eq!(block_drive_kind(&odd), DriveKind::Other);
}

const MOUNTINFO: &str = "\
22 1 259:2 / / rw,relatime shared:1 - ext4 /dev/nvme0n1p2 rw
40 22 8:17 / /media/me/KIT\\040A rw,nosuid,nodev,noexec,relatime shared:2 - vfat /dev/sdb1 rw,uid=1000,fmask=0022,showexec,utf8
41 22 8:33 / /media/me/EX rw,nosuid,nodev,relatime shared:3 - exfat /dev/sdc1 rw,uid=1000
42 41 8:33 /sub /home/me/bound rw,relatime shared:3 - exfat /dev/sdc1 rw
";

#[test]
fn linux_mount_options_from_mountinfo() {
    let at = |maj, min, p: &str| mount_from_info(MOUNTINFO, maj, min, Path::new(p));
    let kit = at(8, 17, "/media/me/KIT A/Atollo").unwrap();
    assert_eq!(kit.point, PathBuf::from("/media/me/KIT A"));
    assert_eq!(kit.fs_type, "vfat");
    assert!(kit.noexec() && kit.showexec());
    let ex = at(8, 33, "/media/me/EX/kit").unwrap();
    assert!(!ex.noexec() && !ex.showexec());
    // The same device mounted twice: the mount that holds the path.
    assert_eq!(
        at(8, 33, "/home/me/bound/kit").unwrap().point,
        PathBuf::from("/home/me/bound")
    );
    assert_eq!(at(8, 99, "/media/me/EX"), None);
    assert_eq!(at(259, 2, "/home/me").unwrap().fs_type, "ext4");
    assert_eq!(unescape_mount(r"a\040b\134c\9"), r"a b\c\9");
}

#[test]
fn linux_mount_of_the_temp_folder() {
    let m = mount_of(&std::env::temp_dir()).unwrap();
    assert!(std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .starts_with(&m.point));
    assert!(!m.fs_type.is_empty());
}
