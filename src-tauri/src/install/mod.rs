//! Installers of the components (port of nabre/dev-kit `_prepare-*.ps1` and
//! `_languages.ps1`; specification in docs/app/CATALOG.md §2).
//!
//! Each component says where its latest build is and how to verify it
//! (`latest`): a release made of one or more parts (Java with Maven and
//! Gradle, CMake with Ninja...). The shared path installs each part:
//! download into `data\tmp\setup\<id>`, checksum, extraction, check of the
//! key file, then the swap into `data\apps\<platform>`. The saved version
//! joins the versions of the parts, so a new build of any part is noticed.
//! Installers made of processes (Rust, pnpm, R) implement `custom`.

mod agents;
mod core;
mod editor;
mod lang;
mod native;
mod web;

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

/// Where a part goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dest {
    /// The component's own folder, replaced as a whole.
    Own,
    /// Another folder of `data\apps\<platform>` (Maven, Gradle).
    Folder(&'static str),
    /// Single files copied into `data\apps\<platform>\tools`, found anywhere
    /// in the extracted tree (uv, gh, Ninja, micromamba).
    Tools(Vec<&'static str>),
    /// One file replaced in place (`<folder>\<name>`): the old one is renamed
    /// first, so a program in use can still be updated (Claude, Antigravity).
    File {
        folder: &'static str,
        name: &'static str,
        from: String,
    },
}

/// An Authenticode signature to check on a file of the new tree before it
/// is put in place (`Assert-Signature` of the kit).
#[derive(Debug, Clone)]
pub struct Sign {
    /// Path of the file in the extracted tree.
    pub file: String,
    /// Regex the signer subject must match.
    pub publisher: &'static str,
    /// The signature is the only check: an unverifiable revocation fails.
    pub required: bool,
}

/// One downloadable file of a release.
#[derive(Debug, Clone)]
pub struct Part {
    pub url: String,
    pub file_name: String,
    pub expect: Expect,
    pub kind: Kind,
    pub inner: Inner,
    pub dest: Dest,
    pub sign: Option<Sign>,
}

/// The latest build of a component for this kit.
#[derive(Debug, Clone)]
pub struct Release {
    pub version: String,
    pub parts: Vec<Part>,
}

impl Release {
    /// The common case: one archive that becomes the component's folder.
    pub fn single(
        version: String,
        url: String,
        file_name: String,
        expect: Expect,
        kind: Kind,
        inner: Inner,
    ) -> Release {
        Release {
            version,
            parts: vec![Part {
                url,
                file_name,
                expect,
                kind,
                inner,
                dest: Dest::Own,
                sign: None,
            }],
        }
    }
}

/// What the installer of one component provides.
pub trait Installer: Sync {
    /// The latest build for this kit's platform.
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release>;
    /// Work on the new tree of the `Own` part before the swap (keep the
    /// user's packages...). Default: nothing.
    fn before_swap(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        _tree: &Path,
        _old: &Path,
    ) -> Result<()> {
        Ok(())
    }
    /// Work done after the swap (config files, markers). Default: nothing.
    fn finish(&self, _ctx: &Ctx, _component: &Component, _dest: &Path) -> Result<()> {
        Ok(())
    }
    /// Installers made of processes instead of archives: install `release`
    /// themselves. Default: the shared path.
    fn custom(&self) -> bool {
        false
    }
    fn install_custom(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        _release: &Release,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        bail!("no custom installer")
    }
}

/// The installer of `id`, if this version of the app has one.
pub fn installer(id: &str) -> Option<&'static dyn Installer> {
    Some(match id {
        "claude" => &agents::Claude,
        "codex" => &agents::Codex,
        "antigravity" => &agents::Antigravity,
        "git" => &core::Git,
        "python" => &core::Python,
        "node" => &core::Node,
        "uv" => &core::Uv,
        "gh" => &core::Gh,
        "php" => &web::Php,
        "apache" => &web::Apache,
        "rust" => &native::Rust,
        "go" => &lang::Go,
        "java" => &lang::Java,
        "kotlin" => &lang::Kotlin,
        "dotnet" => &lang::Dotnet,
        "dart" => &lang::Dart,
        "zig" => &lang::Zig,
        "julia" => &lang::Julia,
        "bun" => &lang::Bun,
        "deno" => &lang::Deno,
        "typst" => &lang::Typst,
        "tinytex" => &lang::TinyTex,
        "pnpm" => &native::Pnpm,
        "cmake" => &lang::Cmake,
        "cc" => &lang::Cc,
        "ruby" => &lang::Ruby,
        "r" => &native::R,
        "pwsh" => &editor::Pwsh,
        "terminal" => &editor::Terminal,
        "vscodium-server" => &editor::Server,
        "vscodium" => &editor::Vscodium,
        _ => return None,
    })
}

/// Progress of one installation, for the interface and the command line.
pub trait Progress {
    fn note(&mut self, text: &str);
    fn bytes(&mut self, done: u64, total: Option<u64>);
}

/// The work folder of a component: `data\tmp\setup\<id>`, emptied.
pub fn work_dir(ctx: &Ctx, component: &Component) -> Result<PathBuf> {
    let work = ctx.kit.data().join("tmp").join("setup").join(&component.id);
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).with_context(|| work.display().to_string())?;
    Ok(work)
}

/// Downloads and verifies one file into `work`.
pub fn fetch(
    ctx: &Ctx,
    url: &str,
    file_name: &str,
    expect: &Expect,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<PathBuf> {
    let file = work.join(file_name);
    progress.note(&format!("download {url}"));
    ctx.net
        .download(url, &file, ctx.cancel, &mut |done, total| {
            progress.bytes(done, total)
        })?;
    verify::check(&file, expect)?;
    progress.note("checksum ok");
    Ok(file)
}

/// Installs `release`: every part, then the finishing step and the version.
pub fn install(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    progress: &mut dyn Progress,
) -> Result<()> {
    let Some(inst) = installer(&component.id) else {
        bail!("{}: no installer", component.id);
    };
    if inst.custom() {
        inst.install_custom(ctx, component, release, progress)?;
    } else {
        let work = work_dir(ctx, component)?;
        let result = release.parts.iter().enumerate().try_for_each(|(i, part)| {
            install_part(
                ctx,
                inst,
                component,
                part,
                &work.join(i.to_string()),
                progress,
            )
        });
        let _ = fs::remove_dir_all(&work);
        result?;
        if let Some(folder) = component.folder_path(ctx.kit) {
            inst.finish(ctx, component, &folder)?;
        } else {
            inst.finish(ctx, component, &ctx.kit.apps().join("tools"))?;
        }
    }
    component.save_version(ctx.kit, &release.version)?;
    Ok(())
}

fn install_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<()> {
    fs::create_dir_all(work)?;
    let file = fetch(
        ctx,
        &part.url,
        &part.file_name,
        &part.expect,
        work,
        progress,
    )?;
    let extracted = work.join("x");
    archive::extract(&file, part.kind, &extracted)?;
    if part.kind != Kind::File {
        let _ = fs::remove_file(&file);
    }
    let tree = archive::pick_inner(&extracted, &part.inner)?;
    if let Some(sign) = &part.sign {
        let file = tree.join(sign.file.replace('/', "\\"));
        let publisher = pattern(sign.publisher)?;
        match crate::signature::check(&file, &publisher, sign.required)? {
            crate::signature::Verdict::Valid { subject } => {
                progress.note(&format!("signature ok: {subject}"))
            }
            crate::signature::Verdict::RevocationUnknown { subject } => {
                progress.note(&format!(
                    "signature of {subject}: revocation not verifiable (offline?)"
                ));
            }
        }
    }
    let apps = ctx.kit.apps();
    match &part.dest {
        Dest::Own => {
            let Some(folder) = component.folder_path(ctx.kit) else {
                bail!("{} has no folder of its own", component.id);
            };
            check_key(component, &folder, &tree)?;
            inst.before_swap(ctx, component, &tree, &folder)?;
            if let Some(parent) = folder.parent() {
                archive::clean_leftovers(parent);
            }
            archive::replace_dir(&tree, &folder)
        }
        Dest::Folder(name) => {
            let folder = apps.join(name);
            archive::clean_leftovers(&apps);
            archive::replace_dir(&tree, &folder)
        }
        Dest::Tools(names) => {
            let tools = apps.join("tools");
            fs::create_dir_all(&tools)?;
            for name in names {
                let found = find_file(&tree, name)
                    .with_context(|| format!("{name} not found in {}", part.file_name))?;
                archive::replace_file(&found, &tools.join(name))?;
            }
            Ok(())
        }
        Dest::File { folder, name, from } => {
            let src = tree.join(from.replace('/', "\\"));
            if !src.is_file() {
                bail!("{from} not found in {}", part.file_name);
            }
            let dir = apps.join(folder);
            fs::create_dir_all(&dir)?;
            archive::replace_file(&src, &dir.join(name))
        }
    }
}

/// First file called `name` (case-insensitive) under `dir`.
pub fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in fs::read_dir(&d).ok()?.flatten() {
            let path = entry.path();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                stack.push(path);
            } else if entry
                .file_name()
                .to_string_lossy()
                .eq_ignore_ascii_case(name)
            {
                return Some(path);
            }
        }
    }
    None
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

/// `x86_64` or `aarch64`, for Rust-style target names.
fn triple_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "aarch64"
    } else {
        "x86_64"
    }
}

fn pattern(re: &str) -> Result<regex::Regex> {
    regex::Regex::new(re).context("asset pattern")
}

/// A component published as a GitHub release asset matching `re` (GitHub's
/// SHA-256 `digest`), installed as one archive (`Get-GitHubAsset` +
/// `Install-Archive` of the kit).
fn github_single(ctx: &Ctx, repo: &str, re: &str, kind: Kind, inner: Inner) -> Result<Release> {
    let rel = ctx.net.github_latest(repo)?;
    let asset = rel
        .asset(&pattern(re)?)
        .ok_or_else(|| anyhow::anyhow!("{repo} {}: no asset matching {re}", rel.tag_name))?;
    let version = rel
        .tag_name
        .trim_start_matches("bun-")
        .trim_start_matches('v')
        .to_owned();
    Ok(Release::single(
        version,
        asset.browser_download_url.clone(),
        asset.name.clone(),
        Expect::Sha256(crate::net::github_digest(asset)?),
        kind,
        inner,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;
    use crate::kit::tests::TempDir;

    #[test]
    fn every_catalog_component_has_an_installer() {
        for c in Catalog::get().all() {
            assert!(installer(&c.id).is_some(), "{} has no installer", c.id);
        }
    }

    #[test]
    fn find_file_is_case_insensitive_and_recursive() {
        let tmp = TempDir::new("findfile");
        let deep = tmp.0.join("a").join("b");
        fs::create_dir_all(&deep).unwrap();
        fs::write(deep.join("UV.EXE"), b"").unwrap();
        assert_eq!(find_file(&tmp.0, "uv.exe"), Some(deep.join("UV.EXE")));
        assert_eq!(find_file(&tmp.0, "gh.exe"), None);
    }
}
