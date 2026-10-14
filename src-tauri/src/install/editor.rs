//! Editor and terminal: VSCodium web server, PowerShell 7, Windows Terminal
//! (new components, docs/app/CATALOG.md and DECISIONS.md D-23, D-26).

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use regex::Regex;

use super::{arch_of, Ctx, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::net::{first_sha256, github_digest};
use crate::verify::Expect;

fn pattern(re: &str) -> Result<Regex> {
    Regex::new(re).context("asset pattern")
}

/// PowerShell 7, the official zip (not the "fxdependent" build).
pub struct Pwsh;

impl Installer for Pwsh {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let rel = ctx.net.github_latest("PowerShell/PowerShell")?;
        let arch = arch_of(ctx, component);
        let asset = rel
            .asset(&pattern(&format!(
                r"^PowerShell-\d+\.\d+\.\d+-win-{arch}\.zip$"
            ))?)
            .ok_or_else(|| anyhow!("no PowerShell zip for {arch} in {}", rel.tag_name))?;
        Ok(Release {
            version: rel.tag_name.trim_start_matches('v').to_owned(),
            url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            expect: Expect::Sha256(github_digest(asset)?),
            kind: Kind::Zip,
            inner: Inner::Auto,
        })
    }
}

/// Windows Terminal, the unpackaged zip in portable mode.
pub struct Terminal;

impl Installer for Terminal {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let rel = ctx.net.github_latest("microsoft/terminal")?;
        let arch = arch_of(ctx, component);
        let asset = rel
            .asset(&pattern(&format!(
                r"^Microsoft\.WindowsTerminal_[\d.]+_{arch}\.zip$"
            ))?)
            .ok_or_else(|| anyhow!("no Windows Terminal zip for {arch} in {}", rel.tag_name))?;
        Ok(Release {
            version: rel.tag_name.trim_start_matches('v').to_owned(),
            url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            expect: Expect::Sha256(github_digest(asset)?),
            kind: Kind::Zip,
            inner: Inner::Auto,
        })
    }

    /// `.portable` next to the exe: settings stay in the folder, not in
    /// `%LOCALAPPDATA%` (Windows Terminal portable mode).
    fn finish(&self, _ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        fs::write(dest.join(".portable"), b"").context("terminal .portable")
    }
}

/// VSCodium web server ("reh-web"); x64 only, also on ARM64.
pub struct Server;

impl Installer for Server {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let rel = ctx.net.github_latest("VSCodium/vscodium")?;
        let name = format!("vscodium-reh-web-win32-x64-{}.tar.gz", rel.tag_name);
        let asset = rel
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow!("no {name} in VSCodium {}", rel.tag_name))?;
        // The checksum published by VSCodium; GitHub's digest must agree.
        let sums = rel
            .assets
            .iter()
            .find(|a| a.name == format!("{name}.sha256"))
            .ok_or_else(|| anyhow!("VSCodium publishes no {name}.sha256"))?;
        let published = first_sha256(&ctx.net.text(&sums.browser_download_url)?)
            .ok_or_else(|| anyhow!("{name}.sha256 holds no SHA-256"))?;
        if let Ok(digest) = github_digest(asset) {
            if digest != published {
                return Err(anyhow!("{name}: VSCodium and GitHub checksums differ"));
            }
        }
        Ok(Release {
            version: rel.tag_name.clone(),
            url: asset.browser_download_url.clone(),
            file_name: name,
            expect: Expect::Sha256(published),
            kind: Kind::TarGz,
            inner: Inner::Auto,
        })
    }
}
