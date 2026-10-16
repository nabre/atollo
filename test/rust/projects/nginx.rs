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
    assert!(conf.contains("listen 127.0.0.1:7106 default_server;"));
    assert!(conf.contains("server_name shop.localhost api.shop.localhost localhost;"));
    assert!(conf.contains("root \"D:/kit/projects/shop/public\";"));
    assert!(conf.contains("fastcgi_pass 127.0.0.1:9123;"));
    assert!(conf.contains("location /media/ {"));
    assert!(conf.contains(&format!("alias \"{projs_fwd}/shop/storage/media/\";")));
    assert!(conf.contains(&format!("alias \"{projs_fwd}/common/assets/\";")));
    assert!(!conf.contains("location /up/") && !conf.contains("x;y"));
}
