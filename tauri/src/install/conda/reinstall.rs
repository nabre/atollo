//! After a move: an environment created again from the kept packages by
//! `micromamba create --offline`, or the reason it goes back to the network.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::list::{check, file_url, Fallback};
use super::{lock_name, locks_dir, pkgs_dir};
use crate::install::native::{mamba_envs, micromamba_path, run};
use crate::install::{installer, work_dir, Ctx, Installer, Progress};
use crate::kit::Kit;
use crate::net::verify::{self, Expect};
use crate::setup::Component;

/// The version, folder and line of `line` of `c` (`None`: the default).
fn located(kit: &Kit, c: &Component, line: Option<&str>) -> Option<(String, PathBuf, String)> {
    match line {
        None => {
            let version = c.installed_version(kit)?;
            let dir = if c.per_line() {
                crate::versions::install_dir(kit, c, &version)
            } else {
                c.folder_path(kit)?
            };
            let line = crate::versions::dir_name(&version);
            Some((version, dir, line))
        }
        Some(line) => {
            let version = crate::versions::saved(kit, c)
                .into_iter()
                .find(|l| l.line == line)?
                .version?;
            Some((
                version,
                crate::versions::saved_dir(kit, c, line),
                line.to_owned(),
            ))
        }
    }
}

/// After a move: the environment of `line` of `component` (`None`: the
/// default folder) created again from the kept packages, without the
/// network; its mark is removed. Returns the version, or `None` when it
/// must be installed from the network as before (the reason in the log and
/// the notes).
pub fn reinstall(
    ctx: &Ctx,
    component: &Component,
    line: Option<&str>,
    progress: &mut dyn Progress,
) -> Option<String> {
    let target = ctx.kit.target();
    let inst = installer(&component.id).filter(|i| !i.relocatable(target))?;
    let (version, dir, line_name) = located(ctx.kit, component, line)?;
    let prefix = inst.conda_prefix(target, &dir)?;
    let started = std::time::Instant::now();
    match restore(
        ctx, inst, component, &version, &dir, &prefix, &line_name, progress,
    ) {
        Ok(Ok(())) => {
            match line {
                None => component.clear_reinstall(ctx.kit),
                Some(l) => component.clear_line_reinstall(ctx.kit, l),
            }
            let text = format!(
                "{} {line_name}: created again from the kept packages, without the network ({} s)",
                component.id,
                started.elapsed().as_secs()
            );
            crate::common::log::info(&text);
            progress.note(&text);
            Some(version)
        }
        Ok(Err(fallback)) => {
            let text = format!(
                "{} {line_name}: installed again from the network: {fallback}",
                component.id
            );
            crate::common::log::warn(&text);
            progress.note(&text);
            None
        }
        Err(e) => {
            let text = format!(
                "{} {line_name}: not created from the kept packages ({e:#}), installed again from the network",
                component.id
            );
            crate::common::log::warn(&text);
            progress.note(&text);
            None
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn restore(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    version: &str,
    dir: &Path,
    prefix: &Path,
    line: &str,
    progress: &mut dyn Progress,
) -> Result<Result<(), Fallback>> {
    let kit = ctx.kit;
    let mm = micromamba_path(kit);
    if !mm.is_file() {
        return Ok(Err(Fallback::NoMicromamba));
    }
    let name = lock_name(&component.id, line);
    let locks = locks_dir(kit);
    let list = fs::read_to_string(locks.join(format!("{name}.txt"))).ok();
    let record = fs::read_to_string(locks.join(format!("{name}.json"))).ok();
    let pkgs = pkgs_dir(kit);
    let archives = match check(
        list.as_deref(),
        record.as_deref(),
        kit.target().conda_subdir(),
        &pkgs,
    ) {
        Ok(a) => a,
        Err(f) => return Ok(Err(f)),
    };
    progress.note(&format!(
        "{} kept packages checked (SHA-256 of the list)",
        archives.len()
    ));
    let guard = work_dir(ctx, component)?;
    let work = guard.path();
    // Copied out of the kit and checked again: micromamba reads the copies,
    // so a file changed on the kit's disk after the check is never used.
    let input = work.join("kept");
    fs::create_dir_all(&input)?;
    let mut explicit = String::from("@EXPLICIT\n");
    for a in &archives {
        let copy = input.join(&a.file);
        fs::copy(pkgs.join(&a.file), &copy).with_context(|| copy.display().to_string())?;
        if !verify::matches(&copy, &Expect::Sha256(a.sha256.clone()))? {
            return Ok(Err(Fallback::Changed(a.file.clone())));
        }
        explicit.push_str(&format!("{}#{}\n", file_url(&copy), a.sha256));
    }
    let list_file = work.join("explicit.txt");
    fs::write(&list_file, explicit)?;
    let envs = mamba_envs(work);
    let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
    // What an interrupted run set aside next to it goes first.
    if let Some(parent) = prefix.parent() {
        crate::net::archive::clean_leftovers(parent);
    }
    let created = crate::net::archive::rebuild_in_place(prefix, || -> Result<()> {
        let prefix_arg = prefix.display().to_string();
        let list_arg = list_file.display().to_string();
        progress.note(&format!("micromamba --offline: {name}"));
        run(
            ctx,
            &mm,
            &[
                "create",
                "--no-rc",
                "--offline",
                "-y",
                "-q",
                "-p",
                &prefix_arg,
                "-f",
                &list_arg,
            ],
            &envs,
            progress,
        )?;
        let _ = fs::remove_dir_all(prefix.join("pkgs"));
        inst.after_conda(ctx, component, dir, progress)?;
        if component.per_line() {
            crate::versions::write_version(dir, version)?;
        }
        Ok(())
    });
    drop(guard);
    created.map(Ok)
}

#[cfg(test)]
#[path = "../../../../test/rust/install/conda/reinstall.rs"]
mod tests;
