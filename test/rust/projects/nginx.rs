use super::*;

/// A php-cgi on `port` with a fixed key.
fn cgi(port: u16) -> FastCgi {
    FastCgi {
        port,
        key: "0123abcd".repeat(8),
    }
}

#[test]
#[cfg_attr(not(windows), ignore = "alias folders created with backslash paths")]
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
        Some(&cgi(9123)),
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
    // The key the kit's guard asks of every request (D-99).
    assert!(conf.contains(&format!(
        "            fastcgi_param DEVKIT_FCGI_KEY {};\r\n        }}",
        cgi(9123).key
    )));
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
    let conf = |fcgi: Option<&FastCgi>| {
        project_conf(nginx, Some("site"), doc, run, 7107, fcgi, false, &[], &[])
    };

    let static_only = conf(None);
    assert!(static_only.contains(block), "{static_only}");
    assert!(!static_only.contains("fastcgi_pass"));

    let with_php = conf(Some(&cgi(9124)));
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
        Some(&cgi(9125)),
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
    for fcgi in [None, Some(cgi(9127))] {
        let conf = project_conf(
            nginx,
            Some("site"),
            doc,
            run,
            7111,
            fcgi.as_ref(),
            false,
            &[],
            &[],
        );
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
    for fcgi in [None, Some(cgi(9126))] {
        let conf = project_conf(
            nginx,
            Some("blog"),
            doc,
            run,
            7110,
            fcgi.as_ref(),
            false,
            &[],
            &[],
        );
        let at = conf.find(deny).unwrap_or_else(|| panic!("{conf}"));
        assert!(at < conf.find("location ~* \\.(?:php").unwrap(), "{conf}");
    }
}

/// WEB-8: no link followed in an alias' folder where Nginx can refuse them;
/// Nginx for Windows has no `disable_symlinks` and would not start with it.
#[test]
#[cfg_attr(not(windows), ignore = "alias folders created with backslash paths")]
fn aliases_follow_no_links_where_nginx_can() {
    let tmp = crate::kit::tests::TempDir::new("nginx-alias-links");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join(r"shop\storage\media")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/media".into());
    let aliases = alias::resolve(&projs.join("shop"), &projs, &aliases);
    let lines = alias_locations(&aliases, "").join("\r\n");
    assert!(lines.starts_with("location /media/ {"), "{lines}");
    assert_eq!(
        lines.contains("disable_symlinks on;"),
        !cfg!(windows),
        "{lines}"
    );
}

/// BUG-V3-08: the programs of Nginx, PHP and Apache in each system's layout.
#[test]
fn server_programs_per_system() {
    use crate::kit::Target;
    let tmp = crate::kit::tests::TempDir::new("server-exes");
    let dir = tmp.0.join("srv");
    let win = Kit::at_target(tmp.0.join("w"), Target::WIN32_X64);
    let linux = Kit::at_target(tmp.0.join("l"), Target::LINUX_X64);
    let mac = Kit::at_target(tmp.0.join("m"), Target::DARWIN_ARM64);
    assert_eq!(nginx_exe(&win, &dir), dir.join("nginx.exe"));
    assert_eq!(php_cgi_exe(&win, &dir), dir.join("php-cgi.exe"));
    assert_eq!(
        crate::projects::apache::httpd_exe(&win, &dir),
        dir.join("bin").join("httpd.exe")
    );
    assert_eq!(php_cgi_exe(&mac, &dir), dir.join("bin").join("php-cgi"));
    assert_eq!(
        crate::projects::apache::httpd_exe(&linux, &dir),
        dir.join("bin").join("httpd")
    );
    // sbin/nginx by default; bin/nginx when only that one exists.
    assert_eq!(nginx_exe(&linux, &dir), dir.join("sbin").join("nginx"));
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::write(dir.join("bin").join("nginx"), "").unwrap();
    assert_eq!(nginx_exe(&mac, &dir), dir.join("bin").join("nginx"));
    fs::create_dir_all(dir.join("sbin")).unwrap();
    fs::write(dir.join("sbin").join("nginx"), "").unwrap();
    assert_eq!(nginx_exe(&linux, &dir), dir.join("sbin").join("nginx"));
}

/// D-99 «Resta»: php-cgi runs nothing without the key Nginx sends. The key
/// is an ini value of its command line (out of reach of a request) and the
/// guard path stays one ini value with a space or a `;` in it.
#[test]
fn php_cgi_guard_arguments() {
    let guard = Path::new("D:/my kit;x/data/kit/fcgi-guard.php");
    assert_eq!(
        guard_args(guard, "abc"),
        [
            "-d",
            "devkit.fcgi_key=abc",
            "-d",
            "auto_prepend_file=\"D:/my kit;x/data/kit/fcgi-guard.php\""
        ]
    );
    let php = include_str!("../../../runtime/fcgi-guard.php");
    assert!(php.contains("get_cfg_var('devkit.fcgi_key')"));
    assert!(php.contains(&format!("$_SERVER['{FCGI_KEY_PARAM}']")));
    assert!(php.contains("hash_equals(") && php.contains("http_response_code(403)"));
}
