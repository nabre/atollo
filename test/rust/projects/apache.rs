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
    assert!(with_site.contains(
        "    ServerAlias 127.0.0.1 [::1] *.localhost shop.localhost api.shop.localhost\r\n"
    ));
    assert!(with_site.contains("Alias \"/media\" \"${CP_PROJECT_FWD}/storage/media\""));
    assert!(with_site.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
    assert!(!with_site.contains("/up") && !with_site.contains(".."));
}

/// SEC-V3-03: not in LAN mode only a local `Host` is served; the first
/// VirtualHost, which answers any other one, refuses everything. In LAN
/// mode an IP of the PC is the `Host`: no VirtualHost.
#[test]
fn only_local_hosts_without_lan() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\n";
    let root = Path::new(r"D:\kit\projects\shop\public");
    let dir = Path::new(r"D:\kit\data\projects\shop\apache");
    let conf = |lan| {
        project_conf_with_site(
            base,
            Some("shop"),
            root,
            dir,
            7112,
            lan,
            &["api.shop.localhost".into()],
            &[],
        )
        .unwrap()
    };
    let local = conf(false);
    let refuse = "<VirtualHost *:7112>\r\n    ServerName refused.invalid\r\n    <Location \"/\">\r\n        Require all denied\r\n    </Location>\r\n</VirtualHost>";
    let site = "<VirtualHost *:7112>\r\n    ServerName localhost\r\n    ServerAlias 127.0.0.1 [::1] *.localhost shop.localhost api.shop.localhost\r\n</VirtualHost>";
    let at = local.find(refuse).unwrap_or_else(|| panic!("{local}"));
    assert_eq!(
        local.find("<VirtualHost"),
        Some(at),
        "the refusing one first"
    );
    assert!(local.find(site).unwrap() > at, "{local}");
    assert_eq!(local.matches("<VirtualHost").count(), 2);
    // Before the user's conf.d (a VirtualHost of theirs is never the first).
    assert!(local.find(site).unwrap() < local.find("IncludeOptional").unwrap());
    // The site's directives stay in the main server, inherited by both.
    assert!(local.find("DocumentRoot").unwrap() < at);
    assert!(!local[..at].contains("ServerAlias"));

    let lan = conf(true);
    assert!(!lan.contains("<VirtualHost"), "{lan}");
    assert!(lan.contains("\r\nServerAlias shop.localhost api.shop.localhost\r\n"));
}

/// A3 handoff: `.php` goes to PHP in any case, and without a PHP module it
/// is refused, never sent as source.
#[test]
fn php_is_case_insensitive_and_denied_without_module() {
    let blocks = php_module_blocks("CP_PHP_FWD").join("\r\n");
    assert!(!blocks.contains("<FilesMatch \"\\.php$\">"), "{blocks}");
    assert_eq!(
        blocks
            .matches("<FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">\r\n        SetHandler application/x-httpd-php")
            .count(),
        3
    );
    let deny = "<IfModule !php_module>\r\n    <IfModule !php7_module>\r\n        <IfModule !php5_module>\r\n            <FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">\r\n                Require all denied\r\n            </FilesMatch>\r\n        </IfModule>\r\n    </IfModule>\r\n</IfModule>";
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

/// SEC-V3-01: `.phtml` and `.phar` run as PHP (refused without it), `.inc`
/// and `.php3`…`.php8` are never served, PHP or not.
#[test]
fn other_php_files_are_run_or_refused() {
    let run = regex::Regex::new(&format!("(?i){PHP_RUN}")).unwrap();
    let never = regex::Regex::new(&format!("(?i){PHP_NEVER}")).unwrap();
    for f in ["a.php", "INDEX.PHP", "view.phtml", "tool.PHAR"] {
        assert!(run.is_match(f) && !never.is_match(f), "{f}");
    }
    for f in [
        "db.inc",
        "CONFIG.INC",
        "old.php3",
        "old.php5",
        "x.PHP7",
        "y.php8",
    ] {
        assert!(never.is_match(f) && !run.is_match(f), "{f}");
    }
    for f in ["a.php.txt", "a.html", "php", "a.php9", "a.include"] {
        assert!(!run.is_match(f) && !never.is_match(f), "{f}");
    }

    let blocks = php_module_blocks("CP_PHP_FWD").join("\r\n");
    let refuse =
        "<FilesMatch \"(?i)\\.(?:inc|php[3-8])$\">\r\n    Require all denied\r\n</FilesMatch>";
    // Outside any `IfModule`/`IfFile`: with PHP too.
    let at = blocks.find(refuse).unwrap_or_else(|| panic!("{blocks}"));
    assert!(at > blocks.rfind("</IfModule>").unwrap(), "{blocks}");
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

/// An older kit.conf (created once): `.php` matched in any case and refused
/// without a PHP module, once; the user's other lines kept.
#[test]
fn old_kit_conf_is_migrated_once() {
    let mut old = vec![
        "DocumentRoot \"${CP_ROOT_FWD}/data/apache/htdocs\"".to_owned(),
        "# mine".to_owned(),
    ];
    for (dll, ts, module) in [
        ("php8apache2_4.dll", "php8ts.dll", "php_module"),
        ("php7apache2_4.dll", "php7ts.dll", "php7_module"),
        ("php5apache2_4.dll", "php5ts.dll", "php5_module"),
    ] {
        old.extend([
            format!("<IfFile \"${{CP_APPS_FWD}}/php/{dll}\">"),
            format!("    LoadFile \"${{CP_APPS_FWD}}/php/{ts}\""),
            format!("    LoadModule {module} \"${{CP_APPS_FWD}}/php/{dll}\""),
            "    <FilesMatch \"\\.php$\">".to_owned(),
            "        SetHandler application/x-httpd-php".to_owned(),
            "    </FilesMatch>".to_owned(),
            "</IfFile>".to_owned(),
        ]);
    }
    old.extend([
        String::new(),
        "IncludeOptional \"${CP_ROOT_FWD}/data/apache/conf.d/*.conf\"".to_owned(),
        String::new(),
    ]);
    let old = old.join("\r\n");

    let new = migrate_kit_conf(&old);
    assert!(!new.contains("\"\\.php$\""), "{new}");
    assert!(!new.contains("\"(?i)\\.php$\""), "{new}");
    assert!(!new.contains("${CP_APPS_FWD}/php/"), "{new}");
    assert_eq!(
        new.matches("<FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">")
            .count(),
        4
    );
    assert_eq!(
        new.matches("<FilesMatch \"(?i)\\.(?:inc|php[3-8])$\">")
            .count(),
        1
    );
    assert!(new.contains("# mine"));
    let deny = new.find("<IfModule !php_module>").expect("deny block");
    assert!(deny > new.rfind("</IfFile>").unwrap());
    assert!(deny < new.find("IncludeOptional").unwrap());
    assert_eq!(migrate_kit_conf(&new), new, "idempotent");

    // SEC-V3-01, SEC-V3-02: a kit.conf of the previous kit (`(?i)\.php$`,
    // deny block already there) gets the other PHP files, the `.inc`
    // refusal and the hidden files refusal.
    let hidden = hidden_deny("").join("\r\n");
    let previous = migrate_kit_conf(&old)
        .replace("\\.(?:php|phtml|phar)$", "\\.php$")
        .replace(&format!("\r\n{hidden}"), "")
        .replace(
            "<FilesMatch \"(?i)\\.(?:inc|php[3-8])$\">\r\n    Require all denied\r\n</FilesMatch>\r\n",
            "",
        );
    assert!(!previous.contains("php[3-8]"), "{previous}");
    assert!(!previous.contains("LocationMatch"), "{previous}");
    assert_eq!(migrate_kit_conf(&previous), new);

    // SEC-V3-02: hidden files refused, once, before the user's includes.
    assert_eq!(new.matches(&hidden).count(), 1, "{new}");
    assert!(new.find(&hidden).unwrap() < new.find("IncludeOptional").unwrap());
    let unhidden = new.replace(&format!("\r\n{hidden}"), "");
    assert!(!unhidden.contains("LocationMatch"), "{unhidden}");
    let rehidden = migrate_kit_conf(&unhidden);
    assert_eq!(rehidden.matches(&hidden).count(), 1, "{rehidden}");
    assert_eq!(rehidden.len(), new.len());
    assert_eq!(migrate_kit_conf(&rehidden), rehidden);

    // A kit.conf written from php_module_blocks and hidden_deny is left as
    // it is.
    let current = format!(
        "{}\r\n{hidden}\r\n",
        php_module_blocks("CP_PHP_FWD").join("\r\n")
    );
    assert_eq!(migrate_kit_conf(&current), current);
    // No module block at all: the deny goes at the end.
    let bare = migrate_kit_conf("DocumentRoot \"x\"");
    assert!(bare.starts_with("DocumentRoot \"x\"\r\n<IfModule !php_module>"));
}
