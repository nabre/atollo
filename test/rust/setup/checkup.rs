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

#[test]
fn php_header_version() {
    let head = "<?php\n/** Adminer\n* @license x\n* @version 6.1.2\n*/namespace\n";
    assert_eq!(header_version(head), Some("6.1.2"));
    // phpMyAdmin's libraries/classes/Version.php.
    let pma = "<?php\nfinal class Version\n{\n    public const VERSION = '5.2.3' . VERSION_SUFFIX;\n    public const SERIES = '5.2';\n";
    assert_eq!(header_version(pma), Some("5.2.3"));
    assert_eq!(header_version("<?php echo 1;"), None);
}

/// `@` in the check program is the kit root, as in the key.
#[test]
#[cfg_attr(not(windows), ignore = "runs a batch file (@echo) as pnpm")]
fn check_program_at_the_kit_root() {
    let tmp = TempDir::new("checkup-root");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let pnpm = Catalog::get().find("pnpm").unwrap();
    for p in pnpm.key_paths(&kit) {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, b"@echo 10.0.0\r\n").unwrap();
    }
    let result = check(&kit, &KitEnv::default(), pnpm).unwrap();
    assert!(result.ok, "{}", result.answer);
    assert_eq!(result.answer, "10.0.0");
}

/// Half installed: in the check, failed, with the missing part.
#[test]
#[cfg_attr(not(windows), ignore = "expects the Windows part ninja.exe")]
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
    assert!(result.answer.contains("ninja.exe"), "{}", result.answer);
}
