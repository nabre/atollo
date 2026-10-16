use super::*;

#[test]
fn hosts_file() {
    let text = "github.com:\n    users:\n        user:\n            oauth_token: x\n        nabre:\n    git_protocol: https\n    user: nabre\n    oauth_token: gho_x\nexample.com:\n    user: other\n";
    assert_eq!(hosts_user(text).as_deref(), Some("nabre"));
    assert_eq!(hosts_user("example.com:\n    user: other\n"), None);
    assert_eq!(hosts_user(""), None);
}

#[test]
fn code_of_the_sign_in() {
    assert_eq!(
        one_time_code("! First copy your one-time code: 1A2B-3C4D").as_deref(),
        Some("1A2B-3C4D")
    );
    assert_eq!(
        one_time_code(
            "Open this URL to continue in your web browser: https://github.com/login/device"
        ),
        None
    );
    assert_eq!(one_time_code("abcd-efgh"), None);
}

#[test]
fn gh_helpers() {
    assert!(is_gh_helper(HELPER));
    assert!(is_gh_helper(
        r"!'C:\\Kit\\apps\\tools\\gh.exe' auth git-credential"
    ));
    assert!(!is_gh_helper("manager"));
    assert!(!is_gh_helper(""));
}
