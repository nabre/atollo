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

#[test]
fn safe_directory_is_the_kits_folder() {
    assert_eq!(
        safe_pattern(std::path::Path::new(r"D:\Kits\Atollo")),
        "D:/Kits/Atollo/*"
    );
    assert_eq!(
        safe_pattern(std::path::Path::new("/Volumes/USB/Atollo/")),
        "/Volumes/USB/Atollo/*"
    );
    // Written by older kits, or by this kit where it was: replaced.
    assert!(kit_safe_value("*"));
    assert!(kit_safe_value("Z:/gone/kit/*"));
    let tmp = crate::kit::tests::TempDir::new("safe-dir");
    std::fs::create_dir_all(tmp.0.join("data").join("app")).unwrap();
    assert!(kit_safe_value(&safe_pattern(&tmp.0)));
    // The user's own: kept.
    std::fs::create_dir_all(tmp.0.join("work")).unwrap();
    assert!(!kit_safe_value(&safe_pattern(&tmp.0.join("work"))));
    assert!(!kit_safe_value("C:/Users/me/repo"));
}

#[test]
fn identity_is_checked() {
    let ok = clean_identity(&Identity {
        name: "  Ada Lovelace ".into(),
        email: " 1+ada@users.noreply.github.com ".into(),
    })
    .unwrap();
    assert_eq!(ok.name, "Ada Lovelace");
    assert_eq!(ok.email, "1+ada@users.noreply.github.com");
    assert!(
        clean_identity(&Identity::default()).is_ok(),
        "both empty: removed"
    );
    for email in ["ada", "ada@host", "a da@x.org", "<ada@x.org>", "@x.org"] {
        assert!(
            clean_identity(&Identity {
                name: "Ada".into(),
                email: email.into()
            })
            .is_err(),
            "{email}"
        );
    }
    assert!(clean_identity(&Identity {
        name: "Ada\nEvil".into(),
        email: String::new()
    })
    .is_err());
    assert_eq!(
        noreply(583231, "octocat"),
        "583231+octocat@users.noreply.github.com"
    );
}
