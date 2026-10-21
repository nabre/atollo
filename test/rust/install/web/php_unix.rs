use super::super::php::{composer_launchers, COMPOSER_UNIX};
use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;

/// The catalog pins the kit's own PHP builds (issue #4, D-133) for each
/// line it offers, with a SHA-256 for the four macOS and Linux targets.
#[test]
fn php_catalog_pins_cover_unix() {
    assert!(!php_build_pins().is_empty(), "no pinned PHP line");
    for (line, pin) in php_build_pins() {
        assert_eq!(
            crate::versions::line_of(&pin.version).as_deref(),
            Some(line.as_str())
        );
        let mut targets: Vec<&str> = pin.sha256_by_target.keys().map(String::as_str).collect();
        targets.sort_unstable();
        assert_eq!(
            targets,
            ["darwin-arm64", "darwin-x64", "linux-arm64", "linux-x64"],
            "{line}"
        );
        for t in [Target::LINUX_X64, Target::DARWIN_ARM64] {
            assert!(
                php_build_part(php_build_pins(), t, line).is_some(),
                "{line} {t}"
            );
        }
    }
}

/// D-112: static-php-cli's archives for exactly the system and the
/// architecture of each Unix target, none for Windows (windows.php.net).
#[test]
fn static_php_assets_per_target() {
    let want = [
        (Target::DARWIN_ARM64, "macos-aarch64"),
        (Target::DARWIN_X64, "macos-x86_64"),
        (Target::LINUX_X64, "linux-x86_64"),
        (Target::LINUX_ARM64, "linux-aarch64"),
    ];
    for (target, suffix) in want {
        assert_eq!(
            static_php_assets(target, "8.4.23").unwrap(),
            [
                format!("php-8.4.23-cli-{suffix}.tar.gz"),
                format!("php-8.4.23-fpm-{suffix}.tar.gz")
            ],
            "{target}"
        );
    }
    for target in [Target::WIN32_X64, Target::WIN32_ARM64] {
        assert!(static_php_assets(target, "8.4.23").is_none(), "{target}");
        assert!(static_php_os(target).is_none());
    }
}

/// D-112: the lines offered are those with both archives pinned in
/// the catalog for the target, the newest version of each; an archive of
/// another architecture never stands in.
#[test]
fn static_php_lines_from_the_pins() {
    let pins: BTreeMap<String, String> = [
        "php-8.4.20-cli-linux-x86_64.tar.gz",
        "php-8.4.20-fpm-linux-x86_64.tar.gz",
        "php-8.4.23-cli-linux-x86_64.tar.gz",
        "php-8.4.23-fpm-linux-x86_64.tar.gz",
        // No FPM archive pinned for x86_64: not offered there.
        "php-8.5.8-cli-linux-x86_64.tar.gz",
        "php-8.5.8-cli-linux-aarch64.tar.gz",
        "php-8.5.8-fpm-linux-aarch64.tar.gz",
        "README.txt",
    ]
    .into_iter()
    .map(|f| (f.to_owned(), "a".repeat(64)))
    .collect();
    assert_eq!(
        static_php_versions(&pins, Target::LINUX_X64),
        [("8.4".to_owned(), "8.4.23".to_owned())]
    );
    assert_eq!(
        static_php_versions(&pins, Target::LINUX_ARM64),
        [("8.5".to_owned(), "8.5.8".to_owned())]
    );
    assert!(static_php_versions(&pins, Target::DARWIN_ARM64).is_empty());
    assert!(static_php_versions(&pins, Target::WIN32_X64).is_empty());
    let parts = static_php_parts(&pins, Target::LINUX_X64, "8.4.23").unwrap();
    assert_eq!(parts.len(), 2);
    assert_eq!(
        parts[0].url,
        "https://dl.static-php.dev/static-php-cli/bulk/php-8.4.23-cli-linux-x86_64.tar.gz"
    );
    assert_eq!(parts[0].dest, Dest::Own);
    assert_eq!(parts[0].kind, Kind::TarGz);
    assert_eq!(parts[1].dest, Dest::Inside("php-fpm"));
    assert!(matches!(&parts[1].expect, Expect::Sha256(s) if s == &"a".repeat(64)));
    // An archive with no pin is never downloaded.
    assert!(static_php_parts(&pins, Target::LINUX_X64, "8.5.8").is_err());
}

/// The pins of this catalog: 8.4 and 8.5 on the four Unix targets, every
/// hash a SHA-256, nothing for Windows.
#[test]
fn static_php_pins_in_the_catalog() {
    let catalog: Value =
        serde_json::from_str(include_str!("../../../../tauri/catalog/components.json")).unwrap();
    let pins = static_php_pins(&catalog);
    assert_eq!(pins.len(), 16);
    for target in [
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
        Target::LINUX_X64,
        Target::LINUX_ARM64,
    ] {
        let lines: Vec<String> = static_php_versions(&pins, target)
            .into_iter()
            .map(|(l, _)| l)
            .collect();
        assert_eq!(lines, ["8.5", "8.4"], "{target}");
    }
    assert!(static_php_versions(&pins, Target::WIN32_X64).is_empty());
    assert_eq!(&pins, catalog_pins());
    // A bad value is not a pin.
    let bad =
        json!({"components": [{"id": "php", "sha256ByFile": {"a.tar.gz": "xyz", "b.tar.gz": 1}}]});
    assert!(static_php_pins(&bad).is_empty());
}

/// An archive whose hash is not the pinned one is refused (and deleted).
#[test]
fn static_php_other_hash_refused() {
    let tmp = TempDir::new("static-php-hash");
    let file = tmp.0.join("php-8.4.23-cli-linux-x86_64.tar.gz");
    fs::write(&file, b"not the pinned archive").unwrap();
    let pinned = Expect::Sha256(
        "5fa2b5f1cc9d7f79b19718926b4f4f0bb6949db52073b2fabf8ae52de3993af5".to_owned(),
    );
    let e = crate::net::verify::check(&file, &pinned)
        .unwrap_err()
        .to_string();
    assert!(e.contains("checksum does not match"), "{e}");
    assert!(!file.exists());
}

/// D-112: `php` of the CLI archive goes to `bin/php`, `php-fpm` of the FPM
/// one to `sbin/php-fpm`; Composer's launcher in `bin/`, on the PATH there.
#[test]
fn static_php_layout_and_composer() {
    let tmp = TempDir::new("static-php-layout");
    let tree = tmp.0.join("tree");
    fs::create_dir_all(&tree).unwrap();
    assert!(static_php_layout(&tree).is_err(), "no php in the archive");
    fs::write(tree.join("php"), b"elf").unwrap();
    static_php_layout(&tree).unwrap();
    assert!(tree.join("bin").join("php").is_file() && !tree.join("php").exists());
    // Again (an update of the same tree): nothing to move, still fine.
    static_php_layout(&tree).unwrap();

    fs::write(tree.join("php-fpm"), b"elf").unwrap();
    static_php_fpm_in_place(&tree).unwrap();
    assert!(tree.join("sbin").join("php-fpm").is_file() && !tree.join("php-fpm").exists());
    static_php_fpm_in_place(&tree).unwrap();

    composer_launchers(&tree).unwrap();
    let sh = fs::read(tree.join("bin").join("composer")).unwrap();
    assert_eq!(sh, COMPOSER_UNIX);
    let text = String::from_utf8(sh).unwrap();
    assert!(text.starts_with("#!/bin/sh\n") && !text.contains('\r'));
    assert!(text.contains("\"$d/php\" \"$d/../composer.phar\""));
    // A folder of windows.php.net gets no bin/composer.
    let win = tmp.0.join("win");
    fs::create_dir_all(&win).unwrap();
    composer_launchers(&win).unwrap();
    assert!(!win.join("bin").exists());
}

/// The kit's PHP builds (issue #4): a pinned line comes from the release of
/// this repository, one tarball with `bin/php` and `sbin/php-fpm` as the
/// folder; never on Windows, never without a valid SHA-256.
#[test]
fn php_kit_builds_pinned() {
    let sha = "cd".repeat(32);
    let catalog = json!({ "components": [{ "id": "php", "sha256ByLine": {
        "8.4": { "version": "8.4.26-1", "sha256ByTarget": {
            "linux-x64": sha, "darwin-arm64": sha.to_uppercase(), "linux-arm64": "nothex" } }
    } }] });
    let pins = line_pins_in(&catalog, "php");
    let (version, part) = php_build_part(&pins, Target::LINUX_X64, "8.4").unwrap();
    assert_eq!(version, "8.4.26-1");
    assert_eq!(part.file_name, "php-8.4.26-1-linux-x64.tar.gz");
    assert_eq!(
        part.url,
        "https://github.com/nabre/atollo/releases/download/php-v8.4.26-1/php-8.4.26-1-linux-x64.tar.gz"
    );
    assert_eq!(part.expect, Expect::Sha256(sha.clone()));
    assert!(matches!(part.kind, Kind::TarGz));
    assert!(matches!(part.dest, Dest::Own));
    let (_, upper) = php_build_part(&pins, Target::DARWIN_ARM64, "8.4").unwrap();
    assert_eq!(upper.expect, Expect::Sha256(sha.clone()), "lower case");
    assert!(
        php_build_part(&pins, Target::LINUX_ARM64, "8.4").is_none(),
        "not a SHA-256"
    );
    assert!(
        php_build_part(&pins, Target::DARWIN_X64, "8.4").is_none(),
        "no pin"
    );
    assert!(
        php_build_part(&pins, Target::LINUX_X64, "8.5").is_none(),
        "no line"
    );
    assert!(
        php_build_part(&pins, Target::WIN32_X64, "8.4").is_none(),
        "windows.php.net"
    );
}

/// The lines offered on macOS and Linux: static-php-cli's, with the kit's
/// build in place of a line it pins and its own lines added.
#[test]
fn unix_php_lines_prefer_the_kit_build() {
    let sha = "ef".repeat(32);
    let builds = line_pins_in(
        &json!({ "components": [{ "id": "php", "sha256ByLine": {
            "8.4": { "version": "8.4.26-1", "sha256ByTarget": { "linux-x64": sha } },
            "8.6": { "version": "8.6.0-2", "sha256ByTarget": { "linux-x64": sha } }
        } }] }),
        "php",
    );
    let bulk: BTreeMap<String, String> = [
        "php-8.4.23-cli-linux-x86_64.tar.gz",
        "php-8.4.23-fpm-linux-x86_64.tar.gz",
        "php-8.5.8-cli-linux-x86_64.tar.gz",
        "php-8.5.8-fpm-linux-x86_64.tar.gz",
    ]
    .into_iter()
    .map(|f| (f.to_owned(), sha.clone()))
    .collect();
    assert_eq!(
        unix_php_versions(&builds, &bulk, Target::LINUX_X64),
        vec![
            ("8.6".to_owned(), "8.6.0-2".to_owned()),
            ("8.5".to_owned(), "8.5.8".to_owned()),
            ("8.4".to_owned(), "8.4.26-1".to_owned()),
        ]
    );
    // Where the kit pins nothing, static-php-cli's alone.
    assert_eq!(
        unix_php_versions(&BTreeMap::new(), &bulk, Target::LINUX_X64),
        static_php_versions(&bulk, Target::LINUX_X64)
    );
}
