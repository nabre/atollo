//! Editor, shell and terminal: VSCodium web server, PowerShell 7, Windows
//! Terminal (docs/app/CATALOG.md and DECISIONS.md D-23, D-26, D-68).

use std::path::Path;

use anyhow::{bail, Context, Result};

use super::{arch_of, github_single, Ctx, Installer, Release};
use crate::archive::{Inner, Kind};
use crate::setup::catalog::Component;
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

/// Windows Terminal, the unpackaged zip in portable mode (D-68).
pub struct Terminal;

impl Installer for Terminal {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        github_single(
            ctx,
            "microsoft/terminal",
            &format!(r"^Microsoft\.WindowsTerminal_[\d.]+_{arch}\.zip$"),
            Kind::Zip,
            Inner::Auto,
        )
    }

    /// `.portable` next to the exe: settings stay in the folder, not in
    /// `%LOCALAPPDATA%` (Windows Terminal portable mode).
    fn finish(&self, _ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        std::fs::write(dest.join(".portable"), b"").context("terminal .portable")
    }
}

/// The editor (VSCodium web server, "reh-web"), part of the app; x64 only,
/// also on ARM64. Always the version and SHA-256 fixed in the catalog for
/// this release of Dev-Kit (DECISIONS.md D-58): no lookup of the latest one.
pub struct Server;

impl Installer for Server {
    fn latest(&self, _ctx: &Ctx, component: &Component) -> Result<Release> {
        pinned(component)
    }

    /// The editor of another release may still run (this app's, or one of a
    /// previous start not yet ended): it is part of the app, so it is
    /// stopped, or its folder could not be replaced.
    fn before_swap(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        _tree: &Path,
        old: &Path,
    ) -> Result<()> {
        let running = crate::processes::running_from(&crate::processes::all(), old);
        if !running.is_empty() {
            crate::log::info(&format!("editor update: stopping {running:?}"));
            crate::sessions::stop(&running);
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        Ok(())
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
    use crate::setup::catalog::Catalog;

    #[test]
    fn processes_of_the_old_editor() {
        let proc = |pid, path: &str| crate::processes::Proc {
            pid,
            parent: 0,
            name: String::new(),
            path: path.into(),
        };
        let all = [
            proc(1, r"E:\Kit\data\apps\win32-x64\vscodium-server\node.exe"),
            proc(2, r"E:\Kit\data\apps\win32-x64\VSCODIUM-SERVER\bin\x.exe"),
            proc(
                3,
                r"E:\Kit\data\apps\win32-x64\vscodium-server-old\node.exe",
            ),
            proc(4, r"E:\Kit\data\apps\win32-x64\node\node.exe"),
        ];
        let old = Path::new(r"E:\Kit\data\apps\win32-x64\vscodium-server");
        assert_eq!(crate::processes::running_from(&all, old), [1, 2]);
    }

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
