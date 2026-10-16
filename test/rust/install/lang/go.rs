use super::*;
use serde_json::json;

#[test]
fn go_lines_of_the_list() {
    let go = |v: &str, stable: bool, arch: &str| {
        json!({"version": v, "stable": stable, "files": [
            {"os": "windows", "arch": arch, "kind": "archive", "filename": "x.zip", "sha256": "a"}
        ]})
    };
    let list = [
        go("go1.25.3", true, "amd64"),
        go("go1.26rc1", false, "amd64"),
        go("go1.24.9", true, "amd64"),
        go("go1.24.10", true, "amd64"),
        go("go1.23.1", true, "arm64"),
    ];
    assert_eq!(go_lines(&list, "amd64"), ["1.25", "1.24"]);
    assert_eq!(go_lines(&list, "arm64"), ["1.23"]);
}
