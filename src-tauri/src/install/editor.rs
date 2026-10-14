//! Editor and shell: VSCodium web server, PowerShell 7 (new components,
//! docs/app/CATALOG.md and DECISIONS.md D-23, D-26).

use anyhow::{anyhow, Result};

use super::{arch_of, github_single, Ctx, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::net::{first_sha256, github_digest};
use crate::verify::Expect;

/// PowerShell 7, the official zip (not the "fxdependent" build).
pub struct Pwsh;

impl Installer for Pwsh {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        github_single(
            ctx,
            "PowerShell/PowerShell",
            &format!(r"^PowerShell-\d+\.\d+\.\d+-win-{arch}\.zip$"),
            Kind::Zip,
            Inner::Auto,
        )
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
        Ok(Release::single(
            rel.tag_name.clone(),
            asset.browser_download_url.clone(),
            name,
            Expect::Sha256(published),
            Kind::TarGz,
            Inner::Auto,
        ))
    }
}
