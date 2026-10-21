//! Apache httpd: Apache Lounge's build on Windows, the kit's own pinned
//! build on macOS and Linux (D-116), its `httpd.conf` and the kit's site in
//! `data\apache`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::vcruntime::copy_vc_runtime;
use crate::fsx::{join_rel, write_atomic};
use crate::install::db::{mysql_lines_for, MysqlPin};
use crate::install::lang::version_key;
use crate::install::{pattern, target_of, Ctx, Installer, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Apache httpd: Apache Lounge on Windows (the maintained Windows build),
/// the kit's own build on macOS and Linux (D-116). Port, site and PHP live
/// in `data\apache\kit.conf`, which updates never touch.
///
/// Apache Lounge keeps only the latest build of 2.4, its only line: a new
/// build replaces the folder `apache\2.4`. The kit's builds are pinned per
/// line in the catalog, like MySQL's.
pub struct Apache;

/// The kit's httpd builds (`scripts/build-httpd`, D-116): GitHub releases
/// `httpd-v<version>` of this repository, one tarball per macOS and Linux
/// target with the tree at its root. Each line's version (`2.4.69-1`: httpd
/// and the kit's build number) and the SHA-256 of each tarball are fixed in
/// `sha256ByLine` of `apache`: only those are offered, and a tarball with
/// another hash is refused.
pub(super) const HTTPD_RELEASES: &str = "https://github.com/nabre/atollo/releases/download";

/// The `sha256ByLine` pins of component `id` in a catalog file.
pub fn line_pins_in(catalog: &Value, id: &str) -> BTreeMap<String, MysqlPin> {
    catalog["components"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|c| c["id"] == id)
        .and_then(|c| serde_json::from_value(c["sha256ByLine"].clone()).ok())
        .unwrap_or_default()
}

/// Apache's pins in the catalog embedded in the app.
fn httpd_pins() -> &'static BTreeMap<String, MysqlPin> {
    static PINS: std::sync::OnceLock<BTreeMap<String, MysqlPin>> = std::sync::OnceLock::new();
    PINS.get_or_init(|| {
        serde_json::from_str::<Value>(include_str!("../../../catalog/components.json"))
            .map(|v| line_pins_in(&v, "apache"))
            .unwrap_or_default()
    })
}

/// The tarball of the kit's httpd `version` for a macOS or Linux `target`.
pub fn httpd_file(target: Target, version: &str) -> String {
    format!("httpd-{version}-{target}.tar.gz")
}

/// The address of `file` in the release of `version`.
pub fn httpd_url(version: &str, file: &str) -> String {
    format!("{HTTPD_RELEASES}/httpd-v{version}/{file}")
}

/// The lines of the kit's httpd with a pinned tarball for `target`, newest
/// first.
pub fn httpd_lines_for(pins: &BTreeMap<String, MysqlPin>, target: Target) -> Vec<String> {
    mysql_lines_for(pins, target)
}

/// The release of `line` for `target`, with its pinned SHA-256; an error for
/// a line or target without one (never a download that cannot be verified).
pub fn httpd_release_for(
    pins: &BTreeMap<String, MysqlPin>,
    target: Target,
    line: &str,
) -> Result<Release> {
    if target.os == Os::Win32 {
        bail!("the kit's httpd build is for macOS and Linux; Windows uses Apache Lounge");
    }
    if !httpd_lines_for(pins, target).iter().any(|l| l == line) {
        bail!(
            "Apache {line}: no build with a fixed SHA-256 for {target} in this version of the app"
        );
    }
    let pin = &pins[line];
    let sha = &pin.sha256_by_target[target.as_str()];
    if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Apache {line}: the SHA-256 fixed for {target} is not one");
    }
    let file = httpd_file(target, &pin.version);
    Ok(Release::single(
        pin.version.clone(),
        httpd_url(&pin.version, &file),
        file,
        Expect::Sha256(sha.to_ascii_lowercase()),
        Kind::TarGz,
        Inner::Root,
    ))
}

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

    /// The lines on Apache Lounge, newest first (only 2.4); off Windows the
    /// pinned lines of the kit's build.
    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        if target.os != Os::Win32 {
            return Ok(httpd_lines_for(httpd_pins(), target));
        }
        let mut out: Vec<String> = Vec::new();
        for line in apache_builds(ctx)?
            .iter()
            .filter_map(|b| crate::versions::line_of(&b.version))
        {
            if !out.contains(&line) {
                out.push(line);
            }
        }
        out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        if target.os != Os::Win32 {
            return httpd_release_for(httpd_pins(), target, line);
        }
        let build = apache_builds(ctx)?
            .into_iter()
            .find(|b| crate::versions::line_of(&b.version).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("Apache {line} not available"))?;
        apache_release_of(ctx, &build)
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if target.os != Os::Win32 {
            let pins = httpd_pins();
            let line = httpd_lines_for(pins, target)
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("Apache: no build with a fixed SHA-256 for {target}"))?;
            return httpd_release_for(pins, target, &line);
        }
        let build = apache_builds(ctx)?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("Apache Win64 package not found on apachelounge.com"))?;
        apache_release_of(ctx, &build)
    }

    /// The same on every system: the kit's build ships a `httpd.conf` in
    /// Apache Lounge's shape (`Define SRVROOT`, D-116). The VC++ runtime is
    /// Windows only (`copy_vc_runtime` does nothing elsewhere).
    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        copy_vc_runtime(ctx, &tree.join("bin"))?;
        fs::create_dir_all(tree.join("logs"))?;
        let conf = join_rel(tree, "conf/httpd.conf");
        if conf.is_file() {
            let text = fs::read_to_string(&conf).context("httpd.conf")?;
            fs::write(&conf, httpd_conf(&text)?)?;
        }
        Ok(())
    }

    fn finish(&self, ctx: &Ctx, component: &Component, _dest: &Path) -> Result<()> {
        apache_site(
            ctx.kit.data().join("apache").as_path(),
            target_of(ctx, component).os,
        )
    }
}

/// httpd.conf of the kit: root from the kit variables, no port 80 (it is
/// in kit.conf), mod_rewrite on, kit.conf included. Lines end as in the
/// file: CRLF in Apache Lounge's, LF in the kit's macOS and Linux build.
pub fn httpd_conf(text: &str) -> Result<String> {
    let crlf = text.contains("\r\n");
    let nl = if crlf { "\r\n" } else { "\n" };
    let listen = if crlf {
        "#Listen 80   (port in data\\apache\\kit.conf)$1"
    } else {
        "#Listen 80   (port in data/apache/kit.conf)$1"
    };
    let mut t = pattern(r"(?m)^Define SRVROOT [^\r\n]*")?
        .replace(text, r#"Define SRVROOT "$${CP_APACHE_FWD}""#)
        .into_owned();
    t = pattern(r"(?m)^Listen 80[ \t]*(\r?)$")?
        .replace(&t, listen)
        .into_owned();
    t = pattern(r"(?m)^#(LoadModule rewrite_module [^\r\n]*)")?
        .replace(&t, "$1")
        .into_owned();
    if !t.contains("${CP_APACHE_FWD}") {
        bail!("httpd.conf in an unexpected format (no Define SRVROOT)");
    }
    Ok(format!(
        "{}{nl}{nl}# --- Atollo ---{nl}Include \"${{CP_ROOT_FWD}}/data/apache/kit.conf\"{nl}",
        t.trim_end()
    ))
}

/// `data\apache\kit.conf`, `conf.d\`, `htdocs\` with a test page; each
/// created once (they belong to the user). On Windows PHP is the Apache
/// module; on macOS and Linux PHP runs in php-fpm, which only the kit's
/// pools and project sites start (D-116): this site, started by hand with
/// the stock `httpd.conf`, refuses PHP files (403) and gets an HTML page.
pub fn apache_site(dir: &Path, os: Os) -> Result<()> {
    let windows = os == Os::Win32;
    let nl = if windows { "\r\n" } else { "\n" };
    let sep = if windows { "\\" } else { "/" };
    fs::create_dir_all(dir.join("conf.d"))?;
    let htdocs = dir.join("htdocs");
    fs::create_dir_all(&htdocs)?;
    let conf = dir.join("kit.conf");
    if !conf.exists() {
        let mut lines: Vec<String> = [
            "# Atollo Apache configuration (yours: updates never touch it).",
            "# Paths use variables set by Atollo, so they work on any drive letter.",
            "",
            "# Port: this PC only (127.0.0.1), so the firewall never asks.",
            "Define PORTA 8080",
            "Listen 127.0.0.1:${PORTA}",
            "ServerName localhost:${PORTA}",
            "",
            &format!("# Site folder: data{sep}apache{sep}htdocs{sep} in the kit"),
            "DocumentRoot \"${CP_ROOT_FWD}/data/apache/htdocs\"",
            "<Directory \"${CP_ROOT_FWD}/data/apache/htdocs\">",
            "    Options Indexes FollowSymLinks",
            "    AllowOverride All",
            "    Require all granted",
            "</Directory>",
            "DirectoryIndex index.php index.html",
            "",
            "# Hidden files and folders (.env, .git) are refused, .well-known is served",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        lines.extend(crate::projects::apache::hidden_deny(""));
        if windows {
            lines.extend(
                [
                    "",
                    "# The kit's PHP (Apache module), php.ini in data\\php\\<line>; without it a .php is refused (403).",
                    "# .inc and .php3 ... .php8 are never served.",
                ]
                .map(str::to_owned),
            );
            lines.extend(crate::projects::apache::php_module_blocks("CP_PHP_FWD"));
        } else {
            lines.extend(
                [
                    "",
                    "# PHP runs in php-fpm, started by the kit's web server pools and project sites:",
                    "# here a .php is refused (403). .inc and .php3 ... .php8 are never served.",
                ]
                .map(str::to_owned),
            );
            lines.extend(crate::projects::apache::php_off_blocks());
        }
        lines.extend(
            [
                "".to_owned(),
                format!("# Other configurations (virtual hosts, aliases...): .conf files in data{sep}apache{sep}conf.d"),
                "IncludeOptional \"${CP_ROOT_FWD}/data/apache/conf.d/*.conf\"".to_owned(),
            ],
        );
        let text = lines.join(nl);
        write_atomic(&conf, format!("{text}{nl}").as_bytes())?;
    }
    if fs::read_dir(&htdocs)?.next().is_none() {
        if windows {
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
        } else {
            let page = [
                "<!doctype html>",
                "<meta charset=\"utf-8\">",
                "<title>Atollo</title>",
                "<h1>Apache funziona</h1>",
                "<p>I file del sito stanno nella cartella <code>data/apache/htdocs/</code> del kit.</p>",
            ]
            .join("\n");
            write_atomic(&htdocs.join("index.html"), format!("{page}\n").as_bytes())?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/web/apache.rs"]
mod tests;
