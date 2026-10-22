//! SQLite: the command-line tools of sqlite.org (SHA3-256 of the download
//! page), conda-forge's `sqlite` on Linux ARM64 (D-112).

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};

use crate::install::native::{conda_info, conda_versions, micromamba_create, micromamba_release};
use crate::install::{arch_of, pattern, target_of, Ctx, Installer, Progress, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::Component;

/// From the `PRODUCT,<version>,<path>,<size>,<sha3-256>` lines of
/// sqlite.org's download page: the command-line tools for Windows `arch`.
#[cfg(test)]
fn sqlite_tools(page: &str, arch: &str) -> Result<(String, String, String)> {
    sqlite_tools_for(page, "win", arch)
}

/// From the `PRODUCT,<version>,<path>,<size>,<sha3-256>` lines of
/// sqlite.org's download page: the command-line tools for `os_tag` (`"win"`,
/// `"osx"`, `"linux"`) and `arch` (`"x64"`, `"arm64"`).
pub(crate) fn sqlite_tools_for(
    page: &str,
    os_tag: &str,
    arch: &str,
) -> Result<(String, String, String)> {
    let re = pattern(&format!(
        r"(?m)^PRODUCT,([\d.]+),(\d{{4}}/sqlite-tools-{os_tag}-{arch}-\d+\.zip),\d+,([0-9a-f]{{64}})\s*$"
    ))?;
    let c = re
        .captures(page)
        .ok_or_else(|| anyhow!("SQLite: no tools for {os_tag}-{arch} on the download page"))?;
    Ok((c[1].to_owned(), c[2].to_owned(), c[3].to_owned()))
}

/// sqlite.org publishes no command-line tools for Linux ARM64: there SQLite
/// is conda-forge `sqlite` (D-112).
fn sqlite_from_conda(target: Target) -> bool {
    target == Target::LINUX_ARM64
}

/// The catalog's key of SQLite off Windows is `sqlite/sqlite3`, the file
/// of sqlite.org's zip; in a conda environment the program is
/// `bin/sqlite3`, which finds its libraries from there (`$ORIGIN/../lib`):
/// this launcher at the key starts it.
pub const SQLITE_CONDA_LAUNCHER: &str =
    "#!/bin/sh\n# Atollo: sqlite3 of the conda-forge environment next to this file\nexec \"$(dirname \"$0\")/bin/sqlite3\" \"$@\"\n";

pub struct Sqlite;

impl Installer for Sqlite {
    /// micromamba on Linux ARM64, the shared path elsewhere.
    fn custom(&self, target: Target) -> bool {
        sqlite_from_conda(target)
    }

    /// A conda environment cannot follow the kit (D-92).
    fn relocatable(&self, target: Target) -> bool {
        !sqlite_from_conda(target)
    }

    /// The environment in the folder of its line, and the launcher at the
    /// catalog's key.
    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
        let spec = format!("sqlite={}", release.version);
        micromamba_create(ctx, component, release, &dir, &[&spec], progress)?;
        self.after_conda(ctx, component, &dir, progress)
    }

    fn conda_prefix(&self, target: Target, dir: &Path) -> Option<PathBuf> {
        sqlite_from_conda(target).then(|| dir.to_path_buf())
    }

    /// The launcher at the catalog's key, after the environment is created
    /// from the network or from the kept packages (D-118).
    fn after_conda(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        dir: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        if !dir.join("bin").join("sqlite3").is_file() {
            bail!("bin/sqlite3 not found after the installation");
        }
        let launcher = dir.join("sqlite3");
        crate::fsx::write_atomic(&launcher, SQLITE_CONDA_LAUNCHER.as_bytes())?;
        crate::router::set_executable(&launcher)?;
        Ok(())
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if sqlite_from_conda(target) {
            let subdir = target.conda_subdir();
            let versions = conda_versions(&conda_info(ctx, "sqlite")?, subdir);
            let version = versions
                .first()
                .ok_or_else(|| anyhow!("no conda-forge sqlite for {subdir}"))?;
            return micromamba_release(ctx, version);
        }
        let os_tag = match target.os {
            Os::Win32 => "win",
            Os::Darwin => "osx",
            Os::Linux => "linux",
        };
        let arch = arch_of(ctx, component);
        let page = ctx.net.text("https://sqlite.org/download.html")?;
        // The build of the kit's architecture or an error: never the x64
        // one in its place (D-108), as the catalog says SQLite is native.
        let (version, path, sha3) = sqlite_tools_for(&page, os_tag, arch)?;
        let name = path.rsplit('/').next().unwrap_or("sqlite.zip").to_owned();
        Ok(Release::single(
            version,
            format!("https://sqlite.org/{path}"),
            name,
            Expect::Sha3_256(sha3),
            Kind::Zip,
            Inner::Auto,
        ))
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/sqlite.rs"]
mod tests;
