use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn saved_in_data_and_kept() {
    let tmp = TempDir::new("project-settings");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(load(&kit, "app"), ProjectSettings::default());
    update(&kit, "app", |s| s.agent = Some("codex".into()));
    opened(&kit, "app");
    let file = kit
        .data()
        .join("projects")
        .join("app")
        .join("dev-settings.json");
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"agent\": \"codex\"") && text.contains("lastOpened"));
    // a favorite is written only while it is one
    assert!(!text.contains("pinned"));
    update(&kit, "app", |s| s.pinned = true);
    assert!(load(&kit, "app").pinned);
    update(&kit, "app", |s| s.pinned = false);
    assert!(!std::fs::read_to_string(&file).unwrap().contains("pinned"));
    // a key of a newer version stays
    std::fs::write(&file, r#"{"agent":"claude","future":{"x":1}}"#).unwrap();
    opened(&kit, "app");
    let s = load(&kit, "app");
    assert_eq!(s.agent.as_deref(), Some("claude"));
    assert!(s.other.contains_key("future") && s.last_opened.is_some());
    // never outside data\projects
    for bad in ["..", r"..\x", "a/b", "con"] {
        update(&kit, bad, |s| s.agent = Some("x".into()));
        assert!(dir(&kit, bad).is_none(), "{bad}");
    }
    remove(&kit, "app").unwrap();
    assert!(!file.exists());
    remove(&kit, "app").unwrap();
    remove(&kit, "..").unwrap();
}
