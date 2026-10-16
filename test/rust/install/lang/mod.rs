use super::*;

#[test]
fn kotlin_lines_from_releases() {
    let rel = |tag: &str| GhRelease {
        tag_name: tag.into(),
        assets: Vec::new(),
        prerelease: false,
    };
    let list = [
        rel("v2.2.21"),
        rel("v2.2.20"),
        rel("v2.1.21"),
        rel("v2.0.0"),
    ];
    assert_eq!(release_lines(&list), ["2.2", "2.1", "2.0"]);
}

#[test]
fn versions() {
    assert!(version_key("1.10.2") > version_key("1.9.9"));
    assert!(
        is_release("0.14.1")
            && !is_release("master")
            && !is_release("4.0.0-beta-1")
            && !is_release("")
    );
}
