//! llama.cpp (D-151): the server of the local models (`llm/`). The build
//! `b<n>` fixed in the catalog (`version`), from ggml-org's releases, with
//! the SHA-256 of each target's archive fixed there too (`sha256ByTarget`,
//! taken from GitHub's digest when it was fixed): the releases are rolling,
//! several a day, so the kit moves to a new one with a new catalog.
//!
//! One build per target: the Vulkan one on Windows and Linux, which holds
//! the CPU backends as well and loads the Vulkan one only where a driver is
//! there (no GPU, or no `libvulkan1`: the CPU), the Metal one on macOS.

use anyhow::{anyhow, bail, Result};

use super::{target_of, Ctx, Installer, Release};
use crate::kit::{Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::Component;

pub struct Llama;

/// The archive of build `version` for `target`: its name, kind and where
/// the programs are inside.
pub(crate) fn asset_for(version: &str, target: Target) -> (String, Kind, Inner) {
    let arch = target.arch.as_str();
    match target.os {
        Os::Win32 => (
            format!("llama-{version}-bin-win-vulkan-{arch}.zip"),
            Kind::Zip,
            Inner::Root,
        ),
        Os::Darwin => (
            format!("llama-{version}-bin-macos-{arch}.tar.gz"),
            Kind::TarGz,
            Inner::Single,
        ),
        Os::Linux => (
            format!("llama-{version}-bin-ubuntu-vulkan-{arch}.tar.gz"),
            Kind::TarGz,
            Inner::Single,
        ),
    }
}

/// A build of the catalog: `b` and digits only.
pub(crate) fn valid_version(version: &str) -> bool {
    version
        .strip_prefix('b')
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

impl Installer for Llama {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let version = component
            .version
            .as_deref()
            .ok_or_else(|| anyhow!("llama.cpp: no build fixed in the catalog"))?;
        if !valid_version(version) {
            bail!("llama.cpp: {version:?} is not a build of llama.cpp");
        }
        let sha = component
            .sha256_for(target)
            .ok_or_else(|| anyhow!("llama.cpp {version}: no SHA-256 for {target}"))?;
        let (name, kind, inner) = asset_for(version, target);
        Ok(Release::single(
            version.to_owned(),
            format!("https://github.com/ggml-org/llama.cpp/releases/download/{version}/{name}"),
            name,
            Expect::Sha256(sha.to_owned()),
            kind,
            inner,
        ))
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/llama.rs"]
mod tests;
