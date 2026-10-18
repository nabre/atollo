use std::fs;
use std::path::Path;

use serde_json::{json, Value};

use super::*;
use crate::kit::{tests::TempDir, Arch, Kit};
use crate::servers::pools::{load_store, store_path};
use crate::servers::{WebServerEngine, WebServers};

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

fn write_json(path: &Path, value: &Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// D-119: a kit with a profile of Nginx and two sites of Nginx (one in
/// `panel.json` on an automatic pool, one in the old `pannello.json` on the
/// profile). After the start everything is Caddy's, each file copied first
/// to `data\app\pre-caddy`; a second start changes nothing; the port of the
/// pool of Nginx waits for Caddy and is never pruned meanwhile.
#[test]
fn nginx_pools_and_sites_become_caddy() {
    let tmp = TempDir::new("nginx-to-caddy");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();

    let servers_json = json!({
        "profiles": [
            { "id": "legacy", "name": "Legacy", "engine": "nginx", "serverLine": "1.27",
              "phpLine": "8.4", "port": 8090, "keepAlive": true, "autostart": false },
            { "id": "web", "name": "Web", "engine": "apache", "serverLine": "2.4" }
        ],
        "ports": { "nginx-1.27_php-8.4": 8085, "legacy": 8090, "web": 8091 }
    });
    write_json(&store_path(&kit), &servers_json);
    let original_store = fs::read_to_string(store_path(&kit)).unwrap();

    let shop = kit.projects().join("shop");
    fs::create_dir_all(shop.join("public")).unwrap();
    fs::write(shop.join("public").join("index.php"), "<?php").unwrap();
    let shop_panel = json!({
        "name": "Shop",
        "default": "site",
        "actions": [
            { "id": "site", "label": "sito web Nginx (public)", "nginx": "public",
              "server": "nginx-1.27_php-8.4", "domains": ["shop.localhost"] },
            { "id": "dev", "label": "Dev", "command": "npm run dev" }
        ]
    });
    write_json(&shop.join("panel.json"), &shop_panel);
    let original_shop = fs::read_to_string(shop.join("panel.json")).unwrap();

    let blog = kit.projects().join("blog");
    fs::create_dir_all(&blog).unwrap();
    fs::write(blog.join("index.php"), "<?php").unwrap();
    write_json(
        &blog.join("pannello.json"),
        &json!({ "azioni": [ { "id": "site", "etichetta": "Nginx-Website (.)",
                               "nginx": ".", "server": "legacy" } ] }),
    );

    // Untouched: a project with no Nginx.
    let plain = kit.projects().join("plain");
    fs::create_dir_all(&plain).unwrap();
    let plain_panel = json!({ "actions": [ { "id": "site", "caddy": "." } ] });
    write_json(&plain.join("panel.json"), &plain_panel);
    let original_plain = fs::read_to_string(plain.join("panel.json")).unwrap();

    run(&kit);

    let store = read_json(&store_path(&kit));
    assert_eq!(store["profiles"][0]["engine"], "caddy");
    assert!(store["profiles"][0].get("serverLine").is_none());
    assert_eq!(store["profiles"][0]["phpLine"], "8.4");
    assert_eq!(store["profiles"][1]["engine"], "apache");
    assert_eq!(store["profiles"][1]["serverLine"], "2.4");
    // No Caddy yet: the port of Nginx's pool waits under its id.
    assert_eq!(store["ports"]["nginx-1.27_php-8.4"], 8085);

    let site = &read_json(&shop.join("panel.json"))["actions"][0];
    assert_eq!(site["caddy"], "public");
    assert!(site.get("nginx").is_none());
    assert_eq!(site["label"], "sito web Caddy (public)");
    assert!(site.get("server").is_none(), "{site}");
    assert_eq!(site["domains"][0], "shop.localhost");
    let site = &read_json(&blog.join("pannello.json"))["azioni"][0];
    assert_eq!(site["caddy"], ".");
    assert_eq!(site["etichetta"], "Caddy-Website (.)");
    assert_eq!(site["server"], "legacy");
    assert_eq!(
        fs::read_to_string(plain.join("panel.json")).unwrap(),
        original_plain
    );

    // The copies first, as the files were.
    let copies = kit.app_dir().join("pre-caddy");
    assert_eq!(
        fs::read_to_string(copies.join("servers.json")).unwrap(),
        original_store
    );
    assert_eq!(
        fs::read_to_string(copies.join("projects").join("shop").join("panel.json")).unwrap(),
        original_shop
    );
    assert!(copies
        .join("projects")
        .join("blog")
        .join("pannello.json")
        .is_file());
    assert!(!copies.join("projects").join("plain").exists());

    // Read back as the app reads it.
    let typed = load_store(&kit);
    assert_eq!(typed.profiles[0].engine, WebServerEngine::Caddy);
    assert_eq!(typed.profiles[0].server_line, None);
    let found = crate::detect::detect(&shop, "it");
    let action = found.actions.iter().find(|a| a.id == "site").unwrap();
    assert_eq!(action.caddy_dir.as_deref(), Some("public"));

    // A second start: nothing changes, the first copies stay.
    let after_store = fs::read_to_string(store_path(&kit)).unwrap();
    let after_shop = fs::read_to_string(shop.join("panel.json")).unwrap();
    fs::write(copies.join("servers.json"), "first copy").unwrap();
    run(&kit);
    assert_eq!(fs::read_to_string(store_path(&kit)).unwrap(), after_store);
    assert_eq!(
        fs::read_to_string(shop.join("panel.json")).unwrap(),
        after_shop
    );
    assert_eq!(
        fs::read_to_string(copies.join("servers.json")).unwrap(),
        "first copy"
    );

    // A scan that prunes keeps the port of Nginx's pool while Caddy is
    // not installed.
    WebServers::new(kit.clone()).list("it", true);
    assert_eq!(
        load_store(&kit).ports.get("nginx-1.27_php-8.4"),
        Some(&8085)
    );
}

/// The port of an automatic pool of Nginx moves to Caddy's pool of the
/// same PHP once Caddy is installed; one Caddy's pool already has stays.
#[test]
fn nginx_ports_wait_for_caddy() {
    let mut store: crate::servers::pools::StoredServers = serde_json::from_value(json!({
        "profiles": [ { "id": "nginx-1.28", "name": "Mine", "engine": "nginx" } ],
        "ports": { "nginx-1.27_php-8.4": 8085, "nginx-default": 8086,
                   "nginx-1.26-php-7.4": 8087, "caddy-2.10_php-7.4": 8099,
                   "nginx-1.28": 8088 }
    }))
    .unwrap();
    // A profile of Nginx reads as Caddy.
    assert_eq!(store.profiles[0].engine, WebServerEngine::Caddy);
    assert!(!nginx_ports(&mut store, None));
    assert_eq!(store.ports.len(), 5);

    assert!(nginx_ports(&mut store, Some("2.10")));
    assert_eq!(store.ports.get("caddy-2.10_php-8.4"), Some(&8085));
    assert_eq!(store.ports.get("caddy-2.10"), Some(&8086));
    assert_eq!(store.ports.get("caddy-2.10_php-7.4"), Some(&8099));
    // A profile's id is never an automatic pool's.
    assert_eq!(store.ports.get("nginx-1.28"), Some(&8088));
    assert!(!store
        .ports
        .keys()
        .any(|k| k.starts_with("nginx-") && k != "nginx-1.28"));
    assert!(!nginx_ports(&mut store, Some("2.10")));
}

#[test]
fn nginx_pool_ids_and_actions() {
    assert_eq!(
        parse_nginx_pool("nginx-1.27_php-8.4"),
        Some(Some("8.4".into()))
    );
    assert_eq!(
        parse_nginx_pool("nginx-1.27-php-8.4"),
        Some(Some("8.4".into()))
    );
    assert_eq!(parse_nginx_pool("nginx-default"), Some(None));
    assert_eq!(parse_nginx_pool("nginx-mine"), None);
    assert_eq!(parse_nginx_pool("caddy-2.10"), None);
    assert!(!is_nginx_pool("legacy"));

    // With Caddy installed, a site on an automatic pool of Nginx names
    // Caddy's pool of the same PHP.
    let mut a = json!({ "id": "site", "nginx": "web", "server": "nginx-1.27_php-8.5" });
    assert!(migrate_action(a.as_object_mut().unwrap(), Some("2.10")));
    assert_eq!(
        a,
        json!({ "id": "site", "caddy": "web", "server": "caddy-2.10_php-8.5" })
    );
    // An action with a `caddy` already keeps it; one without Nginx is left.
    let mut a = json!({ "id": "site", "nginx": "a", "caddy": "b" });
    assert!(migrate_action(a.as_object_mut().unwrap(), None));
    assert_eq!(a, json!({ "id": "site", "caddy": "b" }));
    let mut a = json!({ "id": "dev", "label": "Nginx docs", "command": "x" });
    assert!(!migrate_action(a.as_object_mut().unwrap(), None));
    assert_eq!(a["label"], "Nginx docs");
}
