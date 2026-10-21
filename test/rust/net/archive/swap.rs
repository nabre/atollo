use super::*;
use crate::kit::tests::TempDir;
use crate::net::archive::tests::tree;
use anyhow::bail;
use std::fs::File;

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

/// The `.old` of an earlier update still running (it cannot be deleted):
/// the old file goes to a name of its own instead of stopping the
/// update; the stamped leftovers of earlier times go (BUG-V6-23).
#[cfg(windows)]
#[test]
fn replace_file_with_the_old_one_still_running() {
    use std::os::windows::fs::OpenOptionsExt;
    let tmp = TempDir::new("replold");
    let dest = tmp.0.join("tool.exe");
    fs::write(&dest, "1").unwrap();
    let old = tmp.0.join("tool.exe.old");
    fs::write(&old, "0").unwrap();
    fs::write(tmp.0.join("tool.exe.old-123"), "left").unwrap();
    // Without FILE_SHARE_DELETE: neither deleted nor replaced.
    let held = fs::OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&old)
        .unwrap();
    let new = tmp.0.join("new.exe");
    fs::write(&new, "2").unwrap();
    replace_file(&new, &dest).unwrap();
    assert_eq!(fs::read_to_string(&dest).unwrap(), "2");
    assert_eq!(fs::read_to_string(&old).unwrap(), "0");
    assert_eq!(names(&tmp.0), ["new.exe", "tool.exe", "tool.exe.old"]);
    drop(held);
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
    fs::create_dir_all(tmp.0.join("apps").join("tool.atollo-new").join("bin")).unwrap();
    fs::write(
        tmp.0.join("apps").join("tool.atollo-new").join("stale.txt"),
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
        ["tool", "tool.atollo-new", "uv.exe.atollo-new"]
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

/// A rebuild that fails halfway (no network, solver, full disk) leaves the
/// old folder where it was, without what the failed one wrote (BUG-V6-02).
#[test]
fn rebuild_in_place_puts_the_old_one_back_on_failure() {
    let tmp = TempDir::new("rebuildfail");
    let dest = tmp.0.join("r").join("4.5");
    tree(&dest, "old");
    let err = rebuild_in_place(&dest, || -> Result<()> {
        assert!(!dest.exists(), "the old one is set aside first");
        fs::create_dir_all(dest.join("conda-meta")).unwrap();
        fs::write(dest.join("half.txt"), "new").unwrap();
        bail!("solver failed")
    })
    .unwrap_err();
    assert!(err.to_string().contains("solver failed"), "{err:#}");
    assert_eq!(
        fs::read_to_string(dest.join("bin").join("tool.exe")).unwrap(),
        "old"
    );
    assert!(!dest.join("half.txt").exists());
    assert!(!dest.join("conda-meta").exists());
    assert_eq!(names(&tmp.0.join("r")), ["4.5"]);
}

/// A rebuild that works leaves only the new folder; one without an old
/// folder that fails leaves nothing.
#[test]
fn rebuild_in_place_replaces_on_success() {
    let tmp = TempDir::new("rebuildok");
    let dest = tmp.0.join("r").join("4.5");
    tree(&dest, "old");
    rebuild_in_place(&dest, || {
        tree(&dest, "new");
        Ok(())
    })
    .unwrap();
    assert_eq!(
        fs::read_to_string(dest.join("bin").join("tool.exe")).unwrap(),
        "new"
    );
    assert_eq!(names(&tmp.0.join("r")), ["4.5"]);

    let fresh = tmp.0.join("r").join("4.4");
    rebuild_in_place(&fresh, || -> Result<()> {
        tree(&fresh, "half");
        bail!("no network")
    })
    .unwrap_err();
    assert!(!fresh.exists());
}
