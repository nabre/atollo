//! Installers of the components (specification in docs/app/CATALOG.md §2).
//!
//! Each component says where its latest build is and how to verify it
//! (`latest`): a release made of one or more parts (Java with Maven and
//! Gradle, CMake with Ninja...). The shared path installs each part:
//! download into `setup\<id>` of the PC's temporary folder (`data\tmp`
//! without one, D-90), checksum, extraction, check of the key file, then the
//! swap into `data\apps\<platform>` (`archive::stage_into`, a copy first when
//! the kit is on another disk). The saved version
//! joins the versions of the parts, so a new build of any part is noticed.
//! Installers made of processes (Rust, pnpm, R) implement `custom`.
//!
//! Versioned components (`versions/mod.rs`) have one folder per minor line,
//! `data\apps\<platform>\<id>\<line>`: an update installs into that of its
//! line (the old line stays), and another line can be installed next to it
//! (`lines`, `release_line`).

//!
//! The shared path is in `flow` (the work folder, download, installation
//! of a release or of a line, relocation), `parts` (each part placed in
//! its folder) and `keys` (the key file of a component, its target and
//! architecture).

mod agents;
mod android;
mod caddy;
pub mod conda;
mod conpty;
mod core;
mod db;
mod editor;
mod flow;
mod keys;
mod lang;
mod llama;
mod native;
mod parts;
mod web;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use anyhow::{bail, Context, Result};

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::Net;
use crate::setup::Component;

pub(crate) use lang::xcode_selected;
pub use lang::{make_launchers, version_key};
pub use web::{adopt_composer, migrate_php_ini, php_ini_dir};

pub(crate) use flow::is_plain_file_name;
pub use flow::{
    fetch, flag_not_relocatable, install, install_line, release_at, relocate, work_dir,
};
use keys::{arch_of, target_of, triple_arch};
pub use parts::find_file;
pub(crate) use parts::{install_part, put_file};

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
#[cfg_attr(not(windows), allow(dead_code))]
pub struct Sign {
    /// Path of the file in the extracted tree.
    pub file: String,
    /// The organization (`O=`) of the signer, exactly.
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
    /// Checked on Windows only (Authenticode).
    #[cfg_attr(not(windows), allow(dead_code))]
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
    /// Installers made of processes instead of archives on `target`: install
    /// `release` themselves. Default: the shared path.
    fn custom(&self, _target: crate::kit::Target) -> bool {
        false
    }
    /// Its folder keeps working when the kit moves (another drive letter
    /// or folder) on `target`. Not so for a conda environment off Windows
    /// (Git, R), which has its folder written inside: when the kit moves
    /// it is marked to be installed again (`flag_not_relocatable`, D-92).
    /// Default: it can move.
    fn relocatable(&self, _target: crate::kit::Target) -> bool {
        true
    }
    /// A component that follows the kit but writes its folder into some
    /// file (R's Makeconf on Windows): those files are set to the folders
    /// it is in now (`relocate`). Default: nothing to do.
    fn relocate(&self, _kit: &Kit, _component: &Component) -> Result<()> {
        Ok(())
    }
    /// The conda environment of `dir`, the folder of a line (or the
    /// component's own folder), on `target`: the folder itself, or one
    /// inside it (`cc/make`). It is what is created again from the kept
    /// packages when the kit moves (`conda`, D-118). Default: none.
    fn conda_prefix(&self, _target: crate::kit::Target, _dir: &Path) -> Option<PathBuf> {
        None
    }
    /// The conda package whose programs go on the kit's PATH on `target`,
    /// through the launchers of `kit-bin` (`conda::write_launchers`, D-125),
    /// instead of the whole `bin` of the environment. Default: none.
    fn conda_programs(&self, _target: crate::kit::Target) -> Option<&'static str> {
        None
    }
    /// The component's own steps after its conda environment is created in
    /// `dir` (launchers, files rewritten, shims): the same after an
    /// installation from the network and after one from the kept packages
    /// (D-118). Default: nothing.
    fn after_conda(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        _dir: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        Ok(())
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
        "pi" => &agents::Pi,
        "llama" => &llama::Llama,
        "git" => &core::Git,
        "python" => &core::Python,
        "node" => &core::Node,
        "uv" => &core::Uv,
        "gh" => &core::Gh,
        "php" => &web::Php,
        "apache" => &web::Apache,
        "caddy" => &caddy::Caddy,
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
        "vscodium-server" => &editor::Server,
        "conpty" => &conpty::ConPty,
        "mysql" => &db::MySql,
        "mongodb" => &db::MongoDb,
        "sqlite" => &db::Sqlite,
        "phpmyadmin" => &db::PhpMyAdmin,
        "mongo-express" => &db::MongoExpress,
        _ => return None,
    })
}

/// Progress of one installation, for the interface and the command line.
pub trait Progress {
    fn note(&mut self, text: &str);
    fn bytes(&mut self, done: u64, total: Option<u64>);
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
        Expect::Sha256(ctx.net.release_sha256(&rel, asset)?),
        kind,
        inner,
    ))
}

#[cfg(test)]
#[path = "../../../test/rust/install/mod.rs"]
mod tests;
