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

/// One entry of a hand-made tar: a file with its text, a symbolic link or
/// a hard link with its target.
enum Ent<'a> {
    File(&'a str, &'a str),
    Sym(&'a str, &'a str),
    Hard(&'a str, &'a str),
}

fn append_entries<W: Write>(tar: &mut tar::Builder<W>, entries: &[Ent]) {
    for entry in entries {
        let (kind, name, target) = match entry {
            Ent::File(name, text) => {
                tar.append_data(&mut file_header(text.len() as u64), name, text.as_bytes())
                    .unwrap();
                continue;
            }
            Ent::Sym(name, target) => (tar::EntryType::Symlink, name, target),
            Ent::Hard(name, target) => (tar::EntryType::Link, name, target),
        };
        let mut link = tar::Header::new_gnu();
        link.set_entry_type(kind);
        link.set_size(0);
        link.set_mode(0o777);
        link.set_path(name).unwrap();
        link.set_link_name(target).unwrap();
        link.set_cksum();
        tar.append(&link, io::empty()).unwrap();
    }
}

/// `entries` as a `.tar.gz` or a `.tar.xz` at `path`.
fn make_tar(path: &Path, kind: Kind, entries: &[Ent]) {
    match kind {
        Kind::TarXz => make_tar_xz(path, |tar| append_entries(tar, entries)),
        _ => {
            let gz = flate2::write::GzEncoder::new(
                File::create(path).unwrap(),
                flate2::Compression::fast(),
            );
            let mut tar = tar::Builder::new(gz);
            append_entries(&mut tar, entries);
            tar.into_inner().unwrap().finish().unwrap();
        }
    }
}

/// `t -> .` and `s -> t/..` each look inside the target when read alone;
/// together `s` is the parent of the target. A file under `s/` must not
/// land there (links are made last and never crossed).
#[test]
fn tar_symlink_chain_escape_refused() {
    for kind in [Kind::TarGz, Kind::TarXz] {
        let tmp = TempDir::new("tarchain");
        let archive = tmp.0.join("evil.tar");
        make_tar(
            &archive,
            kind,
            &[
                Ent::Sym("t", "."),
                Ent::Sym("s", "t/.."),
                Ent::File("s/pwned", "x"),
            ],
        );
        let out = tmp.0.join("out");
        let _ = extract(&archive, kind, &out);
        assert!(!tmp.0.join("pwned").exists(), "{kind:?}");
        let s = fs::symlink_metadata(out.join("s")).unwrap();
        assert!(s.is_dir(), "{kind:?}: s stays a real folder");

        // The links alone: `t` is inside and made, `s` crosses it and is not.
        let archive = tmp.0.join("links.tar");
        make_tar(&archive, kind, &[Ent::Sym("t", "."), Ent::Sym("s", "t/..")]);
        let out = tmp.0.join("out2");
        extract(&archive, kind, &out).unwrap();
        assert!(fs::symlink_metadata(out.join("s")).is_err(), "{kind:?}");
        #[cfg(unix)]
        assert!(fs::symlink_metadata(out.join("t"))
            .unwrap()
            .file_type()
            .is_symlink());
        #[cfg(not(unix))]
        assert!(fs::symlink_metadata(out.join("t")).is_err());
    }
}

/// The same chain in a zip.
#[test]
fn zip_symlink_chain_escape_refused() {
    let tmp = TempDir::new("zipchain");
    let archive = tmp.0.join("evil.zip");
    {
        let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        zip.add_symlink("t", ".", options).unwrap();
        zip.add_symlink("s", "t/..", options).unwrap();
        zip.start_file("s/pwned", options).unwrap();
        zip.write_all(b"x").unwrap();
        zip.finish().unwrap();
    }
    let out = tmp.0.join("out");
    let _ = extract(&archive, Kind::Zip, &out);
    assert!(!tmp.0.join("pwned").exists());
    assert!(fs::symlink_metadata(out.join("s")).unwrap().is_dir());
}

/// A hard link reached through a link chain (or to a file outside) is not
/// made, and a later file of the same name never truncates the target.
#[test]
fn tar_hardlink_outside_refused() {
    for kind in [Kind::TarGz, Kind::TarXz] {
        let tmp = TempDir::new("tarhard");
        let victim = tmp.0.join("victim.txt");
        fs::write(&victim, "safe").unwrap();
        let archive = tmp.0.join("evil.tar");
        make_tar(
            &archive,
            kind,
            &[
                Ent::Sym("t", "."),
                Ent::Sym("s", "t/.."),
                Ent::Hard("h", "s/victim.txt"),
                Ent::File("h", "pwned"),
            ],
        );
        let out = tmp.0.join("out");
        let _ = extract(&archive, kind, &out);
        assert_eq!(fs::read_to_string(&victim).unwrap(), "safe", "{kind:?}");
        assert_eq!(fs::read_to_string(out.join("h")).unwrap(), "pwned");
    }
}

/// A link already in the target (not from the archive) is never crossed
/// or written through, and a hard link to it is not made.
#[cfg(unix)]
#[test]
fn tar_never_writes_through_a_link() {
    use std::os::unix::fs::MetadataExt;
    let tmp = TempDir::new("tarthrough");
    let victim = tmp.0.join("victim.txt");
    fs::write(&victim, "safe").unwrap();
    let out = tmp.0.join("out");
    fs::create_dir_all(&out).unwrap();
    std::os::unix::fs::symlink(&tmp.0, out.join("up")).unwrap();
    std::os::unix::fs::symlink(&victim, out.join("ext")).unwrap();

    let archive = tmp.0.join("hard.tar.gz");
    make_tar(
        &archive,
        Kind::TarGz,
        &[Ent::Hard("h", "ext"), Ent::File("h", "pwned")],
    );
    extract(&archive, Kind::TarGz, &out).unwrap();
    assert_eq!(fs::read_to_string(&victim).unwrap(), "safe");
    assert_eq!(fs::metadata(&victim).unwrap().nlink(), 1);

    let archive = tmp.0.join("file.tar.gz");
    make_tar(&archive, Kind::TarGz, &[Ent::File("up/pwned", "x")]);
    assert!(extract(&archive, Kind::TarGz, &out).is_err());
    assert!(!tmp.0.join("pwned").exists());

    // A file entry named like the link replaces the link, not its target.
    let archive = tmp.0.join("over.tar.gz");
    make_tar(&archive, Kind::TarGz, &[Ent::File("ext", "new")]);
    extract(&archive, Kind::TarGz, &out).unwrap();
    assert_eq!(fs::read_to_string(&victim).unwrap(), "safe");
    assert_eq!(fs::read_to_string(out.join("ext")).unwrap(), "new");
}

/// Ordinary relative links are kept, even when they come before their
/// target in the archive; one that crosses another link is not.
#[cfg(unix)]
#[test]
fn tar_relative_symlinks_inside_are_created() {
    let tmp = TempDir::new("tarlinks");
    let archive = tmp.0.join("ok.tar.xz");
    make_tar(
        &archive,
        Kind::TarXz,
        &[
            Ent::Sym("pkg/bin/tool", "../lib/tool.sh"),
            Ent::File("pkg/lib/tool.sh", "run"),
            Ent::Sym("pkg/cur", "lib"),
            Ent::Sym("pkg/through", "cur/tool.sh"),
        ],
    );
    let out = tmp.0.join("out");
    extract(&archive, Kind::TarXz, &out).unwrap();
    assert_eq!(fs::read_to_string(out.join("pkg/bin/tool")).unwrap(), "run");
    assert_eq!(
        fs::read_to_string(out.join("pkg/cur/tool.sh")).unwrap(),
        "run"
    );
    assert!(fs::symlink_metadata(out.join("pkg/through")).is_err());
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
#[cfg(windows)]
#[test]
fn replace_dir_with_a_folder_held_open() {
    use std::os::windows::fs::OpenOptionsExt;
    const SHARE_READ_WRITE: u32 = 1 | 2;
    const BACKUP_SEMANTICS: u32 = 0x0200_0000;
    let tmp = TempDir::new("replheld");
    let dest = tmp.0.join("editor");
    let make = |root: &Path, text: &str| {
        fs::create_dir_all(root.join("extensions").join("held")).unwrap();
        fs::write(root.join("extensions").join("held").join("a.txt"), text).unwrap();
        fs::write(root.join("node.exe"), text).unwrap();
    };
    make(&dest, "old");
    fs::write(dest.join("gone.txt"), "old").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARE_READ_WRITE)
        .custom_flags(BACKUP_SEMANTICS)
        .open(dest.join("extensions").join("held"))
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
        fs::read_to_string(dest.join("extensions").join("held").join("a.txt")).unwrap(),
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
        fs::read_to_string(dest.join("extensions").join("held").join("a.txt")).unwrap(),
        "new"
    );
    assert!(dest.join("out").is_dir());
}

/// The antivirus holds a file just extracted for a moment (as on the
/// first install of Java and Kotlin): the swap waits for it.
#[cfg(windows)]
#[test]
fn replace_dir_waits_for_a_scan() {
    use std::os::windows::fs::OpenOptionsExt;
    let tmp = TempDir::new("replscan");
    let new = tmp.0.join("x").join("kotlinc");
    fs::create_dir_all(new.join("lib")).unwrap();
    fs::write(new.join("lib").join("a.jar"), "jar").unwrap();
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(new.join("lib").join("a.jar"))
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
    assert_eq!(
        fs::read_to_string(dest.join("lib").join("a.jar")).unwrap(),
        "jar"
    );
    assert!(!new.exists());
}

fn tree(root: &Path, text: &str) {
    fs::create_dir_all(root.join("bin").join("lib")).unwrap();
    fs::write(root.join("bin").join("tool.exe"), text).unwrap();
    fs::write(root.join("bin").join("lib").join("a.dll"), text).unwrap();
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
    let new = tmp.0.join("temp").join("x");
    tree(&new, "1");
    let dest = tmp.0.join("apps").join("tool");
    stage_into(&new, &dest).unwrap();
    assert_eq!(
        fs::read_to_string(dest.join("bin").join("tool.exe")).unwrap(),
        "1"
    );
    assert!(!new.exists(), "renamed, not copied");
    assert_eq!(names(&tmp.0.join("apps")), ["tool"]);
}

#[cfg(windows)]
#[test]
fn stage_into_from_another_disk_copies() {
    let tmp = TempDir::new("stageother");
    let dest = tmp.0.join("apps").join("tool");
    tree(&dest, "old");
    fs::write(dest.join("gone.txt"), "old").unwrap();
    let new = tmp.0.join("temp").join("x");
    tree(&new, "new");
    // A copy interrupted earlier (the app closed in the middle).
    fs::create_dir_all(tmp.0.join("apps").join("tool.devkit-new").join("bin")).unwrap();
    fs::write(
        tmp.0.join("apps").join("tool.devkit-new").join("stale.txt"),
        "x",
    )
    .unwrap();
    stage_with(&new, &dest, false).unwrap();
    assert_eq!(
        fs::read_to_string(dest.join("bin").join("lib").join("a.dll")).unwrap(),
        "new"
    );
    assert!(!dest.join("gone.txt").exists() && !dest.join("stale.txt").exists());
    assert!(
        new.join("bin").join("tool.exe").is_file(),
        "the source is copied"
    );
    assert_eq!(names(&tmp.0.join("apps")), ["tool"]);

    // The swap refused (a file in use): the installed tree is intact and
    // no copy is left next to it.
    let busy = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(dest.join("bin").join("tool.exe"))
            .unwrap()
    };
    tree(&new, "newer");
    let err = stage_with(&new, &dest, false).unwrap_err();
    assert!(err.is::<InUse>(), "{err:#}");
    drop(busy);
    assert_eq!(
        fs::read_to_string(dest.join("bin").join("tool.exe")).unwrap(),
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
        ["tool", "tool.devkit-new", "uv.exe.devkit-new"]
    );
    clean_leftovers(&apps);
    assert_eq!(names(&apps), ["tool"]);
    assert_eq!(
        fs::read_to_string(apps.join("tool").join("bin").join("tool.exe")).unwrap(),
        "1"
    );
}

#[test]
fn replace_file_leaves_no_staged_copy() {
    let tmp = TempDir::new("replstaged");
    let dest = tmp.0.join("uv.exe");
    let new = tmp.0.join("temp").join("uv.exe");
    fs::create_dir_all(new.parent().unwrap()).unwrap();
    fs::write(&new, "1").unwrap();
    replace_file(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "1");
    assert!(new.is_file(), "the source is copied");
    assert_eq!(names(&tmp.0), ["temp", "uv.exe"]);
}

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
    let fat32 = crate::os::volume::FsKind::Fat32.max_file_size();
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

#[test]
fn plain_file_kind() {
    let tmp = TempDir::new("kindfile");
    let f = tmp.0.join("agy.exe");
    fs::write(&f, "x").unwrap();
    extract(&f, Kind::File, &tmp.0.join("out")).unwrap();
    assert!(tmp.0.join("out").join("agy.exe").is_file());
}

#[cfg(windows)]
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

/// The refusal of exFAT and FAT, injected so it runs on every system.
fn no_links(_: &str, _: &Path) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "no links here"))
}

/// Without links the targets are copied: a file, a folder, a link to a
/// link (whose copy comes first); a link outside, or a folder that would
/// hold itself, is skipped.
#[test]
fn links_become_copies_where_refused() {
    let tmp = TempDir::new("linkcopy");
    let dest = tmp.0.join("out");
    let pkg = dest.join("pkg");
    fs::create_dir_all(pkg.join("lib")).unwrap();
    fs::create_dir_all(pkg.join("bin")).unwrap();
    fs::write(pkg.join("lib").join("tool.sh"), "run").unwrap();
    let doc = pkg.join("share").join("doc");
    fs::create_dir_all(&doc).unwrap();
    fs::write(doc.join("a.txt"), "doc").unwrap();
    fs::write(tmp.0.join("outside.txt"), "secret").unwrap();
    let links = vec![
        (pkg.join("chain"), "bin/tool".to_owned()),
        (pkg.join("bin").join("tool"), "../lib/tool.sh".to_owned()),
        (pkg.join("doc"), "share/doc".to_owned()),
        (doc.join("up"), "..".to_owned()),
        (pkg.join("out"), "../../outside.txt".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(
        fs::read_to_string(pkg.join("bin").join("tool")).unwrap(),
        "run"
    );
    assert_eq!(fs::read_to_string(pkg.join("chain")).unwrap(), "run");
    assert_eq!(
        fs::read_to_string(pkg.join("doc").join("a.txt")).unwrap(),
        "doc"
    );
    for copy in [pkg.join("chain"), pkg.join("doc")] {
        assert!(!fs::symlink_metadata(&copy)
            .unwrap()
            .file_type()
            .is_symlink());
    }
    assert!(!pkg.join("out").exists() && !doc.join("up").exists());
    assert_eq!(skipped.len(), 2, "{skipped:?}");
}

/// Without links, a link whose way crosses another link (`x -> lib64/foo`,
/// `lib64 -> lib`) is copied once that one is a copy; a way that still
/// leaves the folder after it stays skipped.
#[test]
fn links_through_a_copied_link_where_refused() {
    let tmp = TempDir::new("linkchain");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("lib")).unwrap();
    fs::write(dest.join("lib").join("foo"), "so").unwrap();
    let links = vec![
        (dest.join("x"), "lib64/foo".to_owned()),
        (dest.join("y"), "lib64/../../secret".to_owned()),
        (dest.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(fs::read_to_string(dest.join("x")).unwrap(), "so");
    assert_eq!(
        fs::read_to_string(dest.join("lib64").join("foo")).unwrap(),
        "so"
    );
    assert!(!dest.join("y").exists());
    assert_eq!(skipped.len(), 1, "{skipped:?}");

    // With links, a way through a link is never followed.
    let dest2 = tmp.0.join("out2");
    fs::create_dir_all(dest2.join("lib")).unwrap();
    let made = std::cell::RefCell::new(Vec::new());
    let links = vec![
        (dest2.join("x"), "lib64/foo".to_owned()),
        (dest2.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest2, links, |t, p| {
        made.borrow_mut().push((t.to_owned(), p.to_path_buf()));
        Ok(())
    })
    .unwrap();
    assert_eq!(made.borrow().len(), 1);
    assert_eq!(skipped.len(), 1, "{skipped:?}");
}

/// Without links, a folder copied in place of a link (`d -> a`) waits for a
/// link inside it that resolves only later (`a/x -> lib64/foo`, through
/// the copy of `lib64 -> lib`), so the copy holds it too.
#[test]
fn a_copied_folder_gets_links_resolved_later() {
    let tmp = TempDir::new("linklater");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("a")).unwrap();
    fs::create_dir_all(dest.join("lib")).unwrap();
    fs::write(dest.join("lib").join("foo"), "so").unwrap();
    let links = vec![
        (dest.join("d"), "a".to_owned()),
        (dest.join("a").join("x"), "../lib64/foo".to_owned()),
        (dest.join("lib64"), "lib".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert!(skipped.is_empty(), "{skipped:?}");
    assert_eq!(fs::read_to_string(dest.join("a").join("x")).unwrap(), "so");
    assert_eq!(fs::read_to_string(dest.join("d").join("x")).unwrap(), "so");
}

/// A link whose target can never stay inside (absolute, climbing above the
/// folder) is skipped before anything is made for it: no folder for its
/// parent, no error.
#[test]
fn hopeless_links_make_no_folder() {
    let tmp = TempDir::new("linkhopeless");
    let dest = tmp.0.join("out");
    fs::create_dir_all(&dest).unwrap();
    let links = vec![
        (dest.join("p").join("abs"), "/etc/passwd".to_owned()),
        (dest.join("q").join("up"), "../../x".to_owned()),
        (dest.join("r").join("win"), r"..\x".to_owned()),
    ];
    let skipped = finish_links(&dest, links, no_links).unwrap();
    assert_eq!(skipped.len(), 3, "{skipped:?}");
    for d in ["p", "q", "r"] {
        assert!(!dest.join(d).exists(), "{d}");
    }
}

#[test]
fn lexically_inside_by_text() {
    let top = Path::new("");
    let pkg = Path::new("pkg");
    assert!(lexically_inside(top, "a/b"));
    assert!(lexically_inside(pkg, "../x"));
    assert!(lexically_inside(pkg, "./b/../c"));
    assert!(!lexically_inside(top, "../x"));
    assert!(!lexically_inside(pkg, "../../x"));
    assert!(!lexically_inside(pkg, "a/../../../x"));
    assert!(!lexically_inside(pkg, "/abs"));
    assert!(!lexically_inside(pkg, ""));
    assert!(!lexically_inside(pkg, r"a\b"));
    assert!(!lexically_inside(pkg, "C:x"));
}

/// Links to folders nested in each other double the tree at each step once
/// each is a copy: past the budget the extraction stops with a clear error.
#[test]
fn copies_of_nested_links_have_a_budget() {
    let tmp = TempDir::new("linkbomb");
    let dest = tmp.0.join("out");
    fs::create_dir_all(dest.join("d0")).unwrap();
    fs::write(dest.join("d0").join("f"), vec![b'x'; 1024]).unwrap();
    let mut links = Vec::new();
    for i in 1..12 {
        fs::create_dir_all(dest.join(format!("d{i}"))).unwrap();
        for n in ["a", "b"] {
            links.push((dest.join(format!("d{i}")).join(n), format!("../d{}", i - 1)));
        }
    }
    let err = finish_links_within(&dest, links.clone(), no_links, 0).unwrap_err();
    assert!(err.to_string().contains("too much space"), "{err:#}");
    // The same links within the default budget: 2^11 KiB, well inside.
    let dest2 = tmp.0.join("out2");
    fs::create_dir_all(dest2.join("d0")).unwrap();
    fs::write(dest2.join("d0").join("f"), vec![b'x'; 1024]).unwrap();
    let links2: Vec<_> = links
        .into_iter()
        .map(|(p, t)| (dest2.join(p.strip_prefix(&dest).unwrap()), t))
        .collect();
    for i in 1..12 {
        fs::create_dir_all(dest2.join(format!("d{i}"))).unwrap();
    }
    assert!(finish_links(&dest2, links2, no_links).unwrap().is_empty());
    assert_eq!(tree_size(&dest2.join("d11")), 2048 * 1024);
}

/// A tar hard link where the file system has none (exFAT, FAT) is a copy;
/// another refusal is an error, not a missing file.
#[test]
fn hard_links_become_copies_where_refused() {
    let tmp = TempDir::new("hardcopy");
    let target = tmp.0.join("a");
    fs::write(&target, "data").unwrap();
    hard_link_or_copy(&target, &tmp.0.join("b"), |_, _| {
        Err(io::Error::new(io::ErrorKind::Unsupported, "no links here"))
    })
    .unwrap();
    assert_eq!(fs::read_to_string(tmp.0.join("b")).unwrap(), "data");
    let denied = hard_link_or_copy(&target, &tmp.0.join("c"), |_, _| {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
    });
    assert!(denied.is_err());
    assert!(!tmp.0.join("c").exists());
    hard_link_or_copy(&target, &tmp.0.join("d"), |t, p| fs::hard_link(t, p)).unwrap();
    assert_eq!(fs::read_to_string(tmp.0.join("d")).unwrap(), "data");
}

/// A refusal for another reason (permissions) is an error, not a copy.
#[test]
fn other_link_errors_stop() {
    let tmp = TempDir::new("linkerr");
    let dest = tmp.0.join("out");
    fs::create_dir_all(&dest).unwrap();
    fs::write(dest.join("a"), "x").unwrap();
    let denied = |_: &str, _: &Path| -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "denied"))
    };
    let links = vec![(dest.join("b"), "a".to_owned())];
    assert!(finish_links(&dest, links, denied).is_err());
    assert!(!dest.join("b").exists());
}

#[cfg(unix)]
#[test]
fn links_unsupported_codes() {
    for code in [libc::EPERM, libc::ENOTSUP, libc::EOPNOTSUPP] {
        assert!(
            links_unsupported(&io::Error::from_raw_os_error(code)),
            "{code}"
        );
    }
    assert!(!links_unsupported(&io::Error::from_raw_os_error(
        libc::EACCES
    )));
    assert!(!links_unsupported(&io::Error::from_raw_os_error(
        libc::ENOENT
    )));
}

#[test]
fn links_unsupported_kind() {
    assert!(links_unsupported(&io::Error::new(
        io::ErrorKind::Unsupported,
        "x"
    )));
    assert!(!links_unsupported(&io::Error::new(
        io::ErrorKind::PermissionDenied,
        "x"
    )));
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
