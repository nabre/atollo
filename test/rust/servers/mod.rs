use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn profile_crud_and_vhost_generation() {
    let tmp = TempDir::new("servers-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());

    servers
        .save_profile(
            WebServerProfileInput {
                id: String::new(),
                name: "Legacy Apache 5.6".into(),
                engine: WebServerEngine::Apache,
                server_line: Some("2.4".into()),
                php_line: Some("5.6".into()),
                port: Some(8095),
            },
            "it",
        )
        .unwrap();

    let view = servers.list("it");
    assert_eq!(view.pools.len(), 1);
    assert_eq!(view.pools[0].id, "legacy-apache-5.6");
    assert_eq!(view.pools[0].port, 8095);
    assert!(view.pools[0].manual);

    let projects = kit.projects();
    fs::create_dir_all(projects.join(r"portale\storage\uploads")).unwrap();
    fs::create_dir_all(projects.join(r"common\assets")).unwrap();
    let mut aliases = BTreeMap::new();
    aliases.insert("/media".into(), "storage/uploads".into());
    aliases.insert("/shared".into(), "@common/assets".into());
    aliases.insert("/up".into(), "../../Windows".into());
    aliases.insert("/q".into(), "a\"\r\nInclude x".into());
    let site = AttachedSite {
        project: "portale".into(),
        folder: "public".into(),
        server_name: "portale.localhost".into(),
        domains: vec!["admin.portale.localhost".into(), "x\r\nListen 80".into()],
        aliases,
        url: "http://portale.localhost:8095".into(),
    };
    let vhost = apache_vhost_conf(
        8095,
        &projects.join("portale"),
        &projects,
        &projects.join(r"portale\public"),
        &site,
    );
    assert!(vhost.contains("<VirtualHost *:8095>"));
    assert!(vhost.contains("ServerName portale.localhost"));
    assert!(vhost.contains("ServerAlias admin.portale.localhost\r\n"));
    assert!(vhost.contains("Alias \"/media\" \"${CP_PROJECTS_FWD}/portale/storage/uploads\""));
    assert!(vhost.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
    assert!(!vhost.contains("..") && !vhost.contains("Include") && !vhost.contains("Listen"));

    servers.remove_profile("legacy-apache-5.6").unwrap();
    assert!(servers.list("it").pools.is_empty());
}

#[test]
fn nginx_pool_conf_and_vhosts_have_no_duplicate_localhost() {
    let tmp = TempDir::new("servers-nginx-pool");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let servers = WebServers::new(kit.clone());

    let projects = kit.projects();
    fs::create_dir_all(projects.join(r"alpha\public")).unwrap();
    fs::create_dir_all(projects.join(r"beta\public")).unwrap();

    let pool = WebServerPool {
        id: "nginx-default".into(),
        name: "Nginx".into(),
        engine: WebServerEngine::Nginx,
        server_line: None,
        php_line: None,
        port: 8088,
        running: false,
        pid: None,
        manual: false,
        sites: vec![
            AttachedSite {
                project: "alpha".into(),
                folder: "public".into(),
                server_name: "alpha.localhost".into(),
                domains: vec!["api.alpha.localhost".into()],
                aliases: BTreeMap::new(),
                url: "http://alpha.localhost:8088".into(),
            },
            AttachedSite {
                project: "beta".into(),
                folder: "public".into(),
                server_name: "beta.localhost".into(),
                domains: Vec::new(),
                aliases: BTreeMap::new(),
                url: "http://beta.localhost:8088".into(),
            },
        ],
        conf_dir: r"data\servers\nginx-default".into(),
    };

    let conf_path = servers.write_pool_configs(&pool, None, false).unwrap();
    let main_conf = fs::read_to_string(&conf_path).unwrap();
    assert!(main_conf.contains("listen 127.0.0.1:8088 default_server;"));
    assert!(main_conf.contains("server_name localhost;"));

    let vhosts_dir = conf_path.parent().unwrap().join("vhosts");
    let alpha_vhost = fs::read_to_string(vhosts_dir.join("alpha.conf")).unwrap();
    let beta_vhost = fs::read_to_string(vhosts_dir.join("beta.conf")).unwrap();

    assert!(alpha_vhost.contains("listen 127.0.0.1:8088;"));
    assert!(!alpha_vhost.contains("default_server"));
    assert!(alpha_vhost.contains("server_name alpha.localhost api.alpha.localhost;"));
    assert!(!alpha_vhost.contains("server_name alpha.localhost api.alpha.localhost localhost;"));

    assert!(beta_vhost.contains("listen 127.0.0.1:8088;"));
    assert!(!beta_vhost.contains("default_server"));
    assert!(beta_vhost.contains("server_name beta.localhost;"));
}
