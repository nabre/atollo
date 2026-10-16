use super::*;
use serde_json::json;

#[test]
fn zig_lines_of_the_index() {
    let zig = json!({
        "master": {"x86_64-windows": {}},
        "0.15.2": {"x86_64-windows": {}},
        "0.15.1": {"x86_64-windows": {}},
        "0.14.1": {"x86_64-windows": {}, "aarch64-windows": {}}
    });
    assert_eq!(zig_lines(&zig, "x86_64-windows"), ["0.15", "0.14"]);
    assert_eq!(zig_lines(&zig, "aarch64-windows"), ["0.14"]);
}
