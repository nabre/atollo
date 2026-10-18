//! Claude Code, Codex, Antigravity (port of nabre/atollo
//! `_prepare-core.ps1:10-104`).

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{target_of, Ctx, Dest, Installer, Part, Release, Sign};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

fn str_at<'a>(v: &'a Value, path: &[&str]) -> Result<&'a str> {
    let mut cur = v;
    for key in path {
        cur = cur
            .get(key)
            .ok_or_else(|| anyhow!("missing {}", path.join(".")))?;
    }
    cur.as_str()
        .ok_or_else(|| anyhow!("{} is not text", path.join(".")))
}

/// npm package metadata: version, tarball, integrity.
fn npm(ctx: &Ctx, url: &str) -> Result<(String, String, String)> {
    let meta: Value = ctx.net.json(url)?;
    Ok((
        str_at(&meta, &["version"])?.to_owned(),
        str_at(&meta, &["dist", "tarball"])?.to_owned(),
        str_at(&meta, &["dist", "integrity"])?.to_owned(),
    ))
}

/// Claude Code: the native package `@anthropic-ai/claude-code-<os>-<arch>`
/// (channel `latest`), verified with npm integrity and (on Windows) Anthropic's
/// signature; `claude(.exe)` replaced as a file, so a running Claude can be updated.
pub struct Claude;

impl Installer for Claude {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let slug = target.as_str();
        let (version, tarball, integrity) = npm(
            ctx,
            &format!("https://registry.npmjs.org/@anthropic-ai/claude-code-{slug}/latest"),
        )?;
        let bin = if target.is_windows() {
            "claude.exe"
        } else {
            "claude"
        };
        Ok(Release {
            parts: vec![Part {
                url: tarball,
                file_name: format!("claude-{version}.tgz"),
                expect: Expect::NpmIntegrity(integrity),
                kind: Kind::TarGz,
                inner: Inner::Named("package".into()),
                dest: Dest::File {
                    folder: "claude",
                    name: bin,
                    from: bin.into(),
                },
                sign: target.is_windows().then(|| Sign {
                    file: "claude.exe".into(),
                    publisher: "Anthropic",
                    required: false,
                }),
            }],
            version,
        })
    }
}

/// Codex: the native build of `@openai/codex` (`<version>-<os>-<arch>`),
/// whose `package/vendor/<target>` folder becomes `codex/`.
pub struct Codex;

impl Installer for Codex {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let meta: Value = ctx
            .net
            .json("https://registry.npmjs.org/@openai/codex/latest")?;
        let version = str_at(&meta, &["version"])?.to_owned();
        let slug = target_of(ctx, component).as_str();
        let (_, tarball, integrity) = npm(
            ctx,
            &format!("https://registry.npmjs.org/@openai/codex/{version}-{slug}"),
        )?;
        Ok(Release::single(
            version.clone(),
            tarball,
            format!("codex-{version}.tgz"),
            Expect::NpmIntegrity(integrity),
            Kind::TarGz,
            Inner::SingleUnder("package/vendor".into()),
        ))
    }
}

/// Antigravity CLI: one binary from Google's update manifest, SHA-512.
/// `agy install` is not run: the kit's environment is enough.
pub struct Antigravity;

impl Installer for Antigravity {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let platform = format!("{}_{}", target.go_os(), target.go_arch());
        let meta: Value = ctx.net.json(&format!(
            "https://antigravity-cli-auto-updater-974169037036.us-central1.run.app/manifests/{platform}.json"
        ))?;
        let version = str_at(&meta, &["version"])?.to_owned();
        let url = str_at(&meta, &["url"])
            .context("Antigravity manifest")?
            .to_owned();
        let sha512 = str_at(&meta, &["sha512"])
            .context("Antigravity manifest")?
            .to_owned();
        if sha512.len() != 128 || !sha512.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("Antigravity manifest without a valid SHA-512");
        }
        let bin = if target.is_windows() {
            "agy.exe"
        } else {
            "agy"
        };
        let (kind, file_name, from) = antigravity_download(&url, &version, target);
        Ok(Release {
            parts: vec![Part {
                url,
                file_name,
                expect: Expect::Sha512(sha512),
                kind,
                inner: Inner::Root,
                dest: Dest::File {
                    folder: "antigravity",
                    name: bin,
                    from,
                },
                sign: None,
            }],
            version,
        })
    }
}

/// How the manifest's download is unpacked: a `.tar.gz` (macOS and Linux
/// since 1.4: one binary named `antigravity` inside) or the program itself
/// (Windows' `.exe`). The kind, the downloaded file's name and the file
/// that becomes `agy`.
pub(crate) fn antigravity_download(
    url: &str,
    version: &str,
    target: crate::kit::Target,
) -> (Kind, String, String) {
    let name = url
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        (
            Kind::TarGz,
            format!("agy-{version}.tar.gz"),
            "antigravity".to_owned(),
        )
    } else {
        let file_name = format!("agy-{version}{}", target.exe_suffix());
        (Kind::File, file_name.clone(), file_name)
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/agents.rs"]
mod tests;
