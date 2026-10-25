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
    // macOS names; Linux's magic numbers are in os/linux.
    assert_eq!(FsKind::from_name("msdos"), FsKind::Fat32);
    assert_eq!(FsKind::from_name("exfat"), FsKind::ExFat);
    assert_eq!(FsKind::from_name("apfs"), FsKind::Posix);
    assert!(FsKind::Posix.has_permissions());
}

#[test]
fn startup_notices() {
    let mount = |fs_type: &str, options: &[&str]| Mount {
        point: PathBuf::from("/media/me/KIT"),
        fs_type: fs_type.into(),
        options: options.iter().map(|o| o.to_string()).collect(),
    };
    let folder = Path::new("/home/me/kit/data");
    let notice_for =
        |os, fs, mount: Option<&Mount>, lang| notice_for(os, fs, false, mount, folder, lang);
    let noexec = mount("ext4", &["rw", "noexec"]);
    let linux = notice_for(Os::Linux, FsKind::Posix, Some(&noexec), "it").unwrap();
    assert!(linux.blocking);
    assert!(
        linux
            .text
            .contains("sudo mount -o remount,exec '/media/me/KIT'"),
        "{}",
        linux.text
    );
    let mac = notice_for(Os::Darwin, FsKind::Posix, Some(&noexec), "it").unwrap();
    assert!(
        mac.text.contains("sudo mount -u -o exec '/media/me/KIT'"),
        "{}",
        mac.text
    );
    // A label is chosen by whoever made the stick: nothing in it expands
    // when the user pastes the command.
    let crafted = Mount {
        point: PathBuf::from("/media/me/$(rm x)`y`'z"),
        ..noexec.clone()
    };
    let text = notice_for(Os::Linux, FsKind::Posix, Some(&crafted), "it")
        .unwrap()
        .text;
    assert!(
        text.contains(r"remount,exec '/media/me/$(rm x)`y`'\''z'"),
        "{text}"
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

/// The `statvfs` flag alone blocks the start: on a btrfs subvolume (the
/// default `/home` of Fedora and openSUSE) `mountinfo` has no line with
/// the folder's device, and the kit must not start anyway.
#[test]
fn noexec_flag_blocks_without_the_mount() {
    let folder = Path::new("/home/me/kit/data");
    for os in [Os::Linux, Os::Darwin] {
        let n = notice_for(os, FsKind::Posix, true, None, folder, "it").unwrap();
        assert!(n.blocking, "{os:?}");
        // The command names the folder when the mount point is unknown.
        assert!(n.text.contains("'/home/me/kit/data'"), "{}", n.text);
    }
    // The flag set, the mount found without the option: still blocking,
    // on the mount point.
    let found = Mount {
        point: PathBuf::from("/home"),
        fs_type: "btrfs".into(),
        options: vec!["rw".into()],
    };
    let n = notice_for(Os::Linux, FsKind::Posix, true, Some(&found), folder, "it").unwrap();
    assert!(
        n.blocking && n.text.contains("remount,exec '/home'"),
        "{}",
        n.text
    );
    // Nothing set, nothing found: the kit starts.
    assert_eq!(
        notice_for(Os::Linux, FsKind::Posix, false, None, folder, "it"),
        None
    );
    assert_eq!(
        notice_for(Os::Win32, FsKind::Ntfs, true, None, folder, "it"),
        None
    );
}

#[cfg(windows)]
#[test]
fn no_startup_problem_on_windows() {
    let tmp = crate::kit::tests::TempDir::new("volume-win");
    let kit = Kit::at(tmp.0.clone(), crate::kit::Arch::X64).with_host(None);
    assert_eq!(disk_notice(&kit, "it"), None);
    assert!(!kit_noexec(&kit));
    assert!(!kit.volume().unwrap().noexec);
}
