//! PHP: windows.php.net's Thread Safe x64 builds (with the archived EOL
//! lines), the installer of every system, and the Composer of each PHP
//! folder with its launchers.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::php_ini::{ensure_php_ini, migrate_php_ini};
use super::php_unix::{
    catalog_pins, php_build_pins, static_php_fpm_in_place, static_php_layout, unix_php_release,
    unix_php_versions,
};
use super::vcruntime::{copy_vc_runtime, LogProgress};
use crate::common::fsx::{join_rel, write_atomic};
use crate::install::lang::{is_release, version_key};
use crate::install::{fetch, pattern, target_of, Ctx, Dest, Installer, Part, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::hash_in_list;
use crate::net::verify::Expect;
use crate::setup::Component;

/// PHP Thread Safe x64 (the Apache module needs it), newest branch or the
/// one in the settings; in each of its folders, the Composer that runs on it.
pub struct Php;

/// `(version, zip path, sha256)` of the TS x64 build of `branch` (or the
/// newest branch) in windows.php.net's `releases.json`.
pub fn pick_php(releases: &Value, branch: Option<&str>) -> Result<(String, String, String)> {
    let obj = releases
        .as_object()
        .ok_or_else(|| anyhow!("PHP releases"))?;
    let branches = obj
        .keys()
        .filter(|k| is_release(k) && k.split('.').count() == 2);
    let chosen = match branch {
        Some(b) => obj
            .keys()
            .find(|k| *k == b)
            .ok_or_else(|| anyhow!("PHP {b} not available"))?,
        None => branches
            .max_by_key(|k| version_key(k))
            .ok_or_else(|| anyhow!("no PHP branch"))?,
    };
    let rel = &obj[chosen];
    let re = pattern(r"^ts-v[sc]\d+-x64$")?;
    let build = rel
        .as_object()
        .and_then(|b| b.iter().find(|(k, _)| re.is_match(k)))
        .map(|(_, v)| v)
        .ok_or_else(|| anyhow!("PHP {chosen}: no Thread Safe x64 build"))?;
    let get = |p: &str| build.pointer(p).and_then(Value::as_str).map(str::to_owned);
    Ok((
        rel.get("version")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("PHP version"))?
            .to_owned(),
        get("/zip/path").ok_or_else(|| anyhow!("PHP zip path"))?,
        get("/zip/sha256").ok_or_else(|| anyhow!("PHP zip sha256"))?,
    ))
}

/// `(version, URL)` of the newest Composer that runs on PHP `php` (`8.4.13`),
/// from getcomposer.org's `versions` (the list `composer self-update`
/// reads, newest first): the stable one, or the 2.2 LTS for a PHP older
/// than the stable's `min-php` (`70205` = 7.2.5).
pub fn pick_composer(versions: &Value, php: &str) -> Result<(String, String)> {
    let php_id = {
        let n: Vec<u64> = php
            .split(|c: char| !c.is_ascii_digit())
            .filter(|p| !p.is_empty())
            .take(3)
            .filter_map(|p| p.parse().ok())
            .collect();
        n.first().copied().unwrap_or(0) * 10000
            + n.get(1).copied().unwrap_or(0) * 100
            + n.get(2).copied().unwrap_or(0)
    };
    let entry = ["stable", "2.2"]
        .iter()
        .filter_map(|ch| versions.get(*ch).and_then(Value::as_array))
        .flatten()
        .find(|e| e.get("min-php").and_then(Value::as_u64).unwrap_or(0) <= php_id)
        .ok_or_else(|| anyhow!("Composer: no stable version for PHP {php}"))?;
    let get = |k: &str| {
        entry
            .get(k)
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("Composer versions: no {k}"))
    };
    let path = get("path")?;
    if !path.starts_with('/') {
        bail!("Composer versions: unexpected path {path}");
    }
    Ok((
        get("version")?.to_owned(),
        format!("https://getcomposer.org{path}"),
    ))
}

/// PHP `version` with the Composer that runs on it, inside its folder: each
/// version of PHP has its own (`php\8.4\composer.phar`).
fn php_release(ctx: &Ctx, version: String, path: String, sha: String) -> Result<Release> {
    php_release_of(ctx, version, vec![php_part(path, sha)])
}

/// The release of PHP `version` made of `parts`, Composer added after them.
pub(super) fn php_release_of(ctx: &Ctx, version: String, mut parts: Vec<Part>) -> Result<Release> {
    let (composer_version, composer) =
        pick_composer(&ctx.net.json("https://getcomposer.org/versions")?, &version)?;
    let composer_sha = hash_in_list(
        &ctx.net.text(&format!("{composer}.sha256sum"))?,
        "composer.phar",
    )
    .ok_or_else(|| anyhow!("Composer: no SHA-256"))?;
    parts.push(Part {
        url: composer,
        file_name: "composer.phar".into(),
        expect: Expect::Sha256(composer_sha),
        kind: Kind::File,
        inner: Inner::Root,
        dest: Dest::Inside("composer.phar"),
        sign: None,
    });
    Ok(Release {
        version: format!("{version} (Composer {composer_version})"),
        parts,
    })
}

/// The branches of `releases.json` with a Thread Safe x64 build, newest first.
pub fn php_lines(releases: &Value) -> Vec<String> {
    let mut out: Vec<String> = releases
        .as_object()
        .into_iter()
        .flatten()
        .map(|(k, _)| k.clone())
        .filter(|k| crate::versions::valid_line(k) && pick_php(releases, Some(k)).is_ok())
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out
}

const PHP_RELEASES: &str = "https://windows.php.net/downloads/releases";

/// Final EOL Thread Safe x64 builds archived on windows.php.net (`(line, version, path, sha256)`).
const PHP_ARCHIVES: &[(&str, &str, &str, &str)] = &[
    (
        "8.1",
        "8.1.34",
        "archives/php-8.1.34-Win32-vs16-x64.zip",
        "8e17e0804fe48d3a032c9bef16f0f922996e0b1b237061b7ce94485394db5d1b",
    ),
    (
        "8.0",
        "8.0.30",
        "archives/php-8.0.30-Win32-vs16-x64.zip",
        "8697972b5f704885e3444422ede2b5c25d025f614ea96dc67de74dc582471562",
    ),
    (
        "7.4",
        "7.4.33",
        "archives/php-7.4.33-Win32-vc15-x64.zip",
        "cdbb85b45f38f282f05764ca08648b5f92db99c75b2fb3848eb4a559f6553b48",
    ),
    (
        "7.3",
        "7.3.33",
        "archives/php-7.3.33-Win32-VC15-x64.zip",
        "eabf265d817ea534eeeeb51995693a8fd215bc6cfc42d5fcd5e5cd56a1d5da53",
    ),
    (
        "7.2",
        "7.2.34",
        "archives/php-7.2.34-Win32-VC15-x64.zip",
        "3a3d026aacf8da57ff01ace2f78f8f57494744bfb669f8545cc8f9c4ff0450a5",
    ),
    (
        "7.1",
        "7.1.33",
        "archives/php-7.1.33-Win32-VC14-x64.zip",
        "2f6b0c6b62de88cf5cc16e68ac6d5e3b73fc01bcb03b51fe8c8534965b8e808c",
    ),
    (
        "7.0",
        "7.0.33",
        "archives/php-7.0.33-Win32-VC14-x64.zip",
        "8a22c93732b576507888e011b147df80beee7b7e24f527cb6e1282b5228350c2",
    ),
    (
        "5.6",
        "5.6.40",
        "archives/php-5.6.40-Win32-VC11-x64.zip",
        "b312df0b16ec645adfd7ad39ce6f9dd294a4dfeb8130ee8041f8af0f89f62904",
    ),
];

/// The PHP zip of a branch, the part that becomes the `php` folder.
fn php_part(path: String, sha: String) -> Part {
    let file_name = path.rsplit('/').next().unwrap_or(&path).to_owned();
    Part {
        url: format!("{PHP_RELEASES}/{path}"),
        file_name,
        expect: Expect::Sha256(sha),
        kind: Kind::Zip,
        inner: Inner::Root,
        dest: Dest::Own,
        sign: None,
    }
}

impl Installer for Php {
    fn has_lines(&self) -> bool {
        true
    }

    /// Composer comes after the zip (`php_release`); on macOS and Linux the
    /// CLI archive has `php` at its root, put in `bin/` by `before_swap`.
    fn inside(&self) -> &'static [&'static str] {
        &["composer.phar", "bin/php"]
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        if !target.is_windows() {
            return Ok(unix_php_versions(php_build_pins(), catalog_pins(), target)
                .into_iter()
                .map(|(line, _)| line)
                .collect());
        }
        let mut out = php_lines(&ctx.net.json(&format!("{PHP_RELEASES}/releases.json"))?);
        for (line, _, _, _) in PHP_ARCHIVES {
            if !out.iter().any(|l| l == line) {
                out.push((*line).to_owned());
            }
        }
        out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        if !target.is_windows() {
            return unix_php_release(ctx, target, Some(line));
        }
        let releases: Value = ctx.net.json(&format!("{PHP_RELEASES}/releases.json"))?;
        let (version, path, sha) = match pick_php(&releases, Some(line)) {
            Ok(found) => found,
            Err(e) => PHP_ARCHIVES
                .iter()
                .find(|(l, _, _, _)| *l == line)
                .map(|(_, v, p, s)| ((*v).to_owned(), (*p).to_owned(), (*s).to_owned()))
                .ok_or(e)?,
        };
        php_release(ctx, version, path, sha)
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        // The newest line; others come from the Tools page (D-80, D-137).
        let target = target_of(ctx, component);
        if !target.is_windows() {
            return unix_php_release(ctx, target, None);
        }
        let releases: Value = ctx.net.json(&format!("{PHP_RELEASES}/releases.json"))?;
        let (version, path, sha) = pick_php(&releases, None)?;
        php_release(ctx, version, path, sha)
    }

    /// VC++ DLLs next to `php.exe`; `cacert.pem` for HTTPS from PHP (PHP does
    /// not read the Windows store; a failure is only a warning).
    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        copy_vc_runtime(ctx, tree)?;
        if !ctx.kit.os().is_windows() {
            static_php_layout(tree)?;
        }
        let ssl = join_rel(tree, "extras/ssl");
        fs::create_dir_all(&ssl)?;
        let pem = (|| -> Result<()> {
            let sha = hash_in_list(
                &ctx.net.text("https://curl.se/ca/cacert.pem.sha256")?,
                "cacert.pem",
            )
            .ok_or_else(|| anyhow!("cacert.pem: no SHA-256"))?;
            let file = fetch(
                ctx,
                "https://curl.se/ca/cacert.pem",
                "cacert.pem",
                &Expect::Sha256(sha),
                &ssl,
                &mut LogProgress,
            )?;
            debug_assert!(file.is_file());
            Ok(())
        })();
        if let Err(e) = pem {
            crate::common::log::warn(&format!(
                "PHP: CA certificates not downloaded ({e:#}): HTTPS from PHP may fail"
            ));
        }
        Ok(())
    }

    /// `data\php\<line>\php.ini` of this line once (then it is the user's),
    /// Composer launchers.
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        if !ctx.kit.os().is_windows() {
            static_php_fpm_in_place(dest)?;
        }
        migrate_php_ini(ctx.kit)?;
        ensure_php_ini(&ctx.kit.data(), dest)?;
        composer_launchers(dest)?;
        // An older kit's Composer, one for every PHP, first on the PATH.
        let tools = ctx.kit.apps().join("tools");
        for name in ["composer.phar", "composer.cmd", "composer"] {
            let _ = fs::remove_file(tools.join(name));
        }
        Ok(())
    }
}

/// An older kit had one Composer for every PHP, in `tools\`: each folder of
/// PHP gets a copy with its launchers (its next update brings its own), then
/// the old one goes. A folder of PHP whose Composer has no `composer.ps1`
/// (an older kit's) gets its launchers again. At start, with
/// `versions::migrate`.
pub fn adopt_composer(kit: &crate::kit::Kit) -> Result<()> {
    let tools = kit.apps().join("tools");
    let old = tools.join("composer.phar");
    let Some(php) = crate::setup::Catalog::get().find("php") else {
        return Ok(());
    };
    let dirs = std::iter::once(crate::versions::default_dir(kit, php)).chain(
        crate::versions::saved(kit, php)
            .into_iter()
            .map(|l| crate::versions::saved_dir(kit, php, &l.line)),
    );
    for dir in dirs.filter(|d| d.join("php.exe").is_file() || join_rel(d, "bin/php").is_file()) {
        if old.is_file() && !dir.join("composer.phar").is_file() {
            fs::copy(&old, dir.join("composer.phar")).with_context(|| dir.display().to_string())?;
        }
        if dir.join("composer.phar").is_file()
            && (old.is_file() || !dir.join("composer.ps1").is_file())
        {
            composer_launchers(&dir)?;
        }
    }
    if old.is_file() {
        for name in ["composer.phar", "composer.cmd", "composer"] {
            let _ = fs::remove_file(tools.join(name));
        }
    }
    Ok(())
}

/// The launcher of Composer in `bin/` of static-php-cli's PHP (macOS and
/// Linux), on the PATH with `php` (env/path.rs): the `php` beside it runs the
/// `composer.phar` of its own folder.
pub const COMPOSER_UNIX: &[u8] =
    b"#!/bin/sh\nd=\"$(dirname \"$0\")\"\nexec \"$d/php\" \"$d/../composer.phar\" \"$@\"\n";

/// `composer.cmd`, `composer.ps1` and `composer` (Git Bash / POSIX sh) next
/// to `composer.phar` in a folder of PHP: they run it with the `php.exe` or
/// `bin/php` beside them, so each version of PHP uses its own Composer.
/// PowerShell takes `composer.ps1` before `composer.cmd`: through cmd.exe
/// the `^` of `vendor/package:^1.2` would be lost, and the wizard's lines
/// run with PowerShell (D-190). A folder of static-php-cli (`bin/php`, macOS
/// and Linux) also gets `bin/composer` ([`COMPOSER_UNIX`]), since `bin/` is
/// the folder on the PATH there.
pub fn composer_launchers(dir: &Path) -> Result<()> {
    write_atomic(
        &dir.join("composer.cmd"),
        b"@\"%~dp0php.exe\" \"%~dp0composer.phar\" %*\r\n",
    )?;
    write_atomic(
        &dir.join("composer.ps1"),
        b"$php = Join-Path $PSScriptRoot 'bin/php'\r\n\
          if (-not (Test-Path -LiteralPath $php)) { $php = Join-Path $PSScriptRoot 'php.exe' }\r\n\
          & $php (Join-Path $PSScriptRoot 'composer.phar') @args\r\n\
          exit $LASTEXITCODE\r\n",
    )?;
    let sh = dir.join("composer");
    write_atomic(
        &sh,
        b"#!/bin/sh\nd=\"$(dirname \"$0\")\"\nif [ -x \"$d/bin/php\" ]; then\n  exec \"$d/bin/php\" \"$d/composer.phar\" \"$@\"\nfi\nexec \"$d/php.exe\" \"$d/composer.phar\" \"$@\"\n",
    )?;
    let _ = crate::platform::files::set_executable(&sh);
    let bin = dir.join("bin");
    if bin.join("php").is_file() {
        let launcher = bin.join("composer");
        write_atomic(&launcher, COMPOSER_UNIX)?;
        let _ = crate::platform::files::set_executable(&launcher);
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/web/php.rs"]
mod tests;
