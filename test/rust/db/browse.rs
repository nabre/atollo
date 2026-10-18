use super::*;

#[test]
fn names_are_quoted() {
    assert_eq!(ident("a`b"), "`a``b`");
    assert_eq!(literal(r"it's\x"), r"'it\'s\\x'");
    assert!(is_system("mysql", "MySQL"));
    assert!(is_system("mongodb", "local"));
    assert!(!is_system("mongodb", "shop"));
}

#[test]
fn sql_values_hide_binary_and_cut_text() {
    assert_eq!(sql_value("id", "int"), "`id`");
    assert_eq!(sql_value("t", "longtext"), "LEFT(`t`, 2000)");
    assert!(sql_value("b", "BLOB").contains("'$bytes', OCTET_LENGTH(`b`)"));
}

#[test]
fn documents_become_columns_in_order() {
    let answer: Value = serde_json::from_str(
        r#"{"total": 7, "docs": [{"_id": {"$oid": "1"}, "a": 1}, {"_id": {"$oid": "2"}, "b": [2]}]}"#,
    )
    .unwrap();
    let r = documents(&answer, 0);
    let names: Vec<&str> = r.columns.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["_id", "a", "b"]);
    assert_eq!(r.rows[1][1], Value::Null);
    assert_eq!(r.rows[1][2], serde_json::json!([2]));
    assert_eq!(r.total, 7);
}

#[test]
fn json_lines_skip_blanks_and_fail_on_garbage() {
    assert_eq!(json_lines("1\n\n[2]\n").unwrap().len(), 2);
    assert!(json_lines("ERROR 1146").is_err());
}
