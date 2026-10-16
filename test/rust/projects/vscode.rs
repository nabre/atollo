use super::*;
use crate::kit::tests::TempDir;
use std::fs;

#[test]
fn settings_round_trip_keeping_other_keys() {
    let tmp = TempDir::new("vscode-settings");
    let d = tmp.0.join("p");
    fs::create_dir_all(d.join(".vscode")).unwrap();
    assert!(!get(&d).exists);
    fs::write(
        d.join(".vscode").join("settings.json"),
        r#"{"python.analysis.typeCheckingMode":"strict","files.exclude":{"**/.git":false,"**/dist":true}}"#,
    )
    .unwrap();
    let v = get(&d);
    assert_eq!(v.settings.hidden, ["**/dist"]);
    let s = EditorSettings {
        format_on_save: Some(true),
        tab_size: Some(4),
        insert_spaces: Some(true),
        hidden: vec!["**/vendor".into()],
    };
    set(&d, &s).unwrap();
    let text = fs::read_to_string(d.join(".vscode").join("settings.json")).unwrap();
    let o: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(o["python.analysis.typeCheckingMode"], "strict");
    assert_eq!(o["files.exclude"]["**/.git"], false, "false patterns stay");
    assert_eq!(o["files.exclude"]["**/vendor"], true);
    assert!(
        o["files.exclude"].get("**/dist").is_none(),
        "unchecked: removed"
    );
    assert_eq!(get(&d).settings, s);
    assert!(set(
        &d,
        &EditorSettings {
            tab_size: Some(0),
            ..s.clone()
        }
    )
    .is_err());
    // Comments: shown as read only, never written.
    fs::write(d.join(".vscode").join("settings.json"), "// mine\n{}").unwrap();
    assert!(get(&d).read_only);
    assert!(set(&d, &s).is_err());
    assert_eq!(
        fs::read_to_string(d.join(".vscode").join("settings.json")).unwrap(),
        "// mine\n{}"
    );
}
