use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn remotes_and_pages() {
    for url in [
        "https://github.com/Nabre/Dev-Kit.git",
        "https://x-access-token@github.com/nabre/dev-kit/",
        "git@github.com:nabre/dev-kit.git",
        "ssh://git@github.com/nabre/dev-kit",
    ] {
        assert_eq!(
            github_full_name(url).as_deref(),
            Some("nabre/dev-kit"),
            "{url}"
        );
    }
    for url in [
        "https://notgithub.com/a/b",
        "https://github.com/a",
        "https://github.com/a/b/c",
        "https://gitlab.com/a/b",
    ] {
        assert_eq!(github_full_name(url), None, "{url}");
    }
    assert_eq!(
        renamed_url("https://github.com/a/old.git", "new"),
        "https://github.com/a/new.git"
    );
    assert_eq!(
        renamed_url("git@github.com:a/old", "new"),
        "git@github.com:a/new"
    );
    let link = r#"<https://api.github.com/user/repos?page=2>; rel="next", <https://api.github.com/user/repos?page=5>; rel="last""#;
    assert_eq!(
        next_page(link).as_deref(),
        Some("https://api.github.com/user/repos?page=2")
    );
    assert_eq!(next_page(r#"<https://x>; rel="prev""#), None);
    assert_eq!(next_page(""), None);
}

#[test]
fn names() {
    assert!(valid_full_name("nabre/dev-kit"));
    for bad in [
        "nabre",
        "a/b/c",
        "../x",
        "a/-b",
        "a/b c",
        "https://github.com/a/b",
    ] {
        assert!(!valid_full_name(bad), "{bad}");
    }
    assert!(valid_repo_name("dev-kit_2.0"));
    for bad in ["", ".", "..", "a b", "a/b", "é"] {
        assert!(!valid_repo_name(bad), "{bad}");
    }
}

#[test]
fn repository_from_the_api() {
    let v = json!({
        "full_name": "nabre/dev-kit", "private": true, "fork": false,
        "description": null, "language": "Rust", "stargazers_count": 3,
        "pushed_at": "2026-10-14T08:00:00Z", "html_url": "https://github.com/nabre/dev-kit",
        "permissions": { "admin": true }
    });
    let r = repo_from(&v).unwrap();
    assert_eq!((r.owner.as_str(), r.name.as_str()), ("nabre", "dev-kit"));
    assert!(r.private && r.admin && !r.fork);
    assert_eq!(r.description, "");
    assert_eq!(r.stars, 3);
    assert!(repo_from(&json!({ "name": "x" })).is_none());
}

#[test]
fn local_clones() {
    let tmp = TempDir::new("github-clones");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let repo = |dir: &str, url: &str| {
        let git = kit.projects().join(dir).join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(
            git.join("config"),
            format!("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = {url}\n"),
        )
        .unwrap();
    };
    repo("dev-kit", "https://github.com/nabre/dev-kit.git");
    repo("other", "https://gitlab.com/nabre/other.git");
    repo("nabre-dev-kit", "https://github.com/nabre/dev-kit");
    fs::create_dir_all(kit.projects().join("plain")).unwrap();
    let c = clones(&kit);
    assert_eq!(c.len(), 1);
    assert_eq!(c.get("nabre/dev-kit").map(String::as_str), Some("dev-kit"));
}
