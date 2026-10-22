use std::fs;

use super::*;
use crate::detect::{set_site_with_options, SiteKind, SiteOptions};
use crate::env::KitEnv;
use crate::servers::tests::*;
use crate::servers::*;
use crate::test_support::{fake_server, prepared_kit};

/// N-S4c: a fresh list drops the ports of pools that no longer exist; the
/// cached poll leaves `servers.json` as it is.
#[test]
fn fresh_list_prunes_ports_of_gone_pools() {
    let (_tmp, kit) = prepared_kit("servers-prune-ports");
    let servers = WebServers::new(kit.clone());
    let mut ports = BTreeMap::new();
    ports.insert("apache-2.2-php-5.6".to_owned(), 8123);
    ports.insert("keep".to_owned(), 8097);
    let store = StoredServers {
        profiles: vec![WebServerProfileInput {
            id: "keep".into(),
            name: "Keep".into(),
            engine: WebServerEngine::Caddy,
            server_line: None,
            php_line: None,
            port: Some(8097),
            keep_alive: true,
            autostart: false,
        }],
        ports,
    };
    save_store(&kit, &store).unwrap();

    servers.list("it", false);
    assert_eq!(load_store(&kit).ports.len(), 2);
    servers.list("it", true);
    let ports = load_store(&kit).ports;
    assert_eq!(ports.keys().collect::<Vec<_>>(), ["keep"]);
    assert_eq!(ports["keep"], 8097);
}

/// A port kept in `servers.json` by a pool gone since does not block a
/// profile: saving it drops the ports of the pools that no longer exist.
#[test]
fn save_profile_takes_the_port_of_a_gone_pool() {
    let (_tmp, kit) = prepared_kit("servers-gone-port");
    let servers = WebServers::new(kit.clone());
    let mut store = StoredServers::default();
    store.ports.insert("apache-2.2_php-5.6".to_owned(), 8124);
    save_store(&kit, &store).unwrap();
    servers
        .save_profile(
            WebServerProfileInput {
                id: "mine".into(),
                name: "Mine".into(),
                engine: WebServerEngine::Caddy,
                server_line: None,
                php_line: None,
                port: Some(8124),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let ports = load_store(&kit).ports;
    assert_eq!(ports.keys().collect::<Vec<_>>(), ["mine"]);

    // Opening the app drops them too; the port of a pool of Nginx's waits
    // for Caddy (no Caddy in this kit) to move to its pool.
    let mut store = load_store(&kit);
    store.ports.insert("caddy-2.10".to_owned(), 8125);
    store.ports.insert("nginx-1.20".to_owned(), 8126);
    save_store(&kit, &store).unwrap();
    servers.autostart(&KitEnv::default(), "it");
    let ports = load_store(&kit).ports;
    assert!(!ports.contains_key("caddy-2.10"));
    assert_eq!(ports.get("nginx-1.20"), Some(&8126));
}

/// A `servers.json` that cannot be read is never written again as if
/// empty (SEC-V6-01): the page shows no profile, saving or removing one is
/// refused, the file stays as it is.
#[test]
fn unreadable_servers_json_is_never_overwritten() {
    let (_tmp, kit) = prepared_kit("servers-damaged");
    let servers = WebServers::new(kit.clone());
    let damaged = r#"{"profiles": [{"id": "mine", "#;
    fs::create_dir_all(store_path(&kit).parent().unwrap()).unwrap();
    fs::write(store_path(&kit), damaged).unwrap();
    servers.scan("it", true);
    let input = WebServerProfileInput {
        id: "other".into(),
        name: "Other".into(),
        engine: WebServerEngine::Caddy,
        server_line: None,
        php_line: None,
        port: Some(8124),
        keep_alive: true,
        autostart: false,
    };
    let e = servers.save_profile(input, "it").unwrap_err();
    assert!(format!("{e:#}").contains("servers.json"), "{e:#}");
    assert!(servers.remove_profile("mine").is_err());
    assert_eq!(fs::read_to_string(store_path(&kit)).unwrap(), damaged);
}

/// GAP-RUN: the pool of a project's site is the profile it was assigned to.
#[test]
fn pool_of_finds_assigned_project() {
    let (_tmp, kit) = prepared_kit("servers-pool-of");
    let servers = WebServers::new(kit.clone());
    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Legacy".into(),
                engine: WebServerEngine::Caddy,
                server_line: None,
                php_line: None,
                port: Some(8096),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);

    servers
        .assign_project(
            "shop",
            "public",
            WebServerEngine::Caddy,
            Some("legacy".into()),
            &KitEnv::default(),
            "it",
        )
        .unwrap();
    let (pool, site) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.id, "legacy");
    assert_eq!(site.url, "http://shop.localhost:8096");
    assert_eq!(servers.pool_of("shop", "other", "it").map(|p| p.0.id), None);
    assert_eq!(servers.pool_of("nope", "site", "it").map(|p| p.0.id), None);

    // I3: moved to another profile behind the cache's back (no
    // invalidation): the cached scan still says "legacy", pool_of does not.
    servers
        .save_profile(
            WebServerProfileInput {
                id: "modern".into(),
                name: "Modern".into(),
                engine: WebServerEngine::Caddy,
                server_line: None,
                php_line: None,
                port: Some(8091),
                keep_alive: true,
                autostart: false,
            },
            "it",
        )
        .unwrap();
    assert!(servers
        .list("it", false)
        .pools
        .iter()
        .any(|p| p.id == "legacy" && p.sites.iter().any(|s| s.project == "shop")));
    set_site_with_options(
        &shop,
        "public",
        SiteKind::Caddy,
        &SiteOptions {
            server: Some("modern".into()),
            ..SiteOptions::default()
        },
        "it",
    )
    .unwrap();
    let cached = servers.list("it", false);
    assert!(cached
        .pools
        .iter()
        .any(|p| p.id == "legacy" && p.sites.iter().any(|s| s.project == "shop")));
    let (pool, site) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.id, "modern");
    assert_eq!(site.url, "http://shop.localhost:8091");
    // Not running: no live port to attach a run to.
    assert_eq!(servers.running_port("modern"), None);
    assert!(!servers.is_running("modern"));
}

/// Defect 7 of the pools (D-101): a project is put in the pools once per site:
/// the detected `apache` and `caddy` give way to the sites of `panel.json`,
/// and only the first of them is taken.
#[test]
fn pooled_actions_drop_detected_duplicates() {
    use crate::detect::{Action, Kind};
    let web = |id: &str, apache: bool, custom: bool| {
        let mut a = Action::for_test(id, "", Kind::Web);
        if apache {
            a.apache_dir = Some("public".into());
        } else {
            a.caddy_dir = Some("public".into());
        }
        a.custom = custom;
        a
    };
    let ids = |actions: &[Action]| -> Vec<String> {
        pooled_actions(actions)
            .into_iter()
            .map(|a| a.id.clone())
            .collect()
    };
    let cmd = Action::for_test("php:server", "php -S", Kind::Web);
    let detected = [web("apache", true, false), web("caddy", false, false), cmd];
    assert_eq!(ids(&detected), ["apache"]);
    let mut with_site = detected.to_vec();
    with_site.push(web("site", false, true));
    with_site.push(web("docs", true, true));
    assert_eq!(ids(&with_site), ["site", "docs"]);
    assert!(ids(&[Action::for_test("dev", "npm run dev", Kind::Web)]).is_empty());
}

/// Defect 6: two pools never share a port: a profile is refused the port
/// of another one, and a new pool never falls back on a pool's port.
#[test]
fn ports_are_unique_among_pools() {
    let (_tmp, kit) = prepared_kit("servers-unique-ports");
    let servers = WebServers::new(kit.clone());
    let profile = |id: &str, port: u16| WebServerProfileInput {
        id: id.into(),
        name: id.into(),
        engine: WebServerEngine::Caddy,
        server_line: None,
        php_line: None,
        port: Some(port),
        keep_alive: true,
        autostart: false,
    };
    servers.save_profile(profile("one", 8085), "it").unwrap();
    let err = servers
        .save_profile(profile("two", 8085), "it")
        .unwrap_err()
        .to_string();
    assert!(err.contains("8085") && err.contains("one"), "{err}");
    // The same profile saved again keeps its port.
    servers.save_profile(profile("one", 8085), "it").unwrap();
    servers.save_profile(profile("two", 8084), "it").unwrap();

    // Every port of the range taken by a pool but one: that one, even
    // when another program holds it.
    let mut store = load_store(&kit);
    for (i, p) in (POOL_PORT_FROM..=POOL_PORT_TO).enumerate() {
        if p != 8500 {
            store.ports.insert(format!("p{i}"), p);
        }
    }
    let mut dirty = false;
    assert_eq!(
        servers.assign_or_get_port(&mut store, &mut dirty, "new", None),
        8500
    );
    assert!(dirty);

    // A servers.json of before: an automatic pool stored on the port a
    // profile was given by hand gets a port of its own.
    let mut store = load_store(&kit);
    store.ports.insert("one".to_owned(), 8085);
    store.ports.insert("auto".to_owned(), 8085);
    let mut dirty = false;
    let p = servers.assign_or_get_port(&mut store, &mut dirty, "auto", None);
    assert_ne!(p, 8085);
    assert!(dirty);
    assert_eq!(
        servers.assign_or_get_port(&mut store, &mut dirty, "one", Some(8085)),
        8085
    );
}

/// BUG-V5-05: a pool is never saved on the port of a database version,
/// of a project's action or the editor's, and the owner is named; a
/// database version is never set on a pool's.
#[test]
fn pool_ports_are_not_those_of_databases_projects_or_editor() {
    let (_tmp, kit) = prepared_kit("servers-ports-taken");
    fs::write(
        kit.app_dir().join("services.json"),
        r#"{ "mysql": { "ports": { "8.4": 3307, "8.0": 8081 } } }"#,
    )
    .unwrap();
    let shop = kit.project_data().join("shop");
    fs::create_dir_all(&shop).unwrap();
    fs::write(shop.join("ports.json"), r#"{ "ports": { "site": 7150 } }"#).unwrap();
    let servers = WebServers::new(kit.clone());
    let profile = |port: u16| WebServerProfileInput {
        id: "legacy".into(),
        name: "Legacy".into(),
        engine: WebServerEngine::Caddy,
        server_line: None,
        php_line: None,
        port: Some(port),
        keep_alive: true,
        autostart: false,
    };
    let refused = |port: u16| {
        servers
            .save_profile(profile(port), "en")
            .unwrap_err()
            .to_string()
    };
    assert_eq!(
        refused(3307),
        "Port 3307 already belongs to database MySQL 8.4: choose another one."
    );
    assert_eq!(refused(7010), "Ports 7000 to 7099 belong to the editor");
    assert_eq!(
        refused(7150),
        "Port 7150 already belongs to project shop → site: choose another one."
    );
    assert!(load_store(&kit).profiles.is_empty(), "nothing saved");

    servers.save_profile(profile(8095), "en").unwrap();
    servers.save_profile(profile(8095), "en").unwrap();
    let taken = crate::ports::ports_taken(&kit);
    assert_eq!(taken[&8095], crate::ports::Owner::Pool("legacy".into()));
    // A new pool's port is none of theirs either.
    let (mut store, mut dirty) = (load_store(&kit), false);
    let auto = servers.assign_or_get_port(&mut store, &mut dirty, "auto", None);
    assert!(auto != 8081 && auto != 8095, "{auto}");

    // The other way round: the database version on the pool's port.
    let services = crate::db::services::Services::new(kit.clone());
    let err = services.set("mysql", Some(8095), false, "en").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Port 8095 already belongs to server legacy: choose another one."
    );
}

/// D-101: an older `servers.json` (no options) keeps its profiles
/// alive and starts none with the app; the options are saved and read back.
/// A profile of Nginx's of before reads as a Caddy one.
#[test]
fn profile_options_default_and_round_trip() {
    let old: StoredServers = serde_json::from_str(
        r#"{ "profiles": [ { "id": "legacy", "name": "Legacy", "engine": "nginx",
            "port": 8090 } ], "ports": { "legacy": 8090 } }"#,
    )
    .unwrap();
    assert_eq!(old.profiles[0].engine, WebServerEngine::Caddy);
    assert!(old.profiles[0].keep_alive);
    assert!(!old.profiles[0].autostart);

    let (_tmp, kit) = prepared_kit("servers-options");
    let servers = WebServers::new(kit.clone());
    let mut profile = old.profiles[0].clone();
    profile.keep_alive = false;
    profile.autostart = true;
    servers.save_profile(profile, "it").unwrap();
    let text = fs::read_to_string(store_path(&kit)).unwrap();
    assert!(text.contains("\"keepAlive\": false") && text.contains("\"autostart\": true"));
    assert!(text.contains("\"engine\": \"caddy\""), "{text}");
    let view = servers.list("it", true);
    let saved = view.pools[0].profile.as_ref().unwrap();
    assert!(!saved.keep_alive && saved.autostart);
    assert_eq!(
        saved.server_line, None,
        "the profile as saved, not resolved"
    );

    // With the app: tried, and with no Caddy in this kit only logged.
    servers.autostart(&KitEnv::default(), "it");
    assert!(!servers.is_running("legacy"));
}

/// BUG-V3-02: names that give the same label in one pool stay distinct.
#[test]
fn name_sites_keeps_names_unique() {
    let mut sites = vec![
        site("My Shop", "public", 8080),
        site("my-shop", "public", 8080),
        site("___", "public", 8080),
    ];
    sites[1].action = "web".into();
    name_sites(&mut sites, 8080);
    let names: Vec<&str> = sites.iter().map(|s| s.server_name.as_str()).collect();
    assert_eq!(
        names,
        ["my-shop.localhost", "my-shop-2.localhost", "site.localhost"]
    );
    assert_eq!(sites[1].url, "http://my-shop-2.localhost:8080");
    assert_eq!(host_label("Ünï Côde_42"), "n-c-de-42");
}

/// D-101: an automatic pool lists (and serves) the sites the
/// projects chose in `panel.json` and those started, not every PHP project
/// that was only offered a site (Caddy since D-111).
#[test]
fn automatic_pools_list_chosen_or_started_sites() {
    let (_tmp, kit) = prepared_kit("servers-chosen-sites");
    fake_server(&kit, "caddy", "2.11.7");
    let servers = WebServers::new(kit.clone());
    let projects = kit.projects();
    // Offered Caddy next to `php -S`.
    let plain = projects.join("plain").join("public");
    fs::create_dir_all(&plain).unwrap();
    fs::write(plain.join("index.php"), "<?php").unwrap();
    // A site written in panel.json.
    let shop = projects.join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(shop.join("public").join("index.php"), "<?php").unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "caddy": "public" } ] }"#,
    )
    .unwrap();

    let projects_of = |pools: &[WebServerPool]| -> Vec<String> {
        pools
            .iter()
            .flat_map(|p| p.sites.iter().map(|s| s.project.clone()))
            .collect()
    };
    let all = servers.list_all("it", true);
    assert_eq!(projects_of(&all.pools), ["plain", "shop"]);
    let shown = servers.list("it", true);
    assert_eq!(projects_of(&shown.pools), ["shop"]);
    // Every project goes to the same pool: it is not left out with them.
    let (pool, _) = servers.pool_of("plain", "caddy", "it").unwrap();
    assert_eq!(pool.engine, WebServerEngine::Caddy);
    assert!(shown.pools.iter().any(|p| p.id == pool.id));

    let pool_dir = kit.data().join("servers").join(&pool.id);
    // The sites written in the pool's Caddyfile.
    let served = || -> Vec<&'static str> {
        let text = fs::read_to_string(pool_dir.join("Caddyfile")).unwrap();
        ["plain", "shop"]
            .into_iter()
            .filter(|p| text.contains(&format!("http://{p}.localhost:{}", pool.port)))
            .collect()
    };
    servers.write_pool_configs(&pool, None, false).unwrap();
    assert_eq!(served(), ["shop"]);

    // Started by its project: listed and served.
    servers
        .started
        .lock()
        .unwrap()
        .insert(("plain".into(), "caddy".into()));
    assert_eq!(
        projects_of(&servers.list("it", true).pools),
        ["plain", "shop"]
    );
    servers.write_pool_configs(&pool, None, false).unwrap();
    assert_eq!(served(), ["plain", "shop"]);

    // Stopped: out again (the pool is not running, nothing to reload).
    servers
        .detach(&pool.id, "plain", "caddy", &KitEnv::default(), "it")
        .unwrap();
    assert_eq!(projects_of(&servers.list("it", true).pools), ["shop"]);

    // A chosen site whose project stopped: still listed, marked detached.
    let shop_site = |servers: &WebServers| {
        servers
            .list("it", false)
            .pools
            .into_iter()
            .flat_map(|p| p.sites)
            .find(|s| s.project == "shop")
            .unwrap()
    };
    assert!(!shop_site(&servers).detached);
    servers
        .detach(&pool.id, "shop", "site", &KitEnv::default(), "it")
        .unwrap();
    let shop = shop_site(&servers);
    assert!(shop.detached);
    assert_eq!(shop.server_name, "shop.localhost");
}

/// D-101: a site marked `dedicated` keeps its own server (in no
/// pool); a new profile never takes an id a site's `server` means otherwise.
#[test]
fn dedicated_sites_and_reserved_profile_ids() {
    let (_tmp, kit) = prepared_kit("servers-dedicated");
    let servers = WebServers::new(kit.clone());
    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "apache": "public", "server": "dedicated" } ] }"#,
    )
    .unwrap();
    assert_eq!(servers.pool_of("shop", "site", "it").map(|p| p.0.id), None);

    let profile = |name: &str| WebServerProfileInput {
        id: String::new(),
        name: name.into(),
        engine: WebServerEngine::Apache,
        server_line: None,
        php_line: None,
        port: None,
        keep_alive: true,
        autostart: false,
    };
    for (name, id) in [
        ("Apache 2.4", "apache-2.4-profile"),
        ("Dedicated", "dedicated-profile"),
        ("Auto", "auto-profile"),
        ("Apache test", "apache-test"),
    ] {
        servers.save_profile(profile(name), "it").unwrap();
        assert!(
            load_store(&kit).profiles.iter().any(|p| p.id == id),
            "{name}"
        );
    }
}

/// The pools of the page: an automatic pool with no site left is not
/// shown, unless it runs or it is the only pool (the one to start).
#[test]
fn shown_pools_drop_empty_automatic_ones() {
    let mut hidden = site("plain", "public", 8081);
    hidden.chosen = false;
    let started = BTreeSet::new();
    let pools = vec![
        caddy_pool("caddy-2.11", 8081, vec![hidden.clone()]),
        caddy_pool("caddy-2.10", 8082, vec![site("shop", "public", 8082)]),
    ];
    let ids =
        |pools: &[WebServerPool]| -> Vec<String> { pools.iter().map(|p| p.id.clone()).collect() };
    assert_eq!(ids(&shown_pools(pools.clone(), &started)), ["caddy-2.10"]);
    let mut running = pools.clone();
    running[0].running = true;
    let shown = shown_pools(running, &started);
    assert_eq!(ids(&shown), ["caddy-2.11", "caddy-2.10"]);
    assert!(shown[0].sites.is_empty());
    let only = shown_pools(vec![pools[0].clone()], &started);
    assert_eq!(ids(&only), ["caddy-2.11"]);
}

/// SC-2: automatic ids name their lines (`apache-2.4_php-7.4`); the ids of
/// before are read too, and a profile's id is not one.
#[test]
fn auto_pool_ids_name_their_lines() {
    let apache = WebServerEngine::Apache;
    assert_eq!(
        auto_pool_id(apache, Some("2.4"), Some("7.4")),
        "apache-2.4_php-7.4"
    );
    assert_eq!(
        auto_pool_id(WebServerEngine::Caddy, Some("2.11"), None),
        "caddy-2.11"
    );
    assert_eq!(auto_pool_id(apache, None, None), "apache-default");
    let lines = |s: Option<&str>, p: Option<&str>| {
        Some((apache, s.map(str::to_owned), p.map(str::to_owned)))
    };
    assert_eq!(
        parse_auto_pool_id("apache-2.4_php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(
        parse_auto_pool_id("apache-2.4-php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(
        parse_auto_pool_id("apache-default_php-8.4"),
        lines(None, Some("8.4"))
    );
    assert_eq!(
        canonical_auto_pool("apache-2.4_php-7.4"),
        lines(Some("2.4"), Some("7.4"))
    );
    assert_eq!(canonical_auto_pool("apache-2.4-php-7.4"), None);
    for other in [
        "legacy",
        "apache",
        "apache-test",
        "caddy-2.11-php-x",
        // Nginx is no engine of the pools any more (its ports: `migrate`).
        "nginx-1.27",
        "iis-1.0",
    ] {
        assert_eq!(parse_auto_pool_id(other), None, "{other}");
    }
}

/// SC-2: the ports stored under the ids of before move to the new ones (a
/// port already stored there wins); a profile's id stays as it is, and so
/// does a pool of Nginx's (moved to Caddy's by `migrate`, not here).
#[test]
fn ports_move_to_the_new_automatic_ids() {
    let mut store = StoredServers::default();
    store.ports.insert("apache-2.4-php-7.4".into(), 8081);
    store.ports.insert("caddy-2.11-php-8.4".into(), 8082);
    store.ports.insert("caddy-2.11_php-8.4".into(), 8090);
    store.ports.insert("caddy-2.10".into(), 8083);
    store.ports.insert("nginx-1.27-php-8.4".into(), 8085);
    store.ports.insert("apache-2.2-php-5.6".into(), 8084);
    store.profiles.push(WebServerProfileInput {
        id: "apache-2.2-php-5.6".into(),
        name: "Old".into(),
        engine: WebServerEngine::Apache,
        server_line: None,
        php_line: None,
        port: Some(8084),
        keep_alive: true,
        autostart: false,
    });
    assert!(migrate_port_ids(&mut store));
    let ports: Vec<(&str, u16)> = store.ports.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    assert_eq!(
        ports,
        [
            ("apache-2.2-php-5.6", 8084),
            ("apache-2.4_php-7.4", 8081),
            ("caddy-2.10", 8083),
            ("caddy-2.11_php-8.4", 8090),
            ("nginx-1.27-php-8.4", 8085),
        ]
    );
    assert!(!migrate_port_ids(&mut store));
}

/// SC-2: a running automatic pool no project goes to any more stays a pool
/// of the list (not an orphan), rebuilt from its id; a site naming an
/// automatic pool in `panel.json` (old id too) goes to the lines it names;
/// a site of Nginx's (`"nginx"` in `panel.json`) is a Caddy one.
#[test]
fn automatic_pools_come_back_from_their_ids() {
    let (_tmp, kit) = prepared_kit("servers-stable-ids");
    fake_server(&kit, "caddy", "2.11.7");
    let servers = WebServers::new(kit.clone());
    fake_running(&servers, caddy_pool("caddy-2.9", 8083, Vec::new()));
    let view = servers.list("it", true);
    let old = view.pools.iter().find(|p| p.id == "caddy-2.9").unwrap();
    assert!(!old.orphan && old.running);
    assert_eq!(old.server_line.as_deref(), Some("2.9"));
    assert!(load_store(&kit).ports.contains_key("caddy-2.9"));
    servers.stop("caddy-2.9");

    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(
        shop.join("panel.json"),
        r#"{ "actions": [ { "id": "site", "nginx": "public", "server": "caddy-2.10-php-8.3" } ] }"#,
    )
    .unwrap();
    let (pool, _) = servers.pool_of("shop", "site", "it").unwrap();
    assert_eq!(pool.engine, WebServerEngine::Caddy);
    assert_eq!(pool.id, "caddy-2.10_php-8.3");
    assert_eq!(pool.server_line.as_deref(), Some("2.10"));
    assert_eq!(pool.php_line.as_deref(), Some("8.3"));
    assert!(!pool.manual);
}
