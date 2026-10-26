//! The key file that proves a component is installed, and the target
//! and architecture a component is installed for.

use std::fs;
use std::path::Path;

use anyhow::{bail, Result};

use crate::setup::Component;

use super::Ctx;

/// The key file, and the other parts in the component's own folder, must be
/// in the new tree before anything is replaced.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) fn check_key(component: &Component, inside: &[&str], tree: &Path) -> Result<()> {
    check_key_for(
        crate::versions::target_from_path(tree),
        component,
        inside,
        tree,
    )
}

pub(crate) fn check_key_for(
    target: crate::kit::Target,
    component: &Component,
    inside: &[&str],
    tree: &Path,
) -> Result<()> {
    let Some(folder) = &component.folder else {
        return Ok(());
    };
    for k in std::iter::once(component.key_for(target)).chain(component.keys_for(target)) {
        let norm = k.replace('\\', "/");
        let Some(rel) = norm
            .strip_prefix(folder.as_str())
            .and_then(|r| r.strip_prefix('/'))
        else {
            continue;
        };
        if inside.contains(&rel) {
            continue;
        }
        let path = crate::common::fsx::join_rel(tree, rel);
        let ok = if rel.is_empty() {
            fs::read_dir(tree).is_ok_and(|mut d| d.next().is_some())
        } else {
            path.is_file()
        };
        if !ok {
            bail!("the archive does not contain {k}");
        }
    }
    Ok(())
}

/// [`crate::kit::Target`] for downloading a component: the kit's own, since
/// every component is installed with its native build.
pub(super) fn target_of(ctx: &Ctx, _component: &Component) -> crate::kit::Target {
    ctx.kit.target()
}

/// `x64` or `arm64` as used by most download names.
pub(super) fn arch_of(ctx: &Ctx, component: &Component) -> &'static str {
    target_of(ctx, component).arch.as_str()
}

/// `x86_64` or `aarch64`, for Rust-style target names.
pub(super) fn triple_arch(ctx: &Ctx, component: &Component) -> &'static str {
    target_of(ctx, component).gnu_arch()
}
