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

/// D-111: the stock files of WordPress, Laravel and Drupal are what Caddy's
/// front controller does; any other directive is a rule of the project's own.
#[test]
fn htaccess_rules_of_its_own() {
    use crate::detect::php::{apache_available, apache_offered, htaccess_has_own_rules};
    use crate::kit::Target;
    assert!(!htaccess_has_own_rules(LARAVEL_HTACCESS));
    assert!(!htaccess_has_own_rules(WORDPRESS_HTACCESS));
    // Drupal's stock file denies files Caddy's rules serve: its own rules.
    assert!(htaccess_has_own_rules(
        "#\n# Apache/PHP/Drupal settings:\n#\n<FilesMatch \"x\">\n  Require all denied\n</FilesMatch>\n"
    ));
    assert!(!htaccess_has_own_rules("RewriteEngine On\n\n# a comment\n"));
    assert!(!htaccess_has_own_rules(""));
    assert!(htaccess_has_own_rules(OWN_HTACCESS));
    assert!(htaccess_has_own_rules("AuthType Basic\n"));
    // Apache where the catalog has it (D-116), only for rules of its own.
    let own = [OWN_HTACCESS.to_owned()];
    let stock = [WORDPRESS_HTACCESS.to_owned()];
    assert!(apache_offered(Target::WIN32_X64, &own));
    assert!(!apache_offered(Target::WIN32_X64, &stock));
    assert!(!apache_offered(Target::WIN32_X64, &[]));
    for t in Target::ALL {
        assert_eq!(apache_offered(t, &own), apache_available(t), "{t}");
        assert!(!apache_offered(t, &stock), "{t}");
    }
}

#[test]
fn laravel_and_plain() {
    let l = project("php-laravel", &[("composer.json", "{}"), ("artisan", "")]);
    let p = detect(&l.0, "it");
    assert_eq!(p.platform, "Laravel");
    // Its site with Caddy after `artisan serve` (D-111).
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

    // Rules of its own: Apache first, on every system since D-116.
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
    assert_eq!(
        ids(&p),
        ["php:artisan", "apache", "caddy", "composer:install"]
    );
    assert_eq!(p.actions[1].apache_dir.as_deref(), Some("public"));
    assert!(p.requires.contains(&"apache".to_owned()));

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

    // With rules of its own: Apache first, on every system since D-116.
    let own = project(
        "php-apache",
        &[
            ("public/index.php", "<?php"),
            ("public/.htaccess", OWN_HTACCESS),
        ],
    );
    let p = detect(&own.0, "it");
    assert_eq!(p.default.as_deref(), Some("apache"));
    assert_eq!(p.actions[0].apache_dir.as_deref(), Some("public"));
    assert_eq!(p.actions[1].caddy_dir.as_deref(), Some("public"));
    assert!(p.requires.contains(&"apache".to_owned()));

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
    assert!(p.requires.contains(&"mysql".to_owned()));
    assert!(p.requires.contains(&"caddy".to_owned()));
    assert!(p.requires.contains(&"php".to_owned()));
    assert!(!p.requires.contains(&"apache".to_owned()));
}

/// Deny rules in subfolders (review of D-111): Caddy reads no `.htaccess`,
/// so a private or uploads folder with rules of its own makes Apache first
/// on Windows and a warning everywhere; stock files, dependencies and
/// folders past the bounds do not count.
#[test]
fn nested_htaccess_rules() {
    use crate::detect::php::{nested_htaccess, NESTED_DEPTH};
    let deny = "Require all denied\n";
    let drupal_files = "SetHandler Drupal_Security_Do_Not_Remove_See_SA_2006_006\n<IfModule mod_php.c>\n  php_flag engine off\n</IfModule>\n";
    let deep = format!("public/{}.htaccess", "d/".repeat(NESTED_DEPTH + 1));
    let p = project(
        "php-nested-ht",
        &[
            ("composer.json", "{}"),
            ("artisan", ""),
            ("public/index.php", "<?php"),
            ("public/.htaccess", LARAVEL_HTACCESS),
            ("public/private/.htaccess", deny),
            ("public/sites/default/files/.htaccess", drupal_files),
            ("public/assets/.htaccess", "RewriteEngine On\n"),
            ("public/vendor/x/.htaccess", deny),
            (deep.as_str(), deny),
        ],
    );
    assert_eq!(
        nested_htaccess(&p.0, "public"),
        [
            "public/private/.htaccess",
            "public/sites/default/files/.htaccess"
        ]
    );
    let found = detect(&p.0, "it");
    assert!(found
        .warnings
        .iter()
        .any(|w| w.contains("public/private/.htaccess") && w.contains("Caddy")));
    assert_eq!(
        ids(&found),
        ["php:artisan", "apache", "caddy", "composer:install"]
    );
    // The site's own `.htaccess` at the root is not a nested one.
    let flat = project(
        "php-flat-ht",
        &[("index.php", "<?php"), (".htaccess", WORDPRESS_HTACCESS)],
    );
    assert!(nested_htaccess(&flat.0, ".").is_empty());
}

/// Issue #4: the walk of [`nested_htaccess`] that stops at its bound says
/// so (`cut`), and the detection warns that a `.htaccess` may be missed.
#[test]
fn nested_htaccess_cut_is_a_warning() {
    use crate::detect::php::{nested_htaccess_within, Nested, NESTED_ENTRIES};
    let p = project(
        "php-nested-cut",
        &[
            ("index.php", "<?php"),
            ("a/b.txt", ""),
            ("a/c.txt", ""),
            ("a/d.txt", ""),
        ],
    );
    assert_eq!(
        nested_htaccess_within(&p.0, ".", 100),
        Nested {
            files: vec![],
            cut: false
        }
    );
    assert!(nested_htaccess_within(&p.0, ".", 3).cut);
    let warned = |dir: &std::path::Path| {
        detect(dir, "it")
            .warnings
            .iter()
            .any(|w| w.contains(&NESTED_ENTRIES.to_string()) && w.contains("Caddy"))
    };
    assert!(!warned(&p.0));
    let many = p.0.join("uploads");
    std::fs::create_dir_all(&many).unwrap();
    for i in 0..=NESTED_ENTRIES {
        std::fs::write(many.join(format!("{i}.jpg")), b"").unwrap();
    }
    assert!(warned(&p.0));
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
            r#"php -d "atollo.router=router.php" -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'atollo.router=router.php' -S 127.0.0.1:{port} -t public "$CP_DATA/kit/php-router.php""#
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
            r#"php -d "atollo.router=..\router.php" -S 127.0.0.1:{port} -t public "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'atollo.router=../router.php' -S 127.0.0.1:{port} -t public "$CP_DATA/kit/php-router.php""#
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
            r#"php -d "atollo.router=router.php" -S 127.0.0.1:{port} -t . "%CP_DATA%\kit\php-router.php""#,
            r#"php -d 'atollo.router=router.php' -S 127.0.0.1:{port} -t . "$CP_DATA/kit/php-router.php""#
        )
    );

    let router = include_str!("../../../runtime/php-router.php");
    assert!(router.contains("well-known") && router.contains(".inc") && router.contains("403"));
    // Same never-serve set as Apache and Caddy (SEC-V3-01), local hosts only
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

/// A `php -S` with the kit's router, ended on drop.
struct PhpServer(std::process::Child);

impl Drop for PhpServer {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The status of `GET target` with `Host: host` from the server on `port`,
/// retried while it starts, or when the connection is reset before the
/// answer (Windows, with the machine busy).
fn status_of(port: u16, host: &str, target: &str) -> u16 {
    use std::io::{Read, Write};
    let started = std::time::Instant::now();
    loop {
        let mut answer = Vec::new();
        if let Ok(mut stream) = std::net::TcpStream::connect(("127.0.0.1", port)) {
            let request =
                format!("GET {target} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
            if stream.write_all(request.as_bytes()).is_ok() {
                // What came before a reset is enough.
                let mut buf = [0u8; 1024];
                while let Ok(n) = stream.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    answer.extend_from_slice(&buf[..n]);
                }
            }
        }
        let answer = String::from_utf8_lossy(&answer);
        if let Some(status) = answer.split(' ').nth(1).and_then(|s| s.parse().ok()) {
            return status;
        }
        assert!(
            started.elapsed().as_secs() < 10,
            "php -S on {port}, {target}: no status in {answer:?}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

/// BUG-V5-01: the router's 421 for a `Host` that is not local only outside
/// the local network; with `ATOLLO_ALLOW_LAN=1` a phone asking for the PC's
/// address gets the page, and hidden files stay refused either way. Needs
/// `php` on PATH (skipped otherwise).
#[test]
fn php_router_on_the_network() {
    let php_found = std::process::Command::new("php")
        .arg("-v")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !php_found {
        eprintln!("php not on PATH: skipped");
        return;
    }
    let tmp = crate::kit::tests::TempDir::new("php-router-lan");
    let root = tmp.0.join("public");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("index.php"), "<?php echo 'ok';").unwrap();
    std::fs::write(root.join(".env"), "SECRET=1").unwrap();
    let router = tmp.0.join("php-router.php");
    std::fs::write(&router, include_str!("../../../runtime/php-router.php")).unwrap();

    for lan in [false, true] {
        let port = {
            let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
            l.local_addr().unwrap().port()
        };
        let mut cmd = std::process::Command::new("php");
        cmd.arg("-S")
            .arg(format!("127.0.0.1:{port}"))
            .arg("-t")
            .arg(&root)
            .arg(&router)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null());
        if lan {
            cmd.env("ATOLLO_ALLOW_LAN", "1");
        } else {
            cmd.env_remove("ATOLLO_ALLOW_LAN");
        }
        let _server = PhpServer(cmd.spawn().unwrap());

        let network = status_of(port, "192.168.1.20:8000", "/");
        assert_eq!(network, if lan { 200 } else { 421 }, "lan {lan}");
        assert_eq!(status_of(port, &format!("localhost:{port}"), "/"), 200);
        assert_eq!(status_of(port, "blog.localhost", "/"), 200);
        for target in ["/.env", "/.env.", "/%2eenv", "/.git/config"] {
            let host = if lan {
                "192.168.1.20:8000"
            } else {
                "localhost"
            };
            assert_eq!(status_of(port, host, target), 403, "lan {lan}: {target}");
        }
    }
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
    assert!(p.requires.contains(&"mysql".to_owned()));

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
        assert!(p.requires.contains(&"mysql".to_owned()));
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

/// The CMS from their archive, without Composer: Caddy first and MySQL.
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
        for id in ["caddy", "php", "mysql"] {
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
        .contains(&"mysql".to_owned()));
}
