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
    assert!(tmp
        .canonicalize()
        .unwrap()
        .starts_with(root_of(&tmp).unwrap()));
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
