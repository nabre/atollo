use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};

#[test]
fn laravel_and_plain() {
    let l = project("php-laravel", &[("composer.json", "{}"), ("artisan", "")]);
    let p = detect(&l.0, "it");
    assert_eq!(p.platform, "Laravel");
    assert_eq!(ids(&p), ["php:artisan", "composer:install"]);
    assert!(p.warnings.iter().any(|w| w.contains("composer install")));

    let plain = project("php-plain", &[("public/index.php", "<?php")]);
    let p = detect(&plain.0, "it");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(p.actions[0].command, "php -S 127.0.0.1:{port} -t public");

    let ht = project(
        "php-apache",
        &[
            ("public/index.php", "<?php"),
            ("public/.htaccess", "RewriteEngine On"),
        ],
    );
    let p = detect(&ht.0, "it");
    assert_eq!(p.default.as_deref(), Some("apache"));
    assert_eq!(p.actions[0].apache_dir.as_deref(), Some("public"));
    assert!(p.requires.contains(&"apache".to_owned()));
}
