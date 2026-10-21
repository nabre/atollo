use super::*;

use crate::kit::tests::TempDir;
use crate::kit::Arch;

/// The key is made once and kept; the settings keep what they were given.
#[test]
fn key_and_settings_are_kept() {
    let tmp = TempDir::new("llm-settings");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(key(&kit), None);
    let first = ensure_key(&kit).unwrap();
    assert_eq!(first.len(), 64);
    assert_eq!(ensure_key(&kit).unwrap(), first);
    assert_eq!(key(&kit).as_deref(), Some(first.as_str()));
    assert_eq!(load(&kit), LlmSettings::default());
    update(&kit, |s| {
        s.port = Some(9932);
        s.pi_model = Some("gpt-oss-20b".into());
        Ok(())
    })
    .unwrap();
    let s = load(&kit);
    assert_eq!(
        (s.port, s.pi_model.as_deref(), s.autostart),
        (Some(9932), Some("gpt-oss-20b"), false)
    );
}

/// A file that cannot be read is not replaced by the defaults.
#[test]
fn a_damaged_file_is_left_alone() {
    let tmp = TempDir::new("llm-settings-bad");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    std::fs::write(path(&kit), "{ not json").unwrap();
    assert!(update(&kit, |_| Ok(())).is_err());
    assert_eq!(std::fs::read_to_string(path(&kit)).unwrap(), "{ not json");
}
