//! Installers of the components (port of nabre/atollo `_prepare-*.ps1` and
//! `_languages.ps1`; specification in docs/app/CATALOG.md §2).
//!
//! Each component says where its latest build is and how to verify it
//! (`latest`): a release made of one or more parts (Java with Maven and
//! Gradle, CMake with Ninja...). The shared path installs each part:
//! download into `data\tmp\setup\<id>`, checksum, extraction, check of the
//! key file, then the swap into `data\apps\<platform>`. The saved version
//! joins the versions of the parts, so a new build of any part is noticed.
//! Installers made of processes (Rust, pnpm, R) implement `custom`.
//!
//! Versioned components (`versions.rs`) have one folder per minor line,
//! `data\apps\<platform>\<id>\<line>`: an update installs into that of its
//! line (the old line stays), and another line can be installed next to it
//! (`lines`, `release_line`).

mod agents;
mod android;
mod core;
mod db;
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

pub use lang::{make_launchers, version_key};
pub use web::adopt_composer;

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
    /// One file put into the component's own folder after its swap, so each
    /// version has its own (Composer in every folder of PHP).
    Inside(&'static str),
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
    /// A versioned custom installer (R): `release` into `target`, the
    /// folder of its line (the default's or a saved one).
    fn install_custom_at(
        &self,
        _ctx: &Ctx,
        component: &Component,
        _release: &Release,
        _target: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        bail!("{}: versions cannot be saved", component.id)
    }
    /// Files of its own folder that a `Dest::Inside` part puts there after
    /// the swap (Composer in PHP's): not looked for in the archive.
    fn inside(&self) -> &'static [&'static str] {
        &[]
    }
    /// Other lines (`major.minor`) can be downloaded: `lines` and
    /// `release_line` are implemented.
    fn has_lines(&self) -> bool {
        false
    }
    /// The lines available for this kit's platform, newest first.
    fn lines(&self, _ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(Vec::new())
    }
    /// The newest build of `line`; only its `Own` part is installed.
    fn release_line(&self, _ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        bail!("{}: no version {line} to download", component.id)
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
        "android" => &android::Android,
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
        "mariadb" => &db::MariaDb,
        "mongodb" => &db::MongoDb,
        "sqlite" => &db::Sqlite,
        "adminer" => &db::Adminer,
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

/// Downloads and verifies one file into `work`; the same file added by hand
/// (`downloads.rs`) is used instead, and a failed download says where to
/// get it in the browser (`ByHand`).
pub fn fetch(
    ctx: &Ctx,
    url: &str,
    file_name: &str,
    expect: &Expect,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<PathBuf> {
    let file = work.join(file_name);
    if crate::downloads::take(ctx.kit, file_name, expect, &file)? {
        progress.note("file added by hand, checksum ok");
        return Ok(file);
    }
    progress.note(&format!("download {url}"));
    let downloaded = ctx
        .net
        .download(url, &file, ctx.cancel, &mut |done, total| {
            progress.bytes(done, total)
        });
    match downloaded {
        Err(e) if !e.is::<crate::net::Cancelled>() => {
            return Err(crate::net::ByHand {
                url: url.to_owned(),
                file_name: file_name.to_owned(),
                cause: e,
            }
            .into())
        }
        other => other?,
    }
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
        if component.versioned {
            let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
            crate::versions::write_version(&dir, &release.version)?;
        }
    } else {
        let work = work_dir(ctx, component)?;
        // A versioned component goes into the folder of its line, next to
        // the others (versions.rs).
        let own = if component.versioned {
            Some(crate::versions::install_dir(
                ctx.kit,
                component,
                &release.version,
            ))
        } else {
            component.folder_path(ctx.kit)
        };
        let result = release.parts.iter().enumerate().try_for_each(|(i, part)| {
            install_part(
                ctx,
                inst,
                component,
                part,
                own.as_deref(),
                &work.join(i.to_string()),
                progress,
            )
        });
        let _ = fs::remove_dir_all(&work);
        result?;
        match &own {
            Some(folder) => inst.finish(ctx, component, folder)?,
            None => inst.finish(ctx, component, &ctx.kit.apps().join("tools"))?,
        }
        if let (true, Some(folder)) = (component.versioned, &own) {
            crate::versions::write_version(folder, &release.version)?;
        }
    }
    // The installed version first: it says which folder of a versioned
    // component is the default, where its key is looked for.
    let previous = component.installed_version(ctx.kit);
    component.save_version(ctx.kit, &release.version)?;
    // Every part, also those outside its folder (Maven, Ninja, Composer, the
    // yarn shim): otherwise every preparation would install it again.
    if let Some(part) = component.missing_part(ctx.kit) {
        match previous {
            Some(v) => component.save_version(ctx.kit, &v)?,
            None => component.forget_version(ctx.kit),
        }
        bail!(
            "{}: {} missing after the installation",
            component.id,
            part.display()
        );
    }
    Ok(())
}

/// Installs `release` of a versioned component as a saved line: only its
/// own folder and what goes inside it (Composer), not Maven or Gradle, into
/// `data\apps\<platform>\<id>\<line>`. Returns the line.
pub fn install_line(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    progress: &mut dyn Progress,
) -> Result<String> {
    let Some(inst) = installer(&component.id) else {
        bail!("{}: no installer", component.id);
    };
    let line = crate::versions::line_of(&release.version)
        .ok_or_else(|| anyhow::anyhow!("{}: version {}", component.id, release.version))?;
    if !component.versioned {
        bail!("{}: versions cannot be saved", component.id);
    }
    if crate::versions::default_line(ctx.kit, component).as_deref() == Some(line.as_str()) {
        bail!("{} {line}: it is the default version", component.id);
    }
    let target = crate::versions::saved_dir(ctx.kit, component, &line);
    if inst.custom() {
        inst.install_custom_at(ctx, component, release, &target, progress)?;
        if !crate::versions::has_key(component, &target) {
            bail!(
                "{}: {} missing after the installation",
                component.id,
                component.key
            );
        }
        crate::versions::write_version(&target, &release.version)?;
        return Ok(line);
    }
    if !release.parts.iter().any(|p| p.dest == Dest::Own) {
        bail!("{}: nothing to install", component.id);
    }
    let work = work_dir(ctx, component)?;
    // Its folder and what goes inside it (Composer), in order.
    let result = release
        .parts
        .iter()
        .filter(|p| matches!(p.dest, Dest::Own | Dest::Inside(_)))
        .enumerate()
        .try_for_each(|(i, part)| {
            install_part(
                ctx,
                inst,
                component,
                part,
                Some(&target),
                &work.join(i.to_string()),
                progress,
            )
        });
    let _ = fs::remove_dir_all(&work);
    result?;
    inst.finish(ctx, component, &target)?;
    if !crate::versions::has_key(component, &target) {
        bail!(
            "{}: {} missing after the installation",
            component.id,
            component.key
        );
    }
    crate::versions::write_version(&target, &release.version)?;
    Ok(line)
}

#[allow(clippy::too_many_arguments)]
fn install_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    own: Option<&Path>,
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
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            check_key(component, inst.inside(), &tree)?;
            inst.before_swap(ctx, component, &tree, folder)?;
            if let Some(parent) = folder.parent() {
                archive::clean_leftovers(parent);
            }
            archive::replace_dir(&tree, folder)
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
        Dest::Inside(name) => {
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            let found = find_file(&tree, name)
                .with_context(|| format!("{name} not found in {}", part.file_name))?;
            archive::replace_file(&found, &folder.join(name))
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

/// The key file, and the other parts in the component's own folder, must be
/// in the new tree before anything is replaced.
fn check_key(component: &Component, inside: &[&str], tree: &Path) -> Result<()> {
    let Some(folder) = &component.folder else {
        return Ok(());
    };
    for k in std::iter::once(&component.key).chain(&component.keys) {
        let key = PathBuf::from(k.replace('/', "\\"));
        let Ok(rel) = key.strip_prefix(folder) else {
            continue;
        };
        if inside.iter().any(|name| rel == Path::new(name)) {
            continue;
        }
        let path = tree.join(rel);
        let ok = if rel.as_os_str().is_empty() {
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

    #[test]
    fn check_key_wants_every_part_of_the_folder() {
        let tmp = TempDir::new("checkkey");
        let server = Catalog::get().find("vscodium-server").unwrap();
        let tree = tmp.0.join("tree");
        fs::create_dir_all(tree.join("out")).unwrap();
        fs::write(tree.join(r"out\server-main.js"), b"").unwrap();
        let err = check_key(server, &[], &tree).unwrap_err();
        assert!(err.to_string().contains("node.exe"), "{err}");
        fs::write(tree.join("node.exe"), b"").unwrap();
        check_key(server, &[], &tree).unwrap();

        // Parts in other folders (Maven, Gradle) are not in this tree.
        let java = Catalog::get().find("java").unwrap();
        let tree = tmp.0.join("jdk");
        fs::create_dir_all(tree.join("bin")).unwrap();
        fs::write(tree.join(r"bin\java.exe"), b"").unwrap();
        check_key(java, &[], &tree).unwrap();

        // Composer goes inside PHP's folder after the zip: not in it.
        let php = Catalog::get().find("php").unwrap();
        let tree = tmp.0.join("php");
        fs::create_dir_all(&tree).unwrap();
        fs::write(tree.join("php.exe"), b"").unwrap();
        assert!(check_key(php, &[], &tree).is_err());
        check_key(php, installer("php").unwrap().inside(), &tree).unwrap();
    }
}
