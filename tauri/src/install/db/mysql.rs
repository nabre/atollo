//! MySQL (D-110): Oracle's generic builds, each line's version and the
//! SHA-256 of its files pinned in the catalog, stripped of what the kit
//! never runs.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use super::remove_debug_symbols;
use crate::install::lang::version_key;
use crate::install::{target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// MySQL's builds (D-110): Oracle publishes an MD5 and a GPG signature, no
/// SHA-256. Each line's version and the SHA-256 of each of its files are
/// fixed in the catalog (`sha256ByLine` of `mysql`), taken from the files
/// after their signature was verified with Oracle's MySQL release key
/// (fingerprint `BCA4 3417 C3B4 85DD 128E C6D4 B7B3 B788 A8D3 785C`): only
/// those lines are offered, and a file with another hash is refused.
const MYSQL_CDN: &str = "https://cdn.mysql.com";

/// The version of a line of MySQL and the SHA-256 of its file per download
/// target (`win32-x64`, `darwin-*`, `linux-*`; Windows ARM64 takes x64's).
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MysqlPin {
    pub version: String,
    pub sha256_by_target: std::collections::BTreeMap<String, String>,
}

/// `sha256ByLine` of the `mysql` entry of a catalog file (`components.json`).
pub fn mysql_pins_in(catalog: &Value) -> std::collections::BTreeMap<String, MysqlPin> {
    catalog["components"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c["id"] == "mysql")
        .and_then(|c| serde_json::from_value(c["sha256ByLine"].clone()).ok())
        .unwrap_or_default()
}

/// The pins of the catalog embedded in the app.
pub fn mysql_pins() -> &'static std::collections::BTreeMap<String, MysqlPin> {
    static PINS: std::sync::OnceLock<std::collections::BTreeMap<String, MysqlPin>> =
        std::sync::OnceLock::new();
    PINS.get_or_init(|| {
        serde_json::from_str::<Value>(include_str!("../../../catalog/components.json"))
            .map(|v| mysql_pins_in(&v))
            .unwrap_or_default()
    })
}

/// The file of MySQL `version` for the download `target` (`x64Only` on
/// Windows ARM64: the caller gives x64) and its archive kind: Oracle's
/// generic builds, glibc 2.28 on Linux (the minimal one on x64, the only
/// one on ARM64), macOS 15 on both Mac CPUs.
pub fn mysql_file(target: Target, version: &str) -> (String, Kind) {
    match (target.os, target.arch) {
        (Os::Win32, _) => (format!("mysql-{version}-winx64.zip"), Kind::Zip),
        (Os::Darwin, Arch::Arm64) => (format!("mysql-{version}-macos15-arm64.tar.gz"), Kind::TarGz),
        (Os::Darwin, Arch::X64) => (
            format!("mysql-{version}-macos15-x86_64.tar.gz"),
            Kind::TarGz,
        ),
        (Os::Linux, Arch::X64) => (
            format!("mysql-{version}-linux-glibc2.28-x86_64-minimal.tar.xz"),
            Kind::TarXz,
        ),
        (Os::Linux, Arch::Arm64) => (
            format!("mysql-{version}-linux-glibc2.28-aarch64.tar.xz"),
            Kind::TarXz,
        ),
    }
}

/// The address of `file` of `line` on Oracle's CDN.
pub fn mysql_url(line: &str, file: &str) -> String {
    format!("{MYSQL_CDN}/Downloads/MySQL-{line}/{file}")
}

/// The lines with a pinned file for the download `target`, newest first.
pub fn mysql_lines_for(
    pins: &std::collections::BTreeMap<String, MysqlPin>,
    target: Target,
) -> Vec<String> {
    let mut lines: Vec<String> = pins
        .iter()
        .filter(|(line, pin)| {
            crate::versions::valid_line(line)
                && crate::versions::line_of(&pin.version).as_deref() == Some(line.as_str())
                && pin.sha256_by_target.contains_key(target.as_str())
        })
        .map(|(line, _)| line.clone())
        .collect();
    lines.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    lines
}

/// The release of `line` for the download `target`, with its pinned
/// SHA-256; an error for a line or target without one (never a download
/// that cannot be verified).
pub fn mysql_release_for(
    pins: &std::collections::BTreeMap<String, MysqlPin>,
    target: Target,
    line: &str,
) -> Result<Release> {
    if !mysql_lines_for(pins, target).iter().any(|l| l == line) {
        bail!(
            "MySQL {line}: no build with a fixed SHA-256 for {target} in this version of the app"
        );
    }
    let pin = &pins[line];
    let sha = &pin.sha256_by_target[target.as_str()];
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("MySQL {line}: the SHA-256 fixed for {target} is not one");
    }
    let (file, kind) = mysql_file(target, &pin.version);
    Ok(Release::single(
        pin.version.clone(),
        mysql_url(line, &file),
        file,
        Expect::Sha256(sha.to_ascii_lowercase()),
        kind,
        Inner::Single,
    ))
}

/// The parts of a MySQL build the kit never runs (D-110, `mysql::STRIP_*`
/// of the service): removed from the new tree before the swap.
pub fn strip_mysql(tree: &Path) -> Result<()> {
    use crate::db::services::mysql::{STRIP_DIRS, STRIP_FILES};
    for dir in STRIP_DIRS {
        let path = crate::fsx::join_rel(tree, dir);
        if path.is_dir() {
            crate::net::archive::remove_dir(&path)?;
        }
    }
    for file in STRIP_FILES {
        let _ = fs::remove_file(crate::fsx::join_rel(tree, file));
    }
    remove_debug_symbols(tree)
}

pub struct MySql;

impl Installer for MySql {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let pins = mysql_pins();
        let line = mysql_lines_for(pins, target)
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("MySQL: no build with a fixed SHA-256 for {target}"))?;
        mysql_release_for(pins, target, &line)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        Ok(mysql_lines_for(mysql_pins(), target_of(ctx, component)))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        mysql_release_for(mysql_pins(), target_of(ctx, component), line)
    }

    /// No test suite, debug symbols or debug builds; on Windows the kit's
    /// copy of the VC++ runtime next to `mysqld.exe`, as for MongoDB (the
    /// zip has none).
    fn before_swap(&self, ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        strip_mysql(tree)?;
        if ctx.kit.os().is_windows() {
            crate::install::web::copy_vc_runtime(ctx, &tree.join("bin"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/mysql.rs"]
mod tests;
