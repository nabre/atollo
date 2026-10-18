//! PHP: Laravel (and Statamic), Symfony, the CMS (WordPress, also with
//! Bedrock; PrestaShop, Drupal, Joomla, TYPO3), plain PHP (built-in server
//! `php -S`, and a site of Caddy proposed beside it).
//!
//! Caddy is the web server proposed on every system (Q-3, D-111): it reads
//! no `.htaccess`, and gets the rules of a front controller instead
//! (`projects/caddy.rs`), which is what the stock `.htaccess` of WordPress,
//! Laravel and Drupal say. Apache is proposed too only on Windows (the only
//! system with the kit's Apache) and only for a project whose `.htaccess`
//! has rules of its own: then it comes first.

use std::path::Path;

use super::{exists, list_dir, php_server, read, read_json, tr, Action, Found, Kind};
use crate::kit::Os;

/// The CMS with a database: the kit's MariaDB in `requires` (`mod.rs`).
pub(super) const CMS: [&str; 5] = ["WordPress", "PrestaShop", "Drupal", "Joomla", "TYPO3"];

/// The `.htaccess` files of a PHP project, besides `<root>\.htaccess`.
pub(super) const HTACCESS: [&str; 4] = [
    ".htaccess",
    r"public\.htaccess",
    r"web\.htaccess",
    r"htdocs\.htaccess",
];

/// The directives of the stock `.htaccess` of WordPress and Laravel (and of
/// small files that only turn rewriting on or listings off), lowercase with
/// single spaces: what Caddy's front controller does already.
const KNOWN_RULES: [&str; 26] = [
    "<ifmodule mod_rewrite.c>",
    "<ifmodule rewrite_module>",
    "<ifmodule mod_negotiation.c>",
    "</ifmodule>",
    "rewriteengine on",
    "rewritebase /",
    // WordPress
    r"rewriterule ^index\.php$ - [l]",
    "rewritecond %{request_filename} !-f",
    "rewritecond %{request_filename} !-d",
    "rewriterule . /index.php [l]",
    "rewriterule .* - [e=http_authorization:%{http:authorization}]",
    // Laravel
    "options -multiviews -indexes",
    "options -multiviews",
    "rewritecond %{http:authorization} .",
    "rewritecond %{http:x-xsrf-token} .",
    "rewriterule .* - [e=http_x_xsrf_token:%{http:x-xsrf-token}]",
    "rewritecond %{request_uri} (.+)/$",
    "rewriterule ^ %1 [l,r=301]",
    "rewriterule ^(.*)/$ /$1 [l,r=301]",
    "rewriterule ^ index.php [l]",
    // Listings and index only.
    "options -indexes",
    "options +followsymlinks",
    "options -indexes +followsymlinks",
    "directoryindex index.php",
    "directoryindex index.php index.html",
    "directoryindex index.html index.php",
];

/// Drupal's stock `.htaccess`, recognised by its first lines: long, but
/// all of it is what its front controller and Caddy's rules do.
const DRUPAL_HTACCESS: &str = "Apache/PHP/Drupal settings";

/// A `.htaccess` with rules of its own: any directive besides those of the
/// stock files of WordPress, Laravel and Drupal ([`KNOWN_RULES`]).
/// Comments and empty lines are not rules.
pub fn htaccess_has_own_rules(text: &str) -> bool {
    if text.contains(DRUPAL_HTACCESS) {
        return false;
    }
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .any(|l| !KNOWN_RULES.contains(&l.to_ascii_lowercase().as_str()))
}

/// Apache is proposed for a PHP project on `os`: only Windows has the
/// kit's Apache, and only one of the `htaccess` texts has rules of its own
/// (Caddy serves the others as they are).
pub fn apache_offered(os: Os, htaccess: &[String]) -> bool {
    os.is_windows() && htaccess.iter().any(|t| htaccess_has_own_rules(t))
}

/// The texts of the `.htaccess` files of the project in `dir` whose site is
/// `root` (the project folder and the usual document roots).
pub(super) fn htaccess_texts(dir: &Path, root: &str) -> Vec<String> {
    let site = format!(r"{root}\.htaccess");
    HTACCESS
        .iter()
        .copied()
        .chain((root != ".").then_some(site.as_str()))
        .filter(|f| exists(dir, f))
        .map(|f| read(dir, f))
        .collect()
}

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
    let caddy = || Action::caddy("caddy", &tr(lang, "sito web Caddy ({0})", &[&shown]), root);
    let has_htaccess =
        exists(dir, ".htaccess") || (root != "." && exists(dir, &format!(r"{root}\.htaccess")));
    // Rules of its own Caddy cannot read: Apache first, on Windows.
    let with_apache = apache_offered(Os::current(), &htaccess_texts(dir, root));
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
        // Its site after `artisan serve`: Apache first for rules of its own.
        if with_apache {
            actions.push(apache());
        }
        actions.push(caddy());
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
        let has_nginx = super::has_nginx(dir, root);
        // A CMS or an `.htaccess`: its site first, the main button (Apache
        // before Caddy for rules of its own). Otherwise proposed after
        // `php -S`.
        let site_first = has_htaccess || cms.is_some();
        if site_first {
            if with_apache {
                actions.push(apache());
            }
            actions.push(caddy());
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
        if !site_first {
            if with_apache {
                actions.push(apache());
            }
            actions.push(caddy());
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
