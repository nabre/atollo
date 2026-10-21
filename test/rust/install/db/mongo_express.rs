use super::*;
use crate::setup::catalog::Catalog;

/// The kit's build: the release of this repository named after the
/// version, the zip with the tree at its root.
#[test]
fn release_of_the_kit_build() {
    let sha = "AB".repeat(32);
    let r = mongo_express_release("1.1.0-rc.4-1", &sha).unwrap();
    assert_eq!(r.version, "1.1.0-rc.4-1");
    let part = &r.parts[0];
    assert_eq!(
        part.url,
        "https://github.com/nabre/atollo/releases/download/mongo-express-v1.1.0-rc.4-1/mongo-express-1.1.0-rc.4-1.zip"
    );
    assert_eq!(part.file_name, "mongo-express-1.1.0-rc.4-1.zip");
    assert_eq!(part.expect, Expect::Sha256("ab".repeat(32)));
    assert_eq!(part.kind, Kind::Zip);
    assert_eq!(part.inner, Inner::Root);
}

/// Never a download that cannot be checked, never a version that is not
/// one of the kit's builds in the address.
#[test]
fn refuses_what_cannot_be_checked() {
    for sha in ["", "abc", &"g".repeat(64), &"a".repeat(63)] {
        assert!(
            mongo_express_release("1.1.0-rc.4-1", sha).is_err(),
            "{sha:?}"
        );
    }
    for good in ["1.1.0-rc.4-1", "1.2.0-1", "2.0.10-12"] {
        assert!(valid_version(good), "{good}");
    }
    for bad in [
        "",
        "1.1.0",
        "1.1.0-rc-4-1",
        "1.1.0-rc.4",
        "1.1.0-rc.4-",
        "1.1-1",
        "../1.1.0-1",
        "1.1.0-1/../x",
        "1.1.0-beta.1-1",
    ] {
        assert!(!valid_version(bad), "{bad}");
        assert!(mongo_express_release(bad, &"a".repeat(64)).is_err(), "{bad}");
    }
}

/// The catalog entry: a database tool on MongoDB and the kit's Node.js,
/// available where MongoDB is, at a version of the kit's build, checked by its
/// package.json; its SHA-256, once pinned, is one.
#[test]
fn catalog_entry() {
    let catalog = Catalog::get();
    let c = catalog.find("mongo-express").expect("mongo-express in the catalog");
    let mongodb = catalog.find("mongodb").unwrap();
    assert_eq!(c.group, crate::setup::catalog::Group::Databases);
    assert!(!c.def);
    for need in ["node", "mongodb"] {
        assert!(c.needs_on_every_target().any(|n| n == need), "{need}");
    }
    // Pure JavaScript on the kit's Node.js: native everywhere, available
    // wherever MongoDB is.
    assert!(!c.x64_only);
    for target in crate::kit::Target::ALL {
        assert_eq!(
            c.is_available_on(target),
            mongodb.is_available_on(target),
            "{target}"
        );
    }
    assert!(c.minor_folder && !c.versioned);
    assert_eq!(c.key, "mongo-express/app.js");
    assert!(c.keys.iter().any(|k| k == "mongo-express/build-assets.json"));
    let version = c.version.as_deref().expect("a version fixed");
    assert!(valid_version(version), "{version}");
    if let Some(sha) = c.sha256.as_deref() {
        assert!(mongo_express_release(version, sha).is_ok(), "{sha}");
    }
    assert_eq!(
        c.check,
        [
            "mongo-express/package.json".to_owned(),
            crate::setup::checkup::PACKAGE_JSON.to_owned()
        ]
    );
    assert!(crate::install::installer("mongo-express").is_some());
}
