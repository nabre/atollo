use crate::detect::tests::{ids, project, sh};
use crate::detect::{detect, Kind};

/// The stock `.htaccess` of Laravel (`public\.htaccess`, Laravel 11).
const LARAVEL_HTACCESS: &str = r"<IfModule mod_rewrite.c>
    <IfModule mod_negotiation.c>
        Options -MultiViews -Indexes
    </IfModule>

    RewriteEngine On

    # Handle Authorization Header
    RewriteCond %{HTTP:Authorization} .
    RewriteRule .* - [E=HTTP_AUTHORIZATION:%{HTTP:Authorization}]

    # Handle X-XSRF-Token Header
    RewriteCond %{HTTP:x-xsrf-token} .
    RewriteRule .* - [E=HTTP_X_XSRF_TOKEN:%{HTTP:X-XSRF-Token}]

    # Redirect Trailing Slashes If Not A Folder...
    RewriteCond %{REQUEST_FILENAME} !-d
    RewriteCond %{REQUEST_URI} (.+)/$
    RewriteRule ^ %1 [L,R=301]

    # Send Requests To Front Controller...
    RewriteCond %{REQUEST_FILENAME} !-d
    RewriteCond %{REQUEST_FILENAME} !-f
    RewriteRule ^ index.php [L]
</IfModule>
";

/// The stock `.htaccess` WordPress writes.
const WORDPRESS_HTACCESS: &str = r"# BEGIN WordPress
<IfModule mod_rewrite.c>
RewriteEngine On
RewriteRule .* - [E=HTTP_AUTHORIZATION:%{HTTP:Authorization}]
RewriteBase /
RewriteRule ^index\.php$ - [L]
RewriteCond %{REQUEST_FILENAME} !-f
RewriteCond %{REQUEST_FILENAME} !-d
RewriteRule . /index.php [L]
</IfModule>
# END WordPress
";

/// A `.htaccess` with a rule Caddy cannot know.
const OWN_HTACCESS: &str = "RewriteEngine On\nRewriteRule ^shop/(.*)$ catalog.php?p=$1 [L,QSA]\n";

/// T3.3.6: the stock files of WordPress, Laravel and Drupal are what Caddy's
/// front controller does; any other directive is a rule of the project's own.
#[test]
fn htaccess_rules_of_its_own() {
    use crate::detect::php::{apache_offered, htaccess_has_own_rules};
    use crate::kit::Os;
    assert!(!htaccess_has_own_rules(LARAVEL_HTACCESS));
    assert!(!htaccess_has_own_rules(WORDPRESS_HTACCESS));
    assert!(!htaccess_has_own_rules(
        "#\n# Apache/PHP/Drupal settings:\n#\n<FilesMatch \"x\">\n  Require all denied\n</FilesMatch>\n"
    ));
    assert!(!htaccess_has_own_rules("RewriteEngine On\n\n# a comment\n"));
    assert!(!htaccess_has_own_rules(""));
    assert!(htaccess_has_own_rules(OWN_HTACCESS));
    assert!(htaccess_has_own_rules("AuthType Basic\n"));
    // Apache only on Windows, and only for rules of its own.
    let own = [OWN_HTACCESS.to_owned()];
    let stock = [WORDPRESS_HTACCESS.to_owned()];
    assert!(apache_offered(Os::Win32, &own));
    assert!(!apache_offered(Os::Win32, &stock));
    assert!(!apache_offered(Os::Win32, &[]));
    for os in [Os::Darwin, Os::Linux] {
        assert!(!apache_offered(os, &own), "{os:?}");
    }
}

#[test]
fn laravel_and_plain() {
    let l = project("php-laravel", &[("composer.json", "{}"), ("artisan", "")]);
    let p = detect(&l.0, "it");
    assert_eq!(p.platform, "Laravel");
    // Its site with Caddy after `artisan serve` (Q-3, D-111).
    assert_eq!(ids(&p), ["php:artisan", "caddy", "composer:install"]);
    assert!(p.warnings.iter().any(|w| w.contains("composer install")));

    // Laravel's stock public\.htaccess: Caddy does what it says.
    let ht = project(
        "php-laravel-ht",
        &[
            ("composer.json", "{}"),
            ("artisan", ""),
            ("public/index.php", "<?php"),
            ("public/.htaccess", LARAVEL_HTACCESS),
        ],
    );
    let p = detect(&ht.0, "it");
    assert_eq!(ids(&p), ["php:artisan", "caddy", "composer:install"]);
    assert_eq!(p.default.as_deref(), Some("php:artisan"));
    assert_eq!(p.actions[1].caddy_dir.as_deref(), Some("public"));
    assert!(!p.requires.contains(&"apache".to_owned()));

    // Rules of its own: Apache first on Windows, Caddy only elsewhere.
    let own = project(
        "php-laravel-own",
        &[
            ("composer.json", "{}"),
            ("artisan", ""),
            ("public/index.php", "<?php"),
            ("public/.htaccess", OWN_HTACCESS),
        ],
    );
    let p = detect(&own.0, "it");
    if cfg!(windows) {
        assert_eq!(
            ids(&p),
            ["php:artisan", "apache", "caddy", "composer:install"]
        );
        assert_eq!(p.actions[1].apache_dir.as_deref(), Some("public"));
        assert!(p.requires.contains(&"apache".to_owned()));
    } else {
        assert_eq!(ids(&p), ["php:artisan", "caddy", "composer:install"]);
        assert!(!p.requires.contains(&"apache".to_owned()));
    }

    let plain = project("php-plain", &[("public/index.php", "<?php")]);
    let p = detect(&plain.0, "it");
    assert_eq!(p.kind, Kind::Web);
    assert_eq!(
        p.actions[0].command,
        sh(
            r#"php -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#,
            r#"php -S 127.0.0.1:{port} -t public "$CP_DATA/kit/php-router.php""#
        )
    );
    // Caddy proposed after `php -S`, not required.
    assert_eq!(ids(&p), ["php:server", "caddy"]);
    assert_eq!(p.default.as_deref(), Some("php:server"));
    assert_eq!(p.actions[1].caddy_dir.as_deref(), Some("public"));
    assert!(!p.requires.contains(&"caddy".to_owned()));
    assert!(!p.requires.contains(&"apache".to_owned()));

    // A `.htaccess`: the site first, Caddy (it is a stock one).
    let ht = project(
        "php-htaccess",
        &[
            ("public/index.php", "<?php"),
            ("public/.htaccess", "RewriteEngine On"),
        ],
    );
    let p = detect(&ht.0, "it");
    assert_eq!(p.default.as_deref(), Some("caddy"));
    assert_eq!(p.actions[0].caddy_dir.as_deref(), Some("public"));
    assert!(p.requires.contains(&"caddy".to_owned()));
    assert!(!p.requires.contains(&"apache".to_owned()));

    // With rules of its own: Apache first on Windows, Caddy elsewhere.
    let own = project(
        "php-apache",
        &[
            ("public/index.php", "<?php"),
            ("public/.htaccess", OWN_HTACCESS),
        ],
    );
    let p = detect(&own.0, "it");
    if cfg!(windows) {
        assert_eq!(p.default.as_deref(), Some("apache"));
        assert_eq!(p.actions[0].apache_dir.as_deref(), Some("public"));
        assert_eq!(p.actions[1].caddy_dir.as_deref(), Some("public"));
        assert!(p.requires.contains(&"apache".to_owned()));
    } else {
        assert_eq!(p.default.as_deref(), Some("caddy"));
        assert!(!ids(&p).contains(&"apache"));
        assert!(!p.requires.contains(&"apache".to_owned()));
    }

    let wp = project(
        "php-wp",
        &[
            ("index.php", "<?php"),
            ("wp-settings.php", "<?php"),
            (".htaccess", WORDPRESS_HTACCESS),
        ],
    );
    let p = detect(&wp.0, "it");
    assert_eq!(p.platform, "WordPress");
    assert_eq!(p.default.as_deref(), Some("caddy"));
    assert!(p.requires.contains(&"mariadb".to_owned()));
    assert!(p.requires.contains(&"caddy".to_owned()));
    assert!(p.requires.contains(&"php".to_owned()));
    assert!(!p.requires.contains(&"apache".to_owned()));
}

/// `php -S` always goes through the kit's router (no `.env`, `.git`, `.inc`):
/// a project's own `router.php` runs after it, its path given relative to
/// the document root (SEC-V3-02, D-97).
#[test]
fn php_server_router() {
    let own = project(
        "php-router",
        &[
            ("public/index.php", "<?php"),
            ("public/router.php", "<?php"),
            ("router.php", "<?php"),
        ],
    );
    let p = detect(&own.0, "it");
    assert_eq!(
        p.actions[0].command,
        sh(
            r#"php -d "devkit.router=router.php" -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'devkit.router=router.php' -S 127.0.0.1:{port} -t public "$CP_DATA/kit/php-router.php""#
        )
    );
    let up = project(
        "php-router-up",
        &[("public/index.php", "<?php"), ("router.php", "<?php")],
    );
    let p = detect(&up.0, "it");
    assert_eq!(
        p.actions[0].command,
        sh(
            r#"php -d "devkit.router=..\router.php" -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'devkit.router=../router.php' -S 127.0.0.1:{port} -t public "$CP_DATA/kit/php-router.php""#
        )
    );
    let root = project(
        "php-router-root",
        &[("index.php", "<?php"), ("router.php", "<?php")],
    );
    let p = detect(&root.0, "it");
    assert_eq!(
        p.actions[0].command,
        sh(
            r#"php -d "devkit.router=router.php" -S 127.0.0.1:{port} -t . "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'devkit.router=router.php' -S 127.0.0.1:{port} -t . "$CP_DATA/kit/php-router.php""#
        )
    );

    let router = include_str!("../../../runtime/php-router.php");
    assert!(router.contains("well-known") && router.contains(".inc") && router.contains("403"));
    // Same never-serve set as Apache and Nginx (SEC-V3-01), local hosts only
    // (SEC-V3-03).
    assert!(router.contains("php[3-8]|phtml|phar") && router.contains("421"));
    // The path is cut at `?` by hand, not read with `parse_url` (`//.env`),
    // and short 8.3 names (`ENV~1`) are refused.
    assert!(!router.contains("parse_url(") && router.contains(r"'/~\d/'"));
    assert!(router.trim_end().ends_with("return false;"));
    // The checks leave no variable to the project's router, required in the
    // global scope with its return value.
    assert!(router.contains("(static function () {"));
    assert!(router.contains("return require $_SERVER['DOCUMENT_ROOT']"));
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
    assert_eq!(p.actions[0].caddy_dir.as_deref(), Some("web"));
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
        assert_eq!(p.actions[0].caddy_dir.as_deref(), Some(docroot));
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

/// The CMS from their archive, without Composer: Caddy first and MariaDB.
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
        assert_eq!(p.default.as_deref(), Some("caddy"));
        assert_eq!(p.actions[0].caddy_dir.as_deref(), Some("."));
        for id in ["caddy", "php", "mariadb"] {
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
