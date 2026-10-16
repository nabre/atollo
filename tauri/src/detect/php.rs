//! PHP: Laravel (and Statamic), Symfony, the CMS (WordPress, also with
//! Bedrock; TYPO3, Drupal), plain PHP (built-in server `php -S`).

use std::path::Path;

use super::{exists, list_dir, quote, read_json, tr, Action, Found, Kind};

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let manifest = read_json(dir, "composer.json");
    let composer = manifest.is_some();
    // A package in composer.json's `require` (the CMS made with Composer).
    let requires = |package: &str| {
        manifest
            .as_ref()
            .and_then(|m| m.get("require")?.get(package))
            .is_some()
    };
    let php_in_root = list_dir(dir)
        .iter()
        .any(|f| f.to_ascii_lowercase().ends_with(".php"));
    let docroot = ["public", "web", "htdocs"]
        .into_iter()
        .find(|d| exists(dir, &format!(r"{d}\index.php")));
    if !composer && !php_in_root && docroot.is_none() {
        return None;
    }
    let mut actions = Vec::new();
    let mut warnings = Vec::new();
    let mut platform = "PHP".to_owned();
    let mut reason = if composer {
        "composer.json".to_owned()
    } else {
        tr(lang, "file {0}", &[".php"])
    };
    if exists(dir, "artisan") {
        platform = if requires("statamic/cms") {
            "Statamic"
        } else {
            "Laravel"
        }
        .into();
        reason = "artisan".into();
        actions.push(Action::new(
            "php:artisan",
            "php artisan serve",
            "php artisan serve --host=127.0.0.1 --port={port}",
            Kind::Web,
        ));
    } else {
        let is_wp = exists(dir, "wp-config.php")
            || exists(dir, "wp-settings.php")
            || exists(dir, r"wp-includes\version.php")
            || requires("roots/wordpress");
        if exists(dir, r"bin\console") {
            platform = "Symfony".into();
            reason = "bin/console".into();
        } else if is_wp {
            platform = "WordPress".into();
            reason = if requires("roots/wordpress") {
                "composer.json (roots/wordpress)"
            } else {
                "wp-settings.php"
            }
            .into();
        } else if requires("typo3/cms-core") {
            platform = "TYPO3".into();
            reason = "composer.json (typo3/cms-core)".into();
        } else if requires("drupal/core-recommended") || requires("drupal/core") {
            platform = "Drupal".into();
            reason = "composer.json (drupal/core)".into();
        }
        let root = docroot.unwrap_or(".");
        let shown = if root == "." {
            tr(lang, "cartella del progetto", &[])
        } else {
            root.to_owned()
        };
        let has_htaccess =
            exists(dir, ".htaccess") || (root != "." && exists(dir, &format!(r"{root}\.htaccess")));
        let has_nginx = exists(dir, "nginx.conf")
            || (root != "." && exists(dir, &format!(r"{root}\nginx.conf")));
        if has_htaccess || is_wp {
            actions.push(Action::apache(
                "apache",
                &tr(lang, "sito web Apache ({0})", &[&shown]),
                root,
            ));
        }
        if has_nginx {
            actions.push(Action::nginx(
                "nginx",
                &tr(lang, "sito web Nginx ({0})", &[&shown]),
                root,
            ));
        }
        actions.push(Action::new(
            "php:server",
            &tr(lang, "php -S (cartella {0})", &[root]),
            &format!("php -S 127.0.0.1:{{port}} -t {}", quote(root)),
            Kind::Web,
        ));
    }
    if composer {
        actions.push(Action::new(
            "composer:install",
            "composer install",
            "composer install",
            Kind::Command,
        ));
        if !exists(dir, "vendor") {
            warnings.push(tr(
                lang,
                "Dipendenze PHP non installate: esegui prima \"composer install\".",
                &[],
            ));
        }
    }
    Some(Found {
        platform,
        kind: Kind::Web,
        reason,
        actions,
        warnings,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/detect/php.rs"]
mod tests;
