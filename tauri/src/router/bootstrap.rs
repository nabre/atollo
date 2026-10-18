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

use std::fs;
use std::path::Path;

use anyhow::Result;

use super::bins::exe_name;
use super::fs::{ensure_tree_executables, set_executable};
use super::target::{can_execute_on_host, effective_target};
use crate::install::{self, Ctx, Dest, Installer, Progress, Release};
use crate::kit::{Kit, Target};
use crate::net::archive;
use crate::net::verify::{self, Expect};
use crate::setup::catalog::Component;

/// Uses a pre-downloaded bootstrap archive from `data/apps/<target>/.bootstrap`
/// when present and matching `expect`. Removes the staged file once consumed on
/// a host that can execute `kit.target()`.
pub fn take(kit: &Kit, file_name: &str, expect: &Expect, dest: &Path) -> Result<bool> {
    let boot_dir = kit.apps().join(".bootstrap");
    let candidate = boot_dir.join(file_name);
    if !candidate.is_file() {
        return Ok(false);
    }
    if let Err(e) = verify::check(&candidate, expect) {
        crate::log::warn(&format!("bootstrap {file_name} discarded: {e}"));
        let _ = fs::remove_file(&candidate);
        return Ok(false);
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(&candidate, dest)?;
    if can_execute_on_host(kit.target()) {
        let _ = fs::remove_file(&candidate);
        let _ = fs::remove_dir(&boot_dir);
    }
    Ok(true)
}

/// Returns `true` when `component` uses a process-based custom installer that
/// cannot run for `ctx.kit.target()` on the current host and should therefore
/// have its bootstrap pre-downloaded and staged on disk.
pub fn should_stage(ctx: &Ctx, inst: &dyn Installer, component: &Component) -> bool {
    let target = effective_target(ctx.kit.target(), component);
    if !inst.custom(target) {
        return false;
    }
    !can_execute_on_host(target) || (component.id == "rust" && ctx.kit.target() != Target::detect())
}

/// Returns `true` when `component`'s bootstrap has been pre-downloaded for `kit`
/// and is waiting for its target OS to complete installation on first startup.
pub fn is_staged(kit: &Kit, component: &Component) -> bool {
    !component.is_installed(kit)
        && component.needs_reinstall(kit)
        && component.installed_version(kit).is_some()
}

/// Downloads and verifies every part of `release` into `data/apps/<target>/.bootstrap`
/// and stages the bootstrap binaries/tools into `data/apps/<target>`, recording
/// the version and `.reinstall` marker.
pub fn stage(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    release: &Release,
    line: Option<(&str, &Path)>,
    progress: &mut dyn Progress,
) -> Result<()> {
    let work = install::work_dir(ctx, component)?;
    let boot_dir = ctx.kit.apps().join(".bootstrap");
    fs::create_dir_all(&boot_dir)?;
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
        install::put_file(&file, &boot_dir.join(&part.file_name))?;
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
