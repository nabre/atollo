//! Caddy, the kit's web server on every target (Q-3, D-111, WP-3.1): the
//! official build from GitHub `caddyserver/caddy`, one binary per archive
//! (`caddy.exe` or `caddy` at the archive root, next to its licence).
//!
//! Every target has a native build, Windows ARM64 included. Each archive is
//! checked against GitHub's SHA-256 `digest` (releases from 2.10.1 on), the
//! older ones against the SHA-512 in the release's `caddy_<v>_checksums.txt`.
//! A release with neither is not offered.

use anyhow::{anyhow, Result};

use super::{pattern, target_of, Ctx, Installer, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::{github_digest, GhAsset, GhRelease};
use crate::setup::catalog::Component;

pub struct Caddy;

const REPO: &str = "caddyserver/caddy";

/// Caddy's name of the system of `target` in its asset names.
pub(crate) fn caddy_os(target: Target) -> &'static str {
    match target.os {
        Os::Win32 => "windows",
        Os::Darwin => "mac",
        Os::Linux => "linux",
    }
}

/// The pattern of the archive for `target`, any version, and its kind:
/// `caddy_<v>_windows_amd64.zip`, `caddy_<v>_mac_arm64.tar.gz`... Only the
/// exact system and architecture: never a build of another one.
pub(crate) fn caddy_asset_for(target: Target) -> (String, Kind) {
    let (os, arch) = (caddy_os(target), target.go_arch());
    match target.os {
        Os::Win32 => (
            format!(r"^caddy_\d+\.\d+\.\d+_{os}_{arch}\.zip$"),
            Kind::Zip,
        ),
        Os::Darwin | Os::Linux => (
            format!(r"^caddy_\d+\.\d+\.\d+_{os}_{arch}\.tar\.gz$"),
            Kind::TarGz,
        ),
    }
}

/// The version of a release tag: `v2.11.7` → `2.11.7`.
pub(crate) fn caddy_version(tag: &str) -> &str {
    tag.trim_start_matches('v')
}

/// The checksum file of a release: `caddy_2.11.7_checksums.txt`.
pub(crate) fn caddy_checksums_name(version: &str) -> String {
    format!("caddy_{version}_checksums.txt")
}

/// The hash of `file_name` in Caddy's checksum file (`<hex>  <name>` lines,
/// SHA-512 since Caddy 2; a SHA-256 line is read too). The name must match
/// exactly: `caddy_<v>_linux_amd64.tar.gz` is not its `.deb` or `.sig`.
pub fn caddy_hash_in(text: &str, file_name: &str) -> Option<Expect> {
    text.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        if name != file_name || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match hash.len() {
            128 => Some(Expect::Sha512(hash.to_ascii_lowercase())),
            64 => Some(Expect::Sha256(hash.to_ascii_lowercase())),
            _ => None,
        }
    })
}

/// The checksum file published with `release`, if any.
fn checksums_of(release: &GhRelease) -> Option<&GhAsset> {
    let name = caddy_checksums_name(caddy_version(&release.tag_name));
    release.assets.iter().find(|a| a.name == name)
}

/// `asset` of `release` can be checked: GitHub's digest or the checksum file.
pub(crate) fn caddy_verifiable(release: &GhRelease, asset: &GhAsset) -> bool {
    github_digest(asset).is_ok() || checksums_of(release).is_some()
}

/// The stable releases of Caddy with a verifiable archive for `target`,
/// newest first (its last 100 releases: betas and RCs left out).
pub(crate) fn caddy_stable(all: Vec<GhRelease>, target: Target) -> Result<Vec<GhRelease>> {
    let re = pattern(&caddy_asset_for(target).0)?;
    let mut out: Vec<GhRelease> = all
        .into_iter()
        .filter(|r| !r.prerelease && super::lang::is_release(caddy_version(&r.tag_name)))
        .filter(|r| r.asset(&re).is_some_and(|a| caddy_verifiable(r, a)))
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(super::lang::version_key(caddy_version(&r.tag_name))));
    Ok(out)
}

fn caddy_releases(ctx: &Ctx, target: Target) -> Result<Vec<GhRelease>> {
    let all: Vec<GhRelease> = ctx.net.json(&format!(
        "https://api.github.com/repos/{REPO}/releases?per_page=100"
    ))?;
    caddy_stable(all, target)
}

/// What the archive must hash to: GitHub's SHA-256, else the release's
/// checksum file (downloaded). No hash, no installation.
fn caddy_expect(ctx: &Ctx, release: &GhRelease, asset: &GhAsset) -> Result<Expect> {
    if let Ok(hash) = github_digest(asset) {
        return Ok(Expect::Sha256(hash));
    }
    let file = checksums_of(release)
        .ok_or_else(|| anyhow!("Caddy {}: no checksum for {}", release.tag_name, asset.name))?;
    let text = ctx.net.text(&file.browser_download_url)?;
    caddy_hash_in(&text, &asset.name)
        .ok_or_else(|| anyhow!("{}: no hash for {}", file.name, asset.name))
}

fn caddy_release_from(ctx: &Ctx, target: Target, release: &GhRelease) -> Result<Release> {
    let (pat, kind) = caddy_asset_for(target);
    let asset = release
        .asset(&pattern(&pat)?)
        .ok_or_else(|| anyhow!("Caddy {}: no build for {target}", release.tag_name))?;
    Ok(Release::single(
        caddy_version(&release.tag_name).to_owned(),
        asset.browser_download_url.clone(),
        asset.name.clone(),
        caddy_expect(ctx, release, asset)?,
        kind,
        Inner::Root,
    ))
}

/// Versioned like the other servers (D-80): a pool or a site can keep its
/// line while a newer one is installed next to it.
impl Installer for Caddy {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if let Ok(rel) = ctx.net.github_latest(REPO) {
            if let Ok(r) = caddy_release_from(ctx, target, &rel) {
                return Ok(r);
            }
        }
        let releases = caddy_releases(ctx, target)?;
        let rel = releases
            .first()
            .ok_or_else(|| anyhow!("no Caddy release found for {target}"))?;
        caddy_release_from(ctx, target, rel)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        Ok(super::lang::release_lines(&caddy_releases(ctx, target)?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        let releases = caddy_releases(ctx, target)?;
        let rel = releases
            .iter()
            .find(|r| crate::versions::line_of(&r.tag_name).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("Caddy {line} not available for {target}"))?;
        caddy_release_from(ctx, target, rel)
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/caddy.rs"]
mod tests;
