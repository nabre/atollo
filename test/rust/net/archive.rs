use super::*;
use crate::kit::tests::TempDir;
use std::io::Write;

#[test]
fn unsafe_names_are_refused() {
    let base = Path::new(r"D:\x");
    assert_eq!(
        safe_join(base, "a/b.txt"),
        Some(PathBuf::from(r"D:\x\a\b.txt"))
    );
    assert_eq!(safe_join(base, "./a"), Some(PathBuf::from(r"D:\x\a")));
    for bad in [
        "../evil",
        "a/../../evil",
        "/abs",
        r"C:\abs",
        "C:rel",
        r"\\server\share\x",
        "",
    ] {
        assert_eq!(safe_join(base, bad), None, "{bad}");
    }
}

fn make_zip(path: &Path, files: &[(&str, &str)]) {
    let mut zip = zip::ZipWriter::new(File::create(path).unwrap());
    for (name, content) in files {
        zip.start_file(*name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(content.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
}

#[test]
fn zip_extract_and_inner() {
    let tmp = TempDir::new("zip");
    let archive = tmp.0.join("a.zip");
    make_zip(
        &archive,
        &[("top-1.0/bin/tool.exe", "x"), ("top-1.0/readme.txt", "r")],
    );
    let out = tmp.0.join("out");
    extract(&archive, Kind::Zip, &out).unwrap();
    let inner = pick_inner(&out, &Inner::Single).unwrap();
    assert!(inner.join("bin").join("tool.exe").is_file());
    assert_eq!(pick_inner(&out, &Inner::Auto).unwrap(), inner);
    assert_eq!(pick_inner(&out, &Inner::Root).unwrap(), out);
    assert_eq!(
        pick_inner(&out, &Inner::SingleThen("bin")).unwrap(),
        inner.join("bin")
    );

    let flat = tmp.0.join("flat.zip");
    make_zip(&flat, &[("pwsh.exe", "x"), ("Modules/a.psd1", "m")]);
    let out2 = tmp.0.join("out2");
    extract(&flat, Kind::Zip, &out2).unwrap();
    assert!(pick_inner(&out2, &Inner::Single).is_err());
    assert_eq!(pick_inner(&out2, &Inner::Auto).unwrap(), out2);
}

#[test]
fn zip_slip_is_refused() {
    let tmp = TempDir::new("zipslip");
    let archive = tmp.0.join("evil.zip");
    make_zip(&archive, &[("../evil.txt", "x")]);
    assert!(extract(&archive, Kind::Zip, &tmp.0.join("out")).is_err());
    assert!(!tmp.0.join("evil.txt").exists());
}

#[test]
fn tar_gz_extract() {
    let tmp = TempDir::new("targz");
    let archive = tmp.0.join("a.tar.gz");
    {
        let gz = flate2::write::GzEncoder::new(
            File::create(&archive).unwrap(),
            flate2::Compression::fast(),
        );
        let mut tar = tar::Builder::new(gz);
        let mut header = tar::Header::new_gnu();
        header.set_size(5);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, "server/bin/run.cmd", &b"hello"[..])
            .unwrap();
        tar.into_inner().unwrap().finish().unwrap();
    }
    let out = tmp.0.join("out");
    extract(&archive, Kind::TarGz, &out).unwrap();
    assert_eq!(
        fs::read_to_string(out.join("server").join("bin").join("run.cmd")).unwrap(),
        "hello"
    );
}

/// A `.tar.xz` written by `fill`, compressed in process (no `xz` binary).
fn make_tar_xz(path: &Path, fill: impl FnOnce(&mut tar::Builder<lzma_rust2::XzWriter<File>>)) {
    let xz = lzma_rust2::XzWriter::new(
        File::create(path).unwrap(),
        lzma_rust2::XzOptions::with_preset(1),
    )
    .unwrap();
    let mut tar = tar::Builder::new(xz);
    fill(&mut tar);
    tar.into_inner().unwrap().finish().unwrap();
}

/// The header of a regular file of `size` bytes, mode `0o644`.
fn file_header(size: u64) -> tar::Header {
    let mut header = tar::Header::new_gnu();
    header.set_size(size);
    header.set_mode(0o644);
    header.set_cksum();
    header
}

#[test]
fn tar_xz_extract() {
    let tmp = TempDir::new("tarxz");
    let archive = tmp.0.join("a.tar.xz");
    make_tar_xz(&archive, |tar| {
        tar.append_data(&mut file_header(5), "server/bin/run", &b"hello"[..])
            .unwrap();
        tar.append_data(&mut file_header(4), "server/readme.txt", &b"read"[..])
            .unwrap();
    });
    let out = tmp.0.join("out");
    extract(&archive, Kind::TarXz, &out).unwrap();
    let run = out.join("server").join("bin").join("run");
    assert_eq!(fs::read_to_string(&run).unwrap(), "hello");
    assert_eq!(
        fs::read_to_string(out.join("server").join("readme.txt")).unwrap(),
        "read"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = |p: &Path| fs::metadata(p).unwrap().permissions().mode();
        assert_ne!(mode(&run) & 0o111, 0, "files in bin/ are executable");
        assert_eq!(
            mode(&out.join("server").join("readme.txt")) & 0o111,
            0,
            "others keep their mode"
        );
    }
}

/// `tar::Builder` refuses `..` in names: the header is written by hand, as
/// a hostile archive would be.
#[test]
fn tar_xz_slip_is_refused() {
    let tmp = TempDir::new("tarxzslip");
    let archive = tmp.0.join("evil.tar.xz");
    make_tar_xz(&archive, |tar| {
        let name = b"../evil.txt";
        let mut header = tar::Header::new_old();
        header.as_old_mut().name[..name.len()].copy_from_slice(name);
        header.set_size(1);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, &b"x"[..]).unwrap();
    });
    let out = tmp.0.join("out");
    assert!(extract(&archive, Kind::TarXz, &out).is_err());
    assert!(!tmp.0.join("evil.txt").exists());
}

/// A symbolic link to an absolute path is not created; the rest is.
#[cfg(unix)]
#[test]
fn tar_xz_absolute_symlink_skipped() {
    let tmp = TempDir::new("tarxzlink");
    let archive = tmp.0.join("link.tar.xz");
    make_tar_xz(&archive, |tar| {
        let mut link = tar::Header::new_gnu();
        link.set_entry_type(tar::EntryType::Symlink);
        link.set_size(0);
        link.set_mode(0o777);
        link.set_path("server/passwd").unwrap();
        link.set_link_name("/etc/passwd").unwrap();
        link.set_cksum();
        tar.append(&link, io::empty()).unwrap();
        tar.append_data(&mut file_header(2), "server/ok.txt", &b"ok"[..])
            .unwrap();
    });
    let out = tmp.0.join("out");
    extract(&archive, Kind::TarXz, &out).unwrap();
    assert!(fs::symlink_metadata(out.join("server").join("passwd")).is_err());
    assert_eq!(
        fs::read_to_string(out.join("server").join("ok.txt")).unwrap(),
        "ok"
    );
}

#[test]
fn seven_z_signature_search_spans_buffers() {
    let tmp = TempDir::new("7zsig");
    let file = tmp.0.join("stub.exe");
    let mut bytes = vec![0u8; 1024 * 1024 - 3];
    bytes.extend_from_slice(&SEVEN_Z);
    bytes.extend_from_slice(b"rest");
    fs::write(&file, &bytes).unwrap();
    assert_eq!(seven_z_offsets(&file).unwrap(), vec![1024 * 1024 - 3]);
}

#[test]
fn replace_file_works_while_in_use() {
    let tmp = TempDir::new("replfile");
    let dest = tmp.0.join("tool.exe");
    fs::write(&dest, "1").unwrap();
    let held = File::open(&dest).unwrap();
    let new = tmp.0.join("new.exe");
    fs::write(&new, "2").unwrap();
    replace_file(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "2");
    drop(held);
    fs::write(&new, "3").unwrap();
    replace_file(&new, &dest).unwrap();
    assert!(
        !tmp.0.join("tool.exe.old").exists(),
        "old removed once free"
    );
}

/// A program of the PC keeps a folder of the tree open (as Cloud Code's
/// indexer did with `extensions\terminal-suggest` of the editor): the
/// swap still works. A file really in use stops it, and nothing changes.
#[test]
fn replace_dir_with_a_folder_held_open() {
    use std::os::windows::fs::OpenOptionsExt;
    const SHARE_READ_WRITE: u32 = 1 | 2;
    const BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let tmp = TempDir::new("replheld");
    let dest = tmp.0.join("editor");
    let make = |root: &Path, text: &str| {
        fs::create_dir_all(root.join(r"extensions\held")).unwrap();
        fs::write(root.join(r"extensions\held\a.txt"), text).unwrap();
        fs::write(root.join("node.exe"), text).unwrap();
    };
    make(&dest, "old");
    fs::write(dest.join("gone.txt"), "old").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARE_READ_WRITE)
        .custom_flags(BACKUP_SEMANTICS)
        .open(dest.join(r"extensions\held"))
        .unwrap();
    assert!(
        fs::rename(&dest, tmp.0.join("x")).is_err(),
        "Windows refuses the whole folder"
    );

    let new = tmp.0.join("new");
    make(&new, "new");
    fs::create_dir_all(new.join("out")).unwrap();
    replace_dir(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join("node.exe")).unwrap(), "new");
    assert_eq!(
        fs::read_to_string(dest.join(r"extensions\held\a.txt")).unwrap(),
        "new"
    );
    assert!(dest.join("out").is_dir() && !dest.join("gone.txt").exists());
    assert!(!new.exists());
    drop(held);
    let names: Vec<String> = fs::read_dir(&tmp.0)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        names.iter().all(|n| !n.starts_with("editor.old")),
        "{names:?}"
    );

    // A file really in use: InUse, and the folder is as before.
    let busy = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(dest.join("node.exe"))
        .unwrap();
    let newer = tmp.0.join("newer");
    make(&newer, "newer");
    let err = replace_dir(&newer, &dest).unwrap_err();
    assert!(err.is::<InUse>(), "{err:#}");
    drop(busy);
    assert_eq!(fs::read_to_string(dest.join("node.exe")).unwrap(), "new");
    assert_eq!(
        fs::read_to_string(dest.join(r"extensions\held\a.txt")).unwrap(),
        "new"
    );
    assert!(dest.join("out").is_dir());
}

/// The antivirus holds a file just extracted for a moment (as on the
/// first install of Java and Kotlin): the swap waits for it.
#[test]
fn replace_dir_waits_for_a_scan() {
    use std::os::windows::fs::OpenOptionsExt;
    let tmp = TempDir::new("replscan");
    let new = tmp.0.join(r"x\kotlinc");
    fs::create_dir_all(new.join("lib")).unwrap();
    fs::write(new.join(r"lib\a.jar"), "jar").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(new.join(r"lib\a.jar"))
        .unwrap();
    let dest = tmp.0.join("kotlin");
    assert!(
        fs::rename(&new, &dest).is_err(),
        "Windows refuses the folder while the file is open"
    );
    let scan = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(600));
        drop(held);
    });
    replace_dir(&new, &dest).unwrap();
    scan.join().unwrap();
    assert_eq!(fs::read_to_string(dest.join(r"lib\a.jar")).unwrap(), "jar");
    assert!(!new.exists());
}

fn tree(root: &Path, text: &str) {
    fs::create_dir_all(root.join(r"bin\lib")).unwrap();
    fs::write(root.join(r"bin\tool.exe"), text).unwrap();
    fs::write(root.join(r"bin\lib\a.dll"), text).unwrap();
}

fn names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn stage_into_on_the_same_disk_moves() {
    let tmp = TempDir::new("stagesame");
    let new = tmp.0.join(r"temp\x");
    tree(&new, "1");
    let dest = tmp.0.join(r"apps\tool");
    stage_into(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join(r"bin\tool.exe")).unwrap(), "1");
    assert!(!new.exists(), "renamed, not copied");
    assert_eq!(names(&tmp.0.join("apps")), ["tool"]);
}

#[test]
fn stage_into_from_another_disk_copies() {
    let tmp = TempDir::new("stageother");
    let dest = tmp.0.join(r"apps\tool");
    tree(&dest, "old");
    fs::write(dest.join("gone.txt"), "old").unwrap();
    let new = tmp.0.join(r"temp\x");
    tree(&new, "new");
    // A copy interrupted earlier (the app closed in the middle).
    fs::create_dir_all(tmp.0.join(r"apps\tool.atollo-new\bin")).unwrap();
    fs::write(tmp.0.join(r"apps\tool.atollo-new\stale.txt"), "x").unwrap();
    stage_with(&new, &dest, false).unwrap();
    assert_eq!(
        fs::read_to_string(dest.join(r"bin\lib\a.dll")).unwrap(),
        "new"
    );
    assert!(!dest.join("gone.txt").exists() && !dest.join("stale.txt").exists());
    assert!(new.join(r"bin\tool.exe").is_file(), "the source is copied");
    assert_eq!(names(&tmp.0.join("apps")), ["tool"]);

    // The swap refused (a file in use): the installed tree is intact and
    // no copy is left next to it.
    let busy = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(dest.join(r"bin\tool.exe"))
            .unwrap()
    };
    tree(&new, "newer");
    let err = stage_with(&new, &dest, false).unwrap_err();
    assert!(err.is::<InUse>(), "{err:#}");
    drop(busy);
    assert_eq!(
        fs::read_to_string(dest.join(r"bin\tool.exe")).unwrap(),
        "new"
    );
    assert_eq!(names(&tmp.0.join("apps")), ["tool"]);
}

#[test]
fn interrupted_copy_is_cleaned() {
    let tmp = TempDir::new("stageleft");
    let apps = tmp.0.join("apps");
    tree(&apps.join("tool"), "1");
    // What a copy stopped in the middle leaves: only the staged folder
    // (and a staged single file of `replace_file`).
    tree(&staged_path(&apps.join("tool")), "half");
    fs::write(staged_path(&apps.join("uv.exe")), "half").unwrap();
    assert_eq!(
        names(&apps),
        ["tool", "tool.atollo-new", "uv.exe.atollo-new"]
    );
    clean_leftovers(&apps);
    assert_eq!(names(&apps), ["tool"]);
    assert_eq!(
        fs::read_to_string(apps.join(r"tool\bin\tool.exe")).unwrap(),
        "1"
    );
}

#[test]
fn replace_file_leaves_no_staged_copy() {
    let tmp = TempDir::new("replstaged");
    let dest = tmp.0.join("uv.exe");
    let new = tmp.0.join(r"temp\uv.exe");
    fs::create_dir_all(new.parent().unwrap()).unwrap();
    fs::write(&new, "1").unwrap();
    replace_file(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "1");
    assert!(new.is_file(), "the source is copied");
    assert_eq!(names(&tmp.0), ["temp", "uv.exe"]);
}

/// A file over 4 GiB (sparse: nothing written) does not fit on FAT32;
/// the error names it with its size.
#[test]
fn check_fits_refuses_a_file_too_large_for_fat32() {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::IO::DeviceIoControl;
    const FSCTL_SET_SPARSE: u32 = 0x0009_00C4;
    let tmp = TempDir::new("fits");
    let new = tmp.0.join("x");
    tree(&new, "small");
    let fat32 = crate::os::volume::FsKind::Fat32.max_file_size();
    fits(&new, fat32, "FAT32").unwrap();
    check_fits(&new, &tmp.0.join(r"apps\tool")).unwrap();

    let big = new.join(r"bin\lib\model.bin");
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
    check_fits(&new, &tmp.0.join(r"apps\tool")).unwrap();
}

#[test]
fn check_space_needs_the_whole_tree() {
    let tmp = TempDir::new("space");
    let new = tmp.0.join("x");
    tree(&new, "12345");
    let dest = tmp.0.join(r"apps\tool");
    has_space(&new, 10, &dest).unwrap();
    let err = has_space(&new, 9, &dest).unwrap_err().to_string();
    assert!(
        err.contains("10 bytes needed") && err.contains("9 bytes free"),
        "{err}"
    );
    check_space(&new, &dest).unwrap();
    assert_eq!(human(3 << 29), "1.5 GiB");
}

#[test]
fn plain_file_kind() {
    let tmp = TempDir::new("kindfile");
    let f = tmp.0.join("agy.exe");
    fs::write(&f, "x").unwrap();
    extract(&f, Kind::File, &tmp.0.join("out")).unwrap();
    assert!(tmp.0.join("out").join("agy.exe").is_file());
}

#[test]
fn replace_and_remove() {
    let tmp = TempDir::new("replace");
    let dest = tmp.0.join("apps").join("tool");
    let new1 = tmp.0.join("new1");
    fs::create_dir_all(&new1).unwrap();
    fs::write(new1.join("v.txt"), "1").unwrap();
    replace_dir(&new1, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join("v.txt")).unwrap(), "1");

    let new2 = tmp.0.join("new2");
    fs::create_dir_all(&new2).unwrap();
    fs::write(new2.join("v.txt"), "2").unwrap();
    replace_dir(&new2, &dest).unwrap();
    assert_eq!(fs::read_to_string(dest.join("v.txt")).unwrap(), "2");
    let names: Vec<_> = fs::read_dir(dest.parent().unwrap())
        .unwrap()
        .flatten()
        .map(|e| e.file_name())
        .collect();
    assert_eq!(
        names,
        vec![std::ffi::OsString::from("tool")],
        "old folder removed"
    );

    // A file locked (open without FILE_SHARE_DELETE) blocks the swap and
    // leaves everything as it was.
    let held = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(dest.join("v.txt"))
            .unwrap()
    };
    let new3 = tmp.0.join("new3");
    fs::create_dir_all(&new3).unwrap();
    let err = replace_dir(&new3, &dest).unwrap_err();
    assert!(err.is::<InUse>(), "{err:#}");
    assert!(new3.exists());
    assert!(remove_dir(&dest).unwrap_err().is::<InUse>());
    drop(held);
    remove_dir(&dest).unwrap();
    assert!(!dest.exists());
}
