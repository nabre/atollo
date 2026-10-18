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

/// D-112: GNU by default on both Windows, `gnullvm` on ARM64 (linked by
/// llvm-mingw's clang); MSVC only when chosen; the native triple elsewhere,
/// whatever the setting says.
#[test]
fn rust_host_per_target_and_setting() {
    for (toolchain, x64, arm64) in [
        (None, "x86_64-pc-windows-gnu", "aarch64-pc-windows-gnullvm"),
        (
            Some("gnu"),
            "x86_64-pc-windows-gnu",
            "aarch64-pc-windows-gnullvm",
        ),
        (
            Some("msvc"),
            "x86_64-pc-windows-msvc",
            "aarch64-pc-windows-msvc",
        ),
    ] {
        assert_eq!(rust_host_for(Target::WIN32_X64, toolchain), x64);
        assert_eq!(rust_host_for(Target::WIN32_ARM64, toolchain), arm64);
        for t in [
            Target::DARWIN_ARM64,
            Target::DARWIN_X64,
            Target::LINUX_X64,
            Target::LINUX_ARM64,
        ] {
            assert_eq!(rust_host_for(t, toolchain), t.rust_triple(), "{t}");
        }
    }
}

/// Only the `gnullvm` host needs C, C++ (llvm-mingw) to link.
#[test]
fn only_gnullvm_needs_cc() {
    assert!(host_needs_cc("aarch64-pc-windows-gnullvm"));
    for t in Target::ALL {
        for toolchain in [None, Some("msvc")] {
            let host = rust_host_for(t, toolchain);
            assert_eq!(
                host_needs_cc(host),
                t == Target::WIN32_ARM64 && toolchain.is_none(),
                "{t} {toolchain:?}"
            );
        }
    }
}

#[test]
fn rustup_init_args_for_both_arm64_hosts() {
    for host in ["aarch64-pc-windows-gnullvm", "aarch64-pc-windows-msvc"] {
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

/// BUG-V3-06: conda writes the environment's folder into R's Makeconf
/// (`LOCAL_SOFT`); only that line changes, line endings kept.
#[test]
fn makeconf_follows_the_folder_of_r() {
    let text = "SHELL = /bin/sh\r\nLOCAL_SOFT ?= D:/Kit/data/apps/win32-x64/r/4.6/Library\r\nLIBR = -L\"$(R_HOME)/bin/x64\"\r\n";
    let new = relocated_makeconf(text, Path::new(r"E:\Kit\data\apps\win32-x64\r\4.6")).unwrap();
    assert_eq!(
        new,
        "SHELL = /bin/sh\r\nLOCAL_SOFT ?= E:/Kit/data/apps/win32-x64/r/4.6/Library\r\nLIBR = -L\"$(R_HOME)/bin/x64\"\r\n"
    );
    assert_eq!(
        relocated_makeconf(&new, Path::new(r"E:\Kit\data\apps\win32-x64\r\4.6")),
        None,
        "already right: nothing to write"
    );
    let plain = "LOCAL_SOFT = C:/old/Library\n";
    assert_eq!(
        relocated_makeconf(plain, Path::new("D:/new/r")).as_deref(),
        Some("LOCAL_SOFT = D:/new/r/Library\n")
    );
    assert_eq!(relocated_makeconf("CC = gcc\n", Path::new("D:/r")), None);
}

/// The kit moved to another drive letter or folder: at the next start every
/// line of R on Windows (the default and a saved one) names its own folder.
#[test]
fn moving_the_kit_rewrites_makeconf_on_windows() {
    let tmp = TempDir::new("makeconf-moved");
    let write = |root: &Path, line: &str, prefix: &str| {
        let file = join_rel(
            root,
            &format!("data/apps/win32-x64/r/{line}/lib/R/etc/x64/Makeconf"),
        );
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, format!("CC = gcc\nLOCAL_SOFT ?= {prefix}/Library\n")).unwrap();
        file
    };
    let old = tmp.0.join("one");
    let first = Kit::at_target(old.clone(), Target::WIN32_X64);
    first.prepare().unwrap();
    let now = tmp.0.join("two");
    fs::rename(&old, &now).unwrap();
    let old_r = old.join("data/apps/win32-x64/r/4.6");
    let files = [
        write(&now, "4.6", &old_r.to_string_lossy().replace('\\', "/")),
        write(&now, "4.5", "D:/bld/r-base_1/_h_env"),
    ];
    let kit = Kit::at_target(now.clone(), Target::WIN32_X64);
    assert_eq!(kit.prepare().unwrap(), Some(old.clone()));
    for (file, line) in files.iter().zip(["4.6", "4.5"]) {
        let prefix = now
            .join("data/apps/win32-x64/r")
            .join(line)
            .to_string_lossy()
            .replace('\\', "/");
        assert_eq!(
            fs::read_to_string(file).unwrap(),
            format!("CC = gcc\nLOCAL_SOFT ?= {prefix}/Library\n"),
            "{line}"
        );
    }

    // Off Windows R is installed again instead (D-92): nothing rewritten.
    let linux = Kit::at_target(tmp.0.join("linux"), Target::LINUX_X64);
    let file = join_rel(
        linux.root(),
        "data/apps/linux-x64/r/4.6/lib/R/etc/x64/Makeconf",
    );
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(&file, "LOCAL_SOFT ?= /old/Library\n").unwrap();
    linux.prepare().unwrap();
    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "LOCAL_SOFT ?= /old/Library\n"
    );
}
