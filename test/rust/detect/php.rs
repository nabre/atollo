use crate::detect::tests::{ids, project};
use crate::detect::{detect, Kind};

#[test]
fn laravel_and_plain() {
    let l = project("php-laravel", &[("composer.json", "{}"), ("artisan", "")]);
    let p = detect(&l.0, "it");
    assert_eq!(p.platform, "Laravel");
    assert_eq!(ids(&p), ["php:artisan", "composer:install"]);
    assert!(p.warnings.iter().any(|w| w.contains("composer install")));

    // Laravel's public\.htaccess: Apache after `artisan serve`.
    let ht = project(
        "php-laravel-ht",
        &[
            ("composer.json", "{}"),
            ("artisan", ""),
            ("public/index.php", "<?php"),
            ("public/.htaccess", "RewriteEngine On"),
        ],
    );
    let p = detect(&ht.0, "it");
    assert_eq!(ids(&p), ["php:artisan", "apache", "composer:install"]);
    assert_eq!(p.default.as_deref(), Some("php:artisan"));
    assert_eq!(p.actions[1].apache_dir.as_deref(), Some("public"));

    let plain = project("php-plain", &[("public/index.php", "<?php")]);
    let p = detect(&plain.0, "it");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(
        p.actions[0].command,
        r#"php -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#
    );
    // Apache proposed after `php -S`, not required.
    assert_eq!(ids(&p), ["php:server", "apache"]);
    assert_eq!(p.default.as_deref(), Some("php:server"));
    assert_eq!(p.actions[1].apache_dir.as_deref(), Some("public"));
    assert!(!p.requires.contains(&"apache".to_owned()));

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

    let wp = project(
        "php-wp",
        &[("index.php", "<?php"), ("wp-settings.php", "<?php")],
    );
    let p = detect(&wp.0, "it");
    assert_eq!(p.platform, "WordPress");
    assert!(p.requires.contains(&"mariadb".to_owned()));
    assert!(p.requires.contains(&"apache".to_owned()));
    assert!(p.requires.contains(&"php".to_owned()));
}

/// `php -S` goes through the kit's router (no `.env`, `.git`, `.inc`), unless
/// the project has a `router.php` of its own (SEC-V3-02).
#[test]
fn php_server_router() {
    let own = project(
        "php-router",
        &[
            ("public/index.php", "<?php"),
            ("public/router.php", "<?php"),
        ],
    );
    let p = detect(&own.0, "it");
    assert_eq!(
        p.actions[0].command,
        r"php -S 127.0.0.1:{port} -t public public\router.php"
    );
    let root = project(
        "php-router-root",
        &[("index.php", "<?php"), ("router.php", "<?php")],
    );
    let p = detect(&root.0, "it");
    assert_eq!(
        p.actions[0].command,
        "php -S 127.0.0.1:{port} -t . router.php"
    );

    let router = include_str!("../../../runtime/php-router.php");
    assert!(router.contains("well-known") && router.contains(".inc") && router.contains("403"));
    assert!(router.trim_end().ends_with("return false;"));
}

/// The CMS of the wizard, made with Composer: the public folder is a
/// subfolder, `.env` stays out of the site.
#[test]
fn cms_with_composer() {
    let bedrock = project(
        "php-bedrock",
        &[
            ("composer.json", r#"{"require": {"roots/wordpress": "^6"}}"#),
            ("web/index.php", "<?php"),
            (".env", "DATABASE_URL=mysql://a:b@127.0.0.1:3306/a"),
        ],
    );
    let p = detect(&bedrock.0, "it");
    assert_eq!(p.platform, "WordPress");
    assert_eq!(p.actions[0].apache_dir.as_deref(), Some("web"));
    assert!(p.requires.contains(&"mariadb".to_owned()));

    for (pkg, docroot, platform) in [
        ("typo3/cms-core", "public", "TYPO3"),
        ("drupal/core-recommended", "web", "Drupal"),
    ] {
        let manifest = format!(r#"{{"require": {{"{pkg}": "*"}}}}"#);
        let (index, htaccess) = (
            format!("{docroot}/index.php"),
            format!("{docroot}/.htaccess"),
        );
        let site = project(
            &format!("php-{platform}"),
            &[
                ("composer.json", manifest.as_str()),
                (index.as_str(), "<?php"),
                (htaccess.as_str(), "RewriteEngine On"),
            ],
        );
        let p = detect(&site.0, "it");
        assert_eq!(p.platform, platform);
        assert_eq!(p.actions[0].apache_dir.as_deref(), Some(docroot));
        assert!(p.requires.contains(&"mariadb".to_owned()));
    }

    let statamic = project(
        "php-statamic",
        &[
            ("composer.json", r#"{"require": {"statamic/cms": "^6"}}"#),
            ("artisan", ""),
        ],
    );
    assert_eq!(detect(&statamic.0, "it").platform, "Statamic");
}

/// The CMS from their archive, without Composer: Apache first and MariaDB.
#[test]
fn cms_from_archive() {
    for (marker, platform) in [
        // PrestaShop has a Symfony `bin\console`: still PrestaShop.
        ("classes/PrestaShopAutoload.php", "PrestaShop"),
        ("core/lib/Drupal.php", "Drupal"),
        ("administrator/manifests/files/joomla.xml", "Joomla"),
        ("typo3/sysext/core/ext_emconf.php", "TYPO3"),
    ] {
        let site = project(
            &format!("cms-{platform}"),
            &[("index.php", "<?php"), (marker, ""), ("bin/console", "")],
        );
        let p = detect(&site.0, "it");
        assert_eq!(p.platform, platform);
        assert_eq!(p.default.as_deref(), Some("apache"));
        assert_eq!(p.actions[0].apache_dir.as_deref(), Some("."));
        for id in ["apache", "php", "mariadb"] {
            assert!(p.requires.contains(&id.to_owned()), "{platform}: {id}");
        }
    }
    // Statamic keeps its contents in files: no database.
    let statamic = project(
        "cms-statamic",
        &[
            ("composer.json", r#"{"require": {"statamic/cms": "^6"}}"#),
            ("artisan", ""),
        ],
    );
    assert!(!detect(&statamic.0, "it")
        .requires
        .contains(&"mariadb".to_owned()));
}
