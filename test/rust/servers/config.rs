use std::fs;
use std::path::Path;

use super::*;
use crate::kit::{tests::TempDir, Os};
use crate::projects::alias;
use crate::projects::fastcgi::FastCgi;
use crate::servers::config::apache_pool_conf;
use crate::servers::tests::*;
use crate::servers::*;
use crate::test_support::prepared_kit;

/// N-S1: a site whose document root is missing or a junction is not served
/// at all (never the project root in its place, with `.env` and `.git`).
#[test]
fn pool_skips_site_with_missing_or_linked_docroot() {
    let (tmp, kit) = prepared_kit("servers-docroot");
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();

    fs::create_dir_all(projects.join("good").join("public")).unwrap();
    // `missing` has no public\ folder: only its secrets at the root.
    fs::create_dir_all(projects.join("missing")).unwrap();
    fs::write(projects.join("missing").join(".env"), "DB_PASSWORD=x").unwrap();
    // `linked` has public\ as a junction to a folder outside the projects.
    fs::create_dir_all(projects.join("linked")).unwrap();
    let outside = tmp.0.join("outside");
    fs::create_dir_all(&outside).unwrap();
    crate::platform::safe_path::tests::junction(&projects.join("linked").join("public"), &outside);

    let pool = caddy_pool(
        "caddy-docroot",
        8089,
        vec![
            site("good", "public", 8089),
            site("missing", "public", 8089),
            site("linked", "public", 8089),
        ],
    );
    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let all = fs::read_to_string(&conf_path).unwrap();
    assert!(all.contains("http://good.localhost:8089 {"), "{all}");
    assert!(all.contains("good/public"), "{all}");
    assert!(
        !all.contains("missing") && !all.contains("linked") && !all.contains("outside"),
        "{all}"
    );
}

/// N-S5: a Host no project names (an IP from the LAN) gets Apache's own
/// page from the first VirtualHost, never the first project.
#[test]
fn apache_pool_conf_has_default_vhost_first() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n";
    let conf = apache_pool_conf(
        base,
        Path::new(r"C:\kit\data\servers\p"),
        8081,
        true,
        Os::Win32,
        None,
    )
    .unwrap();
    let default = conf
        .find("<VirtualHost *:8081>\r\n    ServerName localhost\r\n    DocumentRoot \"${SRVROOT}/htdocs\"\r\n</VirtualHost>")
        .expect("default vhost");
    let include = conf
        .find("IncludeOptional \"C:/kit/data/servers/p/vhosts/*.conf\"")
        .unwrap();
    assert!(default < include);
    assert_eq!(conf.matches("<VirtualHost").count(), 1);
    assert_eq!(conf.find("<VirtualHost"), Some(default));
    // A3 handoff: no PHP module, no `.php` sent as source.
    assert!(conf.contains("<IfModule !php_module>"));
    assert!(conf.contains(
        "<FilesMatch \"(?i)\\.(?:php|phtml|phar)$\">\r\n                Require all denied"
    ));
}

/// Out of LAN mode the pool's first VirtualHost refuses a Host nobody names
/// with 403, as a project's own Apache (D-98); its local names keep
/// Apache's page, and both come before the projects' sites.
#[test]
fn apache_pool_conf_refuses_foreign_hosts() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\r\nServerRoot \"${SRVROOT}\"\r\n";
    let conf = apache_pool_conf(
        base,
        Path::new(r"C:\kit\data\servers\p"),
        8081,
        false,
        Os::Win32,
        None,
    )
    .unwrap();
    let refused = conf
        .find("<VirtualHost *:8081>\r\n    ServerName refused.invalid\r\n    <Location \"/\">\r\n        Require all denied")
        .expect("refusing vhost");
    let local = conf
        .find("<VirtualHost *:8081>\r\n    ServerName localhost\r\n    ServerAlias 127.0.0.1 [::1]\r\n    DocumentRoot \"${SRVROOT}/htdocs\"\r\n</VirtualHost>")
        .expect("local vhost");
    let include = conf
        .find("IncludeOptional \"C:/kit/data/servers/p/vhosts/*.conf\"")
        .unwrap();
    assert_eq!(conf.find("<VirtualHost"), Some(refused));
    assert!(refused < local && local < include);
    assert_eq!(conf.matches("<VirtualHost").count(), 2);
    assert!(!conf.contains("*.localhost"));
}

/// BUG-V3-02: two web actions of one project in the same pool are two
/// site blocks with their own name in the Caddyfile; a stale vhost file goes.
#[test]
fn two_actions_of_one_project_get_distinct_vhosts() {
    let (_tmp, kit) = prepared_kit("servers-two-actions");
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: "legacy".into(),
                name: "Legacy".into(),
                engine: WebServerEngine::Caddy,
                server_line: None,
                php_line: None,
                port: Some(8087),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::create_dir_all(shop.join("docs")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [
            { "id": "npm:docs", "caddy": "docs", "server": "legacy" },
            { "id": "site", "caddy": "public", "server": "legacy" } ] }"#,
    )
    .unwrap();
    let view = servers.list("it", true);
    let pool = view.pools.iter().find(|p| p.id == "legacy").unwrap();
    let names: BTreeMap<&str, &str> = pool
        .sites
        .iter()
        .map(|s| (s.action.as_str(), s.server_name.as_str()))
        .collect();
    assert_eq!(names["site"], "shop.localhost");
    assert_eq!(names["npm:docs"], "npm-docs.shop.localhost");
    assert!(names.values().all(|n| alias::valid_domain(n)));
    let docs = pool.sites.iter().find(|s| s.action == "npm:docs").unwrap();
    assert_eq!(docs.url, "http://npm-docs.shop.localhost:8087");

    let vhosts = kit.data().join("servers").join("legacy").join("vhosts");
    fs::create_dir_all(&vhosts).unwrap();
    fs::write(vhosts.join("shop.conf"), "stale").unwrap();
    let conf = servers.write_pool_configs(pool, None, false).unwrap();
    assert!(!vhosts.join("shop.conf").exists());
    let text = fs::read_to_string(conf).unwrap();
    // Each block with its own name and its own folder.
    let block = |name: &str| {
        let start = text
            .find(&format!("http://{name}:8087 {{"))
            .unwrap_or_else(|| panic!("{name}: {text}"));
        let rest = &text[start..];
        let end = rest[1..].find("\nhttp://").map_or(rest.len(), |e| e + 1);
        rest[..end].to_owned()
    };
    let main = block("shop.localhost");
    assert!(
        main.contains("shop/public") && !main.contains("shop/docs"),
        "{main}"
    );
    let other = block("npm-docs.shop.localhost");
    assert!(
        other.contains("shop/docs") && !other.contains("shop/public"),
        "{other}"
    );
}

/// D-111: a Caddy pool's site block: its own name and its domains, its
/// document root, PHP with the start's key, the aliases checked on disk
/// only (no alias that was not resolved in the text, as Apache).
#[test]
fn caddy_site_block_of_a_pool() {
    let tmp = TempDir::new("servers-caddy-site");
    let projs = tmp.0.join("projects");
    fs::create_dir_all(projs.join("shop").join("public")).unwrap();
    fs::create_dir_all(projs.join("blog").join("uploads")).unwrap();
    let mut s = site("shop", "public", 8090);
    s.domains = vec!["api.shop.localhost".into(), "no;way.localhost".into()];
    s.aliases.insert("/blog".into(), "@blog/uploads".into());
    s.aliases.insert("/gone".into(), "nowhere".into());
    s.aliases.insert("/up".into(), "../blog/uploads".into());
    let aliases = crate::projects::caddy::resolve_aliases(&projs.join("shop"), &projs, &s.aliases);
    let fcgi = FastCgi::tcp(9333, "k".repeat(64));
    let doc = projs.join("shop").join("public");
    let block =
        crate::servers::config::caddy_site_conf(8090, false, &doc, &aliases, Some(&fcgi), &s, None)
            .unwrap()
            .join("\n");
    assert!(
        block.starts_with(
            "http://shop.localhost:8090, http://api.shop.localhost:8090 {\n\tbind 127.0.0.1\n"
        ),
        "{block}"
    );
    assert!(block.contains(&format!(
        "\t\troot * \"{}\"\n",
        crate::projects::apache::fwd(&doc)
    )));
    assert!(block.contains("\t\t\tenv ATOLLO_FCGI_KEY {env.ATOLLO_FCGI_KEY}\n"));
    assert!(!block.contains(&"k".repeat(64)), "the key is never written");
    assert!(block.contains("\t\thandle_path /blog/* {\n"));
    assert!(block.contains(&format!(
        "\t\t\troot * \"{}/blog/uploads\"\n",
        crate::projects::apache::fwd(&projs)
    )));
    for absent in ["/gone", "redir /up ", "/up/*", "nowhere", "no;way", ".."] {
        assert!(!block.contains(absent), "{absent}: {block}");
    }
    // A folder Caddy cannot take is not served at all.
    let bad = Path::new("/kit/{x}/public");
    assert!(
        crate::servers::config::caddy_site_conf(8090, false, bad, &[], None, &s, None).is_none()
    );
}

/// A Caddy pool's files: one Caddyfile with the sites it serves, the PHP
/// key, its admin endpoint, the user's snippets only when there are some.
#[test]
fn caddy_pool_writes_one_caddyfile() {
    let (tmp, kit) = prepared_kit("servers-caddy-pool");
    let servers = WebServers::new(kit.clone());
    for p in ["shop", "blog"] {
        fs::create_dir_all(kit.projects().join(p).join("public")).unwrap();
    }
    let pool = caddy_pool(
        "caddy-2.11_php-8.4",
        8091,
        vec![site("shop", "public", 8091), site("blog", "public", 8091)],
    );
    let fcgi = FastCgi::tcp(9444, "z".repeat(64));
    let sock = tmp.0.join("host").join("caddy-1").join("a.sock");
    let conf = servers
        .write_pool_configs_with(&pool, Some(&fcgi), false, Some(&sock))
        .unwrap();
    assert_eq!(conf.file_name().unwrap(), "Caddyfile");
    let text = fs::read_to_string(&conf).unwrap();
    if crate::projects::caddy::socket_path_usable(&sock) {
        assert!(text.contains(&format!(
            "\tadmin \"{}|0600\"\n",
            crate::projects::caddy::admin_address(&sock)
        )));
    }
    assert!(!text.contains("admin 127.0.0.1") && !text.contains(&"z".repeat(64)));
    assert!(text.contains("http://shop.localhost:8091 {\n"));
    assert!(text.contains("http://blog.localhost:8091 {\n"));
    assert!(text.contains("php_fastcgi 127.0.0.1:9444 {"));
    assert!(text.contains("respond 421"));
    assert!(!text.contains("import "));
    let snippets = kit.data().join("servers").join(&pool.id).join("conf.d");
    fs::write(snippets.join("mine.caddy"), "").unwrap();
    let text = fs::read_to_string(servers.write_pool_configs(&pool, None, false).unwrap()).unwrap();
    assert!(
        text.contains("\tadmin off\n") && text.contains("import \""),
        "{text}"
    );
    assert!(text.contains("respond @php 503"));
}

/// BUG-V5-05: a project's own `caddy\conf.d` is imported in its site block
/// of the pool once it holds a `.caddy` file, not before, and only there.
#[test]
fn caddy_pool_imports_the_project_conf_d() {
    let (_tmp, kit) = prepared_kit("servers-caddy-project-conf-d");
    let servers = WebServers::new(kit.clone());
    for p in ["shop", "blog"] {
        fs::create_dir_all(kit.projects().join(p).join("public")).unwrap();
    }
    let pool = caddy_pool(
        "caddy-conf-d",
        8092,
        vec![site("shop", "public", 8092), site("blog", "public", 8092)],
    );
    let conf_d = kit.project_data().join("shop").join("caddy").join("conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    fs::write(conf_d.join("notes.txt"), "").unwrap();
    let text = fs::read_to_string(servers.write_pool_configs(&pool, None, false).unwrap()).unwrap();
    assert!(!text.contains("import "), "{text}");

    fs::write(
        conf_d.join("cors.caddy"),
        "header Access-Control-Allow-Origin *",
    )
    .unwrap();
    let text = fs::read_to_string(servers.write_pool_configs(&pool, None, false).unwrap()).unwrap();
    let import = format!(
        "http://shop.localhost:8092 {{\n\tbind 127.0.0.1\n\timport \"{}/*.caddy\"\n\troute {{",
        crate::projects::apache::fwd(&conf_d)
    );
    assert!(text.contains(&import) && !import.contains('\\'), "{text}");
    assert_eq!(text.matches("import ").count(), 1, "{text}");
}

/// BUG-V5-05: an Apache pool's VirtualHost of a site includes the
/// project's own `apache\conf.d` once it is there.
#[test]
fn apache_pool_includes_the_project_conf_d() {
    let (_tmp, kit) = prepared_kit("servers-apache-project-conf-d");
    let servers = WebServers::new(kit.clone());
    let conf = crate::servers::config::component_dir(&kit, "apache", None).join("conf");
    fs::create_dir_all(&conf).unwrap();
    fs::write(conf.join("httpd.conf"), "Define SRVROOT \"x\"\r\n").unwrap();
    fs::create_dir_all(kit.projects().join("shop").join("public")).unwrap();
    let mut pool = caddy_pool("apache-conf-d", 8093, vec![site("shop", "public", 8093)]);
    pool.engine = WebServerEngine::Apache;
    let vhost = kit
        .data()
        .join("servers")
        .join(&pool.id)
        .join("vhosts")
        .join(crate::servers::pools::vhost_file(&pool.sites[0]));

    servers.write_pool_configs(&pool, None, false).unwrap();
    let text = fs::read_to_string(&vhost).unwrap();
    assert!(!text.contains("Include"), "{text}");

    let conf_d = kit
        .project_data()
        .join("shop")
        .join("apache")
        .join("conf.d");
    fs::create_dir_all(&conf_d).unwrap();
    servers.write_pool_configs(&pool, None, false).unwrap();
    let text = fs::read_to_string(&vhost).unwrap();
    let include = format!(
        "    IncludeOptional \"{}/*.conf\"\r\n</VirtualHost>",
        crate::projects::apache::fwd(&conf_d)
    );
    assert!(text.contains(&include), "{text}");
}

/// D-116: an Apache pool off Windows has PHP through its php-fpm and its
/// run-time files in the pool's folder.
#[test]
fn apache_pool_conf_off_windows() {
    let base = "Define SRVROOT \"${CP_APACHE_FWD}\"\nServerRoot \"${SRVROOT}\"\n";
    let fcgi = FastCgi::tcp(9002, "k");
    let conf = apache_pool_conf(
        base,
        Path::new("/kit/data/servers/p"),
        8090,
        false,
        Os::Linux,
        Some(&fcgi),
    )
    .unwrap();
    assert!(conf.contains("DefaultRuntimeDir \"/kit/data/servers/p\""));
    assert!(conf.contains("SetHandler \"proxy:fcgi://127.0.0.1:9002\""));
    assert!(!conf.contains("php8apache2_4.dll"));
    let none = apache_pool_conf(
        base,
        Path::new("/kit/data/servers/p"),
        8090,
        false,
        Os::Linux,
        None,
    )
    .unwrap();
    assert!(!none.contains("SetHandler") && none.contains("<IfModule !php_module>"));
}
