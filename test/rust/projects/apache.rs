use super::*;

#[test]
fn conf_of_a_project() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n\r\n# --- Dev-Kit ---\r\nInclude \"${CP_ROOT_FWD}/data/apache/kit.conf\"\r\n";
    let root = Path::new(r"D:\kit\projects\shop\public");
    let dir = Path::new(r"D:\kit\data\projects\shop\apache");
    let c = project_conf(base, root, dir, 7105, false).unwrap();
    assert!(c.starts_with("Define SRVROOT"));
    assert!(!c.contains("kit.conf"));
    assert!(c.contains("Listen 127.0.0.1:7105\r\n"));
    assert!(c.contains("DocumentRoot \"D:/kit/projects/shop/public\""));
    assert!(c.contains("PidFile \"D:/kit/data/projects/shop/apache/httpd.pid\""));
    assert!(c.contains("php8apache2_4.dll"));
    assert!(c.contains("php7apache2_4.dll"));
    assert!(c.contains("php5apache2_4.dll"));
    assert!(c.contains("IncludeOptional \"D:/kit/data/projects/shop/apache/conf.d/*.conf\""));
    let lan = project_conf(base, root, dir, 7105, true).unwrap();
    assert!(lan.contains("Listen 7105\r\n"));
    assert!(project_conf("Listen 80", root, dir, 7105, false).is_err());

    let tmp = crate::kit::tests::TempDir::new("apache-alias");
    let projects = tmp.0.join("projects");
    fs::create_dir_all(projects.join(r"shop\storage\media")).unwrap();
    fs::create_dir_all(projects.join(r"common\assets")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/media".into());
    aliases.insert("/shared".into(), "@common/assets".into());
    aliases.insert("/up".into(), "../common/assets".into());
    let aliases = alias::resolve(&projects.join("shop"), &projects, &aliases);
    let with_site = project_conf_with_site(
        base,
        Some("shop"),
        root,
        dir,
        7105,
        false,
        &["api.shop.localhost".into(), "bad\"domain.localhost".into()],
        &aliases,
    )
    .unwrap();
    assert!(with_site.contains("ServerAlias shop.localhost api.shop.localhost\r\n"));
    assert!(with_site.contains("Alias \"/media\" \"${CP_PROJECT_FWD}/storage/media\""));
    assert!(with_site.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
    assert!(!with_site.contains("/up") && !with_site.contains(".."));
}

/// A3 handoff: `.php` goes to PHP in any case, and without a PHP module it
/// is refused, never sent as source.
#[test]
fn php_is_case_insensitive_and_denied_without_module() {
    let blocks = php_module_blocks("CP_PHP_FWD").join("\r\n");
    assert!(!blocks.contains("<FilesMatch \"\\.php$\">"), "{blocks}");
    assert_eq!(
        blocks
            .matches("<FilesMatch \"(?i)\\.php$\">\r\n        SetHandler application/x-httpd-php")
            .count(),
        3
    );
    let deny = "<IfModule !php_module>\r\n    <IfModule !php7_module>\r\n        <IfModule !php5_module>\r\n            <FilesMatch \"(?i)\\.php$\">\r\n                Require all denied\r\n            </FilesMatch>\r\n        </IfModule>\r\n    </IfModule>\r\n</IfModule>";
    let at = blocks.find(deny).expect("deny block");
    // After the modules are loaded: `IfModule` is read in order.
    assert!(at > blocks.rfind("LoadModule").unwrap());
    assert_eq!(
        blocks.matches("<IfModule").count(),
        blocks.matches("</IfModule>").count()
    );

    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\n";
    let conf = project_conf(
        base,
        Path::new(r"D:\kit\projects\shop\public"),
        Path::new(r"D:\kit\data\projects\shop\apache"),
        7105,
        false,
    )
    .unwrap();
    assert!(conf.contains(deny));
}

/// F3: hidden files and folders (`.env`, `.git/`) are refused but
/// `.well-known`, by URL and by file name, also with the project's root
/// as DocumentRoot.
#[test]
fn hidden_files_are_denied() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\n";
    let conf = project_conf(
        base,
        Path::new(r"D:\kit\projects\blog"),
        Path::new(r"D:\kit\data\projects\blog\apache"),
        7109,
        false,
    )
    .unwrap();
    let by_url =
        "<LocationMatch \"/\\.(?!well-known(/|$))\">\r\n    Require all denied\r\n</LocationMatch>";
    let by_name = "<FilesMatch \"^\\.(?!well-known$)\">\r\n    Require all denied\r\n</FilesMatch>";
    assert!(conf.contains(by_url), "{conf}");
    assert!(conf.contains(by_name), "{conf}");
    // After the `<Directory>` that grants access.
    assert!(conf.find(by_url).unwrap() > conf.find("Require all granted").unwrap());
    assert_eq!(
        hidden_deny("    ")[0],
        format!("    {}", by_url.lines().next().unwrap())
    );
}
