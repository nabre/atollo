use super::*;
use crate::common::fsx::join_rel;
use crate::kit::tests::TempDir;
use std::time::SystemTime;

#[test]
fn in_host_temp_or_data_tmp() {
    let tmp = TempDir::new("temp-work");
    let kit = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch());
    let work = TempWork::new(&kit, "setup/x").unwrap();
    assert_eq!(work.path(), join_rel(&tmp.0, "kit/data/tmp/setup/x"));
    assert!(work.path().join(".lock").is_file());
    let dir = work.path().to_path_buf();
    drop(work);
    assert!(!dir.exists());

    let host = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch())
        .with_host(Some(&tmp.0.join("host")));
    let work = TempWork::new(&host, "setup/x").unwrap();
    assert!(work.path().starts_with(tmp.0.join("host").join("Atollo")));
    // a second user of the same folder starts from empty
    fs::write(work.path().join("old"), "x").unwrap();
    drop(work);
    let again = TempWork::new(&host, "setup/x").unwrap();
    assert!(!again.path().join("old").exists());
}

fn age(dir: &Path, hours: u64) {
    let t = SystemTime::now() - Duration::from_secs(hours * 3600);
    let f = fs::OpenOptions::new()
        .write(true)
        .open(dir.join(".lock"))
        .unwrap();
    f.set_modified(t).unwrap();
}

#[test]
fn sweep_keeps_live_and_new_removes_old() {
    let tmp = TempDir::new("temp-sweep");
    let host = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch())
        .with_host(Some(&tmp.0.join("host")));
    let base = host.system_temp().unwrap().to_path_buf();

    // old and held by a live TempWork: kept
    let live = TempWork::new(&host, "setup/live").unwrap();
    age(live.path(), 48);
    // old and unlocked (a dead app left its lock closed): removed
    let dead = base.join("setup").join("dead");
    fs::create_dir_all(&dead).unwrap();
    fs::write(dead.join(".lock"), "").unwrap();
    fs::write(dead.join("file"), "x").unwrap();
    age(&dead, 48);
    // old, in data\tmp, unlocked: removed
    let old_data = join_rel(host.data(), "tmp/frameworks/1");
    fs::create_dir_all(&old_data).unwrap();
    fs::write(old_data.join(".lock"), "").unwrap();
    age(&old_data, 48);
    // new and unlocked: kept
    let fresh = base.join("frameworks").join("fresh");
    fs::create_dir_all(&fresh).unwrap();
    fs::write(fresh.join(".lock"), "").unwrap();
    // files added by hand: their own 30 days, never swept
    let downloads = base.join("downloads");
    fs::create_dir_all(&downloads).unwrap();
    fs::write(downloads.join(".lock"), "").unwrap();
    age(&downloads, 48);

    remove(sweep(&host));
    assert!(live.path().join(".lock").is_file(), "live work kept");
    assert!(!dead.exists(), "old unlocked removed");
    assert!(!old_data.exists());
    assert!(fresh.is_dir(), "new kept");
    assert!(downloads.is_dir(), "downloads kept");
    drop(live);
}

/// At start the leftovers are only set aside (renamed, quick): their names
/// are free at once, their files deleted afterwards (`remove`, in the
/// background); a live one is neither.
#[test]
fn sweep_frees_the_names_then_remove_deletes() {
    let tmp = TempDir::new("temp-aside");
    let host = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch())
        .with_host(Some(&tmp.0.join("host")));
    let base = host.system_temp().unwrap().to_path_buf();
    let live = TempWork::new(&host, "setup/live").unwrap();
    age(live.path(), 48);
    let dead = base.join("setup").join("dead");
    fs::create_dir_all(&dead).unwrap();
    fs::write(dead.join(".lock"), "").unwrap();
    fs::write(dead.join("file"), "x").unwrap();
    age(&dead, 48);

    let aside = sweep(&host);
    assert_eq!(aside.len(), 1, "{aside:?}");
    assert!(!dead.exists(), "its name is free");
    assert_eq!(aside[0].parent(), dead.parent(), "next to it");
    assert!(aside[0].join("file").is_file(), "not deleted yet");
    assert!(live.path().join(".lock").is_file(), "live work kept");
    remove(aside.clone());
    assert!(!aside[0].exists());
    drop(live);
}

#[test]
fn cleanup_at_exit_removes_programs_temp_only() {
    let tmp = TempDir::new("temp-exit");
    let host = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch())
        .with_host(Some(&tmp.0.join("host")));
    let base = host.system_temp().unwrap().to_path_buf();
    let work = TempWork::new(&host, "setup/x").unwrap();
    fs::create_dir_all(base.join("system").join("a")).unwrap();
    fs::create_dir_all(base.join("claude")).unwrap();
    cleanup_at_exit(&host);
    assert!(!base.join("system").exists() && !base.join("claude").exists());
    assert!(work.path().is_dir());
    cleanup_at_exit(&host); // nothing left: no error
}

#[cfg(windows)]
#[test]
fn every_host_temp_is_private_on_windows() {
    assert!(private_base(Path::new(r"C:\Users\me\AppData\Local\Temp")));
}

#[cfg(unix)]
#[test]
fn only_a_private_host_temp_holds_the_kits_folder() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("temp-private");
    let mine = tmp.0.join("mine");
    fs::create_dir_all(&mine).unwrap();
    fs::set_permissions(&mine, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(private_base(&mine));
    // Writable by others, as a shared /tmp (BUG-V7-17).
    fs::set_permissions(&mine, fs::Permissions::from_mode(0o1777)).unwrap();
    assert!(!private_base(&mine));
    assert!(!private_base(&tmp.0.join("missing")));
}

#[test]
fn falls_back_to_data_tmp_without_space() {
    let tmp = TempDir::new("temp-space");
    let host = Kit::at(tmp.0.join("kit"), crate::test_support::kit_arch())
        .with_host(Some(&tmp.0.join("host")));
    let work = TempWork::new_with_space(&host, "setup/big", u64::MAX).unwrap();
    assert_eq!(
        work.path(),
        host.data().join("tmp").join("setup").join("big")
    );
    let ok = TempWork::new_with_space(&host, "setup/small", 1).unwrap();
    assert!(ok.path().starts_with(host.system_temp().unwrap()));
}
