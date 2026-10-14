//! Installers of the components (port of nabre/atollo `_prepare-*.ps1` and
//! `_languages.ps1`; specification in docs/app/CATALOG.md §2).
//!
//! Each component says where its latest version is and how to verify it
//! (`latest`), then the shared path installs it: download into
//! `data\tmp\setup\<id>`, checksum, extraction, check of the key file,
//! swap into `data\apps\<platform>`, version file, optional finishing step.

mod core;
mod editor;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Context, Result};

use crate::archive::{self, Inner, Kind};
use crate::catalog::Component;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::net::Net;
use crate::verify::{self, Expect};

/// What a step needs from the app.
pub struct Ctx<'a> {
    pub kit: &'a Kit,
    pub net: &'a Net,
    pub env: &'a KitEnv,
    pub cancel: &'a AtomicBool,
}

/// One downloadable build of a component.
#[derive(Debug, Clone)]
pub struct Release {
    pub version: String,
    pub url: String,
    pub file_name: String,
    pub expect: Expect,
    pub kind: Kind,
    pub inner: Inner,
}

/// What the installer of one component provides.
pub trait Installer: Sync {
    /// The latest build for this kit's platform.
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release>;
    /// Work done after the swap (config files, markers). Default: nothing.
    fn finish(&self, _ctx: &Ctx, _component: &Component, _dest: &Path) -> Result<()> {
        Ok(())
    }
}

/// The installer of `id`, if this version of the app has one.
pub fn installer(id: &str) -> Option<&'static dyn Installer> {
    Some(match id {
        "git" => &core::Git,
        "node" => &core::Node,
        "pwsh" => &editor::Pwsh,
        "terminal" => &editor::Terminal,
        "vscodium-server" => &editor::Server,
        _ => return None,
    })
}

/// Progress of one installation, for the interface and the command line.
pub trait Progress {
    fn note(&mut self, text: &str);
    fn bytes(&mut self, done: u64, total: Option<u64>);
}

/// Downloads, verifies, extracts and swaps `release` in place of the
/// component's folder; saves the version.
pub fn install(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    progress: &mut dyn Progress,
) -> Result<()> {
    let Some(folder) = component.folder_path(ctx.kit) else {
        bail!("{} has no folder of its own", component.id);
    };
    let work = ctx.kit.data().join("tmp").join("setup").join(&component.id);
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).with_context(|| work.display().to_string())?;
    let result = (|| {
        let file = work.join(&release.file_name);
        progress.note(&format!("download {}", release.url));
        ctx.net
            .download(&release.url, &file, ctx.cancel, &mut |done, total| {
                progress.bytes(done, total)
            })?;
        verify::check(&file, &release.expect)?;
        progress.note("checksum ok");

        let extracted = work.join("x");
        archive::extract(&file, release.kind, &extracted)?;
        let _ = fs::remove_file(&file);
        let tree = archive::pick_inner(&extracted, &release.inner)?;
        check_key(component, &folder, &tree)?;
        if let Some(parent) = folder.parent() {
            archive::clean_leftovers(parent);
        }
        archive::replace_dir(&tree, &folder)
    })();
    let _ = fs::remove_dir_all(&work);
    result?;
    installer(&component.id)
        .map(|i| i.finish(ctx, component, &folder))
        .transpose()?;
    component.save_version(ctx.kit, &release.version)?;
    Ok(())
}

/// The key file must be in the new tree before anything is replaced.
fn check_key(component: &Component, folder: &Path, tree: &Path) -> Result<()> {
    let key = PathBuf::from(component.key.replace('/', "\\"));
    let Ok(rel) = key.strip_prefix(folder.file_name().unwrap_or_default()) else {
        return Ok(());
    };
    let path = tree.join(rel);
    let ok = if rel.as_os_str().is_empty() {
        fs::read_dir(tree).is_ok_and(|mut d| d.next().is_some())
    } else {
        path.is_file()
    };
    if !ok {
        bail!("the archive does not contain {}", component.key);
    }
    Ok(())
}

/// `x64` or `arm64` as used by most download names, honouring `x64Only`.
fn arch_of(ctx: &Ctx, component: &Component) -> &'static str {
    if component.x64_only {
        "x64"
    } else {
        ctx.kit.arch().as_str()
    }
}
