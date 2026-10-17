use super::*;

#[cfg(windows)]
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

#[cfg(unix)]
#[test]
fn system_disk() {
    let tmp = std::env::temp_dir();
    let v = info(&tmp).unwrap();
    assert!(v.free > 0);
    assert!(same_volume(&tmp, &tmp));
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

#[test]
fn unix_names() {
    // macOS names, and Linux magic numbers through `linux_name`.
    assert_eq!(FsKind::from_name("msdos"), FsKind::Fat32);
    assert_eq!(FsKind::from_name("exfat"), FsKind::ExFat);
    assert_eq!(FsKind::from_name("apfs"), FsKind::Posix);
    assert!(FsKind::Posix.has_permissions());
    assert_eq!(FsKind::from_name(linux_name(0x4d44).0), FsKind::Fat32);
    assert_eq!(FsKind::from_name(linux_name(0xef53).0), FsKind::Posix);
    assert_eq!(linux_name(0x6969), ("", true));
    assert_eq!(FsKind::from_name(linux_name(1).0), FsKind::Other);
}

/// T1.7.2: a partition's `removable` is its disk's; a disk on USB is
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
    let kit = at(8, 17, "/media/me/KIT A/Dev-Kit").unwrap();
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
fn startup_notices() {
    let mount = |fs_type: &str, options: &[&str]| Mount {
        point: PathBuf::from("/media/me/KIT"),
        fs_type: fs_type.into(),
        options: options.iter().map(|o| o.to_string()).collect(),
    };
    let noexec = mount("ext4", &["rw", "noexec"]);
    let linux = notice_for(Os::Linux, FsKind::Posix, Some(&noexec), "it").unwrap();
    assert!(linux.blocking);
    assert!(
        linux
            .text
            .contains("sudo mount -o remount,exec \"/media/me/KIT\""),
        "{}",
        linux.text
    );
    let mac = notice_for(Os::Darwin, FsKind::Posix, Some(&noexec), "it").unwrap();
    assert!(
        mac.text.contains("sudo mount -u -o exec \"/media/me/KIT\""),
        "{}",
        mac.text
    );
    let en = notice_for(Os::Linux, FsKind::Posix, Some(&noexec), "en").unwrap();
    assert_ne!(en.text, linux.text);

    let showexec = mount("vfat", &["rw", "showexec"]);
    let fat = notice_for(Os::Linux, FsKind::Fat32, Some(&showexec), "it").unwrap();
    assert!(fat.blocking && fat.text.contains("exFAT"), "{}", fat.text);
    // showexec means nothing on macOS: FAT32 is a warning there.
    let mac_fat = notice_for(Os::Darwin, FsKind::Fat32, Some(&showexec), "it").unwrap();
    assert!(!mac_fat.blocking);

    let ex = notice_for(
        Os::Linux,
        FsKind::ExFat,
        Some(&mount("exfat", &["rw"])),
        "it",
    )
    .unwrap();
    assert!(!ex.blocking && ex.text.contains("exFAT"));
    assert_eq!(
        notice_for(
            Os::Linux,
            FsKind::Posix,
            Some(&mount("ext4", &["rw"])),
            "it"
        ),
        None
    );
    assert_eq!(notice_for(Os::Darwin, FsKind::Posix, None, "it"), None);
    // Windows: nothing changes, whatever the disk.
    for fs in [FsKind::Fat32, FsKind::ExFat, FsKind::Ntfs] {
        assert_eq!(notice_for(Os::Win32, fs, Some(&noexec), "it"), None);
    }
}

#[cfg(windows)]
#[test]
fn no_startup_problem_on_windows() {
    let tmp = crate::kit::tests::TempDir::new("volume-win");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64).with_host(None);
    assert_eq!(disk_notice(&kit, "it"), None);
    assert_eq!(startup_problem(&kit, "it"), None);
    assert!(!kit.volume().unwrap().noexec);
}

#[cfg(target_os = "linux")]
#[test]
fn linux_mount_of_the_temp_folder() {
    let m = mount_of(&std::env::temp_dir()).unwrap();
    assert!(std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .starts_with(&m.point));
    assert!(!m.fs_type.is_empty());
}
