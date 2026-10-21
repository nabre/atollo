//! PHP for macOS and Linux (D-112): the kit's own builds pinned per line
//! (issue #4) or static-php-cli's archives with a SHA-256 fixed in the
//! catalog, and the layout of their folder.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::apache::{line_pins_in, HTTPD_RELEASES};
use super::php::php_release_of;
use crate::install::db::{mysql_lines_for, MysqlPin};
use crate::install::lang::{is_release, version_key};
use crate::install::{Ctx, Dest, Part, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;

/// PHP for macOS and Linux (D-112): the kit's own builds where the catalog
/// pins one for the line ([`php_build_release`], issue #4: with
/// `pdo_sqlite` and `pdo_pgsql`), otherwise the builds of static-php-cli,
/// one static binary per archive, `php` (CLI) and `php-fpm` (the FastCGI
/// server of Caddy, `projects/fastcgi.rs`), with their extensions built in.
/// Its "bulk" set: the only one with `intl`, `mysqli` (phpMyAdmin,
/// WordPress) and `sodium`. Its `php -m` does not name `pdo_sqlite`, but
/// PDO has the `sqlite` driver (`PDO::getAvailableDrivers()`: mysql,
/// pgsql, sqlite; a new Laravel migrates on SQLite with it).
pub const STATIC_PHP: &str = "https://dl.static-php.dev/static-php-cli/bulk";

/// static-php-cli's name of the system of `target`; `None` on Windows,
/// whose PHP comes from windows.php.net.
pub fn static_php_os(target: Target) -> Option<&'static str> {
    match target.os {
        Os::Darwin => Some("macos"),
        Os::Linux => Some("linux"),
        Os::Win32 => None,
    }
}

/// The two archives of PHP `version` for `target`, CLI then FPM:
/// `php-8.4.23-cli-linux-x86_64.tar.gz`, `php-8.4.23-fpm-linux-x86_64.tar.gz`.
/// Only the exact system and architecture, never a build of another one.
pub fn static_php_assets(target: Target, version: &str) -> Option<[String; 2]> {
    let os = static_php_os(target)?;
    let arch = target.gnu_arch();
    Some(["cli", "fpm"].map(|sapi| format!("php-{version}-{sapi}-{os}-{arch}.tar.gz")))
}

/// The SHA-256 the catalog fixes for static-php-cli's archives
/// (`sha256ByFile` of `php`, by file name): static-php-cli publishes no
/// hash, so only an archive with a fixed hash is installed, and one with
/// another hash is refused (D-108).
pub fn static_php_pins(catalog: &Value) -> BTreeMap<String, String> {
    catalog
        .get("components")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|c| c.get("id").and_then(Value::as_str) == Some("php"))
        .and_then(|c| c.get("sha256ByFile"))
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .filter_map(|(file, sha)| {
            let sha = sha.as_str()?.to_ascii_lowercase();
            (sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()))
                .then(|| (file.clone(), sha))
        })
        .collect()
}

/// The pins of this build of the app's catalog.
pub(super) fn catalog_pins() -> &'static BTreeMap<String, String> {
    static PINS: std::sync::OnceLock<BTreeMap<String, String>> = std::sync::OnceLock::new();
    PINS.get_or_init(|| {
        serde_json::from_str(include_str!("../../../catalog/components.json"))
            .map(|v: Value| static_php_pins(&v))
            .unwrap_or_default()
    })
}

/// The versions of PHP `pins` has both archives of for `target`, one per
/// line (its newest), newest first: `[("8.5", "8.5.8"), ("8.4", "8.4.23")]`.
pub fn static_php_versions(
    pins: &BTreeMap<String, String>,
    target: Target,
) -> Vec<(String, String)> {
    let mut best: BTreeMap<String, String> = BTreeMap::new();
    for file in pins.keys() {
        let Some(version) = file
            .strip_prefix("php-")
            .and_then(|rest| rest.split_once("-cli-"))
            .map(|(v, _)| v)
        else {
            continue;
        };
        if !is_release(version) {
            continue;
        }
        let Some(assets) = static_php_assets(target, version) else {
            continue;
        };
        if !assets.iter().all(|a| pins.contains_key(a)) {
            continue;
        }
        let Some(line) = crate::versions::line_of(version) else {
            continue;
        };
        let newer = best
            .get(&line)
            .is_none_or(|v| version_key(version) > version_key(v));
        if newer {
            best.insert(line, version.to_owned());
        }
    }
    let mut out: Vec<(String, String)> = best.into_iter().collect();
    out.sort_by_key(|(l, _)| std::cmp::Reverse(version_key(l)));
    out
}

/// The parts of PHP `version` for `target` from static-php-cli: the CLI
/// archive is the folder (`php` put in `bin/` by `before_swap`), the FPM one
/// its `php-fpm` (put in `sbin/` by `finish`), each checked against its
/// fixed hash.
pub fn static_php_parts(
    pins: &BTreeMap<String, String>,
    target: Target,
    version: &str,
) -> Result<Vec<Part>> {
    let assets = static_php_assets(target, version)
        .ok_or_else(|| anyhow!("PHP from static-php-cli is not for {target}"))?;
    let [cli, fpm] = assets.map(|file| {
        let sha = pins.get(&file).cloned();
        (file, sha)
    });
    let part = |(file, sha): (String, Option<String>), dest: Dest| -> Result<Part> {
        let sha = sha.ok_or_else(|| anyhow!("{file}: no SHA-256 fixed in the catalog"))?;
        Ok(Part {
            url: format!("{STATIC_PHP}/{file}"),
            file_name: file,
            expect: Expect::Sha256(sha),
            kind: Kind::TarGz,
            inner: Inner::Root,
            dest,
            sign: None,
        })
    };
    Ok(vec![
        part(cli, Dest::Own)?,
        part(fpm, Dest::Inside("php-fpm"))?,
    ])
}

/// The release of `line` (the newest pinned with `None`) for `target`
/// from static-php-cli.
fn static_php_release(ctx: &Ctx, target: Target, line: Option<&str>) -> Result<Release> {
    let pins = catalog_pins();
    let versions = static_php_versions(pins, target);
    let found = match line {
        Some(l) => versions.iter().find(|(vl, _)| vl == l),
        None => versions.first(),
    };
    let (_, version) = found.ok_or_else(|| match line {
        Some(l) => anyhow!("PHP {l} is not available for {target} (no build with a fixed hash)"),
        None => anyhow!("no PHP build with a fixed hash for {target}"),
    })?;
    php_release_of(
        ctx,
        version.clone(),
        static_php_parts(pins, target, version)?,
    )
}

/// The kit's PHP builds for macOS and Linux (`scripts/build-php`, issue #4):
/// releases `php-v<version>` of this repository, one tarball per target with
/// `bin/php` and `sbin/php-fpm`, built with static-php-cli from the
/// extensions of its "bulk" set but `swoole-hook-sqlite` and
/// `swoole-hook-pgsql`, so with the real `pdo_sqlite` and `pdo_pgsql`. Each
/// line's version (`8.4.23-1`: PHP and the kit's build number) and the
/// SHA-256 of each tarball are fixed in `sha256ByLine` of `php`.
pub(super) fn php_build_pins() -> &'static BTreeMap<String, MysqlPin> {
    static PINS: std::sync::OnceLock<BTreeMap<String, MysqlPin>> = std::sync::OnceLock::new();
    PINS.get_or_init(|| {
        serde_json::from_str::<Value>(include_str!("../../../catalog/components.json"))
            .map(|v| line_pins_in(&v, "php"))
            .unwrap_or_default()
    })
}

/// The tarball of the kit's PHP `version` (`8.4.23-1`) for a macOS or Linux
/// `target`.
pub fn php_build_file(target: Target, version: &str) -> String {
    format!("php-{version}-{target}.tar.gz")
}

/// The address of `file` in the release of the kit's PHP `version`.
pub fn php_build_url(version: &str, file: &str) -> String {
    format!("{HTTPD_RELEASES}/php-v{version}/{file}")
}

/// The part of the kit's PHP build of `line` for `target` when `pins` has one
/// with a valid SHA-256, and its version (`8.4.23-1`).
pub fn php_build_part(
    pins: &BTreeMap<String, MysqlPin>,
    target: Target,
    line: &str,
) -> Option<(String, Part)> {
    if target.os == Os::Win32 || !mysql_lines_for(pins, target).iter().any(|l| l == line) {
        return None;
    }
    let pin = &pins[line];
    let sha = pin
        .sha256_by_target
        .get(target.as_str())?
        .to_ascii_lowercase();
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let file = php_build_file(target, &pin.version);
    Some((
        pin.version.clone(),
        Part {
            url: php_build_url(&pin.version, &file),
            file_name: file,
            expect: Expect::Sha256(sha),
            kind: Kind::TarGz,
            inner: Inner::Root,
            dest: Dest::Own,
            sign: None,
        },
    ))
}

/// The lines of PHP for macOS or Linux `target`, newest first, each with the
/// version installed: the kit's build when pinned, otherwise
/// static-php-cli's.
pub fn unix_php_versions(
    builds: &BTreeMap<String, MysqlPin>,
    bulk: &BTreeMap<String, String>,
    target: Target,
) -> Vec<(String, String)> {
    let mut out = static_php_versions(bulk, target);
    for line in mysql_lines_for(builds, target) {
        let version = builds[&line].version.clone();
        match out.iter_mut().find(|(l, _)| *l == line) {
            Some(found) => found.1 = version,
            None => out.push((line, version)),
        }
    }
    out.sort_by_key(|(l, _)| std::cmp::Reverse(version_key(l)));
    out
}

/// The release of `line` (the newest with `None`) for macOS or Linux: the
/// kit's build when the catalog pins one, static-php-cli's otherwise.
pub(super) fn unix_php_release(ctx: &Ctx, target: Target, line: Option<&str>) -> Result<Release> {
    let builds = php_build_pins();
    let newest = unix_php_versions(builds, catalog_pins(), target);
    let wanted = line.or_else(|| newest.first().map(|(l, _)| l.as_str()));
    if let Some((version, part)) = wanted.and_then(|l| php_build_part(builds, target, l)) {
        // Composer is chosen for PHP's version; the release keeps the build.
        let php = version.split('-').next().unwrap_or(&version).to_owned();
        let mut release = php_release_of(ctx, php.clone(), vec![part])?;
        release.version = release.version.replacen(&php, &version, 1);
        return Ok(release);
    }
    static_php_release(ctx, target, line)
}

/// The CLI archive of static-php-cli has `php` at its root: it goes to
/// `bin/php`, the key of PHP on macOS and Linux (`setup/catalog.rs`).
pub fn static_php_layout(tree: &Path) -> Result<()> {
    let bin = tree.join("bin");
    let php = bin.join("php");
    let top = tree.join("php");
    if top.is_file() {
        fs::create_dir_all(&bin)?;
        fs::rename(&top, &php).context("bin/php")?;
    }
    if !php.is_file() {
        bail!("the archive does not contain php");
    }
    Ok(())
}

/// The `php-fpm` of the FPM archive, put at the folder's root by its part
/// (`Dest::Inside`), goes to `sbin/php-fpm` (`projects/fastcgi.rs`).
pub fn static_php_fpm_in_place(dir: &Path) -> Result<()> {
    let loose = dir.join("php-fpm");
    if !loose.is_file() {
        return Ok(());
    }
    let sbin = dir.join("sbin");
    fs::create_dir_all(&sbin)?;
    let fpm = sbin.join("php-fpm");
    let _ = fs::remove_file(&fpm);
    fs::rename(&loose, &fpm).context("sbin/php-fpm")?;
    let _ = crate::router::fs::set_executable(&fpm);
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/web/php_unix.rs"]
mod tests;
