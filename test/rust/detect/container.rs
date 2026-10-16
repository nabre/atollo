use super::*;
use crate::detect::tests::project;

#[test]
fn image_names_and_lines() {
    assert_eq!(
        split_image("php:8.3-apache"),
        ("php".into(), "8.3-apache".into())
    );
    assert_eq!(
        split_image("docker.io/library/nginx:1.27-alpine@sha256:abc"),
        ("nginx".into(), "1.27-alpine".into())
    );
    assert_eq!(
        split_image("localhost:5000/httpd"),
        ("httpd".into(), String::new())
    );
    assert_eq!(line_of_tag("7.4.33-fpm"), Some("7.4".into()));
    assert_eq!(line_of_tag("alpine3.20"), None);
    assert_eq!(line_of_tag("8"), None);
    assert_eq!(line_of_tag(""), None);
    assert_eq!(
        first_word(r#" "nginx:1.27" # web"#).as_deref(),
        Some("nginx:1.27")
    );
    assert_eq!(
        first_word(" 'mariadb:11.4'").as_deref(),
        Some("mariadb:11.4")
    );
    assert_eq!(first_word(" mongo:8.0 # db").as_deref(), Some("mongo:8.0"));
    // Exact names: not a substring of another image.
    assert!(components("nginx-proxy-manager:latest").is_empty());
    assert!(components("httpd-exporter").is_empty());
    assert!(components("myphp:8.3").is_empty());
}

#[test]
fn compose_and_dockerfile() {
    for name in [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yaml",
        "docker-compose.yml",
    ] {
        let tmp = project(
            &format!("compose-{}", name.replace('.', "_")),
            &[(
                name,
                "services:\n  web:\n    image: \"php:8.3-apache\"\n  db:\n    image: mariadb:11.4 # data\n  cache:\n    image: nginx-proxy-manager\n",
            )],
        );
        let found = servers(&tmp.0);
        assert_eq!(found.get("php"), Some(&Some("8.3".into())), "{name}");
        assert_eq!(found.get("apache"), Some(&None), "{name}");
        assert_eq!(found.get("mariadb"), Some(&Some("11.4".into())), "{name}");
        assert!(!found.contains_key("nginx"), "{name}");
        let p = crate::detect::detect(&tmp.0, "it");
        for id in ["apache", "mariadb", "php"] {
            assert!(p.requires.contains(&id.to_owned()), "{name}: {id}");
        }
        assert_eq!(p.lines.get("php").map(String::as_str), Some("8.3"));
    }

    let docker = project(
        "dockerfile",
        &[(
            "Dockerfile",
            "FROM --platform=linux/amd64 php:7.4-fpm AS base\nfrom httpd:2.4\nFROM base\n",
        )],
    );
    let found = servers(&docker.0);
    assert_eq!(found.get("php"), Some(&Some("7.4".into())));
    assert_eq!(found.get("apache"), Some(&Some("2.4".into())));
    assert_eq!(found.len(), 2);
}
