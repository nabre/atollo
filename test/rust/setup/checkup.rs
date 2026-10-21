use super::*;

use crate::kit::{tests::TempDir, Arch};

#[test]
fn every_check_names_an_installed_file() {
    for c in Catalog::get().all() {
        if let Some((program, _)) = c.check.split_first() {
            let top = program.split('/').next().unwrap_or_default();
            let owned = c.folder.as_deref() == Some(top)
                || top == "tools"
                || *program == c.key
                || c.keys.contains(program);
            assert!(owned, "{}: check {program} outside its folder", c.id);
        }
    }
    for c in Catalog::get().all() {
        assert!(!c.check.is_empty(), "{}: no final check", c.id);
    }
}

/// A program that must not run (OpenConsole.exe) is checked by the
/// version in its resources.
#[cfg(windows)]
#[test]
fn file_version_reads_the_resources() {
    let cmd = crate::os::jobs::system_root().join(r"System32\cmd.exe");
    let v = file_version(&cmd).unwrap();
    assert!(v.starts_with("10.0."), "{v}");
    assert!(file_version(std::path::Path::new(r"C:\nope\x.exe")).is_err());
}

#[test]
fn php_header_version() {
    let head = "<?php\n/** Adminer\n* @license x\n* @version 6.1.2\n*/namespace\n";
    assert_eq!(header_version(head), Some("6.1.2"));
    // phpMyAdmin's libraries/classes/Version.php.
    let pma = "<?php\nfinal class Version\n{\n    public const VERSION = '5.2.3' . VERSION_SUFFIX;\n    public const SERIES = '5.2';\n";
    assert_eq!(header_version(pma), Some("5.2.3"));
    assert_eq!(header_version("<?php echo 1;"), None);
}

/// mongo-express's check (D-147): the version of its package.json.
#[test]
fn package_json_version() {
    let tmp = TempDir::new("checkup-package");
    let file = tmp.0.join("package.json");
    std::fs::write(&file, r#"{"name":"mongo-express","version":"1.1.0-rc-4"}"#).unwrap();
    assert_eq!(package_version(&file).unwrap(), "1.1.0-rc-4");
    std::fs::write(&file, r#"{"name":"mongo-express"}"#).unwrap();
    assert!(package_version(&file).is_err());
    std::fs::write(&file, "not json").unwrap();
    assert!(package_version(&file).is_err());
    assert!(package_version(&tmp.0.join("none.json")).is_err());
}

/// `@` in the check program is the kit root, as in the key.
#[test]
fn check_program_at_the_kit_root() {
    let tmp = TempDir::new("checkup-root");
    // The host's own CPU: a kit of another one is only checked on disk
    // (an x64 kit on Linux ARM64 runs nothing).
    let kit = Kit::at(tmp.0.clone(), Arch::detect());
    kit.prepare().unwrap();
    let pnpm = Catalog::get().find("pnpm").unwrap();
    // `pnpm.cmd` a batch file on Windows, `bin/pnpm` a shell script
    // elsewhere.
    let script: &[u8] = if cfg!(windows) {
        b"@echo 10.0.0\r\n"
    } else {
        b"#!/bin/sh\necho 10.0.0\n"
    };
    for p in pnpm.key_paths(&kit) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, script).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    // The batch file runs through cmd.exe with this environment only, as
    // in the app's: Windows on ARM64 starts no process without SystemRoot.
    let mut env = KitEnv::default();
    if let Some(root) = std::env::var_os("SystemRoot") {
        env.set("SystemRoot", root.to_string_lossy());
    }
    let mut result = check(&kit, &env, pnpm).unwrap();
    // A process forked by another test while the script was still open for
    // writing holds the descriptor until its exec: the exec of the script
    // fails with ETXTBSY until then. Retry for a short while there.
    let attempts = if cfg!(unix) { 20 } else { 1 };
    for _ in 1..attempts {
        if result.ok {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        result = check(&kit, &env, pnpm).unwrap();
    }
    assert!(result.ok, "{}", result.answer);
    assert_eq!(result.answer, "10.0.0");
}

/// A check that writes more than a pipe holds (64 KB) answers at once: the
/// output is read while it runs, not after.
#[test]
fn a_long_answer_does_not_wait_for_the_limit() {
    let tmp = TempDir::new("checkup-long");
    let (name, script): (&str, &[u8]) = if cfg!(windows) {
        (
            "long.cmd",
            b"@echo 1.2.3\r\n@for /L %%i in (1,1,3000) do @echo 0123456789012345678901234567890123456789\r\n",
        )
    } else {
        (
            "long.sh",
            b"#!/bin/sh\necho 1.2.3\nyes 0123456789012345678901234567890123456789 | head -n 3000\n",
        )
    };
    let exe = tmp.0.join(name);
    std::fs::write(&exe, script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut env = KitEnv::default();
    for var in ["SystemRoot", "PATH"] {
        if let Some(v) = std::env::var_os(var) {
            env.set(var, v.to_string_lossy());
        }
    }
    let start = Instant::now();
    // ETXTBSY right after writing the script (see above): retry there.
    let mut result = run(&env, &exe, &[]);
    let attempts = if cfg!(unix) { 20 } else { 1 };
    for _ in 1..attempts {
        if result.is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
        result = run(&env, &exe, &[]);
    }
    assert_eq!(result.unwrap(), "1.2.3");
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "waited for the limit"
    );
}

/// Half installed: in the check, failed, with the missing part.
#[test]
fn half_installed_fails_the_check() {
    let tmp = TempDir::new("checkup-half");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let cmake = Catalog::get().find("cmake").unwrap();
    assert!(check(&kit, &KitEnv::default(), cmake).is_none(), "absent");
    let key = &cmake.key_paths(&kit)[0];
    std::fs::create_dir_all(key.parent().unwrap()).unwrap();
    std::fs::write(key, b"").unwrap();
    let result = check(&kit, &KitEnv::default(), cmake).unwrap();
    assert!(!result.ok);
    // `tools/ninja.exe` on Windows, `tools/ninja` elsewhere
    let ninja = crate::fsx::join_rel(kit.apps(), &cmake.keys_for(kit.target())[0]);
    assert!(
        result.answer.contains(&ninja.display().to_string()),
        "{}",
        result.answer
    );
}
