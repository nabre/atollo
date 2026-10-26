use super::*;
use crate::kit::tests::TempDir;
use serde_json::json;

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
    // BUG-V7-38: no sentence in one language in the welcome page.
    let page = fs::read_to_string(tmp.0.join("htdocs").join("index.php")).unwrap();
    assert!(page.contains("<h1>Apache &#10003;</h1>"), "{page}");
    assert!(
        !page.contains("funziona") && !page.contains("cartella"),
        "{page}"
    );
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
            "darwin-arm64": sha, "linux-x64": sha, "linux-arm64": "nothex" } },
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
/// SHA-256 for the four macOS and Linux targets (none for Windows). At
/// least one line: Apache is offered on every system (D-116).
#[test]
fn httpd_catalog_pins_cover_unix() {
    assert!(!httpd_pins().is_empty(), "no pinned httpd line");
    for (line, pin) in httpd_pins() {
        assert_eq!(
            crate::versions::line_of(&pin.version).as_deref(),
            Some(line.as_str())
        );
        let mut targets: Vec<&str> = pin.sha256_by_target.keys().map(String::as_str).collect();
        targets.sort_unstable();
        assert_eq!(
            targets,
            ["darwin-arm64", "linux-arm64", "linux-x64"],
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
