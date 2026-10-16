use super::*;
use serde_json::json;

#[test]
fn java_lines_of_the_info() {
    let info =
        json!({"available_lts_releases": [8, 11, 17, 21, 25], "most_recent_feature_release": 26});
    assert_eq!(
        java_lines(&info),
        ["26.0", "25.0", "21.0", "17.0", "11.0", "8.0"]
    );
}
