//! Claude Code, Codex, Antigravity (port of nabre/atollo
//! `_prepare-core.ps1:10-104`).

use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;

use super::{arch_of, Ctx, Dest, Installer, Part, Release, Sign};
use crate::archive::{Inner, Kind};
use crate::setup::catalog::Component;
use crate::verify::Expect;

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

/// Claude Code: the native package `@anthropic-ai/claude-code-win32-<arch>`
/// (channel `latest`), verified with npm integrity and Anthropic's signature;
/// `claude.exe` replaced as a file, so a running Claude can be updated.
pub struct Claude;

impl Installer for Claude {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        let (version, tarball, integrity) = npm(
            ctx,
            &format!("https://registry.npmjs.org/@anthropic-ai/claude-code-win32-{arch}/latest"),
        )?;
        Ok(Release {
            parts: vec![Part {
                url: tarball,
                file_name: format!("claude-{version}.tgz"),
                expect: Expect::NpmIntegrity(integrity),
                kind: Kind::TarGz,
                inner: Inner::Named("package".into()),
                dest: Dest::File {
                    folder: "claude",
                    name: "claude.exe",
                    from: "claude.exe".into(),
                },
                sign: Some(Sign {
                    file: "claude.exe".into(),
                    publisher: "Anthropic",
                    required: false,
                }),
            }],
            version,
        })
    }
}

/// Codex: the Windows build of `@openai/codex` (`<version>-win32-<arch>`),
/// whose `package/vendor/<target>` folder becomes `codex\`.
pub struct Codex;

impl Installer for Codex {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let meta: Value = ctx
            .net
            .json("https://registry.npmjs.org/@openai/codex/latest")?;
        let version = str_at(&meta, &["version"])?.to_owned();
        let arch = arch_of(ctx, component);
        let (_, tarball, integrity) = npm(
            ctx,
            &format!("https://registry.npmjs.org/@openai/codex/{version}-win32-{arch}"),
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

/// Antigravity CLI: one exe from Google's update manifest, SHA-512.
/// `agy install` is not run: the kit's environment is enough.
pub struct Antigravity;

impl Installer for Antigravity {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let platform = if arch_of(ctx, component) == "arm64" {
            "windows_arm64"
        } else {
            "windows_amd64"
        };
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
        let file_name = format!("agy-{version}.exe");
        Ok(Release {
            parts: vec![Part {
                url,
                file_name: file_name.clone(),
                expect: Expect::Sha512(sha512),
                kind: Kind::File,
                inner: Inner::Root,
                dest: Dest::File {
                    folder: "antigravity",
                    name: "agy.exe",
                    from: file_name,
                },
                sign: None,
            }],
            version,
        })
    }
}
