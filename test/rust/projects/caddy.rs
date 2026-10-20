use super::*;
use crate::kit::tests::TempDir;
use crate::kit::Target;
use crate::os::safe_path::tests::junction;

/// A FastCGI server on `port` with a fixed key.
fn cgi(port: u16) -> FastCgi {
    FastCgi::tcp(port, "0123abcd".repeat(8))
}

/// `caddy.exe` on Windows, `caddy` elsewhere, at the root of its folder:
/// the key of the component on each of the six targets (D-111).
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

/// The project's Caddyfile (D-111): no admin endpoint, no automatic HTTPS,
/// nothing saved outside, its storage in its folder; the site on its names
/// and the local ones, on 127.0.0.1 only, any other `Host` refused with 421;
/// the FastCGI key read from Caddy's environment, never in the text.
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
        Some(&cgi(9123)),
        false,
        &["api.shop.localhost".into(), "x;y.localhost".into()],
        &aliases,
        None,
    )
    .unwrap();
    let start = "{\n\tadmin off\n\tauto_https off\n\tpersist_config off\n\tstorage file_system \"D:/kit/data/projects/shop/caddy/storage\"\n}\n";
    assert!(conf.contains(start), "{conf}");
    assert!(conf.contains(
        "http://shop.localhost:7106, http://api.shop.localhost:7106, http://localhost:7106, http://127.0.0.1:7106, http://[::1]:7106, http://*.localhost:7106 {\n\tbind 127.0.0.1\n\troute {\n"
    ), "{conf}");
    assert!(conf.contains("\t\troot * \"D:/kit/projects/shop/public\"\n"));
    // PHP with the key of the kit's guard (D-99), `.phtml` and `.phar` too.
    assert!(conf.contains(
        "\t\tphp_fastcgi 127.0.0.1:9123 {\n\t\t\tenv ATOLLO_FCGI_KEY {env.ATOLLO_FCGI_KEY}\n\t\t\tsplit .php .phtml .phar\n\t\t}\n\t\tfile_server\n"
    ), "{conf}");
    assert!(!conf.contains(&cgi(9123).key), "the key is never written");
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
/// refused with 404 before anything else, aliases included (D-111, as
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

/// SEC-V4-01: an alias' folder serves static files only, with or without
/// PHP for the site: its scripts are refused, never run.
#[test]
fn aliases_run_no_php() {
    let doc = Path::new("/kit/projects/site/public");
    let tmp = TempDir::new("caddy-alias-nophp");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join("site").join("up")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/up".into(), "up".into());
    let aliases = resolve_aliases(&projs.join("site"), &projs, &aliases);
    let routes = site_routes(doc, &aliases, Some(&cgi(9001)), "")
        .unwrap()
        .join("\n");
    let start = routes.find("handle_path /up/*").unwrap();
    let block = &routes[start..start + routes[start..].find("\n\t}").unwrap()];
    assert!(!block.contains("php_fastcgi"), "{block}");
    assert!(block.contains("@script path_regexp (?i)\\.(?:php|phtml|phar|phps|inc|php[3-8])$\n\t\trespond @script 404\n\t\tfile_server browse"), "{block}");
    // The document root keeps its PHP.
    assert_eq!(routes.matches("php_fastcgi").count(), 1);
}

/// In LAN mode the site answers any `Host` (an IP of the PC), on every
/// interface, and nothing is refused with 421.
#[test]
fn lan_mode_answers_any_host() {
    let doc = Path::new("/kit/projects/shop/public");
    let run = Path::new("/kit/data/projects/shop/caddy");
    let conf = project_conf(Some("shop"), doc, run, 7113, None, true, &[], &[], None).unwrap();
    assert!(conf.contains("http://:7113 {\n\troute {\n"), "{conf}");
    assert!(
        !conf.contains("bind ") && !conf.contains("respond 421"),
        "{conf}"
    );
}

/// The project's own `caddy\conf.d` is imported in its site, before the
/// routes, as in a pool, and only when it holds a `.caddy` (BUG-V5-06).
#[test]
fn project_conf_d_imported_in_the_site() {
    let tmp = TempDir::new("caddy-project-conf-d");
    let conf_d = tmp.0.join("caddy").join("conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    assert!(!has_snippets(&conf_d));
    fs::write(conf_d.join("notes.txt"), "").unwrap();
    assert!(!has_snippets(&conf_d));
    fs::write(
        conf_d.join("cors.CADDY"),
        "header Access-Control-Allow-Origin *\n",
    )
    .unwrap();
    assert!(has_snippets(&conf_d));

    let doc = Path::new("/kit/projects/shop/public");
    let run = Path::new("/kit/data/projects/shop/caddy");
    let conf_d = Path::new("/kit/data/projects/shop/caddy/conf.d");
    let conf = project_conf(
        Some("shop"),
        doc,
        run,
        7114,
        None,
        false,
        &[],
        &[],
        Some(conf_d),
    )
    .unwrap();
    assert!(
        conf.contains("\tbind 127.0.0.1\n\timport \"/kit/data/projects/shop/caddy/conf.d/*.caddy\"\n\troute {\n"),
        "{conf}"
    );
    // Once, in the site: not in the block that refuses the other names.
    assert_eq!(conf.matches("import ").count(), 1, "{conf}");
    let without = project_conf(Some("shop"), doc, run, 7114, None, false, &[], &[], None).unwrap();
    assert!(!without.contains("import "), "{without}");
    // A folder Caddy cannot read is left out, the site still served.
    let odd = project_conf(
        Some("shop"),
        doc,
        run,
        7114,
        None,
        false,
        &[],
        &[],
        Some(Path::new("/kit/{x}")),
    )
    .unwrap();
    assert!(!odd.contains("import ") && odd.contains("route {"), "{odd}");
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
        false,
        &[],
        &[],
        None,
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
    let sock = Path::new("/tmp/Atollo/0123456789abcdef/caddy-01234567/a.sock");
    let conf = pool_conf(
        pool,
        8081,
        Some(sock),
        false,
        std::slice::from_ref(&site),
        false,
    )
    .unwrap();
    assert!(
        conf.contains("\tadmin \"unix//tmp/Atollo/0123456789abcdef/caddy-01234567/a.sock|0600\"\n"),
        "{conf}"
    );
    assert!(!conf.contains("admin 127.0.0.1"));
    assert!(conf.contains("http://shop.localhost:8081 {\n\tbind 127.0.0.1\n\trespond \"x\"\n}\n"));
    assert!(conf.contains(
        "http://localhost:8081, http://127.0.0.1:8081, http://[::1]:8081 {\n\tbind 127.0.0.1\n\trespond \"Atollo: Caddy pool\" 200\n}\n"
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
    let lan = pool_conf(pool, 8081, Some(sock), true, &[lan_site], false).unwrap();
    assert!(
        lan.contains("http://:8081 {\n\trespond \"Atollo: Caddy pool\" 200\n}\n"),
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
    let reload: Vec<String> = reload_args(conf, Path::new("/tmp/Atollo/k/caddy-1/a.sock"))
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
            "unix//tmp/Atollo/k/caddy-1/a.sock"
        ]
    );
}

/// The admin endpoint is never a TCP port (any local program could read the
/// configuration and load another one): a Unix socket on every system,
/// written as Caddy wants it on Unix (`unix//tmp/…`) and on Windows
/// (`unix/C:/…`), owner-only; `admin off` without a usable socket path.
#[test]
fn admin_line_per_system() {
    let run = Path::new("/kit/data/servers/p");
    let line = |sock: Option<&Path>| global_options(sock, run).unwrap()[1].clone();
    assert_eq!(
        line(Some(Path::new("/tmp/Atollo/0a1b/caddy-9f8e7d6c/a.sock"))),
        "\tadmin \"unix//tmp/Atollo/0a1b/caddy-9f8e7d6c/a.sock|0600\""
    );
    assert_eq!(
        line(Some(Path::new(
            r"C:\Users\me\AppData\Local\Temp\Atollo\0a1b\caddy-9f8e7d6c\a.sock"
        ))),
        "\tadmin \"unix/C:/Users/me/AppData/Local/Temp/Atollo/0a1b/caddy-9f8e7d6c/a.sock|0600\""
    );
    assert_eq!(line(None), "\tadmin off");
    // Too long for `sun_path` (macOS: 104 bytes), or not writable: off.
    let long = format!("/tmp/{}/a.sock", "x".repeat(100));
    assert_eq!(line(Some(Path::new(&long))), "\tadmin off");
    assert_eq!(line(Some(Path::new("/tmp/{x}/a.sock"))), "\tadmin off");
}

/// A pool's socket is in a new owner-only folder with a random name, in the
/// first base of [`socket_bases`], removed when the pool ends; none without
/// any base (never in the kit's `data\tmp`, maybe on a disk without
/// permissions).
#[test]
fn admin_socket_in_a_private_folder() {
    let tmp = TempDir::new("caddy-admin-socket");
    // The system's own temporary folder: a test folder's path is too long
    // for a socket (it would rightly give none).
    let host = std::env::temp_dir();
    let kit = Kit::at_target(tmp.0.join("kit"), Target::detect()).with_host(Some(&host));
    kit.prepare().unwrap();
    let bases = socket_bases(&kit);
    let want = if kit.os().is_windows() {
        kit.system_temp()
    } else {
        // `/tmp` itself, never `/tmp/Atollo/<id>` another user may own.
        kit.system_temp().and_then(|t| t.parent()?.parent())
    };
    assert_eq!(bases.last().map(PathBuf::as_path), want);
    let Some(sock) = admin_socket(&kit, "caddy-2.11_php-8.4") else {
        let probe = bases[0]
            .join(socket_parent_name(current_uid()))
            .join("c-0123456789ab")
            .join("a.sock");
        assert!(!socket_path_usable(&probe), "{}", probe.display());
        return;
    };
    let dir = sock.path.parent().unwrap().to_path_buf();
    assert!(bases.iter().any(|b| sock.path.starts_with(b)));
    assert!(is_private_dir(&dir) && sock.path.ends_with("a.sock"));
    assert!(socket_path_usable(&sock.path));
    // Two pools, two folders.
    let other = admin_socket(&kit, "caddy-2.11_php-8.4").unwrap();
    assert_ne!(other.path, sock.path);
    drop(sock);
    assert!(!dir.exists(), "removed with the pool");
    let bare = Kit::at_target(tmp.0.join("bare"), Target::detect());
    bare.prepare().unwrap();
    if socket_bases(&bare).is_empty() {
        assert!(admin_socket(&bare, "p").is_none());
    }
}

/// The kit's folder of sockets is per user on Unix (a shared `/tmp` may be
/// the base): one that is there and not private (another user's, open to
/// others, a link) makes that base unused, never a place for a leaf whose
/// parent someone else could rename (review of D-111).
#[test]
fn socket_parent_is_the_users_own() {
    assert_eq!(socket_parent_name(Some(1000)), "atollo-caddy-1000");
    assert_eq!(socket_parent_name(None), "atollo-caddy");
    assert_eq!(current_uid().is_some(), cfg!(unix));
    let tmp = TempDir::new("caddy-socket-parent");
    let name = socket_parent_name(current_uid());
    let made = socket_parent(&tmp.0).unwrap();
    assert_eq!(made, tmp.0.join(&name));
    assert!(is_private_dir(&made));
    // Already there and ours: reused.
    assert_eq!(socket_parent(&tmp.0).unwrap(), made);
    // A link in its place: that base is not used.
    let other = TempDir::new("caddy-socket-parent-2");
    let target = other.0.join("elsewhere");
    fs::create_dir(&target).unwrap();
    junction(&other.0.join(&name), &target);
    assert!(socket_parent(&other.0).is_none());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let open = TempDir::new("caddy-socket-parent-3");
        let dir = open.0.join(&name);
        fs::create_dir(&dir).unwrap();
        fs::set_permissions(&dir, fs::Permissions::from_mode(0o1777)).unwrap();
        assert!(socket_parent(&open.0).is_none(), "open to others");
        // A base others can write in without the sticky bit: not used.
        let shared = TempDir::new("caddy-socket-base");
        fs::set_permissions(&shared.0, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(!base_trusted(&shared.0));
        assert!(socket_parent(&shared.0).is_none());
        assert!(!shared.0.join(&name).exists(), "nothing made there");
        fs::set_permissions(&shared.0, fs::Permissions::from_mode(0o1777)).unwrap();
        assert!(base_trusted(&shared.0), "sticky, as /tmp");
        assert!(socket_parent(&shared.0).is_some());
        let mode = fs::metadata(&made).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }
    // A socket checks its folder again before a client connects.
    let sock = PrivateSocket::in_leaf(&made, "c-1", "a.sock").unwrap();
    assert!(sock.still_private());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let leaf = sock.path.parent().unwrap();
        fs::set_permissions(leaf, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!sock.still_private());
    }
}

/// The leaf is made here and now, owner-only: a folder that is already
/// there (another user's, pre-created) or a link in its place is refused.
#[test]
fn private_leaf_refuses_what_exists() {
    let tmp = TempDir::new("caddy-private-leaf");
    let leaf = private_leaf(&tmp.0, "c-1").unwrap();
    assert!(is_private_dir(&leaf));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&leaf).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);
    }
    // Already there: refused, not reused.
    assert!(private_leaf(&tmp.0, "c-1").is_err());
    fs::create_dir(tmp.0.join("c-2")).unwrap();
    assert!(private_leaf(&tmp.0, "c-2").is_err());
    // A link in its place: refused, and never taken as private.
    let target = tmp.0.join("elsewhere");
    fs::create_dir(&target).unwrap();
    junction(&tmp.0.join("c-3"), &target);
    assert!(private_leaf(&tmp.0, "c-3").is_err());
    assert!(!is_private_dir(&tmp.0.join("c-3")));
    assert!(!is_private_dir(&tmp.0.join("missing")));
    // A folder others can enter is not private (Unix).
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let open = tmp.0.join("open");
        fs::create_dir(&open).unwrap();
        fs::set_permissions(&open, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!is_private_dir(&open));
        fs::set_permissions(&open, fs::Permissions::from_mode(0o700)).unwrap();
        assert!(is_private_dir(&open));
    }
}

/// Review of D-111: a failed `caddy reload` logs why (the end of its
/// stderr), never nothing.
#[test]
fn reload_failure_keeps_the_reason() {
    let err = b"2026/10/18 info using config\n\nError: adapting config using caddyfile: Caddyfile:12: unrecognized directive: bogus\n";
    assert_eq!(
        reload_failure(err),
        "2026/10/18 info using config | Error: adapting config using caddyfile: Caddyfile:12: unrecognized directive: bogus"
    );
    assert_eq!(reload_failure(b""), "no output");
    assert_eq!(reload_failure(b"\n \n"), "no output");
    let many: String = (1..=9).map(|i| format!("line {i}\n")).collect();
    assert_eq!(
        reload_failure(many.as_bytes()),
        "line 5 | line 6 | line 7 | line 8 | line 9"
    );
    let long = "x".repeat(3000);
    let short = reload_failure(long.as_bytes());
    assert_eq!(short.chars().count(), 2003);
    assert!(short.ends_with("..."));
}

/// Review of D-111: the leaves a crash left in the sockets' folder go at
/// the next socket; those with a live server, young ones and other names
/// stay.
#[cfg(unix)]
#[test]
fn stale_socket_leaves_are_swept() {
    use std::os::unix::net::UnixListener;
    use std::time::Duration;
    // A short base: macOS's temporary folder (/var/folders/…/T/) would put
    // the sockets past sun_path's 104 bytes.
    let base = std::path::PathBuf::from(format!("/tmp/acs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).unwrap();
    struct Gone(std::path::PathBuf);
    impl Drop for Gone {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let _gone = Gone(base.clone());
    let parent = base.join("p");
    private_leaf(&base, "p").unwrap();
    let dead = private_leaf(&parent, "c-dead").unwrap();
    drop(UnixListener::bind(dead.join("a.sock")).unwrap());
    assert!(
        dead.join("a.sock").exists(),
        "the file of a socket that ended"
    );
    let empty = private_leaf(&parent, "c-empty").unwrap();
    let live = private_leaf(&parent, "c-live").unwrap();
    let _listener = UnixListener::bind(live.join("a.sock")).unwrap();
    let other = private_leaf(&parent, "keep").unwrap();

    // Young leaves are never looked at.
    sweep_stale_leaves(&parent, Duration::from_secs(3600));
    assert!(dead.exists() && empty.exists() && live.exists());

    sweep_stale_leaves(&parent, Duration::ZERO);
    assert!(!dead.exists());
    assert!(!empty.exists());
    assert!(live.join("a.sock").exists(), "a server still listens");
    assert!(other.exists(), "not a leaf of the kit");
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
