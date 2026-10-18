use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;
use crate::os::safe_path::tests::junction;

/// A FastCGI server on `port` with a fixed key.
fn cgi(port: u16) -> FastCgi {
    FastCgi {
        port,
        key: "0123abcd".repeat(8),
    }
}

/// `caddy.exe` on Windows, `caddy` elsewhere, at the root of its folder:
/// the key of the component on each of the six targets (WP-3.1).
#[test]
fn caddy_program_per_target() {
    let dir = Path::new("kit/caddy/2.11");
    for t in Target::ALL {
        let want = if t.is_windows() { "caddy.exe" } else { "caddy" };
        assert_eq!(caddy_exe_for(t.os, dir), dir.join(want), "{t}");
        let key = crate::setup::catalog::Catalog::get()
            .find("caddy")
            .unwrap()
            .key_for(t);
        assert_eq!(key, format!("caddy/{want}"), "{t}");
    }
}

/// The project's Caddyfile (§4.4): its own admin port, no automatic HTTPS,
/// nothing saved outside, its storage in its folder; the site on its names
/// and the local ones, on 127.0.0.1 only, any other `Host` refused with 421.
#[test]
fn project_caddyfile_with_php_and_aliases() {
    let tmp = TempDir::new("caddy-conf");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join("shop").join("storage").join("media")).unwrap();
    fs::create_dir_all(projs.join("common").join("assets")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/media".into());
    aliases.insert("/shared".into(), "@common/assets".into());
    aliases.insert("/up".into(), "../common/assets".into());
    let aliases = resolve_aliases(&projs.join("shop"), &projs, &aliases);
    let doc = Path::new(r"D:\kit\projects\shop\public");
    let run = Path::new(r"D:\kit\data\projects\shop\caddy");
    let conf = project_conf(
        Some("shop"),
        doc,
        run,
        7106,
        Some(9321),
        Some(&cgi(9123)),
        false,
        &["api.shop.localhost".into(), "x;y.localhost".into()],
        &aliases,
    )
    .unwrap();
    let start = "{\n\tadmin 127.0.0.1:9321\n\tauto_https off\n\tpersist_config off\n\tstorage file_system \"D:/kit/data/projects/shop/caddy/storage\"\n}\n";
    assert!(conf.contains(start), "{conf}");
    assert!(conf.contains(
        "http://shop.localhost:7106, http://api.shop.localhost:7106, http://localhost:7106, http://127.0.0.1:7106, http://[::1]:7106, http://*.localhost:7106 {\n\tbind 127.0.0.1\n\troute {\n"
    ), "{conf}");
    assert!(conf.contains("\t\troot * \"D:/kit/projects/shop/public\"\n"));
    // PHP with the key of the kit's guard (D-99), `.phtml` and `.phar` too.
    assert!(conf.contains(&format!(
        "\t\tphp_fastcgi 127.0.0.1:9123 {{\n\t\t\tenv DEVKIT_FCGI_KEY {}\n\t\t\tsplit .php .phtml .phar\n\t\t}}\n\t\tfile_server\n",
        cgi(9123).key
    )), "{conf}");
    // Aliases: only those checked on disk, listed, PHP in them too.
    let projs_fwd = fwd(&projs);
    assert!(conf.contains("\t\tredir /media /media/ 308\n\t\thandle_path /media/* {\n"));
    assert!(conf.contains(&format!(
        "\t\t\troot * \"{projs_fwd}/shop/storage/media\"\n"
    )));
    assert!(conf.contains(&format!("\t\t\troot * \"{projs_fwd}/common/assets\"\n")));
    assert!(conf.contains("\t\t\tfile_server browse\n"));
    assert!(!conf.contains("/up") && !conf.contains("x;y"), "{conf}");
    // DNS rebinding: any other Host gets 421 (D-98).
    assert!(conf.contains("http://:7106 {\n\tbind 127.0.0.1\n\trespond 421\n}\n"));
}

/// Hidden files and folders (but `.well-known`) and `.inc`/`.php3`…`.php8`
/// refused with 404 before anything else, aliases included (T3.3.7, as
/// Apache's `hidden_deny`); the PHP files refused with 503 without PHP.
#[test]
fn hidden_files_and_php_without_fastcgi() {
    let doc = Path::new("/kit/projects/site");
    let routes = site_routes(doc, &[], None, "").unwrap().join("\n");
    let hidden = "route {\n\t@hidden {\n\t\tpath_regexp /\\.\n\t\tnot path /.well-known /.well-known/*\n\t}\n\trespond @hidden 404\n\t@never path_regexp (?i)\\.(?:inc|php[3-8])$\n\trespond @never 404\n";
    assert!(routes.starts_with(hidden), "{routes}");
    assert!(routes.contains("\t@php path_regexp (?i)\\.(?:php|phtml|phar)$\n\trespond @php 503\n"));
    assert!(!routes.contains("php_fastcgi"));
    let with_php = site_routes(doc, &[], Some(&cgi(9001)), "")
        .unwrap()
        .join("\n");
    assert!(with_php.starts_with(hidden));
    assert!(!with_php.contains("respond @php 503"), "{with_php}");
    // The refusals come before the aliases, which `handle_path` would take.
    let tmp = TempDir::new("caddy-hidden-alias");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join("site").join("up")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/up".into(), "up".into());
    let aliases = resolve_aliases(&projs.join("site"), &projs, &aliases);
    let routes = site_routes(doc, &aliases, None, "").unwrap().join("\n");
    let never = routes.find("respond @never 404").unwrap();
    let php = routes.find("respond @php 503").unwrap();
    let alias = routes.find("handle_path /up/*").unwrap();
    assert!(never < php && php < alias, "{routes}");
}

/// In LAN mode the site answers any `Host` (an IP of the PC), on every
/// interface, and nothing is refused with 421.
#[test]
fn lan_mode_answers_any_host() {
    let doc = Path::new("/kit/projects/shop/public");
    let run = Path::new("/kit/data/projects/shop/caddy");
    let conf = project_conf(
        Some("shop"),
        doc,
        run,
        7113,
        Some(9400),
        None,
        true,
        &[],
        &[],
    )
    .unwrap();
    assert!(conf.contains("http://:7113 {\n\troute {\n"), "{conf}");
    assert!(
        !conf.contains("bind ") && !conf.contains("respond 421"),
        "{conf}"
    );
}

/// A folder Caddy would read as something else (a placeholder, a quote) is
/// never written: the site is not served rather than served from elsewhere.
#[test]
fn paths_caddy_cannot_take() {
    assert_eq!(
        quoted_path(Path::new(r"C:\My Kit\projects\a")).as_deref(),
        Some("\"C:/My Kit/projects/a\"")
    );
    for bad in ["/kit/{env.HOME}/a", "/kit/a\"b", "/kit/a\nb", "/kit/a`b"] {
        assert_eq!(quoted_path(Path::new(bad)), None, "{bad:?}");
        assert!(site_routes(Path::new(bad), &[], None, "").is_none());
    }
    let err = project_conf(
        None,
        Path::new("/kit/{x}"),
        Path::new("/kit/run"),
        7100,
        None,
        None,
        false,
        &[],
        &[],
    );
    assert!(err.is_err());
}

/// The pool's Caddyfile: its sites, then its own page for its local names
/// (any `Host` in LAN mode), any other `Host` refused with 421; `admin off`
/// for a file written only for the next start; the user's snippets imported
/// only when there are some.
#[test]
fn pool_caddyfile() {
    let pool = Path::new("/kit/data/servers/caddy-2.11_php-8.4");
    let site = site_block(
        8081,
        false,
        &["shop.localhost".into()],
        &["\trespond \"x\"".into()],
    );
    let conf = pool_conf(
        pool,
        8081,
        Some(9500),
        false,
        std::slice::from_ref(&site),
        false,
    )
    .unwrap();
    assert!(conf.contains("\tadmin 127.0.0.1:9500\n"));
    assert!(conf.contains("http://shop.localhost:8081 {\n\tbind 127.0.0.1\n\trespond \"x\"\n}\n"));
    assert!(conf.contains(
        "http://localhost:8081, http://127.0.0.1:8081, http://[::1]:8081 {\n\tbind 127.0.0.1\n\trespond \"Dev-Kit: Caddy pool\" 200\n}\n"
    ), "{conf}");
    assert!(conf.contains("http://:8081 {\n\tbind 127.0.0.1\n\trespond 421\n}\n"));
    assert!(!conf.contains("import"));
    let next = pool_conf(pool, 8081, None, false, &[], true).unwrap();
    assert!(next.contains("\tadmin off\n"));
    assert!(next.contains("import \"/kit/data/servers/caddy-2.11_php-8.4/conf.d/*.caddy\"\n"));
    let lan_site = site_block(
        8081,
        true,
        &["shop.localhost".into()],
        &["\trespond \"x\"".into()],
    );
    let lan = pool_conf(pool, 8081, Some(9500), true, &[lan_site], false).unwrap();
    assert!(
        lan.contains("http://:8081 {\n\trespond \"Dev-Kit: Caddy pool\" 200\n}\n"),
        "{lan}"
    );
    assert!(!lan.contains("bind ") && !lan.contains("respond 421"));
}

/// `caddy run` and `caddy reload` of the instance's own admin endpoint.
#[test]
fn run_and_reload_arguments() {
    let conf = Path::new("/kit/data/servers/p/Caddyfile");
    let run: Vec<String> = run_args(conf)
        .into_iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        run,
        [
            "run",
            "--config",
            "/kit/data/servers/p/Caddyfile",
            "--adapter",
            "caddyfile"
        ]
    );
    let reload: Vec<String> = reload_args(conf, 9600)
        .into_iter()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    assert_eq!(
        reload,
        [
            "reload",
            "--config",
            "/kit/data/servers/p/Caddyfile",
            "--adapter",
            "caddyfile",
            "--address",
            "127.0.0.1:9600"
        ]
    );
    let port = free_port().unwrap();
    assert!(port >= 1024);
}

/// Caddy's file server follows links on every system: an alias whose folder
/// has one inside is left out (with Apache it would be served).
#[test]
fn aliases_with_links_inside_are_left_out() {
    let tmp = TempDir::new("caddy-alias-links");
    let projects = tmp.0.join("projects");
    let shop = projects.join("shop");
    fs::create_dir_all(shop.join("clean").join("a")).unwrap();
    let month = shop.join("dirty").join("2026");
    fs::create_dir_all(&month).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    junction(&month.join("leak"), &outside);
    let mut aliases = BTreeMap::new();
    aliases.insert("/clean".to_owned(), "clean".to_owned());
    aliases.insert("/dirty".to_owned(), "dirty".to_owned());
    aliases.insert("/missing".to_owned(), "nope".to_owned());
    let urls: Vec<String> = resolve_aliases(&shop, &projects, &aliases)
        .into_iter()
        .map(|a| a.url)
        .collect();
    assert_eq!(urls, ["/clean"]);
    let mut skipped = skipped_aliases(&shop, &projects, &aliases);
    skipped.sort();
    assert_eq!(skipped, ["/dirty", "/missing"]);
}

/// The names of a site: `<project>.localhost` and its valid domains once;
/// a project's own Caddy also `localhost` and, out of LAN mode, the local
/// hosts.
#[test]
fn names_of_a_site() {
    let domains = ["API.shop.localhost".to_owned(), "bad".to_owned()];
    assert_eq!(
        site_names(Some("shop"), &domains, false, false),
        ["shop.localhost", "api.shop.localhost"]
    );
    assert_eq!(
        site_names(Some("shop"), &[], true, false),
        [
            "shop.localhost",
            "localhost",
            "127.0.0.1",
            "[::1]",
            "*.localhost"
        ]
    );
    assert_eq!(site_names(None, &[], true, true), ["localhost"]);
}
