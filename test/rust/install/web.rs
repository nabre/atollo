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
