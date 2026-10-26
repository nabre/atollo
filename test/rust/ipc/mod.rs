use super::*;
use crate::test_support::prepared_kit;
use std::fs;

#[test]
fn project_names_cannot_escape() {
    let (_tmp, kit) = prepared_kit("cmd-proj");
    fs::create_dir_all(kit.projects().join("app")).unwrap();
    assert_eq!(project_dir(&kit, None, "it").unwrap(), kit.projects());
    assert_eq!(
        project_dir(&kit, Some("app"), "it").unwrap(),
        kit.projects().join("app")
    );
    for bad in ["..", r"..\data", "a/b", "C:\\x", "missing", "con"] {
        assert!(project_dir(&kit, Some(bad), "it").is_err(), "{bad}");
    }
}

/// «Apri nel browser» opens the local server and the sites of a pool
/// (`<project>.localhost`), nothing else.
#[test]
fn local_urls_include_the_pool_sites() {
    for good in [
        "http://localhost:7100/",
        "http://127.0.0.1:7100/a?b#c",
        "http://shop.localhost:8080/",
        "http://api.shop-2.localhost:8080",
    ] {
        assert!(app::is_local_url(good), "{good}");
    }
    for bad in [
        "https://localhost:7100/",
        "http://example.com:80/",
        "http://localhost.example.com:80/",
        "http://evil.com/.localhost:80/",
        "http://.localhost:80/",
        "http://a..localhost:80/",
        "http://a_b.localhost:80/",
        "http://shop.localhost:99999/",
        "http://localhost:7100/ \"x\"",
    ] {
        assert!(!app::is_local_url(bad), "{bad}");
    }
}

#[test]
fn unavailable_msg_names_the_system() {
    use crate::kit::Target;
    let mac = unavailable_msg("it", "Apache", Target::DARWIN_ARM64);
    assert_eq!(mac, "Apache non è disponibile per darwin-arm64.");
    let linux = unavailable_msg("en", "PHP", Target::LINUX_X64);
    assert_eq!(linux, "PHP is not available for linux-x64.");
    for lang in ["it", "en", "fr", "de"] {
        let text = unavailable_msg(lang, "Caddy", Target::LINUX_ARM64);
        assert!(
            text.contains("linux-arm64") && !text.contains("ARM64"),
            "{text}"
        );
    }
}
