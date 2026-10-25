//! Cross-target bootstrap pre-downloading and staging (`data/apps/<target>/.bootstrap`).
//!
//! When preparing a custom process-based installer (`rust`, `git`/`r` on Unix
//! via `micromamba`, `android` via `sdkmanager`, `pnpm`) for a target that
//! cannot be executed on the current host, the Router downloads and verifies
//! all bootstrap binaries/archives into `data/apps/<target>/.bootstrap` and the
//! component's target directory, records the version, and writes `.reinstall`.
//! On first startup on that target OS, `setup::task::outdated_app` triggers the
//! installer and [`take`] consumes the pre-downloaded files without hitting the
//! network again.
//!
//! Each component has its own folder in `.bootstrap` (`<id>`, `<id>@<line>`
//! for a saved line), so removing it knows which files are its own; the files
//! an older version put at the top of `.bootstrap` are still used.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::bins::exe_name;
use super::target::{can_execute_on_host, effective_target};
use crate::install::{self, Ctx, Dest, Installer, Progress, Release};
use crate::kit::{Kit, Target};
use crate::net::archive::{self, InUse};
use crate::net::verify::{self, Expect};
use crate::platform::files::{ensure_tree_executables, set_executable};
use crate::setup::{Catalog, Component};

/// `data/apps/<target>/.bootstrap`: what is pre-downloaded for that system.
fn boot_dir(kit: &Kit) -> PathBuf {
    kit.apps().join(".bootstrap")
}

/// The folder of the files pre-downloaded for component `id` (`<id>`), or
/// for its saved `line` (`<id>@<line>`). Two components that download the
/// same file (micromamba for Git and R) keep a copy each: removing one never
/// takes the other's.
pub fn staged_dir(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    match line {
        Some(line) => boot_dir(kit).join(format!("{id}@{line}")),
        None => boot_dir(kit).join(id),
    }
}

/// The folders of `id` in `.bootstrap`: its own and those of its saved lines.
fn staged_dirs(kit: &Kit, id: &str) -> Vec<PathBuf> {
    let prefix = format!("{id}@");
    fs::read_dir(boot_dir(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            name == id || name.starts_with(&prefix)
        })
        .map(|e| e.path())
        .collect()
}

/// Some file is pre-downloaded for `id` on `kit`'s system, also when
/// nothing else of it is on disk (its folder stays empty until the first
/// start there): a component the run can still remove.
pub fn has_staged(kit: &Kit, id: &str) -> bool {
    staged_dirs(kit, id)
        .iter()
        .any(|d| fs::read_dir(d).is_ok_and(|mut e| e.next().is_some()))
}

/// Uses a pre-downloaded bootstrap archive from `data/apps/<target>/.bootstrap`
/// when present and matching `expect`: at the top (an older version's
/// layout), then in any component's folder, since the download does not say
/// for which component it is. Returns the staged file copied, which
/// [`used`] removes once the installation succeeds.
pub fn take(kit: &Kit, file_name: &str, expect: &Expect, dest: &Path) -> Result<Option<PathBuf>> {
    let top = boot_dir(kit);
    let mut candidates = vec![top.join(file_name)];
    candidates.extend(
        fs::read_dir(&top)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
            .map(|e| e.path().join(file_name)),
    );
    // A file of the same name in another component's folder may be another
    // version it still waits for: skipped, not deleted.
    let Some(candidate) = candidates
        .into_iter()
        .find(|c| c.is_file() && verify::matches(c, expect).unwrap_or(false))
    else {
        return Ok(None);
    };
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&candidate, dest)?;
    Ok(Some(candidate))
}

/// The installation that took the staged `file` ([`take`]) succeeded: on a
/// host that can execute `kit.target()` it is consumed, with its folder
/// and `.bootstrap` once empty. Elsewhere (staging for another system) it
/// stays for the first start there.
pub fn used(kit: &Kit, file: &Path) {
    if !can_execute_on_host(kit.target()) {
        return;
    }
    let top = boot_dir(kit);
    let _ = fs::remove_file(file);
    if let Some(folder) = file.parent().filter(|p| *p != top) {
        let _ = fs::remove_dir(folder);
    }
    let _ = fs::remove_dir(&top);
}

/// Removes what was pre-downloaded for `c` on `kit`'s system, its saved
/// lines included, and `.bootstrap` once empty. The files at the top of
/// `.bootstrap` (an older version's layout, whose owner is not known) go
/// with the last component staged there. A folder or file in use stops
/// here (`InUse`), as for the component's own folder.
pub fn remove(kit: &Kit, c: &Component) -> Result<()> {
    for dir in staged_dirs(kit, &c.id) {
        archive::remove_dir(&dir)?;
    }
    let others_staged = Catalog::get()
        .all()
        .iter()
        .filter(|o| o.id != c.id)
        .any(|o| is_staged(kit, o) || has_staged(kit, &o.id));
    if !others_staged {
        for entry in fs::read_dir(boot_dir(kit)).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_file() {
                fs::remove_file(&path).map_err(|_| InUse(path.clone()))?;
            }
        }
    }
    let _ = fs::remove_dir(boot_dir(kit));
    Ok(())
}

/// Removes what was pre-downloaded for one line of `id` on `kit`'s system
/// (the line removed from the Tools page): saved `line`, or with `None` the
/// default one; then `.bootstrap` once empty.
pub fn remove_line(kit: &Kit, id: &str, line: Option<&str>) -> Result<()> {
    archive::remove_dir(&staged_dir(kit, id, line))?;
    let _ = fs::remove_dir(boot_dir(kit));
    Ok(())
}

/// Returns `true` when `component` uses a process-based custom installer that
/// cannot run for `ctx.kit.target()` on the current host and should therefore
/// have its bootstrap pre-downloaded and staged on disk.
pub fn should_stage(ctx: &Ctx, inst: &dyn Installer, component: &Component) -> bool {
    stages_for(inst, component, ctx.kit.target())
}

/// [`should_stage`] for a kit of `target`: preparing `component` there
/// from this host pre-downloads its bootstrap, and the installation
/// finishes on the first start on that system (the grid's "completes on
/// first start").
pub fn stages_for(inst: &dyn Installer, component: &Component, target: Target) -> bool {
    let effective = effective_target(target, component);
    if !inst.custom(effective) {
        return false;
    }
    // Android's sdkmanager runs with the kit's own Java, built for `target`:
    // the x64 SDK of Windows ARM64 (emulated) still waits for that system.
    !can_execute_on_host(effective)
        || (component.id == "rust" && target != Target::detect())
        || (component.id == "android" && !can_execute_on_host(target))
}

/// Returns `true` when `component`'s bootstrap has been pre-downloaded for `kit`
/// and is waiting for its target OS to complete installation on first startup.
pub fn is_staged(kit: &Kit, component: &Component) -> bool {
    !component.is_installed(kit)
        && component.needs_reinstall(kit)
        && component.installed_version(kit).is_some()
}

/// Downloads and verifies every part of `release` into the component's folder
/// of `data/apps/<target>/.bootstrap` ([`staged_dir`]) and stages the
/// bootstrap binaries/tools into `data/apps/<target>`, recording the version
/// and `.reinstall` marker.
pub fn stage(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    release: &Release,
    line: Option<(&str, &Path)>,
    progress: &mut dyn Progress,
) -> Result<()> {
    let work = install::work_dir(ctx, component)?;
    let staged = staged_dir(ctx.kit, &component.id, line.map(|(l, _)| l));
    fs::create_dir_all(&staged)?;
    for (i, part) in release.parts.iter().enumerate() {
        let step_work = work.path().join(i.to_string());
        fs::create_dir_all(&step_work)?;
        let file = install::fetch(
            ctx,
            &part.url,
            &part.file_name,
            &part.expect,
            &step_work,
            progress,
        )?;
        install::put_file(&file, &staged.join(&part.file_name))?;
        if component.id == "rust" {
            let rust_dir = ctx.kit.apps().join("rust");
            fs::create_dir_all(&rust_dir)?;
            let init = rust_dir.join(exe_name(ctx.kit.os(), "rustup-init"));
            install::put_file(&file, &init)?;
            let _ = set_executable(&init);
        } else if matches!(part.dest, Dest::Tools(_)) {
            install::install_part(
                ctx,
                inst,
                component,
                part,
                None,
                &work.path().join(format!("tools-{i}")),
                progress,
            )?;
        } else if component.id == "android" {
            let extracted = work.path().join(format!("android-{i}"));
            archive::extract(&file, part.kind, &extracted)?;
            let tree = archive::pick_inner(&extracted, &part.inner)?;
            let latest = ctx
                .kit
                .apps()
                .join("android")
                .join("cmdline-tools")
                .join("latest");
            if let Some(parent) = latest.parent() {
                fs::create_dir_all(parent)?;
            }
            archive::stage_into(&tree, &latest)?;
            ensure_tree_executables(ctx.kit.target(), &latest);
        }
    }
    drop(work);
    // What an earlier release staged here (another version's file) waits
    // for nothing any more.
    for entry in fs::read_dir(&staged).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if !release.parts.iter().any(|p| p.file_name == name) {
            let _ = fs::remove_file(entry.path());
        }
    }
    let marker = format!("bootstrap:{}", ctx.kit.target());
    match line {
        Some((line_str, target_dir)) => {
            fs::create_dir_all(target_dir)?;
            crate::versions::write_version(target_dir, &release.version)?;
            component.mark_line_reinstall(ctx.kit, line_str, &marker)?;
        }
        None => {
            if component.per_line() {
                let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
                fs::create_dir_all(&dir)?;
                crate::versions::write_version(&dir, &release.version)?;
            } else if let Some(folder) = component.folder_path(ctx.kit) {
                fs::create_dir_all(&folder)?;
            }
            component.save_version(ctx.kit, &release.version)?;
            component.mark_reinstall(ctx.kit, &marker)?;
        }
    }
    progress.note(&format!(
        "bootstrap pre-downloaded for {}",
        ctx.kit.target()
    ));
    Ok(())
}
