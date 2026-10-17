use super::*;
use crate::kit::tests::TempDir;
use crate::os::safe_path::tests::junction;
use std::fs;

#[test]
fn targets_parsed() {
    assert_eq!(
        Target::parse("storage/uploads"),
        Some(Target::Project(vec!["storage".into(), "uploads".into()]))
    );
    assert_eq!(Target::parse("."), Some(Target::Project(vec![])));
    assert_eq!(Target::parse(r"public\img"), Target::parse("public/img"));
    assert_eq!(
        Target::parse("@common/assets"),
        Some(Target::Other {
            project: "common".into(),
            parts: vec!["assets".into()]
        })
    );
    for bad in [
        "",
        "../x",
        "../../Windows",
        "a/../b",
        "C:/x",
        r"\\srv\x",
        "/x",
        "storage/",
        ".git",
        "@../x",
        "@common/../x",
        "@con/x",
        "@",
        "@/x",
        "a\"b",
        "a\nb",
        "a\rb",
        "${X}",
        "a;b",
        "a{b",
        "a b",
        "a$b",
    ] {
        assert_eq!(Target::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn urls_and_domains() {
    assert!(valid_url("/media") && valid_url("/a/b.c"));
    for bad in [
        "", "/", "media", "/a/", "/a/../b", "/a\"b", "/a b", "/a\nb", "/a{b",
    ] {
        assert!(!valid_url(bad), "{bad:?}");
    }
    assert!(valid_domain("mio-sito.localhost") && valid_domain("a.b.localhost"));
    for bad in [
        "localhost",
        ".localhost",
        "a.com",
        "A.localhost",
        "a\".localhost",
        "a b.localhost",
    ] {
        assert!(!valid_domain(bad), "{bad:?}");
    }
}

#[test]
fn resolved_only_inside_the_projects_without_links() {
    let tmp = TempDir::new("site-alias");
    let projects = tmp.0.join("projects");
    let shop = projects.join("shop");
    fs::create_dir_all(shop.join("storage").join("uploads")).unwrap();
    fs::create_dir_all(projects.join("common").join("assets")).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    junction(&shop.join("leak"), &outside);

    let mut aliases = BTreeMap::new();
    aliases.insert("/media".to_owned(), "storage/uploads".to_owned());
    aliases.insert("/shared".to_owned(), "@common/assets".to_owned());
    aliases.insert("/root".to_owned(), ".".to_owned());
    aliases.insert("/leak".to_owned(), "leak".to_owned());
    aliases.insert("/missing".to_owned(), "nope".to_owned());
    aliases.insert("/ghost".to_owned(), "@ghost/x".to_owned());
    aliases.insert("/up".to_owned(), "../common/assets".to_owned());
    aliases.insert("bad".to_owned(), "storage".to_owned());

    let found = resolve(&shop, &projects, &aliases);
    let urls: Vec<&str> = found.iter().map(|a| a.url.as_str()).collect();
    assert_eq!(urls, ["/media", "/root", "/shared"]);
    assert_eq!(
        skipped(&shop, &projects, &aliases),
        ["/ghost", "/leak", "/missing", "/up", "bad"]
    );
    let by_url = |u: &str| found.iter().find(|a| a.url == u).unwrap();
    assert_eq!(by_url("/media").dir, shop.join("storage").join("uploads"));
    assert_eq!(
        by_url("/media").apache_path("${CP_PROJECT_FWD}"),
        "${CP_PROJECT_FWD}/storage/uploads"
    );
    assert_eq!(
        by_url("/shared").apache_path("${CP_PROJECT_FWD}"),
        "${CP_PROJECTS_FWD}/common/assets"
    );
    assert_eq!(
        by_url("/root").apache_path("${CP_PROJECT_FWD}"),
        "${CP_PROJECT_FWD}"
    );

    // a project that is itself a junction is not followed either
    junction(&projects.join("linked"), &outside);
    let mut via = BTreeMap::new();
    via.insert("/x".to_owned(), "@linked/.".to_owned());
    assert!(resolve(&shop, &projects, &via).is_empty());
}

/// Nginx for Windows follows a link inside an alias' folder (no
/// `disable_symlinks`): such an alias is left out of its configuration,
/// found at any depth within the limits; a folder past them is not trusted.
#[cfg(windows)]
#[test]
fn nginx_skips_aliases_with_links_inside() {
    let tmp = TempDir::new("site-alias-inside");
    let projects = tmp.0.join("projects");
    let shop = projects.join("shop");
    let clean = shop.join("clean").join("a").join("b");
    fs::create_dir_all(&clean).unwrap();
    fs::write(clean.join("x.txt"), "x").unwrap();
    let month = shop.join("dirty").join("2026").join("10");
    fs::create_dir_all(&month).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    junction(&month.join("leak"), &outside);
    let mut deep = shop.join("deep");
    for i in 0..=INSIDE_DEPTH {
        deep.push(format!("d{i}"));
    }
    fs::create_dir_all(&deep).unwrap();

    assert_eq!(look_inside(&shop.join("clean")), Inside::Clean);
    assert_eq!(
        look_inside(&shop.join("dirty")),
        Inside::Link(month.join("leak"))
    );
    assert_eq!(look_inside(&shop.join("deep")), Inside::TooLarge);

    let mut aliases = BTreeMap::new();
    aliases.insert("/clean".to_owned(), "clean".to_owned());
    aliases.insert("/dirty".to_owned(), "dirty".to_owned());
    aliases.insert("/deep".to_owned(), "deep".to_owned());
    aliases.insert("/missing".to_owned(), "nope".to_owned());
    // Apache does not follow them (no FollowSymLinks): all three served.
    assert_eq!(resolve(&shop, &projects, &aliases).len(), 3);
    let urls: Vec<String> = resolve_nginx(&shop, &projects, &aliases)
        .into_iter()
        .map(|a| a.url)
        .collect();
    assert_eq!(urls, ["/clean"]);
    assert_eq!(
        skipped_nginx(&shop, &projects, &aliases),
        ["/deep", "/dirty", "/missing"]
    );
}
