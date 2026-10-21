//! phpMyAdmin (D-107): the newest release with every language, without its
//! setup pages.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use crate::install::{Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// phpMyAdmin's list of its releases.
const PMA_VERSIONS: &str = "https://www.phpmyadmin.net/home_page/version.json";

/// `(version, zip)` of the newest phpMyAdmin from `version.json`: the
/// build with every language (the interface follows the app's).
pub(crate) fn phpmyadmin_zip(versions: &Value) -> Result<(String, String)> {
    let version = versions
        .get("version")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .ok_or_else(|| anyhow!("phpMyAdmin: no version in version.json"))?;
    Ok((
        version.to_owned(),
        format!(
            "https://files.phpmyadmin.net/phpMyAdmin/{version}/phpMyAdmin-{version}-all-languages.zip"
        ),
    ))
}

/// phpMyAdmin (D-107): the release with every language, SHA-256 from the
/// `.sha256` file next to it; the folder is `phpmyadmin\<line>`.
pub struct PhpMyAdmin;

impl Installer for PhpMyAdmin {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let (version, url) = phpmyadmin_zip(&ctx.net.json(PMA_VERSIONS)?)?;
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("phpMyAdmin.zip")
            .to_owned();
        let sha = crate::net::hash_in_list(&ctx.net.text(&format!("{url}.sha256"))?, &name)
            .ok_or_else(|| anyhow!("phpMyAdmin {version}: no SHA-256"))?;
        Ok(Release::single(
            version,
            url,
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Single,
        ))
    }

    /// Without its setup pages (the kit writes `config.inc.php`).
    fn before_swap(&self, _ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        let setup = tree.join("setup");
        if setup.is_dir() {
            fs::remove_dir_all(&setup)?;
        }
        let _ = fs::remove_file(tree.join("config.sample.inc.php"));
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/phpmyadmin.rs"]
mod tests;
