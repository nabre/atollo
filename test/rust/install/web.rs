use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;

#[test]
fn php_ini_from_template() {
    let template = ";extension_dir = \"ext\"\r\n;extension=curl\r\n;extension=ldap\r\n;date.timezone =\r\n;curl.cainfo =\r\n;openssl.cafile=\r\n";
    let ini = php_ini(template).unwrap();
    assert!(
        ini.contains("extension_dir = \"${CP_PHP}\\ext\"\r\n"),
        "{ini}"
    );
    assert!(ini.contains("\r\nextension=curl\r\n"));
    assert!(
        ini.contains(";extension=ldap"),
        "not in the list: stays off"
    );
    assert!(ini.contains("date.timezone = Europe/Rome"));
    assert!(ini.contains("curl.cainfo = \"${CP_PHP}\\extras\\ssl\\cacert.pem\""));
    assert!(ini.contains("openssl.cafile = \"${CP_PHP}\\extras\\ssl\\cacert.pem\""));
    // The php.ini of an older kit follows the PHP in use too.
    assert_eq!(
        php_ini_paths("extension_dir = \"${CP_APPS}\\php\\ext\"\r\n"),
        "extension_dir = \"${CP_PHP}\\ext\"\r\n"
    );
}

/// PHP before 7.2 names its extensions `php_curl.dll`, before 8.0 GD is
/// `gd2`: each line's template gets the same extensions on.
#[test]
fn php_ini_from_older_templates() {
    let ini = php_ini(";extension=php_curl.dll\r\n;extension=php_gd2.dll\r\n;extension=php_ldap.dll\r\n;extension=gd2\r\n").unwrap();
    assert_eq!(
        ini,
        "extension=php_curl.dll\r\nextension=php_gd2.dll\r\n;extension=php_ldap.dll\r\nextension=gd2\r\n"
    );
}

/// A PHP folder of `kit` (its key for the kit's target: `php.exe` on
/// Windows, `bin/php` elsewhere) with its `php.ini-development`.
fn fake_php(kit: &crate::kit::Kit, dir: &Path, template: &str) {
    let php = crate::setup::catalog::Catalog::get().find("php").unwrap();
    let key = php.key_for(kit.target());
    let file = crate::fsx::join_rel(dir, key.strip_prefix("php/").unwrap());
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, b"").unwrap();
    fs::write(dir.join("php.ini-development"), template).unwrap();
}

/// Defect 10: one php.ini per line of PHP, from that line's template; the
/// single php.ini of an older kit becomes the default line's, with the
/// user's changes. A Windows kit on every host: that php.ini only existed
/// there (the Unix layout: `php_ini_shared_kit_both_orders`).
#[test]
fn one_php_ini_per_line() {
    use crate::kit::{Kit, Target};
    use crate::setup::catalog::Catalog;

    let tmp = TempDir::new("php-ini-lines");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    let data = kit.data();
    let php = Catalog::get().find("php").unwrap();
    let version = "8.4.13 (Composer 2.8.12)";
    let main = crate::versions::install_dir(&kit, php, version);
    fake_php(&kit, &main, ";extension=curl\r\n");
    php.save_version(&kit, version).unwrap();
    let old = crate::versions::saved_dir(&kit, php, "5.6");
    fake_php(&kit, &old, ";extension=php_curl.dll\r\n");
    crate::versions::write_version(&old, "5.6.40").unwrap();
    let legacy = data.join("php").join("php.ini");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "memory_limit = 1G\r\n").unwrap();

    assert_eq!(php_ini_dir(&data, &main), data.join("php").join("8.4"));
    assert_eq!(php_ini_dir(&data, &old), data.join("php").join("5.6"));
    assert_eq!(
        php_ini_dir(&data, &kit.apps().join("php")),
        data.join("php"),
        "an older kit's layout"
    );

    migrate_php_ini(&kit).unwrap();
    let main_ini = data.join("php").join("8.4").join("php.ini");
    let old_ini = data.join("php").join("5.6").join("php.ini");
    assert_eq!(
        fs::read_to_string(&main_ini).unwrap(),
        "memory_limit = 1G\r\n"
    );
    assert!(!legacy.exists());
    assert_eq!(
        fs::read_to_string(&old_ini).unwrap(),
        "extension=php_curl.dll\r\n"
    );

    // Then they are the user's: never written again.
    fs::write(&old_ini, "mine").unwrap();
    migrate_php_ini(&kit).unwrap();
    ensure_php_ini(&data, &old).unwrap();
    assert_eq!(fs::read_to_string(&old_ini).unwrap(), "mine");
    assert_eq!(
        fs::read_to_string(&main_ini).unwrap(),
        "memory_limit = 1G\r\n"
    );

    // The environment points at the php.ini of the PHP it names.
    let mut env = crate::env::KitEnv::default();
    env.set_php(&data, &old);
    let ini_dir = data.join("php").join("5.6");
    assert_eq!(env.get("PHPRC"), Some(ini_dir.to_str().unwrap()));
    assert_eq!(
        env.get("CP_PHPRC_FWD").unwrap(),
        ini_dir.to_str().unwrap().replace('\\', "/")
    );
}

/// The folder of the php.ini for each of the six targets: Windows keeps
/// `data/php/<line>`, macOS and Linux get `data/php/<target>/<line>`.
#[test]
fn php_ini_dir_per_target() {
    use crate::kit::{Kit, Target};
    let tmp = TempDir::new("php-ini-targets");
    for target in [
        Target::WIN32_X64,
        Target::WIN32_ARM64,
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
        Target::LINUX_X64,
        Target::LINUX_ARM64,
    ] {
        let kit = Kit::at_target(tmp.0.join("kit"), target);
        let data = kit.data();
        let dir = kit.apps().join("php").join("8.5");
        let want = if target.os == crate::kit::Os::Win32 {
            data.join("php").join("8.5")
        } else {
            data.join("php").join(target.as_str()).join("8.5")
        };
        assert_eq!(php_ini_dir(&data, &dir), want, "{}", target.as_str());
    }
    // A kit folder named like a target does not count: only below data/.
    let odd = Kit::at_target(tmp.0.join("linux-x64"), Target::WIN32_X64);
    let dir = odd.apps().join("php").join("8.4");
    assert_eq!(
        php_ini_dir(&odd.data(), &dir),
        odd.data().join("php").join("8.4")
    );
}

/// Review of D-112: one kit with PHP 8.5 for Windows and for Linux gets two
/// php.ini, in either order of install: Windows with its extension= lines,
/// Linux with none. The single php.ini of an older kit stays Windows'.
#[test]
fn php_ini_shared_kit_both_orders() {
    use crate::kit::{Kit, Target};
    for windows_first in [true, false] {
        let tmp = TempDir::new("php-ini-shared");
        let win = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
        let linux = win.for_target(Target::LINUX_X64);
        win.prepare().unwrap();
        let data = win.data();
        let win_dir = win.apps().join("php").join("8.5");
        let linux_dir = linux.apps().join("php").join("8.5");
        fake_php(&win, &win_dir, ";extension=curl\r\n");
        fake_php(&linux, &linux_dir, "");
        fs::remove_file(linux_dir.join("php.ini-development")).unwrap();
        let order = if windows_first {
            [&win_dir, &linux_dir]
        } else {
            [&linux_dir, &win_dir]
        };
        for dir in order {
            ensure_php_ini(&data, dir).unwrap();
        }
        let win_ini = fs::read_to_string(data.join("php").join("8.5").join("php.ini")).unwrap();
        let linux_ini = fs::read_to_string(
            data.join("php")
                .join("linux-x64")
                .join("8.5")
                .join("php.ini"),
        )
        .unwrap();
        assert_eq!(
            win_ini, "extension=curl\r\n",
            "windows_first={windows_first}"
        );
        assert_eq!(linux_ini, php_ini_unix(), "windows_first={windows_first}");
        assert!(!linux_ini
            .lines()
            .any(|l| l.trim_start().starts_with("extension")));

        // PHPRC of each system names its own folder.
        let mut env = crate::env::KitEnv::default();
        env.set_php(&data, &linux_dir);
        let want = data.join("php").join("linux-x64").join("8.5");
        assert_eq!(env.get("PHPRC"), Some(want.to_str().unwrap()));
    }

    // An older kit's data/php/php.ini never becomes a Unix PHP's.
    let tmp = TempDir::new("php-ini-legacy-unix");
    let linux = Kit::at_target(tmp.0.clone(), Target::LINUX_X64);
    linux.prepare().unwrap();
    let php = crate::setup::catalog::Catalog::get().find("php").unwrap();
    let version = "8.5.8 (Composer 2.8.12)";
    let main = crate::versions::install_dir(&linux, php, version);
    fake_php(&linux, &main, "");
    fs::remove_file(main.join("php.ini-development")).unwrap();
    php.save_version(&linux, version).unwrap();
    let legacy = linux.data().join("php").join("php.ini");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "extension=curl\r\n").unwrap();
    migrate_php_ini(&linux).unwrap();
    assert_eq!(fs::read_to_string(&legacy).unwrap(), "extension=curl\r\n");
    let ini = php_ini_dir(&linux.data(), &main).join("php.ini");
    assert_eq!(fs::read_to_string(ini).unwrap(), php_ini_unix());
}

/// Apache loads the php.ini of the PHP in use, also in an older kit.conf.
#[test]
fn apache_php_ini_follows_the_line() {
    let blocks = crate::projects::apache::php_module_blocks("CP_PHP_FWD").join("\n");
    assert!(blocks.contains("PHPIniDir \"${CP_PHPRC_FWD}\""));
    assert!(!blocks.contains("/data/php\""));
    assert_eq!(
        crate::projects::apache::migrate_kit_conf("    PHPIniDir \"${CP_ROOT_FWD}/data/php\"\r\n")
            .lines()
            .next(),
        Some("    PHPIniDir \"${CP_PHPRC_FWD}\"")
    );
}

#[test]
fn php_branch_and_build() {
    let releases = json!({
        "8.3": {"version": "8.3.20", "ts-vs16-x64": {"zip": {"path": "php-8.3.20-Win32-vs16-x64.zip", "sha256": "aa"}}},
        "8.4": {"version": "8.4.9", "nts-vs17-x64": {"zip": {"path": "nts.zip", "sha256": "bb"}}, "ts-vs17-x64": {"zip": {"path": "php-8.4.9-Win32-vs17-x64.zip", "sha256": "cc"}}},
    });
    assert_eq!(
        pick_php(&releases, None).unwrap(),
        (
            "8.4.9".into(),
            "php-8.4.9-Win32-vs17-x64.zip".into(),
            "cc".into()
        )
    );
    assert_eq!(pick_php(&releases, Some("8.3")).unwrap().0, "8.3.20");
    assert!(pick_php(&releases, Some("7.4")).is_err());
    assert_eq!(php_lines(&releases), ["8.4", "8.3"]);

    // PHP 7 and 5.6 builds are supported alongside PHP 8.
    let mut with_7 = releases.clone();
    with_7["7.4"] = json!({"version": "7.4.33", "ts-vc15-x64": {"zip": {"path": "php-7.4.33.zip", "sha256": "dd"}}});
    assert_eq!(php_lines(&with_7), ["8.4", "8.3", "7.4"]);
}

#[test]
fn composer_for_each_php() {
    let versions = json!({
        "stable": [
            {"path": "/download/2.10.3/composer.phar", "version": "2.10.3", "min-php": 70205}
        ],
        "preview": [{"path": "/download/2.11.0-RC1/composer.phar", "version": "2.11.0-RC1", "min-php": 70205}],
        "2.2": [{"path": "/download/2.2.30/composer.phar", "version": "2.2.30", "min-php": 50300}]
    });
    let stable = (
        "2.10.3".to_owned(),
        "https://getcomposer.org/download/2.10.3/composer.phar".to_owned(),
    );
    assert_eq!(pick_composer(&versions, "8.4.13").unwrap(), stable);
    assert_eq!(pick_composer(&versions, "7.2.5").unwrap(), stable);
    assert_eq!(
        pick_composer(&versions, "7.1.33").unwrap().0,
        "2.2.30",
        "older PHP: the LTS"
    );
    assert!(pick_composer(&json!({"stable": []}), "8.4.1").is_err());
    assert!(pick_composer(
        &json!({"stable": [{"path": "https://x/composer.phar", "version": "1"}]}),
        "8.4.1"
    )
    .is_err());

    let tmp = crate::kit::tests::TempDir::new("composer-launchers");
    composer_launchers(&tmp.0).unwrap();
    let cmd = fs::read_to_string(tmp.0.join("composer.cmd")).unwrap();
    assert!(
        cmd.contains(r#""%~dp0php.exe" "%~dp0composer.phar""#),
        "{cmd}"
    );
}

#[test]
fn httpd_conf_changes() {
    let text = "Define SRVROOT \"c:/Apache24\"\r\nListen 80\r\n#LoadModule rewrite_module modules/mod_rewrite.so\r\n";
    let out = httpd_conf(text).unwrap();
    assert!(out.starts_with("Define SRVROOT \"${CP_APACHE_FWD}\"\r\n"));
    assert!(out.contains("#Listen 80"));
    assert!(out.contains("\r\nLoadModule rewrite_module"));
    assert!(out.ends_with("Include \"${CP_ROOT_FWD}/data/apache/kit.conf\"\r\n"));
    assert!(httpd_conf("nothing").is_err());
}

#[test]
fn apache_site_created_once() {
    let tmp = TempDir::new("apachesite");
    apache_site(&tmp.0, Os::Win32).unwrap();
    let conf = tmp.0.join("kit.conf");
    assert!(fs::read_to_string(&conf)
        .unwrap()
        .contains("Listen 127.0.0.1:${PORTA}"));
    fs::write(&conf, "mine").unwrap();
    apache_site(&tmp.0, Os::Win32).unwrap();
    assert_eq!(fs::read_to_string(&conf).unwrap(), "mine");
    assert!(tmp.0.join("htdocs").join("index.php").is_file());
}

#[test]
fn apache_site_refuses_php_without_module() {
    let tmp = TempDir::new("apachesitephp");
    apache_site(&tmp.0, Os::Win32).unwrap();
    let text = fs::read_to_string(tmp.0.join("kit.conf")).unwrap();
    assert!(!text.contains("<FilesMatch \"\\.php$\">"));
    assert_eq!(
        text.matches("<FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">")
            .count(),
        4
    );
    // SEC-V3-01: `.inc` and `.php3`…`.php8` never served.
    assert!(text.contains(
        "<FilesMatch \"(?i)\\.(?:inc|php[3-8])$\">\r\n    Require all denied\r\n</FilesMatch>"
    ));
    assert_eq!(
        crate::projects::apache::migrate_kit_conf(&text),
        text,
        "nothing to migrate"
    );
    assert_eq!(text.matches("LoadModule php").count(), 3);
    let deny = text.find("<IfModule !php_module>").unwrap();
    assert!(deny > text.rfind("LoadModule php5_module").unwrap());
    assert!(text[deny..].contains("<IfModule !php7_module>"));
    assert!(text[deny..].contains("<IfModule !php5_module>"));
    assert!(text[deny..].contains("Require all denied"));
    assert!(text.trim_end().ends_with("conf.d/*.conf\""));
    assert!(text.contains("\r\n"));
}

/// SEC-V3-02: the kit's site refuses hidden files (`.env`, `.git/`) but
/// `.well-known`, like the projects' servers.
#[test]
fn apache_site_refuses_hidden_files() {
    let tmp = TempDir::new("apachesitehidden");
    apache_site(&tmp.0, Os::Win32).unwrap();
    let text = fs::read_to_string(tmp.0.join("kit.conf")).unwrap();
    let hidden = crate::projects::apache::hidden_deny("").join("\r\n");
    let at = text.find(&hidden).unwrap_or_else(|| panic!("{text}"));
    assert!(at > text.find("Require all granted").unwrap());
}

/// D-116: the kit's build ships an LF `httpd.conf` in Apache Lounge's
/// shape; the same rewrite keeps its line endings.
#[test]
fn httpd_conf_of_the_kit_build() {
    let text = "Define SRVROOT \"/opt/atollo-httpd\"\nServerRoot \"${SRVROOT}\"\nListen 80\n#LoadModule rewrite_module modules/mod_rewrite.so\n";
    let out = httpd_conf(text).unwrap();
    assert!(!out.contains('\r'), "{out:?}");
    assert!(out.starts_with("Define SRVROOT \"${CP_APACHE_FWD}\"\nServerRoot \"${SRVROOT}\"\n"));
    assert!(out.contains("#Listen 80   (port in data/apache/kit.conf)\n"));
    assert!(out.contains("\nLoadModule rewrite_module"));
    assert!(out.ends_with("\n# --- Atollo ---\nInclude \"${CP_ROOT_FWD}/data/apache/kit.conf\"\n"));
}

/// D-116: off Windows the hand-started site has no PHP (php-fpm is the
/// kit's pools' and sites'): PHP files refused, an HTML test page, LF.
#[test]
fn apache_site_on_unix_refuses_php() {
    for os in [Os::Darwin, Os::Linux] {
        let tmp = TempDir::new("apachesiteunix");
        apache_site(&tmp.0, os).unwrap();
        let text = fs::read_to_string(tmp.0.join("kit.conf")).unwrap();
        assert!(!text.contains('\r'));
        assert!(!text.contains(".dll"), "{text}");
        assert!(!text.contains("data\\"), "no Windows path: {text}");
        assert!(text.contains("<IfModule !php_module>"));
        assert!(text.contains(
            "<FilesMatch \"(?i)\\.(?:inc|php[3-8])$\">\n    Require all denied\n</FilesMatch>"
        ));
        assert!(text.contains(&crate::projects::apache::hidden_deny("").join("\n")));
        assert!(text.trim_end().ends_with("conf.d/*.conf\""));
        assert!(tmp.0.join("htdocs").join("index.html").is_file());
        assert!(!tmp.0.join("htdocs").join("index.php").exists());
    }
}

/// D-116: the kit's httpd for macOS and Linux, only the pinned lines and
/// targets, the release address and the tree at the archive's root.
#[test]
fn httpd_pinned_releases() {
    let sha = "ab".repeat(32);
    let catalog = json!({ "components": [{ "id": "apache", "sha256ByLine": {
        "2.4": { "version": "2.4.69-1", "sha256ByTarget": {
            "darwin-arm64": sha, "darwin-x64": sha, "linux-x64": sha, "linux-arm64": "nothex" } },
        "2.6": { "version": "2.4.69-1", "sha256ByTarget": { "linux-x64": sha } }
    } }] });
    let pins = line_pins_in(&catalog, "apache");
    assert_eq!(pins.len(), 2);
    assert!(line_pins_in(&catalog, "mysql").is_empty());
    // A line whose version is of another line is never offered.
    assert_eq!(
        httpd_lines_for(&pins, Target::LINUX_X64),
        vec!["2.4".to_owned()]
    );
    let r = httpd_release_for(&pins, Target::DARWIN_ARM64, "2.4").unwrap();
    assert_eq!(r.version, "2.4.69-1");
    let part = &r.parts[0];
    assert_eq!(part.file_name, "httpd-2.4.69-1-darwin-arm64.tar.gz");
    assert_eq!(
        part.url,
        "https://github.com/nabre/atollo/releases/download/httpd-v2.4.69-1/httpd-2.4.69-1-darwin-arm64.tar.gz"
    );
    assert_eq!(part.expect, Expect::Sha256(sha.clone()));
    assert!(matches!(part.kind, Kind::TarGz));
    assert!(matches!(part.inner, Inner::Root));
    assert!(
        httpd_release_for(&pins, Target::LINUX_ARM64, "2.4").is_err(),
        "not a SHA-256"
    );
    assert!(httpd_release_for(&pins, Target::LINUX_X64, "2.2").is_err());
    assert!(
        httpd_release_for(&pins, Target::WIN32_X64, "2.4").is_err(),
        "Windows: Apache Lounge"
    );
}

/// The catalog's own pins name real lines of the kit's build, each with a
/// SHA-256 for the four macOS and Linux targets (none for Windows).
#[test]
fn httpd_catalog_pins_cover_unix() {
    for (line, pin) in httpd_pins() {
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
    }
}

#[test]
fn apache_builds_of_the_pages() {
    let page = r#"<a href="/download/VS18/binaries/httpd-2.4.69-261002-Win64-VS18.zip">x</a>
        <a href="/download/VS18/binaries/httpd-2.4.69-261002-win32-vs18.zip">32</a>
        <a href="/download/VS18/binaries/httpd-2.4.69-261002-Win64-VS18.zip.txt">sums</a>
        <a href="/download/VC15/binaries/httpd-2.4.54-win64-VC15.zip">old</a>
        <a href="/download/VS18/binaries/httpd-2.4.69-261002-Win64-VS18.zip">again</a>"#;
    let builds = apache_builds_in(page);
    assert_eq!(builds.len(), 2);
    assert_eq!(builds[0].version, "2.4.69-261002");
    assert_eq!(builds[0].file, "httpd-2.4.69-261002-Win64-VS18.zip");
    assert_eq!(
        builds[0].path,
        "/download/VS18/binaries/httpd-2.4.69-261002-Win64-VS18.zip"
    );
    assert_eq!(builds[1].version, "2.4.54");
}

#[test]
fn cab_headers() {
    let mut bytes = vec![0u8; 10];
    let mut cab = b"MSCF\0\0\0\0".to_vec();
    cab.extend_from_slice(&100u32.to_le_bytes());
    cab.resize(100, 0);
    cab[24] = 3;
    cab[25] = 1;
    bytes.extend_from_slice(&cab);
    bytes.extend_from_slice(b"MSCF\0\0\0\0junk");
    assert_eq!(find_cabs(&bytes), vec![(10, 100)]);
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
        serde_json::from_str(include_str!("../../../tauri/catalog/components.json")).unwrap();
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

/// D-112: the Unix `php.ini`, no `extension=` line (built-in extensions),
/// the CA bundle with `/`, the time zone of the Windows one; written once
/// for a folder of static-php-cli (no template), then the user's.
#[test]
fn php_ini_for_unix() {
    let ini = php_ini_unix();
    assert!(
        !ini.lines().any(|l| l.trim_start().starts_with("extension")),
        "{ini}"
    );
    assert!(!ini.contains('\\') && !ini.contains('\r'));
    assert!(ini.contains("date.timezone = Europe/Rome\n"));
    assert!(ini.contains("curl.cainfo = \"${CP_PHP}/extras/ssl/cacert.pem\"\n"));
    assert!(ini.contains("openssl.cafile = \"${CP_PHP}/extras/ssl/cacert.pem\"\n"));
    assert!(ini.contains("display_errors = On\n") && ini.contains("error_reporting = E_ALL\n"));
    // The known gap of the bulk build is said where the user looks
    // (review of D-112): SQLite through PDO only, no ext-pdo_sqlite.
    assert!(ini.contains("php -m does not list pdo_sqlite") && ini.contains("ext-pdo_sqlite"));
    assert!(ini
        .lines()
        .filter(|l| l.contains("pdo_sqlite"))
        .all(|l| l.starts_with(';')));
    // The Windows one keeps its `\` and its extensions.
    let win = php_ini(";extension=curl\n;curl.cainfo =\n").unwrap();
    assert!(win.contains("extension=curl") && win.contains(r"\extras\ssl\cacert.pem"));

    let tmp = TempDir::new("php-ini-unix");
    let data = tmp.0.join("data");
    let dir = tmp.0.join("apps").join("php").join("8.4");
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("php"), b"elf").unwrap();
    ensure_php_ini(&data, &dir).unwrap();
    let path = data.join("php").join("8.4").join("php.ini");
    assert_eq!(fs::read_to_string(&path).unwrap(), ini);
    fs::write(&path, "; mine\n").unwrap();
    ensure_php_ini(&data, &dir).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), "; mine\n");
}
