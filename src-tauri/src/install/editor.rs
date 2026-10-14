//! Editor and shell: VSCodium web server, PowerShell 7 (new components,
//! docs/app/CATALOG.md and DECISIONS.md D-23, D-26).

use anyhow::{bail, Result};

use super::{arch_of, github_single, Ctx, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
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

/// The editor (VSCodium web server, "reh-web"), part of the app; x64 only,
/// also on ARM64. Always the version and SHA-256 fixed in the catalog for
/// this release of Atollo (DECISIONS.md D-58): no lookup of the latest one.
pub struct Server;

impl Installer for Server {
    fn latest(&self, _ctx: &Ctx, component: &Component) -> Result<Release> {
        pinned(component)
    }
}

fn pinned(component: &Component) -> Result<Release> {
    let (Some(version), Some(sha)) = (&component.version, &component.sha256) else {
        bail!("{}: no version fixed in the catalog", component.id);
    };
    let name = format!("vscodium-reh-web-win32-x64-{version}.tar.gz");
    Ok(Release::single(
        version.clone(),
        format!("https://github.com/VSCodium/vscodium/releases/download/{version}/{name}"),
        name,
        Expect::Sha256(sha.clone()),
        Kind::TarGz,
        Inner::Auto,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    #[test]
    fn the_editor_is_pinned() {
        let server = Catalog::get().find("vscodium-server").unwrap();
        let r = pinned(server).unwrap();
        let version = server.version.as_deref().unwrap();
        assert_eq!(r.version, version);
        assert_eq!(r.parts.len(), 1);
        assert_eq!(
            r.parts[0].url,
            format!("https://github.com/VSCodium/vscodium/releases/download/{version}/vscodium-reh-web-win32-x64-{version}.tar.gz")
        );
        let Expect::Sha256(sha) = &r.parts[0].expect else {
            panic!("sha256")
        };
        assert!(sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
