//! Docker Sandboxes (`sbx`, D-167): the microVM of each project, in the
//! kit like every other program. The official build from GitHub
//! `docker/sbx-releases` (latest stable, GitHub's SHA-256 `digest`):
//!
//! - Windows x64: the per-user `DockerSandboxes.msi`, extracted without
//!   installing it (`Kind::Msi`): `LocalApp\DockerSandboxes\` holds
//!   `bin\sbx.exe` and `libexec\`;
//! - Apple silicon: `Sbx.app` and `bin/sbx`, a link into it;
//! - Linux x64 and ARM64: the static tarball, laid out as its
//!   `install.sh` does (`bin/sbx`, `libexec/`, `libexec/lib/`).
//!
//! No other target: Docker builds none.
//! `sbx` keeps its sandboxes and settings in `%LOCALAPPDATA%` (Windows) or
//! the home: the kit's, so they travel with it.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::{target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::Component;

pub struct Sandboxes;

const REPO: &str = "docker/sbx-releases";

/// The asset of `target`, its kind and the part of it that is the
/// component's folder; `None` where Docker builds none.
pub(crate) fn asset_for(target: Target) -> Option<(&'static str, Kind, Inner)> {
    match (target.os, target.arch) {
        (Os::Win32, Arch::X64) => Some((
            "DockerSandboxes.msi",
            Kind::Msi,
            Inner::Named("LocalApp/DockerSandboxes".into()),
        )),
        (Os::Darwin, Arch::Arm64) => {
            Some(("DockerSandboxes-darwin.tar.gz", Kind::TarGz, Inner::Root))
        }
        (Os::Linux, Arch::X64) => Some((
            "DockerSandboxes-linux-amd64.tar.gz",
            Kind::TarGz,
            Inner::Single,
        )),
        (Os::Linux, Arch::Arm64) => Some((
            "DockerSandboxes-linux-arm64.tar.gz",
            Kind::TarGz,
            Inner::Single,
        )),
        _ => None,
    }
}

/// Where a file of the Linux tarball goes in the component's folder, as
/// its `install.sh` lays it out: `sbx` in `bin`, the library in
/// `libexec/lib`, the shims, `mkfs.*` and the microVM's kernel and root
/// file system in `libexec`. `None`: left out (the script itself, the
/// AppArmor profile, the notices stay where they are).
pub(crate) fn linux_place(name: &str) -> Option<&'static str> {
    match name {
        "sbx" => Some("bin"),
        "libsailor.so" => Some("libexec/lib"),
        n if n.starts_with("containerd-shim-")
            || n.starts_with("mkfs.")
            || n.starts_with("nerdbox-kernel-")
            || n.starts_with("nerdbox-rootfs-") =>
        {
            Some("libexec")
        }
        _ => None,
    }
}

impl Installer for Sandboxes {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let (name, kind, inner) =
            asset_for(target).ok_or_else(|| anyhow!("Docker Sandboxes: no build for {target}"))?;
        let rel = ctx.net.github_latest(REPO)?;
        let asset = rel
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow!("{REPO} {}: no {name}", rel.tag_name))?;
        Ok(Release::single(
            rel.tag_name.trim_start_matches('v').to_owned(),
            asset.browser_download_url.clone(),
            asset.name.clone(),
            Expect::Sha256(ctx.net.release_sha256(&rel, asset)?),
            kind,
            inner,
        ))
    }

    /// The Linux tarball into the layout of its `install.sh`, then the new
    /// tree checked whole ([`check_tree`]) before it replaces the old one.
    fn before_swap(
        &self,
        ctx: &Ctx,
        component: &Component,
        tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        let target = target_of(ctx, component);
        if target.is_linux() {
            lay_out_linux(tree)?;
        }
        check_tree(target, tree)
    }
}

/// A tree of Docker Sandboxes that can start a microVM: `sbx` and, on
/// Windows and Linux, the shim in `libexec` (on macOS it is inside
/// `Sbx.app`). A release laid out differently fails here, with the old
/// tree kept, instead of an `sbx` that is "installed" and never starts.
pub(crate) fn check_tree(target: Target, tree: &Path) -> Result<()> {
    let sbx = tree
        .join("bin")
        .join(format!("sbx{}", target.os.exe_suffix()));
    anyhow::ensure!(
        sbx.symlink_metadata().is_ok(),
        "Docker Sandboxes: no {} in the release",
        sbx.display()
    );
    if target.os == Os::Darwin {
        return Ok(());
    }
    let shim = fs::read_dir(tree.join("libexec"))
        .into_iter()
        .flatten()
        .flatten()
        .any(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("containerd-shim-")
        });
    anyhow::ensure!(shim, "Docker Sandboxes: no containerd-shim in libexec");
    Ok(())
}

/// Moves the files of the Linux tarball in `tree` to their places
/// ([`linux_place`]).
pub(crate) fn lay_out_linux(tree: &Path) -> Result<()> {
    for entry in fs::read_dir(tree)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(dir) = linux_place(&name) else {
            continue;
        };
        let to = crate::common::fsx::join_rel(tree, dir);
        fs::create_dir_all(&to)?;
        fs::rename(entry.path(), to.join(&name))
            .with_context(|| format!("Docker Sandboxes: {name} into {dir}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/install/sandboxes.rs"]
mod tests;
