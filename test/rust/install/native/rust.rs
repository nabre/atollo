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

/// D-112: GNU by default on Windows; MSVC only when chosen; the native
/// triple elsewhere, whatever the setting says.
#[test]
fn rust_host_per_target_and_setting() {
    for (toolchain, x64) in [
        (None, "x86_64-pc-windows-gnu"),
        (Some("gnu"), "x86_64-pc-windows-gnu"),
        (Some("msvc"), "x86_64-pc-windows-msvc"),
    ] {
        assert_eq!(rust_host_for(Target::WIN32_X64, toolchain), x64);
        for t in [Target::DARWIN_ARM64, Target::LINUX_X64, Target::LINUX_ARM64] {
            assert_eq!(rust_host_for(t, toolchain), t.rust_triple(), "{t}");
        }
    }
}

#[test]
fn rustup_init_args_for_both_windows_hosts() {
    for host in ["x86_64-pc-windows-gnu", "x86_64-pc-windows-msvc"] {
        let args = rustup_init_args(host);
        let at = args.iter().position(|a| *a == "--default-host").unwrap();
        assert_eq!(args[at + 1], host);
        assert!(args.contains(&"--no-modify-path"));
        assert_eq!(
            args.windows(2)
                .filter(|w| w[0] == "-c")
                .map(|w| w[1])
                .collect::<Vec<_>>(),
            ["rustfmt", "clippy"]
        );
    }
}

#[test]
fn rust_plan_refuses_a_temp_without_hard_links() {
    assert!(rust_plan(false, false, || false).is_err());
    assert!(rust_plan(true, false, || false).is_err());
    assert!(rust_plan(false, true, || false).is_err());
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
