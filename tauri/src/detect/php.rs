//! PHP: Laravel, Symfony, WordPress, plain PHP (built-in server `php -S`).

use std::path::Path;

use super::{exists, list_dir, quote, read_json, tr, Action, Found, Kind};

pub(super) fn detect(dir: &Path, lang: &str) -> Option<Found> {
    let composer = read_json(dir, "composer.json").is_some();
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
        platform = "Laravel".into();
        reason = "artisan".into();
        actions.push(Action::new(
            "php:artisan",
            "php artisan serve",
            "php artisan serve --host=127.0.0.1 --port={port}",
            Kind::Web,
        ));
    } else {
        let is_wp = exists(dir, "wp-config.php") || exists(dir, "wp-settings.php");
        if exists(dir, r"bin\console") {
            platform = "Symfony".into();
            reason = "bin/console".into();
        } else if is_wp {
            platform = "WordPress".into();
            reason = "wp-settings.php".into();
            warnings.push(tr(
                lang,
                "WordPress richiede un database MySQL/MariaDB, non incluso nel kit.",
                &[],
            ));
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
