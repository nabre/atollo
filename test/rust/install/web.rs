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

/// A PHP folder of `kit` (its key) with its `php.ini-development`.
fn fake_php(dir: &Path, template: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("php.exe"), b"").unwrap();
    fs::write(dir.join("php.ini-development"), template).unwrap();
}

/// Defect 10: one php.ini per line of PHP, from that line's template; the
/// single php.ini of an older kit becomes the default line's, with the
/// user's changes.
#[test]
fn one_php_ini_per_line() {
    use crate::kit::{Arch, Kit};
    use crate::setup::catalog::Catalog;

    let tmp = TempDir::new("php-ini-lines");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let data = kit.data();
    let php = Catalog::get().find("php").unwrap();
    let version = "8.4.13 (Composer 2.8.12)";
    let main = crate::versions::install_dir(&kit, php, version);
    fake_php(&main, ";extension=curl\r\n");
    php.save_version(&kit, version).unwrap();
    let old = crate::versions::saved_dir(&kit, php, "5.6");
    fake_php(&old, ";extension=php_curl.dll\r\n");
    crate::versions::write_version(&old, "5.6.40").unwrap();
    let legacy = data.join(r"php\php.ini");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, "memory_limit = 1G\r\n").unwrap();

    assert_eq!(php_ini_dir(&data, &main), data.join(r"php\8.4"));
    assert_eq!(php_ini_dir(&data, &old), data.join(r"php\5.6"));
    assert_eq!(
        php_ini_dir(&data, &kit.apps().join("php")),
        data.join("php"),
        "an older kit's layout"
    );

    migrate_php_ini(&kit).unwrap();
    let main_ini = data.join(r"php\8.4\php.ini");
    let old_ini = data.join(r"php\5.6\php.ini");
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
    assert_eq!(
        env.get("PHPRC"),
        Some(data.join(r"php\5.6").to_str().unwrap())
    );
    assert_eq!(
        env.get("CP_PHPRC_FWD").unwrap(),
        data.join(r"php\5.6").to_str().unwrap().replace('\\', "/")
    );
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
    apache_site(&tmp.0).unwrap();
    let conf = tmp.0.join("kit.conf");
    assert!(fs::read_to_string(&conf)
        .unwrap()
        .contains("Listen 127.0.0.1:${PORTA}"));
    fs::write(&conf, "mine").unwrap();
    apache_site(&tmp.0).unwrap();
    assert_eq!(fs::read_to_string(&conf).unwrap(), "mine");
    assert!(tmp.0.join(r"htdocs\index.php").is_file());
}

#[test]
fn apache_site_refuses_php_without_module() {
    let tmp = TempDir::new("apachesitephp");
    apache_site(&tmp.0).unwrap();
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
    apache_site(&tmp.0).unwrap();
    let text = fs::read_to_string(tmp.0.join("kit.conf")).unwrap();
    let hidden = crate::projects::apache::hidden_deny("").join("\r\n");
    let at = text.find(&hidden).unwrap_or_else(|| panic!("{text}"));
    assert!(at > text.find("Require all granted").unwrap());
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

/// The `.tar.gz` of the Nginx releases is the source: never picked on macOS
/// or Linux, where Nginx is not available (BUG-23b).
#[test]
fn nginx_asset_only_on_windows() {
    for target in [Target::WIN32_X64, Target::WIN32_ARM64] {
        let (pat, kind) = nginx_asset_for(target).unwrap();
        assert!(pat.ends_with(r"\.zip$") && kind == Kind::Zip, "{target}");
    }
    for target in [
        Target::DARWIN_ARM64,
        Target::DARWIN_X64,
        Target::LINUX_X64,
        Target::LINUX_ARM64,
    ] {
        let e = nginx_asset_for(target).unwrap_err().to_string();
        assert!(e.contains(target.as_str()), "{e}");
    }
}
