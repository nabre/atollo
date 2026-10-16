use super::*;

#[test]
fn nginx_conf_with_aliases_and_php() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\shop\public");
    let run = Path::new(r"D:\kit\data\projects\shop\nginx");
    let tmp = crate::kit::tests::TempDir::new("nginx-alias");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join(r"shop\storage\media")).unwrap();
    fs::create_dir_all(projs.join(r"common\assets")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/media".into());
    aliases.insert("/shared".into(), "@common/assets".into());
    aliases.insert("/up".into(), "../common/assets".into());
    let aliases = alias::resolve(&projs.join("shop"), &projs, &aliases);
    let conf = project_conf(
        nginx,
        Some("shop"),
        doc,
        run,
        7106,
        Some(9123),
        false,
        &["api.shop.localhost".into(), "x;y.localhost".into()],
        &aliases,
    );
    let projs_fwd = fwd(&projs);
    assert!(conf.contains("listen 127.0.0.1:7106;"));
    assert!(conf.contains(
        "server_name shop.localhost api.shop.localhost localhost 127.0.0.1 [::1] *.localhost;"
    ));
    assert!(conf.contains("root \"D:/kit/projects/shop/public\";"));
    assert!(conf.contains("fastcgi_pass 127.0.0.1:9123;"));
    assert!(conf.contains("location /media/ {"));
    assert!(conf.contains(&format!("alias \"{projs_fwd}/shop/storage/media/\";")));
    assert!(conf.contains(&format!("alias \"{projs_fwd}/common/assets/\";")));
    assert!(!conf.contains("location /up/") && !conf.contains("x;y"));
}

/// SEC-V3-03: not in LAN mode the site names the local hosts only and a
/// default server answers 421 to any other `Host`; in LAN mode the site is
/// the default server (an IP of the PC is the `Host`).
#[test]
fn only_local_hosts_without_lan() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\shop\public");
    let run = Path::new(r"D:\kit\data\projects\shop\nginx");
    let conf = |lan| project_conf(nginx, Some("shop"), doc, run, 7113, None, lan, &[], &[]);

    let local = conf(false);
    let refuse = "    server {\r\n        listen 127.0.0.1:7113 default_server;\r\n        server_name _;\r\n        return 421;\r\n    }";
    assert!(local.contains(refuse), "{local}");
    assert_eq!(local.matches("default_server").count(), 1, "{local}");
    assert!(local.contains("server_name shop.localhost localhost 127.0.0.1 [::1] *.localhost;\r\n"));

    let lan = conf(true);
    assert!(!lan.contains("return 421"), "{lan}");
    assert!(lan.contains("listen 7113 default_server;"));
    assert!(lan.contains("server_name shop.localhost localhost;\r\n"));
}

#[test]
fn no_fcgi_blocks_php_source() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\site\public");
    let run = Path::new(r"D:\kit\data\projects\site\nginx");
    let block =
        "        location ~* \\.(?:php|phtml|phar)$ {\r\n            return 503;\r\n        }";
    let conf = |fcgi| project_conf(nginx, Some("site"), doc, run, 7107, fcgi, false, &[], &[]);

    let static_only = conf(None);
    assert!(static_only.contains(block), "{static_only}");
    assert!(!static_only.contains("fastcgi_pass"));

    let with_php = conf(Some(9124));
    assert!(!with_php.contains("return 503;"), "{with_php}");
    assert!(with_php.contains("fastcgi_pass 127.0.0.1:9124;"));
}

/// F2: on NTFS `/INDEX.PHP` is `index.php`: the FastCGI location matches
/// any case, so it is run and never served as source by `location /`.
#[test]
fn fastcgi_location_is_case_insensitive() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\site\public");
    let run = Path::new(r"D:\kit\data\projects\site\nginx");
    let conf = project_conf(
        nginx,
        Some("site"),
        doc,
        run,
        7108,
        Some(9125),
        false,
        &[],
        &[],
    );
    assert!(
        conf.contains("location ~* \\.(?:php|phtml|phar)$ {"),
        "{conf}"
    );
    assert!(!conf.contains("location ~ \\."), "{conf}");
}

/// SEC-V3-01: `.inc` and `.php3`…`.php8` refused with or without PHP,
/// before the PHP location; `.phtml` and `.phar` go where `.php` goes.
#[test]
fn inc_and_old_php_files_are_never_served() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\site\public");
    let run = Path::new(r"D:\kit\data\projects\site\nginx");
    let never = "        location ~* \\.(?:inc|php[3-8])$ {\r\n            deny all;\r\n        }";
    for fcgi in [None, Some(9127)] {
        let conf = project_conf(nginx, Some("site"), doc, run, 7111, fcgi, false, &[], &[]);
        let at = conf.find(never).unwrap_or_else(|| panic!("{conf}"));
        assert!(at < conf.find("location ~* \\.(?:php|phtml|phar)$").unwrap());
    }
}

/// F3: hidden files and folders are refused but `.well-known`, before any
/// other regex location (the first matching one wins), with or without PHP.
#[test]
fn hidden_files_are_denied_first() {
    let nginx = Path::new(r"D:\kit\data\apps\win-x64\nginx\1.28");
    let doc = Path::new(r"D:\kit\projects\blog");
    let run = Path::new(r"D:\kit\data\projects\blog\nginx");
    let deny = "        location ~ /\\.(?!well-known(/|$)) {\r\n            deny all;\r\n        }";
    for fcgi in [None, Some(9126)] {
        let conf = project_conf(nginx, Some("blog"), doc, run, 7110, fcgi, false, &[], &[]);
        let at = conf.find(deny).unwrap_or_else(|| panic!("{conf}"));
        assert!(at < conf.find("location ~* \\.(?:php").unwrap(), "{conf}");
    }
}
