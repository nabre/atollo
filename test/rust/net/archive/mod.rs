use super::formats::{seven_z_offsets, SEVEN_Z};
use super::*;
use crate::kit::tests::TempDir;
use std::io;
use std::io::Write;

#[test]
fn unsafe_names_are_refused() {
    let base = Path::new(r"D:\x");
    // with the system's separator after the base: `D:\x\a\b.txt` on Windows
    assert_eq!(
        safe_join(base, "a/b.txt"),
        Some(base.join("a").join("b.txt"))
    );
    assert_eq!(safe_join(base, "./a"), Some(base.join("a")));
    for bad in ["../evil", "a/../../evil", "/abs", r"\\server\share\x", ""] {
        assert_eq!(safe_join(base, bad), None, "{bad}");
    }
    // A drive or a stream on Windows; an ordinary character elsewhere,
    // always inside the base (PORT-V6-05).
    for colon in [r"C:\abs", "C:rel", "file.txt:stream", "man/foo:bar.1"] {
        if cfg!(windows) {
            assert_eq!(safe_join(base, colon), None, "{colon}");
        } else {
            let joined = safe_join(base, colon).unwrap();
            assert!(joined.starts_with(base), "{colon}");
        }
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

/// The archive of another system unpacked on Windows: its links become
/// copies of their targets, so `python/bin/python3 -> python3.13` and a
/// hard link are real files the installer's key check finds.
#[cfg(windows)]
#[test]
fn tar_links_become_copies_on_windows() {
    let tmp = TempDir::new("tarwinlinks");
    for kind in [Kind::TarGz, Kind::TarXz] {
        let archive = tmp.0.join(format!("python-{kind:?}.tar"));
        make_tar(
            &archive,
            kind,
            &[
                Ent::File("python/bin/python3.13", "elf"),
                Ent::Sym("python/bin/python3", "python3.13"),
                Ent::Sym("python/lib/current", "../bin"),
                Ent::Hard("python/bin/pip3", "python/bin/python3.13"),
                Ent::Sym("python/outside", "../../etc/passwd"),
            ],
        );
        let out = tmp.0.join(format!("out-{kind:?}"));
        extract(&archive, kind, &out).unwrap();
        let bin = out.join("python/bin");
        let python3 = fs::symlink_metadata(bin.join("python3")).unwrap();
        assert!(python3.is_file(), "{kind:?}: a copy, not a link");
        assert_eq!(fs::read_to_string(bin.join("python3")).unwrap(), "elf");
        assert_eq!(fs::read_to_string(bin.join("pip3")).unwrap(), "elf");
        assert_eq!(
            fs::read_to_string(out.join("python/lib/current/python3.13")).unwrap(),
            "elf"
        );
        assert!(fs::symlink_metadata(out.join("python/outside")).is_err());
    }
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
fn plain_file_kind() {
    let tmp = TempDir::new("kindfile");
    let f = tmp.0.join("agy.exe");
    fs::write(&f, "x").unwrap();
    extract(&f, Kind::File, &tmp.0.join("out")).unwrap();
    assert!(tmp.0.join("out").join("agy.exe").is_file());
}

/// A small tree: `bin/tool.exe` and `bin/lib/a.dll`, holding `text`.
pub(super) fn tree(root: &Path, text: &str) {
    fs::create_dir_all(root.join("bin").join("lib")).unwrap();
    fs::write(root.join("bin").join("tool.exe"), text).unwrap();
    fs::write(root.join("bin").join("lib").join("a.dll"), text).unwrap();
}
