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
        if exists(dir, r"bin\console") {
            platform = "Symfony".into();
            reason = "bin/console".into();
        } else if exists(dir, "wp-config.php") || exists(dir, "wp-settings.php") {
            platform = "WordPress".into();
            reason = "wp-settings.php".into();
            warnings.push(tr(
                lang,
                "WordPress richiede un database MySQL/MariaDB, non incluso nel kit.",
                &[],
            ));
        }
        let root = docroot.unwrap_or(".");
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
mod tests {
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
    }
}
