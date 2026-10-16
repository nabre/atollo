//! PHP (with Composer and the Visual C++ runtime) and Apache (port of
//! nabre/atollo `_prepare-web.ps1`).
//!
//! The Visual C++ runtime is not installed: its DLLs are extracted from
//! Microsoft's `vc_redist.x64.exe` (signature required: Microsoft publishes
//! no checksum) and copied next to `php.exe` and `httpd.exe`, the app-local
//! deployment Microsoft allows. PHP and Apache exist only for x64: on ARM64
//! they run emulated with the x64 DLLs.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{fetch, pattern, Ctx, Dest, Installer, Part, Progress, Release};
use crate::fsx::write_atomic;
use crate::net::archive::{Inner, Kind};
use crate::net::hash_in_list;
use crate::net::verify::Expect;
use crate::settings::Settings;
use crate::setup::catalog::Component;

/// A progress sink that only logs, for steps that run inside hooks.
struct LogProgress;

impl Progress for LogProgress {
    fn note(&mut self, text: &str) {
        crate::log::info(text);
    }
    fn bytes(&mut self, _done: u64, _total: Option<u64>) {}
}

fn expand_cab(ctx: &Ctx, cab: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    let system_root = ctx.env.get("SystemRoot").unwrap_or(r"C:\Windows");
    let status =
        crate::os::jobs::kit_command(Path::new(system_root).join(r"System32\expand.exe"), ctx.env)
            .arg(cab)
            .arg("-F:*")
            .arg(dest)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("expand.exe")?;
    if !status.success() {
        bail!("expand.exe failed on {} ({status})", cab.display());
    }
    Ok(())
}

/// Offsets and sizes of the CAB archives inside `bytes` (header `MSCF`,
/// version 1.3, size within the file).
pub fn find_cabs(bytes: &[u8]) -> Vec<(usize, usize)> {
    const MAGIC: &[u8] = b"MSCF\0\0\0\0";
    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 26 <= bytes.len() {
        match bytes[pos..].windows(MAGIC.len()).position(|w| w == MAGIC) {
            None => break,
            Some(i) => {
                let at = pos + i;
                if at + 26 > bytes.len() {
                    break;
                }
                let size = u32::from_le_bytes([
                    bytes[at + 8],
                    bytes[at + 9],
                    bytes[at + 10],
                    bytes[at + 11],
                ]) as usize;
                if size > 64
                    && at + size <= bytes.len()
                    && bytes[at + 24] == 3
                    && bytes[at + 25] == 1
                {
                    out.push((at, size));
                    pos = at + size;
                } else {
                    pos = at + 4;
                }
            }
        }
    }
    out
}

/// The DLLs of the Visual C++ runtime in `tools\vcruntime`, extracted once.
pub fn vc_runtime(ctx: &Ctx, force: bool) -> Result<PathBuf> {
    let folder = ctx.kit.apps().join(r"tools\vcruntime");
    if folder.join("vcruntime140.dll").is_file() && !force {
        return Ok(folder);
    }
    // In the host's temporary folder (D-90), removed when `guard` is dropped.
    let guard = crate::os::temp::TempWork::new(ctx.kit, "setup/vcruntime")?;
    let work = guard.path();
    let result = (|| -> Result<()> {
        let exe = work.join("vc_redist.x64.exe");
        ctx.net.download(
            "https://aka.ms/vc14/vc_redist.x64.exe",
            &exe,
            ctx.cancel,
            &mut |_, _| {},
        )?;
        let microsoft = pattern("O=Microsoft Corporation")?;
        crate::net::signature::check(&exe, &microsoft, true)?;
        let bytes = fs::read(&exe)?;
        let cabs = find_cabs(&bytes);
        if cabs.len() < 2 {
            bail!("vc_redist.x64.exe: unknown format");
        }
        for (n, (at, size)) in cabs.iter().enumerate() {
            let cab = work.join(format!("c{n}.cab"));
            fs::write(&cab, &bytes[*at..*at + *size])?;
            expand_cab(ctx, &cab, &work.join(format!("c{n}")))?;
        }
        // The Burn manifest names the cab of the "Minimum" runtime.
        let manifest = walk(work)
            .into_iter()
            .filter(|p| p.file_name().is_some_and(|n| n == "0"))
            .find_map(|p| {
                fs::read_to_string(&p)
                    .ok()
                    .filter(|t| t.contains("BurnManifest"))
            })
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: manifest not found"))?;
        let re = pattern(r#"<Payload\b[^>]*>"#)?;
        let source = re
            .find_iter(&manifest)
            .map(|m| m.as_str())
            .find(|tag| tag.contains(r"vcRuntimeMinimum_amd64\cab1.cab"))
            .and_then(|tag| {
                pattern(r#"SourcePath="([^"]+)""#)
                    .ok()?
                    .captures(tag)
                    .map(|c| c[1].to_owned())
            })
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: no vcRuntimeMinimum x64"))?;
        let dll_cab = walk(work)
            .into_iter()
            .find(|p| p.file_name().is_some_and(|n| n.to_string_lossy() == source))
            .ok_or_else(|| anyhow!("vc_redist.x64.exe: {source} not found"))?;
        let dlls = work.join("dll");
        expand_cab(ctx, &dll_cab, &dlls)?;
        for entry in fs::read_dir(&dlls)?.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            match name.strip_suffix("_amd64").filter(|n| n.ends_with(".dll")) {
                Some(dll) => fs::rename(entry.path(), dlls.join(dll))?,
                None => fs::remove_file(entry.path())?,
            }
        }
        let key = dlls.join("vcruntime140.dll");
        if !key.is_file() {
            bail!("vcruntime140.dll not found in the Microsoft package");
        }
        crate::net::signature::check(&key, &microsoft, true)?;
        crate::net::archive::stage_into(&dlls, &folder)
    })();
    drop(guard);
    result.map(|()| folder)
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).into_iter().flatten().flatten() {
            if e.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(e.path());
            } else {
                out.push(e.path());
            }
        }
    }
    out
}

pub(super) fn copy_vc_runtime(ctx: &Ctx, dest: &Path) -> Result<()> {
    let vc = vc_runtime(ctx, false)?;
    for entry in fs::read_dir(vc)?.flatten() {
        if entry
            .path()
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dll"))
        {
            fs::copy(entry.path(), dest.join(entry.file_name()))?;
        }
    }
    Ok(())
}

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
        .filter(|k| super::lang::is_release(k) && k.split('.').count() == 2);
    let chosen = match branch {
        Some(b) => obj
            .keys()
            .find(|k| *k == b)
            .ok_or_else(|| anyhow!("PHP {b} not available"))?,
        None => branches
            .max_by_key(|k| super::lang::version_key(k))
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
    let (composer_version, composer) =
        pick_composer(&ctx.net.json("https://getcomposer.org/versions")?, &version)?;
    let composer_sha = hash_in_list(
        &ctx.net.text(&format!("{composer}.sha256sum"))?,
        "composer.phar",
    )
    .ok_or_else(|| anyhow!("Composer: no SHA-256"))?;
    Ok(Release {
        version: format!("{version} (Composer {composer_version})"),
        parts: vec![
            php_part(path, sha),
            Part {
                url: composer,
                file_name: "composer.phar".into(),
                expect: Expect::Sha256(composer_sha),
                kind: Kind::File,
                inner: Inner::Root,
                dest: Dest::Inside("composer.phar"),
                sign: None,
            },
        ],
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
    out.sort_by_key(|l| std::cmp::Reverse(super::lang::version_key(l)));
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

    /// Composer comes after the zip (`php_release`).
    fn inside(&self) -> &'static [&'static str] {
        &["composer.phar"]
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let mut out = php_lines(&ctx.net.json(&format!("{PHP_RELEASES}/releases.json"))?);
        for (line, _, _, _) in PHP_ARCHIVES {
            if !out.iter().any(|l| l == line) {
                out.push((*line).to_owned());
            }
        }
        out.sort_by_key(|l| std::cmp::Reverse(super::lang::version_key(l)));
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
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

    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let releases: Value = ctx.net.json(&format!("{PHP_RELEASES}/releases.json"))?;
        let branch = Settings::load(ctx.kit).settings.php_version;
        let (version, path, sha) = match pick_php(&releases, branch.as_deref()) {
            Ok(found) => found,
            Err(e) => branch
                .as_deref()
                .and_then(|b| PHP_ARCHIVES.iter().find(|(l, _, _, _)| *l == b))
                .map(|(_, v, p, s)| ((*v).to_owned(), (*p).to_owned(), (*s).to_owned()))
                .ok_or(e)?,
        };
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
        let ssl = tree.join(r"extras\ssl");
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
            crate::log::warn(&format!(
                "PHP: CA certificates not downloaded ({e:#}): HTTPS from PHP may fail"
            ));
        }
        Ok(())
    }

    /// `data\php\php.ini` once (then it is the user's), Composer launchers.
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let ini = ctx.kit.data().join(r"php\php.ini");
        if !ini.exists() {
            let template = fs::read_to_string(dest.join("php.ini-development"))
                .context("php.ini-development")?;
            fs::create_dir_all(ini.parent().unwrap_or(dest))?;
            write_atomic(&ini, php_ini(&template)?.as_bytes())?;
        } else if let Ok(text) = fs::read_to_string(&ini) {
            // Written before the saved versions (D-80): the paths follow the
            // PHP in use, not the default folder.
            let new = php_ini_paths(&text);
            if new != text {
                write_atomic(&ini, new.as_bytes())?;
            }
        }
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
/// the old one goes. At start, with `versions::migrate`.
pub fn adopt_composer(kit: &crate::kit::Kit) -> Result<()> {
    let tools = kit.apps().join("tools");
    let old = tools.join("composer.phar");
    let Some(php) = crate::setup::catalog::Catalog::get().find("php") else {
        return Ok(());
    };
    if !old.is_file() {
        return Ok(());
    }
    let dirs = std::iter::once(crate::versions::default_dir(kit, php)).chain(
        crate::versions::saved(kit, php)
            .into_iter()
            .map(|l| crate::versions::saved_dir(kit, php, &l.line)),
    );
    for dir in dirs.filter(|d| d.join("php.exe").is_file()) {
        if !dir.join("composer.phar").is_file() {
            fs::copy(&old, dir.join("composer.phar")).with_context(|| dir.display().to_string())?;
        }
        composer_launchers(&dir)?;
    }
    for name in ["composer.phar", "composer.cmd", "composer"] {
        let _ = fs::remove_file(tools.join(name));
    }
    Ok(())
}

/// `composer.cmd` and `composer` (Git Bash) next to `composer.phar` in a
/// folder of PHP: they run it with the `php.exe` beside them, so each
/// version of PHP uses its own Composer.
pub fn composer_launchers(dir: &Path) -> Result<()> {
    write_atomic(
        &dir.join("composer.cmd"),
        b"@\"%~dp0php.exe\" \"%~dp0composer.phar\" %*\r\n",
    )?;
    write_atomic(
        &dir.join("composer"),
        b"#!/bin/sh\nd=\"$(dirname \"$0\")\"\nexec \"$d/php.exe\" \"$d/composer.phar\" \"$@\"\n",
    )?;
    Ok(())
}

/// `${CP_APPS}\php\…` of an older kit's `php.ini` as `${CP_PHP}\…`.
pub fn php_ini_paths(text: &str) -> String {
    text.replace(r"${CP_APPS}\php\", r"${CP_PHP}\")
}

/// The kit's `php.ini` from `php.ini-development`: paths through the kit
/// variables (valid on any drive letter), common extensions on,
/// Europe/Rome, CA bundle for curl and OpenSSL.
pub fn php_ini(template: &str) -> Result<String> {
    let mut t = template.to_owned();
    let swaps: [(&str, &str); 5] = [
        (
            r#"(?m)^;\s*extension_dir\s*=\s*"ext"[ \t]*(\r?)$"#,
            r#"extension_dir = "$${CP_PHP}\ext"$1"#,
        ),
        (
            r"(?m)^;(extension=(?:curl|fileinfo|gd|intl|mbstring|mysqli|openssl|pdo_mysql|pdo_sqlite|sodium|sqlite3|zip))[ \t]*(\r?)$",
            "$1$2",
        ),
        (
            r"(?m)^;date\.timezone\s*=[^\r\n]*",
            "date.timezone = Europe/Rome",
        ),
        (
            r"(?m)^;curl\.cainfo\s*=[^\r\n]*",
            r#"curl.cainfo = "$${CP_PHP}\extras\ssl\cacert.pem""#,
        ),
        (
            r"(?m)^;openssl\.cafile\s*=[^\r\n]*",
            r#"openssl.cafile = "$${CP_PHP}\extras\ssl\cacert.pem""#,
        ),
    ];
    for (re, with) in swaps {
        t = pattern(re)?.replace_all(&t, with).into_owned();
    }
    Ok(t)
}

/// Apache httpd from Apache Lounge (the maintained Windows build). Port,
/// site and PHP live in `data\apache\kit.conf`, which updates never touch.
///
/// Apache Lounge keeps only the latest build of 2.4, its only line: a new
/// build replaces the folder `apache\2.4`.
pub struct Apache;

const APACHE_SITE: &str = "https://www.apachelounge.com";

/// A Win64 build on Apache Lounge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApacheBuild {
    /// `2.4.66-251206`, or `2.4.54` for the older builds.
    pub version: String,
    /// `/download/VS17/binaries/httpd-2.4.66-251206-Win64-VS17.zip`.
    pub path: String,
    pub file: String,
}

/// The Win64 builds linked by a page of Apache Lounge, in page order.
pub fn apache_builds_in(page: &str) -> Vec<ApacheBuild> {
    let Ok(re) = pattern(
        r#"(?i)href="(/download/[^"/]+/binaries/(httpd-(\d+\.\d+\.\d+(?:-\d+)?)-win64-v[cs]\d+\.zip))""#,
    ) else {
        return Vec::new();
    };
    let mut out: Vec<ApacheBuild> = Vec::new();
    for c in re.captures_iter(page) {
        if out.iter().all(|b| !b.file.eq_ignore_ascii_case(&c[2])) {
            out.push(ApacheBuild {
                version: c[3].to_owned(),
                path: c[1].to_owned(),
                file: c[2].to_owned(),
            });
        }
    }
    out
}

/// The builds on the download page of Apache Lounge, the newest first.
fn apache_builds(ctx: &Ctx) -> Result<Vec<ApacheBuild>> {
    let page = ctx.net.text(&format!("{APACHE_SITE}/download/"))?;
    Ok(apache_builds_in(&page))
}

/// The release of a build, with the SHA-256 Apache Lounge publishes.
fn apache_release_of(ctx: &Ctx, b: &ApacheBuild) -> Result<Release> {
    let url = format!("{APACHE_SITE}{}", b.path);
    let sums = ctx.net.text(&format!("{url}.txt"))?;
    let sha = pattern(r"(?is)SHA256-Checksum for:[^\r\n]*\r?\n\s*([0-9a-f]{64})")?
        .captures(&sums)
        .map(|m| m[1].to_owned())
        .ok_or_else(|| anyhow!("Apache {}: no SHA-256", b.version))?;
    Ok(Release::single(
        b.version.clone(),
        url,
        b.file.clone(),
        Expect::Sha256(sha),
        Kind::Zip,
        Inner::Named("Apache24".into()),
    ))
}

impl Installer for Apache {
    fn has_lines(&self) -> bool {
        true
    }

    /// The lines on Apache Lounge, newest first (only 2.4).
    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        for line in apache_builds(ctx)?
            .iter()
            .filter_map(|b| crate::versions::line_of(&b.version))
        {
            if !out.contains(&line) {
                out.push(line);
            }
        }
        out.sort_by_key(|l| std::cmp::Reverse(super::lang::version_key(l)));
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        let build = apache_builds(ctx)?
            .into_iter()
            .find(|b| crate::versions::line_of(&b.version).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("Apache {line} not available"))?;
        apache_release_of(ctx, &build)
    }

    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let build = apache_builds(ctx)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Apache Win64 package not found on apachelounge.com"))?;
        apache_release_of(ctx, &build)
    }

    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        copy_vc_runtime(ctx, &tree.join("bin"))?;
        fs::create_dir_all(tree.join("logs"))?;
        let conf = tree.join(r"conf\httpd.conf");
        let text = fs::read_to_string(&conf).context("httpd.conf")?;
        fs::write(&conf, httpd_conf(&text)?)?;
        Ok(())
    }

    fn finish(&self, ctx: &Ctx, _component: &Component, _dest: &Path) -> Result<()> {
        apache_site(ctx.kit.data().join("apache").as_path())
    }
}

/// Official Windows build of Nginx from `nginx/nginx` GitHub releases.
pub struct Nginx;

const NGINX_REPO: &str = "nginx/nginx";
const NGINX_ASSET: &str = r"^nginx-\d+\.\d+\.\d+\.zip$";

fn nginx_version(tag: &str) -> &str {
    tag.trim_start_matches("release-").trim_start_matches('v')
}

fn nginx_releases(ctx: &Ctx) -> Result<Vec<crate::net::GhRelease>> {
    let all: Vec<crate::net::GhRelease> = ctx.net.json(&format!(
        "https://api.github.com/repos/{NGINX_REPO}/releases?per_page=100"
    ))?;
    let re = pattern(NGINX_ASSET)?;
    let mut out: Vec<crate::net::GhRelease> = all
        .into_iter()
        .filter(|r| !r.prerelease && super::lang::is_release(nginx_version(&r.tag_name)))
        // Only what can be checked (the oldest have a PGP signature only).
        .filter(|r| r.asset(&re).is_some_and(|a| crate::net::verifiable(r, a)))
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(super::lang::version_key(nginx_version(&r.tag_name))));
    Ok(out)
}

fn nginx_release_from(ctx: &Ctx, rel: &crate::net::GhRelease) -> Result<Release> {
    let file = rel
        .asset(&pattern(NGINX_ASSET)?)
        .ok_or_else(|| anyhow!("Nginx {}: no Windows zip asset", rel.tag_name))?;
    Ok(Release::single(
        nginx_version(&rel.tag_name).to_owned(),
        file.browser_download_url.clone(),
        file.name.clone(),
        Expect::Sha256(ctx.net.release_sha256(rel, file)?),
        Kind::Zip,
        Inner::Single,
    ))
}

impl Installer for Nginx {
    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let mut out = Vec::new();
        for r in nginx_releases(ctx)? {
            if let Some(line) = crate::versions::line_of(nginx_version(&r.tag_name)) {
                if !out.contains(&line) {
                    out.push(line);
                }
            }
        }
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        let releases = nginx_releases(ctx)?;
        let rel = releases
            .iter()
            .find(|r| crate::versions::line_of(nginx_version(&r.tag_name)).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("Nginx {line} not available"))?;
        nginx_release_from(ctx, rel)
    }

    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        if let Ok(rel) = ctx.net.github_latest(NGINX_REPO) {
            if let Ok(r) = nginx_release_from(ctx, &rel) {
                return Ok(r);
            }
        }
        let releases = nginx_releases(ctx)?;
        let rel = releases
            .first()
            .ok_or_else(|| anyhow!("no Nginx release found"))?;
        nginx_release_from(ctx, rel)
    }

    fn before_swap(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        fs::create_dir_all(tree.join("logs"))?;
        fs::create_dir_all(tree.join("temp"))?;
        Ok(())
    }
}

/// httpd.conf of the kit: root from the kit variables, no port 80 (it is
/// in kit.conf), mod_rewrite on, kit.conf included.
pub fn httpd_conf(text: &str) -> Result<String> {
    let mut t = pattern(r"(?m)^Define SRVROOT [^\r\n]*")?
        .replace(text, r#"Define SRVROOT "$${CP_APACHE_FWD}""#)
        .into_owned();
    t = pattern(r"(?m)^Listen 80[ \t]*(\r?)$")?
        .replace(&t, "#Listen 80   (port in data\\apache\\kit.conf)$1")
        .into_owned();
    t = pattern(r"(?m)^#(LoadModule rewrite_module [^\r\n]*)")?
        .replace(&t, "$1")
        .into_owned();
    if !t.contains("${CP_APACHE_FWD}") {
        bail!("httpd.conf in an unexpected format (no Define SRVROOT)");
    }
    Ok(format!(
        "{}\r\n\r\n# --- Atollo ---\r\nInclude \"${{CP_ROOT_FWD}}/data/apache/kit.conf\"\r\n",
        t.trim_end()
    ))
}

/// `data\apache\kit.conf`, `conf.d\`, `htdocs\` with a test page; each
/// created once (they belong to the user).
pub fn apache_site(dir: &Path) -> Result<()> {
    fs::create_dir_all(dir.join("conf.d"))?;
    let htdocs = dir.join("htdocs");
    fs::create_dir_all(&htdocs)?;
    let conf = dir.join("kit.conf");
    if !conf.exists() {
        let text = [
            "# Atollo Apache configuration (yours: updates never touch it).",
            "# Paths use variables set by Atollo, so they work on any drive letter.",
            "",
            "# Port: this PC only (127.0.0.1), so the firewall never asks.",
            "Define PORTA 8080",
            "Listen 127.0.0.1:${PORTA}",
            "ServerName localhost:${PORTA}",
            "",
            "# Site folder: data\\apache\\htdocs\\ in the kit",
            "DocumentRoot \"${CP_ROOT_FWD}/data/apache/htdocs\"",
            "<Directory \"${CP_ROOT_FWD}/data/apache/htdocs\">",
            "    Options Indexes FollowSymLinks",
            "    AllowOverride All",
            "    Require all granted",
            "</Directory>",
            "DirectoryIndex index.php index.html",
            "",
            "# The kit's PHP (Apache module), php.ini in data\\php",
            "<IfFile \"${CP_PHP_FWD}/php8apache2_4.dll\">",
            "    LoadFile \"${CP_PHP_FWD}/php8ts.dll\"",
            "    LoadModule php_module \"${CP_PHP_FWD}/php8apache2_4.dll\"",
            "    PHPIniDir \"${CP_ROOT_FWD}/data/php\"",
            "    <FilesMatch \"\\.php$\">",
            "        SetHandler application/x-httpd-php",
            "    </FilesMatch>",
            "</IfFile>",
            "<IfFile \"${CP_PHP_FWD}/php7apache2_4.dll\">",
            "    LoadFile \"${CP_PHP_FWD}/php7ts.dll\"",
            "    LoadModule php7_module \"${CP_PHP_FWD}/php7apache2_4.dll\"",
            "    PHPIniDir \"${CP_ROOT_FWD}/data/php\"",
            "    <FilesMatch \"\\.php$\">",
            "        SetHandler application/x-httpd-php",
            "    </FilesMatch>",
            "</IfFile>",
            "<IfFile \"${CP_PHP_FWD}/php5apache2_4.dll\">",
            "    LoadFile \"${CP_PHP_FWD}/php5ts.dll\"",
            "    LoadModule php5_module \"${CP_PHP_FWD}/php5apache2_4.dll\"",
            "    PHPIniDir \"${CP_ROOT_FWD}/data/php\"",
            "    <FilesMatch \"\\.php$\">",
            "        SetHandler application/x-httpd-php",
            "    </FilesMatch>",
            "</IfFile>",
            "",
            "# Other configurations (virtual hosts, aliases...): .conf files in data\\apache\\conf.d",
            "IncludeOptional \"${CP_ROOT_FWD}/data/apache/conf.d/*.conf\"",
        ]
        .join("\r\n");
        write_atomic(&conf, format!("{text}\r\n").as_bytes())?;
    }
    if fs::read_dir(&htdocs)?.next().is_none() {
        let page = [
            "<!doctype html>",
            "<meta charset=\"utf-8\">",
            "<title>Atollo</title>",
            "<h1>Apache funziona</h1>",
            "<p>PHP <?= PHP_VERSION ?> &middot; <a href=\"?info\">phpinfo()</a></p>",
            "<p>I file del sito stanno nella cartella <code>data\\apache\\htdocs\\</code> del kit.</p>",
            "<?php if (isset($_GET['info'])) phpinfo(); ?>",
        ]
        .join("\r\n");
        write_atomic(&htdocs.join("index.php"), format!("{page}\r\n").as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/install/web.rs"]
mod tests;
