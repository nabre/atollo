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
    assert_eq!(conda_versions(&info, "win-64"), ["4.6.0", "4.5.3"]);
}

#[test]
fn conda_versions_per_subdir() {
    let info = serde_json::json!({"files": [
        {"version": "2.49.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.52.0.rc1", "attrs": {"subdir": "linux-64"}},
        {"version": "2.52.0rc1", "attrs": {"subdir": "linux-64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-64"}},
        {"version": "2.50.1", "attrs": {"subdir": "osx-arm64"}},
        {"version": "2.51.0", "attrs": {"subdir": "linux-aarch64"}},
        {"version": "2.48.1", "attrs": {"subdir": "noarch"}},
        {"attrs": {"subdir": "linux-64"}},
        {"version": "2.53.0"}
    ]});
    assert_eq!(
        conda_versions(&info, Target::LINUX_X64.conda_subdir()),
        ["2.51.0", "2.49.0"]
    );
    assert_eq!(
        conda_versions(&info, Target::DARWIN_ARM64.conda_subdir()),
        ["2.50.1"]
    );
    assert_eq!(
        conda_versions(&info, Target::LINUX_ARM64.conda_subdir()),
        ["2.51.0"]
    );
    assert!(conda_versions(&info, Target::DARWIN_X64.conda_subdir()).is_empty());
    assert!(conda_versions(&serde_json::json!({}), "linux-64").is_empty());
}

#[test]
fn micromamba_create_args_git() {
    let prefix = Path::new("/kit/data/apps/linux-x64/git/2.51");
    let args = micromamba_create_args(prefix, &["git=2.51.0"]);
    assert_eq!(
        args,
        [
            "create",
            "--no-rc",
            "-y",
            "-q",
            "-p",
            &prefix.display().to_string(),
            "-c",
            "conda-forge",
            "--override-channels",
            "git=2.51.0",
        ]
    );
    // -p is followed by the prefix, the specs come last.
    let p = args.iter().position(|a| a == "-p").unwrap();
    assert_eq!(args[p + 1], prefix.display().to_string());
    assert_eq!(args.last().map(String::as_str), Some("git=2.51.0"));
    // R: several specs, in order.
    let r = micromamba_create_args(prefix, &["r-base=4.5.1", "libblas=*=*openblas"]);
    assert_eq!(&r[r.len() - 2..], ["r-base=4.5.1", "libblas=*=*openblas"]);
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
