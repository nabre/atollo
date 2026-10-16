use super::*;
use crate::kit::tests::TempDir;

#[test]
fn rust_plan_by_kit_and_temp() {
    let never = || -> bool { panic!("temp probed on a kit that does it in place") };
    // NTFS kit, installed: in place, the temp is not even probed.
    assert_eq!(rust_plan(true, true, never).unwrap(), RustPlan::InPlace);
    // FAT/exFAT kit, installed: a copy in the temp.
    assert_eq!(
        rust_plan(true, false, || true).unwrap(),
        RustPlan::CopyUpdate
    );
    // Fresh install: always in the temp, on NTFS or FAT kits alike.
    assert_eq!(rust_plan(false, true, || true).unwrap(), RustPlan::Fresh);
    assert_eq!(rust_plan(false, false, || true).unwrap(), RustPlan::Fresh);
}

#[test]
fn rust_plan_refuses_a_temp_without_hard_links() {
    assert!(rust_plan(false, false, || false).is_err());
    assert!(rust_plan(true, false, || false).is_err());
    assert!(rust_plan(false, true, || false).is_err());
}

#[test]
fn r_versions_of_windows_only() {
    let info = serde_json::json!({"files": [
        {"version": "4.6.1", "attrs": {"subdir": "linux-64"}},
        {"version": "4.6.0", "attrs": {"subdir": "win-64"}},
        {"version": "4.5.3", "attrs": {"subdir": "win-64"}},
        {"version": "4.6.0", "attrs": {"subdir": "win-64"}},
        {"version": "4.7.0rc1", "attrs": {"subdir": "win-64"}}
    ]});
    assert_eq!(r_versions(&info), ["4.6.0", "4.5.3"]);
}

#[test]
fn hard_links_on_the_test_disk() {
    let tmp = TempDir::new("hardlink");
    // The test folder is on the system disk (NTFS).
    assert!(can_hard_link(&tmp.0));
    assert!(
        fs::read_dir(&tmp.0).unwrap().next().is_none(),
        "probe files removed"
    );
}
