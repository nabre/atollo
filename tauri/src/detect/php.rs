//! PHP: Laravel (and Statamic), Symfony, the CMS (WordPress, also with
//! Bedrock; PrestaShop, Drupal, Joomla, TYPO3), plain PHP (built-in server
//! `php -S`, and an Apache of its own proposed beside it).

use std::path::Path;

use super::{exists, list_dir, php_server, read_json, tr, Action, Found, Kind};

/// The CMS with a database: the kit's MariaDB in `requires` (`mod.rs`).
pub(super) const CMS: [&str; 5] = ["WordPress", "PrestaShop", "Drupal", "Joomla", "TYPO3"];

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
    let root = docroot.unwrap_or(".");
    let shown = if root == "." {
        tr(lang, "cartella del progetto", &[])
    } else {
        root.to_owned()
    };
    let apache = || {
        Action::apache(
            "apache",
            &tr(lang, "sito web Apache ({0})", &[&shown]),
            root,
        )
    };
    let has_htaccess =
        exists(dir, ".htaccess") || (root != "." && exists(dir, &format!(r"{root}\.htaccess")));
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
        // Laravel's `public\.htaccess`: Apache too, after `artisan serve`.
        if has_htaccess {
            actions.push(apache());
        }
    } else {
        // The CMS before Symfony: PrestaShop has a `bin\console` too.
        let cms = if exists(dir, "wp-config.php")
            || exists(dir, "wp-settings.php")
            || exists(dir, r"wp-includes\version.php")
        {
            Some(("WordPress", "wp-settings.php"))
        } else if requires("roots/wordpress") {
            Some(("WordPress", "composer.json (roots/wordpress)"))
        } else if exists(dir, r"classes\PrestaShopAutoload.php") {
            Some(("PrestaShop", "classes/PrestaShopAutoload.php"))
        } else if requires("drupal/core-recommended") || requires("drupal/core") {
            Some(("Drupal", "composer.json (drupal/core)"))
        } else if exists(dir, r"core\lib\Drupal.php") {
            Some(("Drupal", "core/lib/Drupal.php"))
        } else if exists(dir, r"administrator\manifests\files\joomla.xml") {
            Some(("Joomla", "administrator/manifests/files/joomla.xml"))
        } else if requires("typo3/cms-core") {
            Some(("TYPO3", "composer.json (typo3/cms-core)"))
        } else if exists(dir, r"typo3\sysext\core\ext_emconf.php") {
            Some(("TYPO3", "typo3/sysext/core"))
        } else {
            None
        };
        if let Some((name, why)) = cms {
            platform = name.into();
            reason = why.into();
        } else if exists(dir, r"bin\console") {
            platform = "Symfony".into();
            reason = "bin/console".into();
        }
        let has_nginx = exists(dir, "nginx.conf")
            || (root != "." && exists(dir, &format!(r"{root}\nginx.conf")));
        // A CMS or an `.htaccess`: Apache first, the main button. Otherwise
        // proposed after `php -S`.
        let apache_first = has_htaccess || cms.is_some();
        if apache_first {
            actions.push(apache());
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
            &php_server(dir, root),
            Kind::Web,
        ));
        if !apache_first {
            actions.push(apache());
        }
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
