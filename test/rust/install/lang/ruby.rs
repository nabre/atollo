use super::*;
use serde_json::json;

#[test]
fn ruby_highest_version_and_build() {
    let digest = format!("sha256:{}", "0".repeat(64));
    let rel = |pre: bool, names: &[&str]| json!({"prerelease": pre, "assets": names.iter().map(|n| json!({"name": n, "digest": digest})).collect::<Vec<_>>()});
    let releases = vec![
        rel(
            false,
            &[
                "rubyinstaller-3.3.9-1-x64.7z",
                "rubyinstaller-3.3.9-1-arm.7z",
            ],
        ),
        rel(
            false,
            &[
                "rubyinstaller-3.4.5-1-x64.7z",
                "rubyinstaller-3.4.5-2-x64.7z",
            ],
        ),
        rel(true, &["rubyinstaller-3.5.0-1-x64.7z"]),
    ];
    assert_eq!(
        pick_ruby_line(&releases, "x64", None).map(|(v, _)| v),
        Some("3.4.5-2".into())
    );
    assert_eq!(
        pick_ruby_line(&releases, "arm", None).map(|(v, _)| v),
        Some("3.3.9-1".into())
    );
    // Without GitHub's SHA-256 (older builds): not offered.
    let old =
        vec![json!({"prerelease": false, "assets": [{"name": "rubyinstaller-3.1.7-1-x64.7z"}]})];
    assert!(ruby_lines(&old, "x64").is_empty());
}
