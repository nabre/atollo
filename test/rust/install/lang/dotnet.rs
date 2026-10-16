use super::*;
use serde_json::json;

#[test]
fn dotnet_lines_of_the_index() {
    let index = json!({"releases-index": [
        {"channel-version": "11.0", "support-phase": "preview"},
        {"channel-version": "10.0", "support-phase": "active"},
        {"channel-version": "9.0", "support-phase": "active"},
        {"channel-version": "6.0", "support-phase": "eol"}
    ]});
    assert_eq!(dotnet_lines(&index), ["10.0", "9.0", "6.0"]);
}
