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
