use super::*;
use crate::kit::tests::TempDir;
use crate::net::archive::tests::tree;
#[cfg(windows)]
use std::fs::File;

/// A file over 4 GiB (sparse: nothing written) does not fit on FAT32;
/// the error names it with its size.
#[cfg(windows)]
#[test]
fn check_fits_refuses_a_file_too_large_for_fat32() {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::IO::DeviceIoControl;
    const FSCTL_SET_SPARSE: u32 = 0x0009_00C4;
    let tmp = TempDir::new("fits");
    let new = tmp.0.join("x");
    tree(&new, "small");
    let fat32 = crate::platform::volume::FsKind::Fat32.max_file_size();
    fits(&new, fat32, "FAT32").unwrap();
    check_fits(&new, &tmp.0.join("apps").join("tool")).unwrap();

    let big = new.join("bin").join("lib").join("model.bin");
    let file = File::create(&big).unwrap();
    // SAFETY: the handle stays valid while `file` lives; no buffers.
    let sparse = unsafe {
        DeviceIoControl(
            HANDLE(file.as_raw_handle()),
            FSCTL_SET_SPARSE,
            None,
            0,
            None,
            0,
            None,
            None,
        )
    };
    assert!(sparse.is_ok(), "{sparse:?}");
    file.set_len((4 << 30) + 1).unwrap();
    drop(file);
    let err = fits(&new, fat32, "FAT32").unwrap_err().to_string();
    assert!(err.contains(r"bin\lib\model.bin"), "{err}");
    assert!(
        err.contains("4294967297 bytes") && err.contains("FAT32"),
        "{err}"
    );
    fits(&new, None, "NTFS").unwrap();
    // A single file is checked too (`replace_file`).
    assert!(fits(&big, fat32, "FAT32").is_err());
    // The test disk (NTFS) has no limit.
    check_fits(&new, &tmp.0.join("apps").join("tool")).unwrap();
}

#[test]
fn check_space_needs_the_whole_tree() {
    let tmp = TempDir::new("space");
    let new = tmp.0.join("x");
    tree(&new, "12345");
    let dest = tmp.0.join("apps").join("tool");
    has_space(&new, 10, &dest).unwrap();
    let err = has_space(&new, 9, &dest).unwrap_err().to_string();
    assert!(
        err.contains("10 bytes needed") && err.contains("9 bytes free"),
        "{err}"
    );
    check_space(&new, &dest).unwrap();
    assert_eq!(human(3 << 29), "1.5 GiB");
}

/// A tree installed from a temporary folder on another disk keeps its
/// links (`bin/npm`, `bin/python3`), with the same relative target; one
/// that leaves the tree is not copied.
#[cfg(unix)]
#[test]
fn copy_tree_keeps_links_inside() {
    use std::os::unix::fs::symlink;
    let tmp = TempDir::new("copylinks");
    let from = tmp.0.join("from");
    fs::create_dir_all(from.join("lib")).unwrap();
    fs::create_dir_all(from.join("bin")).unwrap();
    fs::write(from.join("lib").join("npm-cli.js"), "js").unwrap();
    fs::write(tmp.0.join("sibling.txt"), "no").unwrap();
    symlink("../lib/npm-cli.js", from.join("bin").join("npm")).unwrap();
    symlink("lib", from.join("cur")).unwrap();
    symlink("../sibling.txt", from.join("esc")).unwrap();
    symlink(tmp.0.join("sibling.txt"), from.join("abs")).unwrap();
    let to = tmp.0.join("to");
    copy_tree(&from, &to).unwrap();
    let npm = to.join("bin").join("npm");
    assert!(fs::symlink_metadata(&npm).unwrap().file_type().is_symlink());
    assert_eq!(fs::read_link(&npm).unwrap(), Path::new("../lib/npm-cli.js"));
    assert_eq!(fs::read_to_string(&npm).unwrap(), "js");
    assert_eq!(fs::read_link(to.join("cur")).unwrap(), Path::new("lib"));
    assert_eq!(
        fs::read_to_string(to.join("cur").join("npm-cli.js")).unwrap(),
        "js"
    );
    assert!(
        fs::symlink_metadata(to.join("esc")).is_err(),
        "outside the tree"
    );
    assert!(fs::symlink_metadata(to.join("abs")).is_err(), "absolute");
}

/// On Windows links are skipped by the copy, as before.
#[cfg(windows)]
#[test]
fn copy_tree_copies_files_and_folders() {
    let tmp = TempDir::new("copytree");
    let from = tmp.0.join("from");
    fs::create_dir_all(from.join("bin").join("lib")).unwrap();
    fs::write(from.join("bin").join("tool.exe"), "1").unwrap();
    fs::write(from.join("bin").join("lib").join("a.dll"), "2").unwrap();
    let to = tmp.0.join("to");
    copy_tree(&from, &to).unwrap();
    assert_eq!(
        fs::read_to_string(to.join("bin").join("tool.exe")).unwrap(),
        "1"
    );
    assert_eq!(
        fs::read_to_string(to.join("bin").join("lib").join("a.dll")).unwrap(),
        "2"
    );
}
