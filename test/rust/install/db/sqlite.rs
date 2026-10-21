#[cfg(unix)]
use std::fs;

use super::*;

#[test]
fn sqlite_tools_for_each_arch() {
    let page = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-dll-win-x64-3540000.zip,1450308,{d}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-arm64-3540000.zip,5619849,{a}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-x64-3540000.zip,6982216,{x}\n-->",
        d = "d".repeat(64),
        a = "a".repeat(64),
        x = "c".repeat(64)
    );
    let (v, path, sha) = sqlite_tools(&page, "x64").unwrap();
    assert_eq!(v, "3.54.0");
    assert_eq!(path, "2026/sqlite-tools-win-x64-3540000.zip");
    assert_eq!(sha, "c".repeat(64));
    assert_eq!(sqlite_tools(&page, "arm64").unwrap().2, "a".repeat(64));
}

/// No fallback to x64 for SQLite (D-108): the build of the architecture
/// asked for, or an error.
#[test]
fn sqlite_never_falls_back_to_x64() {
    let both = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-tools-win-x64-3540000.zip,456,{c}\nPRODUCT,3.54.0,2026/sqlite-tools-win-arm64-3540000.zip,457,{a}\n-->",
        c = "c".repeat(64),
        a = "a".repeat(64)
    );
    let (_, path, _) = sqlite_tools_for(&both, "win", "arm64").unwrap();
    assert_eq!(path, "2026/sqlite-tools-win-arm64-3540000.zip");
    let x64_only = format!(
        "<!--\nPRODUCT,3.54.0,2026/sqlite-tools-win-x64-3540000.zip,456,{c}\n-->",
        c = "c".repeat(64)
    );
    assert!(sqlite_tools_for(&x64_only, "win", "arm64").is_err());
    assert!(sqlite_tools_for(&x64_only, "linux", "arm64").is_err());
}

/// D-112: SQLite on Linux ARM64 from conda-forge (sqlite 3.54.x for
/// linux-aarch64 on api.anaconda.org), available there now; sqlite.org's
/// zips elsewhere, and still no x64 build on Linux ARM64.
#[test]
fn sqlite_from_conda_on_linux_arm64() {
    use crate::kit::Target;

    for t in Target::ALL {
        let conda = t == Target::LINUX_ARM64;
        assert_eq!(Sqlite.custom(t), conda, "{t}");
        assert_eq!(Sqlite.relocatable(t), !conda, "{t}");
    }
    let sqlite = crate::setup::catalog::Catalog::get()
        .find("sqlite")
        .unwrap();
    assert!(sqlite.is_available_on(Target::LINUX_ARM64));
    assert_eq!(sqlite.unavailable_reason(Target::LINUX_ARM64), None);
    assert_eq!(sqlite.key_for(Target::LINUX_ARM64), "sqlite/sqlite3");
    assert!(SQLITE_CONDA_LAUNCHER.starts_with("#!/bin/sh\n"));
    assert!(SQLITE_CONDA_LAUNCHER.contains("exec \"$(dirname \"$0\")/bin/sqlite3\" \"$@\""));
    assert!(!SQLITE_CONDA_LAUNCHER.contains('\r'));
}

/// The launcher really starts the environment's sqlite3, from any folder
/// (Unix only: it is a shell script).
#[cfg(unix)]
#[test]
fn sqlite_conda_launcher_runs_the_environment_program() {
    use std::os::unix::fs::PermissionsExt;

    let tmp = crate::kit::tests::TempDir::new("sqlite-launcher");
    let dir = tmp.0.join("sqlite").join("3.54");
    fs::create_dir_all(dir.join("bin")).unwrap();
    let fake = dir.join("bin").join("sqlite3");
    fs::write(&fake, "#!/bin/sh\necho \"fake $1\"\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let launcher = dir.join("sqlite3");
    fs::write(&launcher, SQLITE_CONDA_LAUNCHER).unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o755)).unwrap();
    let out = std::process::Command::new(&launcher)
        .arg("--version")
        .current_dir(&tmp.0)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "fake --version"
    );
}
