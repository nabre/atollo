use super::*;
use crate::test_support::prepared_kit;
use std::fs;

/// Deleting a project drops its database from every version it was created
/// on (`used`), then the one it is on now, each once; a `db.json` of before
/// has only the one it is on.
#[test]
fn drop_lines_cover_every_version_used() {
    let (_tmp, kit) = prepared_kit("services-drop-lines");
    let file = kit
        .project_data()
        .join("shop")
        .join("private")
        .join("db.json");
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    assert!(project_drop_lines(&kit, "shop", "mysql").is_empty());
    fs::write(&file, r#"{ "mysql": "pw", "lines": { "mysql": "8.0" } }"#).unwrap();
    assert_eq!(project_drop_lines(&kit, "shop", "mysql"), ["8.0"]);
    auth::add_project_used_line(&kit, "shop", "mysql", "9.7").unwrap();
    auth::add_project_used_line(&kit, "shop", "mysql", "8.0").unwrap();
    assert_eq!(project_drop_lines(&kit, "shop", "mysql"), ["9.7", "8.0"]);
    auth::set_project_line(&kit, "shop", "mysql", Some("8.4")).unwrap();
    assert_eq!(
        project_drop_lines(&kit, "shop", "mysql"),
        ["9.7", "8.0", "8.4"]
    );
    assert!(project_drop_lines(&kit, "shop", "mongodb").is_empty());
}
