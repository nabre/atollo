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

mod agents;
mod android;
mod caddy;
pub mod conda;
mod conpty;
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

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::net::archive::{self, Inner, Kind};
use crate::net::verify::{self, Expect};
use crate::net::Net;
use crate::os::temp::TempWork;
use crate::setup::catalog::Component;

pub(crate) use lang::xcode_selected;
pub use lang::{make_launchers, version_key};
pub use web::{adopt_composer, migrate_php_ini, php_ini_dir};

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
        _ => return None,
    })
}

/// Progress of one installation, for the interface and the command line.
pub trait Progress {
    fn note(&mut self, text: &str);
    fn bytes(&mut self, done: u64, total: Option<u64>);
}

/// Free space the PC's temporary folder must have for an installation's
/// download and extraction; the sizes are not known before the download, and
/// the largest components (Android SDK, Rust, TeX Live) stay below this.
const SETUP_ROOM: u64 = 4 << 30;

/// The work folder of a component, `setup\<id>` in the PC's temporary folder
/// (D-90; `data\tmp\setup\<id>` without one or with less than `SETUP_ROOM`
/// free there), emptied. It lives as long as the guard: keep it for the
/// whole installation.
pub fn work_dir(ctx: &Ctx, component: &Component) -> Result<TempWork> {
    let rel = format!("setup/{}", component.id);
    TempWork::new_with_space(ctx.kit, &rel, SETUP_ROOM)
        .with_context(|| format!("work folder {rel}"))
}

/// A file [`fetch`] copied in place of a download: pre-downloaded for this
/// system (`router/bootstrap.rs`) or added by hand (`setup/downloads.rs`).
enum Taken {
    Staged(PathBuf),
    ByHand(PathBuf, Expect),
}

impl Taken {
    fn used(self, kit: &Kit) {
        match self {
            Taken::Staged(file) => crate::router::bootstrap::used(kit, &file),
            Taken::ByHand(file, expect) => crate::setup::downloads::used(kit, &file, &expect),
        }
    }
}

thread_local! {
    /// What `fetch` took for the installation running on this thread
    /// ([`keeping_taken`]); `None` outside one.
    static TAKEN: std::cell::RefCell<Option<Vec<Taken>>> = const { std::cell::RefCell::new(None) };
}

/// `file` was taken by `fetch`: used up when the installation succeeds,
/// at once outside an installation.
fn taken(kit: &Kit, file: Taken) {
    let mut file = Some(file);
    TAKEN.with(|t| {
        if let Some(list) = t.borrow_mut().as_mut() {
            list.extend(file.take());
        }
    });
    if let Some(file) = file {
        file.used(kit);
    }
}

/// Runs the installation `run`: the files `fetch` took in place of a
/// download are deleted only when it succeeds. One that fails (extraction,
/// an antivirus, a file in use) finds them again at the next attempt
/// instead of asking for the download by hand again.
fn keeping_taken<T>(kit: &Kit, run: impl FnOnce() -> Result<T>) -> Result<T> {
    /// Puts back the list of an outer installation, also after a panic.
    struct Scope(Option<Option<Vec<Taken>>>);
    impl Scope {
        fn end(mut self) -> Vec<Taken> {
            let outer = self.0.take().flatten();
            TAKEN.with(|t| t.replace(outer)).unwrap_or_default()
        }
    }
    impl Drop for Scope {
        fn drop(&mut self) {
            if let Some(outer) = self.0.take() {
                TAKEN.with(|t| t.replace(outer));
            }
        }
    }
    let scope = Scope(Some(TAKEN.with(|t| t.replace(Some(Vec::new())))));
    let result = run();
    let files = scope.end();
    if result.is_ok() {
        for file in files {
            file.used(kit);
        }
    }
    result
}

/// A file name only (`[A-Za-z0-9._+-]+`, not `.` or `..`): no separator,
/// drive or other character that would put the download elsewhere.
pub(crate) fn is_plain_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().any(|b| b != b'.')
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'+' | b'-'))
}

/// Downloads and verifies one file into `work`; the same file added by hand
/// (`setup/downloads.rs`) is used instead, and a failed download says where to
/// get it in the browser (`ByHand`).
pub fn fetch(
    ctx: &Ctx,
    url: &str,
    file_name: &str,
    expect: &Expect,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<PathBuf> {
    // The name often comes from remote metadata (a version in a manifest,
    // an asset name): one that could leave `work` is refused.
    if !is_plain_file_name(file_name) {
        bail!("refused download name {file_name:?}: only letters, digits and . _ + -");
    }
    let file = work.join(file_name);
    if let Some(staged) = crate::router::bootstrap::take(ctx.kit, file_name, expect, &file)? {
        progress.note("bootstrap pre-downloaded, checksum ok");
        taken(ctx.kit, Taken::Staged(staged));
        return Ok(file);
    }
    if let Some(added) = crate::setup::downloads::take(ctx.kit, file_name, expect, &file)? {
        progress.note("file added by hand, checksum ok");
        taken(ctx.kit, Taken::ByHand(added, expect.clone()));
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
                expect: expect.clone(),
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
    keeping_taken(ctx.kit, || install_now(ctx, component, release, progress))
}

fn install_now(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    progress: &mut dyn Progress,
) -> Result<()> {
    let Some(inst) = installer(&component.id) else {
        bail!("{}: no installer", component.id);
    };
    if crate::router::bootstrap::should_stage(ctx, inst, component) {
        return crate::router::bootstrap::stage(ctx, inst, component, release, None, progress);
    }
    if inst.custom(target_of(ctx, component)) {
        inst.install_custom(ctx, component, release, progress)?;
        if component.per_line() {
            let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
            crate::versions::write_version(&dir, &release.version)?;
        }
    } else {
        let work = work_dir(ctx, component)?;
        // A component with one folder per line goes into that of its line,
        // next to the others (versions/mod.rs).
        let own = if component.per_line() {
            Some(crate::versions::install_dir(
                ctx.kit,
                component,
                &release.version,
            ))
        } else {
            component.folder_path(ctx.kit)
        };
        let parts: Vec<&Part> = release.parts.iter().collect();
        let result = install_parts(
            ctx,
            inst,
            component,
            &parts,
            own.as_deref(),
            work.path(),
            progress,
        );
        drop(work);
        result?;
        match &own {
            Some(folder) => inst.finish(ctx, component, folder)?,
            None => inst.finish(ctx, component, &ctx.kit.apps().join("tools"))?,
        }
        if let (true, Some(folder)) = (component.per_line(), &own) {
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
    // Installed for this root: no longer to be installed again (D-92).
    component.clear_reinstall(ctx.kit);
    // A tool always at its latest version keeps only the new line.
    crate::versions::drop_old_lines(ctx.kit, component);
    Ok(())
}

/// The kit moved from `old_root`: the components on disk that cannot follow
/// it on this kit's target (`Installer::relocatable`) are marked to be
/// installed again (`Component::needs_reinstall`); the automatic setup at
/// start reinstalls them (`setup::task::outdated_app`). Their saved lines
/// are marked one by one (`Component::lines_to_reinstall`). Returns the ids,
/// `<id>@<line>` for a saved line.
/// Nothing on Windows, where every component can move.
pub fn flag_not_relocatable(kit: &Kit, old_root: &str) -> std::io::Result<Vec<String>> {
    let target = kit.target();
    let mut flagged = Vec::new();
    for c in crate::setup::catalog::Catalog::get().all() {
        let fixed = installer(&c.id).is_some_and(|i| !i.relocatable(target));
        if !fixed {
            continue;
        }
        if c.is_present(kit) {
            c.mark_reinstall(kit, old_root)?;
            flagged.push(c.id.clone());
        }
        // The saved lines (R 4.4 next to the default 4.5) are conda
        // environments too (only beside an installed default: otherwise every
        // folder looks saved): each one is marked and installed again at its
        // line by the same start (`setup::runner::run`).
        if c.versioned && c.is_installed(kit) {
            for line in crate::versions::saved(kit, c) {
                c.mark_line_reinstall(kit, &line.line, old_root)?;
                flagged.push(format!("{}@{}", c.id, line.line));
            }
        }
    }
    Ok(flagged)
}

/// The components that follow the kit when it moves (`relocatable`) set
/// the folders written in their files to where they are now
/// (`Installer::relocate`). At every start, not only after a move noticed
/// now: a kit moved before keeps working too; nothing is written when the
/// files are right. A failure is logged, the start goes on.
pub fn relocate(kit: &Kit) {
    let target = kit.target();
    for c in crate::setup::catalog::Catalog::get().all() {
        let Some(inst) = installer(&c.id).filter(|i| i.relocatable(target)) else {
            continue;
        };
        if let Err(e) = inst.relocate(kit, c) {
            crate::log::warn(&format!("{}: relocation: {e:#}", c.id));
        }
    }
}

/// The release of a first installation at `line` instead of the latest:
/// its own folder from `line`, the shared parts (Maven and Gradle, mongosh,
/// the tools) from the latest, as a saved line gets them from the default.
pub fn release_at(ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
    let Some(inst) = installer(&component.id) else {
        bail!("{}: no installer", component.id);
    };
    if !component.versioned || !inst.has_lines() {
        bail!("{}: no version {line} to download", component.id);
    }
    let mut release = inst.release_line(ctx, component, line)?;
    if !inst.custom(target_of(ctx, component)) {
        let latest = inst.latest(ctx, component)?;
        release.parts.extend(
            latest
                .parts
                .into_iter()
                .filter(|p| matches!(p.dest, Dest::Folder(_) | Dest::Tools(_))),
        );
    }
    Ok(release)
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
    keeping_taken(ctx.kit, || {
        install_line_now(ctx, component, release, progress)
    })
}

fn install_line_now(
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
    if crate::router::bootstrap::should_stage(ctx, inst, component) {
        crate::router::bootstrap::stage(
            ctx,
            inst,
            component,
            release,
            Some((&line, &target)),
            progress,
        )?;
        return Ok(line);
    }
    if inst.custom(target_of(ctx, component)) {
        inst.install_custom_at(ctx, component, release, &target, progress)?;
        if !crate::versions::has_key(component, &target) {
            bail!(
                "{}: {} missing after the installation",
                component.id,
                component.key
            );
        }
        crate::versions::write_version(&target, &release.version)?;
        // Installed for this root: no longer to be installed again (D-92).
        component.clear_line_reinstall(ctx.kit, &line);
        return Ok(line);
    }
    if !release.parts.iter().any(|p| p.dest == Dest::Own) {
        bail!("{}: nothing to install", component.id);
    }
    let work = work_dir(ctx, component)?;
    // Its folder and what goes inside it (Composer), in order.
    let parts: Vec<&Part> = release
        .parts
        .iter()
        .filter(|p| matches!(p.dest, Dest::Own | Dest::Inside(_)))
        .collect();
    let result = install_parts(
        ctx,
        inst,
        component,
        &parts,
        Some(&target),
        work.path(),
        progress,
    );
    drop(work);
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
    component.clear_line_reinstall(ctx.kit, &line);
    Ok(line)
}

/// Every part of a release downloaded, checked and extracted, each in its
/// own `work\<i>`, before the first one is put in place: a download that
/// fails (the FPM archive of PHP, Composer) leaves the installed folder as
/// it was, never a new `bin/php` without its `sbin/php-fpm` or
/// `composer.phar` (issue #4).
pub(crate) fn install_parts(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    parts: &[&Part],
    own: Option<&Path>,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<()> {
    let mut trees = Vec::with_capacity(parts.len());
    for (i, part) in parts.iter().enumerate() {
        trees.push(fetch_part(ctx, part, &work.join(i.to_string()), progress)?);
    }
    for (part, tree) in parts.iter().zip(&trees) {
        place_part(ctx, inst, component, part, tree, own)?;
    }
    Ok(())
}

/// One part downloaded and put in place at once (the bootstrap's tools,
/// the parts of MongoDB after its environment).
#[allow(clippy::too_many_arguments)]
pub(crate) fn install_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    own: Option<&Path>,
    work: &Path,
    progress: &mut dyn Progress,
) -> Result<()> {
    let tree = fetch_part(ctx, part, work, progress)?;
    place_part(ctx, inst, component, part, &tree, own)
}

/// Downloads and verifies `part` into `work` and extracts it: the tree it
/// brings (its signature checked on Windows), nothing in place yet.
fn fetch_part(ctx: &Ctx, part: &Part, work: &Path, progress: &mut dyn Progress) -> Result<PathBuf> {
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
    #[cfg(windows)]
    if ctx.kit.os().is_windows() {
        if let Some(sign) = &part.sign {
            let file = crate::fsx::join_rel(&tree, &sign.file);
            match crate::net::signature::check(&file, sign.publisher, sign.required)? {
                crate::net::signature::Verdict::Valid { subject } => {
                    progress.note(&format!("signature ok: {subject}"))
                }
                crate::net::signature::Verdict::RevocationUnknown { subject } => {
                    progress.note(&format!(
                        "signature of {subject}: revocation not verifiable (offline?)"
                    ));
                }
            }
        }
    }
    Ok(tree)
}

/// Puts the tree [`fetch_part`] extracted where `part` goes.
fn place_part(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    part: &Part,
    tree: &Path,
    own: Option<&Path>,
) -> Result<()> {
    let tree = tree.to_path_buf();
    let apps = ctx.kit.apps();
    match &part.dest {
        Dest::Own => {
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            check_key_for(ctx.kit.target(), component, inst.inside(), &tree)?;
            inst.before_swap(ctx, component, &tree, folder)?;
            crate::router::fs::ensure_tree_executables(ctx.kit.target(), &tree);
            if let Some(parent) = folder.parent() {
                archive::clean_leftovers(parent);
            }
            // From the PC's temporary folder: checked, copied next to the
            // folder, then swapped (on the same disk, renamed as before).
            archive::stage_into(&tree, folder)
        }
        Dest::Folder(name) => {
            let folder = apps.join(name);
            crate::router::fs::ensure_tree_executables(ctx.kit.target(), &tree);
            archive::clean_leftovers(&apps);
            archive::stage_into(&tree, &folder)
        }
        Dest::Tools(names) => {
            let tools = apps.join("tools");
            fs::create_dir_all(&tools)?;
            for name in names {
                let found = find_file(&tree, name)
                    .with_context(|| format!("{name} not found in {}", part.file_name))?;
                let dest = tools.join(name);
                put_file(&found, &dest)?;
                let _ = crate::router::fs::set_executable(&dest);
            }
            Ok(())
        }
        Dest::File { folder, name, from } => {
            let src = crate::fsx::join_rel(&tree, from);
            if !src.is_file() {
                bail!("{from} not found in {}", part.file_name);
            }
            // The component's own folder: that of its line (Claude Code).
            let dir = match own {
                Some(o) if component.folder.as_deref() == Some(*folder) => o.to_path_buf(),
                _ => apps.join(folder),
            };
            fs::create_dir_all(&dir)?;
            let dest = dir.join(name);
            put_file(&src, &dest)?;
            let _ = crate::router::fs::set_executable(&dest);
            Ok(())
        }
        Dest::Inside(name) => {
            let Some(folder) = own else {
                bail!("{} has no folder of its own", component.id);
            };
            let found = find_file(&tree, name)
                .with_context(|| format!("{name} not found in {}", part.file_name))?;
            let dest = folder.join(name);
            put_file(&found, &dest)?;
            let _ = crate::router::fs::set_executable(&dest);
            Ok(())
        }
    }
}

/// One file of the new tree in place of `dest`, always copied: checked
/// against the disk of `dest` first, so the installed one stays when it
/// cannot fit.
pub(crate) fn put_file(new: &Path, dest: &Path) -> Result<()> {
    archive::check_fits(new, dest)?;
    archive::check_space(new, dest)?;
    archive::replace_file(new, dest)
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
#[cfg_attr(not(test), allow(dead_code))]
fn check_key(component: &Component, inside: &[&str], tree: &Path) -> Result<()> {
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
        let path = crate::fsx::join_rel(tree, rel);
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

/// Effective [`crate::kit::Target`] for downloading `component`, honouring
/// x64 emulation on `win32-arm64` when `x64_only` is set.
fn target_of(ctx: &Ctx, component: &Component) -> crate::kit::Target {
    let t = ctx.kit.target();
    if component.is_emulated_on(t) {
        t.with_arch(crate::kit::Arch::X64)
    } else {
        t
    }
}

/// `x64` or `arm64` as used by most download names, honouring `x64Only` on Windows.
fn arch_of(ctx: &Ctx, component: &Component) -> &'static str {
    target_of(ctx, component).arch.as_str()
}

/// `x86_64` or `aarch64`, for Rust-style target names.
fn triple_arch(ctx: &Ctx, component: &Component) -> &'static str {
    target_of(ctx, component).gnu_arch()
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
