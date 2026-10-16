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
    assert!(with_site.contains("ServerAlias shop.localhost api.shop.localhost\r\n"));
    assert!(with_site.contains("Alias \"/media\" \"${CP_PROJECT_FWD}/storage/media\""));
    assert!(with_site.contains("Alias \"/shared\" \"${CP_PROJECTS_FWD}/common/assets\""));
    assert!(!with_site.contains("/up") && !with_site.contains(".."));
}
