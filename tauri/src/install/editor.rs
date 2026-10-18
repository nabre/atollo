//! Editor and shell: VSCodium web server, PowerShell 7 (docs/app/CATALOG.md
//! and DECISIONS.md D-23, D-26).

use std::path::Path;

use anyhow::{bail, Result};

use super::{github_single, target_of, Ctx, Installer, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// PowerShell 7, the official archive (not the "fxdependent" build).
pub struct Pwsh;

impl Installer for Pwsh {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let arch = target.arch.as_str();
        let (re, kind) = match target.os {
            Os::Win32 => (
                format!(r"^PowerShell-\d+\.\d+\.\d+-win-{arch}\.zip$"),
                Kind::Zip,
            ),
            Os::Darwin => (
                format!(r"^powershell-\d+\.\d+\.\d+-osx-{arch}\.tar\.gz$"),
                Kind::TarGz,
            ),
            Os::Linux => (
                format!(r"^powershell-\d+\.\d+\.\d+-linux-{arch}\.tar\.gz$"),
                Kind::TarGz,
            ),
        };
        github_single(ctx, "PowerShell/PowerShell", &re, kind, Inner::Auto)
    }
}

/// The editor (VSCodium web server, "reh-web"), part of the app. Always the
/// version fixed in the catalog for this release of Dev-Kit (DECISIONS.md D-58).
pub struct Server;

impl Installer for Server {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if let Some(sha) = component.sha256_for(target) {
            return pinned_with_sha(target, component, sha);
        }
        let Some(version) = &component.version else {
            bail!("{}: no version fixed in the catalog", component.id);
        };
        let name = format!("vscodium-reh-web-{}-{version}.tar.gz", target.as_str());
        let url =
            format!("https://github.com/VSCodium/vscodium/releases/download/{version}/{name}");
        let sha_text = ctx.net.text(&format!("{url}.sha256"))?;
        let sha = first_sha256(&sha_text)
            .ok_or_else(|| anyhow::anyhow!("vscodium-server: no SHA-256 in {name}.sha256"))?;
        pinned_with_sha(target, component, &sha)
    }

    /// The editor of another release may still run (this app's, or one of a
    /// previous start not yet ended): it is part of the app, so it is
    /// stopped, or its folder could not be replaced.
    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        _tree: &Path,
        old: &Path,
    ) -> Result<()> {
        if ctx.kit.os() != Os::current() {
            return Ok(());
        }
        let running = crate::os::processes::running_from(&crate::os::processes::all(), old);
        if !running.is_empty() {
            crate::log::info(&format!("editor update: stopping {running:?}"));
            crate::os::sessions::stop(&running);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Ok(())
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn pinned(component: &Component) -> Result<Release> {
    let Some(sha) = component.sha256_for(Target::WIN32_X64) else {
        bail!("{}: no version fixed in the catalog", component.id);
    };
    pinned_with_sha(Target::WIN32_X64, component, sha)
}

pub(crate) fn pinned_with_sha(target: Target, component: &Component, sha: &str) -> Result<Release> {
    let Some(version) = &component.version else {
        bail!("{}: no version fixed in the catalog", component.id);
    };
    let name = format!("vscodium-reh-web-{}-{version}.tar.gz", target.as_str());
    Ok(Release::single(
        version.clone(),
        format!("https://github.com/VSCodium/vscodium/releases/download/{version}/{name}"),
        name,
        Expect::Sha256(sha.to_owned()),
        Kind::TarGz,
        Inner::Auto,
    ))
}

#[cfg(test)]
#[path = "../../../test/rust/install/editor.rs"]
mod tests;
