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
