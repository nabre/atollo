use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::fs;

#[test]
fn project_names_cannot_escape() {
    let tmp = TempDir::new("cmd-proj");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    fs::create_dir_all(kit.projects().join("app")).unwrap();
    assert_eq!(project_dir(&kit, None, "it").unwrap(), kit.projects());
    assert_eq!(
        project_dir(&kit, Some("app"), "it").unwrap(),
        kit.projects().join("app")
    );
    for bad in ["..", r"..\data", "a/b", "C:\\x", "missing", "con"] {
        assert!(project_dir(&kit, Some(bad), "it").is_err(), "{bad}");
    }
}

#[test]
fn unavailable_msg_names_arm64_only_on_windows_arm64() {
    use crate::kit::Target;
    assert_eq!(
        unavailable_msg("it", "R", Target::WIN32_ARM64),
        "R non è disponibile sui PC ARM64."
    );
    assert_eq!(
        unavailable_msg("en", "R", Target::WIN32_ARM64),
        "R is not available on ARM64 PCs."
    );
}

#[test]
fn unavailable_msg_names_the_system_elsewhere() {
    use crate::kit::Target;
    let mac = unavailable_msg("it", "Apache", Target::DARWIN_ARM64);
    assert_eq!(mac, "Apache non è disponibile per darwin-arm64.");
    let linux = unavailable_msg("en", "PHP", Target::LINUX_X64);
    assert_eq!(linux, "PHP is not available for linux-x64.");
    for lang in ["it", "en", "fr", "de"] {
        let text = unavailable_msg(lang, "Caddy", Target::LINUX_ARM64);
        assert!(
            text.contains("linux-arm64") && !text.contains("ARM64"),
            "{text}"
        );
    }
}
