use super::*;
use crate::kit::{tests::TempDir, Arch};
use sha2::{Digest, Sha256};

fn sha256(bytes: &[u8]) -> Expect {
    Expect::Sha256(
        Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}

#[test]
fn added_files_are_used_by_checksum() {
    let tmp = TempDir::new("downloads");
    let kit = Kit::at(tmp.0.join("kit"), Arch::X64);
    kit.prepare().unwrap();
    let outside = tmp.0.join("Downloads");
    fs::create_dir_all(&outside).unwrap();
    // Renamed by the browser, and another file.
    fs::write(outside.join("tool (1).zip"), b"tool").unwrap();
    fs::write(outside.join("other.zip"), b"other").unwrap();
    let names = add(
        &kit,
        &[outside.join("tool (1).zip"), outside.join("other.zip")],
    )
    .unwrap();
    assert_eq!(names, ["tool (1).zip", "other.zip"]);
    assert!(outside.join("tool (1).zip").is_file(), "the original stays");
    assert!(
        add(&kit, std::slice::from_ref(&outside)).is_err(),
        "a folder is refused"
    );

    let work = tmp.0.join("work");
    fs::create_dir_all(&work).unwrap();
    let dest = work.join("tool.zip");
    assert!(take(&kit, "tool.zip", &sha256(b"missing"), &dest)
        .unwrap()
        .is_none());
    let file = take(&kit, "tool.zip", &sha256(b"tool"), &dest)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), b"tool");
    // Copied: an installation that fails finds it again (BUG-V6-14).
    assert_eq!(file, dir(&kit).join("tool (1).zip"));
    assert!(file.is_file(), "kept until the installation succeeds");
    used(&kit, &file, &sha256(b"tool"));
    assert!(!file.exists(), "used once");
    assert!(dir(&kit).join("other.zip").exists(), "another file is kept");
}

/// With the PC's temporary folder, the files are kept there; those left
/// in `data\downloads` move there and are still found by checksum.
#[test]
fn kept_in_the_host_temp_folder() {
    let tmp = TempDir::new("downloads-host");
    let kit = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&tmp.0.join("host")));
    kit.prepare().unwrap();
    let old = kit.data().join("downloads");
    fs::create_dir_all(&old).unwrap();
    fs::write(old.join("tool.zip"), b"tool").unwrap();
    fs::write(old.join("half.zip.part"), b"x").unwrap();

    let here = dir(&kit);
    assert!(here.starts_with(tmp.0.join("host")), "{}", here.display());
    assert!(here.join("tool.zip").is_file(), "moved on first use");
    assert!(!old.join("tool.zip").exists());
    assert!(old.join("half.zip.part").exists(), "a partial file stays");

    let outside = tmp.0.join("other.zip");
    fs::write(&outside, b"other").unwrap();
    add(&kit, std::slice::from_ref(&outside)).unwrap();
    assert!(here.join("other.zip").is_file());
    assert!(!old.join("other.zip").exists(), "nothing new in the kit");

    let dest = tmp.0.join("tool.zip");
    let file = take(&kit, "tool.zip", &sha256(b"tool"), &dest)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&dest).unwrap(), b"tool");
    used(&kit, &file, &sha256(b"tool"));
    assert!(!here.join("tool.zip").exists(), "used once");
}

#[cfg(windows)]
#[test]
fn moved_across_disks_by_copy() {
    let tmp = TempDir::new("downloads-move");
    let (from, to) = (tmp.0.join("from"), tmp.0.join("to"));
    fs::create_dir_all(&from).unwrap();
    fs::write(from.join("a.zip"), b"a").unwrap();
    // A file held without sharing delete cannot be renamed: copied, and
    // left (with a warning) until it is free.
    let held = {
        use std::os::windows::fs::OpenOptionsExt;
        fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(from.join("a.zip"))
            .unwrap()
    };
    move_files(&from, &to);
    assert_eq!(fs::read(to.join("a.zip")).unwrap(), b"a");
    assert!(!to.join("a.zip.part").exists());
    drop(held);
    move_files(&from, &to);
    assert!(!from.join("a.zip").exists());
    assert_eq!(fs::read(to.join("a.zip")).unwrap(), b"a");
}

/// A failed download notes which component wants the file: removing it
/// removes the file added for it, never another file, nor one another
/// component or system waits for with the same checksum.
#[test]
fn files_added_for_a_component_go_with_it() {
    let tmp = TempDir::new("downloads-forget");
    let host = tmp.0.join("host");
    let kit = Kit::at(tmp.0.join("kit"), Arch::X64).with_host(Some(&host));
    kit.prepare().unwrap();
    // Another system than the kit's own, which is the host's: on Linux
    // runners linux-x64 is the kit itself.
    let other = [crate::kit::Target::LINUX_X64, crate::kit::Target::WIN32_X64]
        .into_iter()
        .find(|t| *t != kit.target())
        .unwrap();
    let elsewhere = kit.for_target(other);
    let dir = dir(&kit);
    fs::create_dir_all(&dir).unwrap();
    for (name, bytes) in [
        ("zig.zip", &b"zig"[..]),
        ("both.zip", b"both"),
        ("elsewhere.zip", b"elsewhere"),
        ("other.zip", b"other"),
    ] {
        fs::write(dir.join(name), bytes).unwrap();
    }
    want(&kit, "zig", &sha256(b"zig"));
    want(&kit, "zig", &sha256(b"zig"));
    want(&kit, "zig", &sha256(b"both"));
    want(&kit, "zig", &sha256(b"elsewhere"));
    want(&kit, "uv", &sha256(b"both"));
    want(&elsewhere, "zig", &sha256(b"elsewhere"));
    assert_eq!(read_wanted(&dir).len(), 5, "noted once");

    forget(&kit, "zig");
    assert!(!dir.join("zig.zip").exists());
    assert!(dir.join("both.zip").is_file(), "uv waits for it");
    assert!(
        dir.join("elsewhere.zip").is_file(),
        "the other system waits for it"
    );
    assert!(dir.join("other.zip").is_file(), "never noted");
    assert_eq!(read_wanted(&dir).len(), 2);
    assert!(read_wanted(&dir).iter().all(|w| !w.is_for(&kit, "zig")));

    // Used by an installation: nobody waits for it any more.
    let dest = tmp.0.join("both.zip");
    let file = take(&kit, "both.zip", &sha256(b"both"), &dest)
        .unwrap()
        .unwrap();
    assert_eq!(read_wanted(&dir).len(), 2, "waited for until used");
    used(&kit, &file, &sha256(b"both"));
    assert_eq!(read_wanted(&dir).len(), 1);
    assert!(
        take(&kit, WANTED, &sha256(b"none"), &dest)
            .unwrap()
            .is_none(),
        "the notes are never a candidate"
    );
}
