//! The path of an installation: the work folder, the download of a
//! release, its installation (or that of another line) and the
//! relocation of the components after a move of the kit.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::kit::Kit;
use crate::net::verify::{self, Expect};
use crate::platform::temp::TempWork;
use crate::setup::Component;

use super::keys::target_of;
use super::parts::install_parts;
use super::{installer, Ctx, Dest, Part, Progress, Release};

/// Free space the PC's temporary folder must have for an installation's
/// download and extraction; the sizes are not known before the download, and
/// the largest components (Android SDK, Rust, TeX Live) stay below this.
pub(super) const SETUP_ROOM: u64 = 4 << 30;

/// The work folder of a component, `setup\<id>` in the PC's temporary folder
/// (D-90; `data\tmp\setup\<id>` without one or with less than `SETUP_ROOM`
/// free there), emptied. It lives as long as the guard: keep it for the
/// whole installation. With the tools sandbox (D-180) it is in the
/// workspace of the sandbox, which runs the programs on its files.
pub fn work_dir(ctx: &Ctx, component: &Component) -> Result<TempWork> {
    let rel = format!("setup/{}", component.id);
    match ctx.guest {
        Some(guest) => {
            let dir = crate::common::fsx::join_rel(guest.work_root(), &rel);
            TempWork::at(&dir).with_context(|| format!("work folder {}", dir.display()))
        }
        None => TempWork::new_with_space(ctx.kit, &rel, SETUP_ROOM)
            .with_context(|| format!("work folder {rel}")),
    }
}

/// A file [`fetch`] copied in place of a download: pre-downloaded for this
/// system (`router/bootstrap.rs`) or added by hand (`setup/downloads.rs`).
pub(super) enum Taken {
    /// Pre-downloaded; `true` when the installation ran the programs of its
    /// system (here or in the tools sandbox): used up once it succeeds.
    Staged(PathBuf, bool),
    ByHand(PathBuf, Expect),
}

impl Taken {
    fn used(self, kit: &Kit) {
        match self {
            Taken::Staged(file, ran) => crate::router::bootstrap::used(kit, &file, ran),
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
pub(super) fn taken(kit: &Kit, file: Taken) {
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
pub(super) fn keeping_taken<T>(kit: &Kit, run: impl FnOnce() -> Result<T>) -> Result<T> {
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
        taken(ctx.kit, Taken::Staged(staged, ctx.runs_target()));
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

pub(super) fn install_now(
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
    // What «Verifica i file» compares with (N-5): the folder as installed.
    let own = if component.per_line() {
        Some(crate::versions::install_dir(
            ctx.kit,
            component,
            &release.version,
        ))
    } else {
        component.folder_path(ctx.kit)
    };
    if let Some(dir) = own {
        crate::setup::integrity::after_install(ctx.kit, component, &dir, true);
    }
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
    for c in crate::setup::Catalog::get().all() {
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
    for c in crate::setup::Catalog::get().all() {
        let Some(inst) = installer(&c.id).filter(|i| i.relocatable(target)) else {
            continue;
        };
        if let Err(e) = inst.relocate(kit, c) {
            crate::common::log::warn(&format!("{}: relocation: {e:#}", c.id));
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

pub(super) fn install_line_now(
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
        crate::setup::integrity::after_install(ctx.kit, component, &target, false);
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
    crate::setup::integrity::after_install(ctx.kit, component, &target, false);
    Ok(line)
}
