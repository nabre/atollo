//! conda-forge through micromamba: the versions of a package, the
//! micromamba that runs on this PC, and the environments it creates where
//! they stay (D-92).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::run;
use crate::fsx::join_rel;
use crate::install::lang::{is_release, version_key};
use crate::install::{fetch, target_of, work_dir, Ctx, Progress, Release};
use crate::kit::{Kit, Target};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::router::target::can_execute_on_host;
use crate::setup::Component;

/// The description of the conda-forge package `pkg`: every file of every
/// version, with its `attrs.subdir`.
pub(in crate::install) fn conda_info(ctx: &Ctx, pkg: &str) -> Result<Value> {
    ctx.net.json(&format!(
        "https://api.anaconda.org/package/conda-forge/{pkg}"
    ))
}

/// The release versions of a conda-forge package (`info` from `conda_info`)
/// built for `subdir` (`win-64`, `osx-arm64`, `linux-64`, ...), newest
/// first; pre-releases (`4.7.0rc1`) are left out.
pub fn conda_versions(info: &Value, subdir: &str) -> Vec<String> {
    let mut out: Vec<String> = info
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|f| f.pointer("/attrs/subdir").and_then(Value::as_str) == Some(subdir))
        .filter_map(|f| f.get("version").and_then(Value::as_str))
        .filter(|v| is_release(v))
        .map(str::to_owned)
        .collect();
    out.sort_by_key(|v| std::cmp::Reverse(version_key(v)));
    out.dedup();
    out
}

/// The micromamba that creates an environment for a kit of `target` on
/// this PC: the kit's own where this PC runs it, else this PC's of the same
/// system. A component in emulation is installed here for a kit this PC
/// cannot run (R x64 for Windows ARM64, prepared from Windows x64): the
/// native micromamba of that kit would not start (os error 216).
pub fn micromamba_runner(target: Target) -> Target {
    micromamba_runner_on(target, Target::detect(), can_execute_on_host(target))
}

/// [`micromamba_runner`] from `host`; `runs` whether `host` runs `target`.
pub fn micromamba_runner_on(target: Target, host: Target, runs: bool) -> Target {
    if runs || host.os != target.os {
        target
    } else {
        host
    }
}

/// `version` of a conda package with the newest micromamba that installs
/// it: the release's one part is micromamba itself (`micromamba_create`),
/// the one of [`micromamba_runner`].
pub(in crate::install) fn micromamba_release(ctx: &Ctx, version: &str) -> Result<Release> {
    let target = micromamba_runner(ctx.kit.target());
    let asset = target.micromamba_asset();
    let mm = ctx.net.github_latest("mamba-org/micromamba-releases")?;
    let base = format!(
        "https://github.com/mamba-org/micromamba-releases/releases/download/{}",
        mm.tag_name
    );
    let sha = first_sha256(&ctx.net.text(&format!("{base}/{asset}.sha256"))?)
        .ok_or_else(|| anyhow!("micromamba: no SHA-256"))?;
    Ok(Release::single(
        // The package only: a new micromamba alone does not rebuild it.
        version.to_owned(),
        format!("{base}/{asset}"),
        format!("micromamba{}", target.exe_suffix()),
        Expect::Sha256(sha),
        crate::net::archive::Kind::File,
        crate::net::archive::Inner::Root,
    ))
}

/// The arguments of `micromamba create` for an environment at `prefix` with
/// `specs` (`git=2.51.0`), from conda-forge only and without any `.condarc`.
/// `platform` is the conda platform of an environment that is not
/// micromamba's own (`win-64` from the native micromamba of Windows ARM64,
/// for a component in emulation); `None` for its own.
pub fn micromamba_create_args(
    prefix: &Path,
    specs: &[&str],
    platform: Option<&str>,
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "create".to_owned(),
        "--no-rc".to_owned(),
        "-y".to_owned(),
        "-q".to_owned(),
        "-p".to_owned(),
        prefix.display().to_string(),
        "-c".to_owned(),
        "conda-forge".to_owned(),
        "--override-channels".to_owned(),
    ];
    if let Some(platform) = platform {
        args.extend(["--platform".to_owned(), platform.to_owned()]);
    }
    args.extend(specs.iter().map(|s| (*s).to_owned()));
    args
}

/// The environment of micromamba with its home, root prefix and package
/// cache (`CONDA_PKGS_DIRS`, which it reads also with `--no-rc`) in the
/// work folder `work`: nothing is written outside the kit and the
/// temporary folder.
pub(in crate::install) fn mamba_envs(work: &Path) -> Vec<(&'static str, PathBuf)> {
    let home = work.join("mamba-home");
    vec![
        ("HOME", home.clone()),
        ("USERPROFILE", home.clone()),
        ("APPDATA", join_rel(&home, "AppData/Roaming")),
        ("LOCALAPPDATA", join_rel(&home, "AppData/Local")),
        ("MAMBA_ROOT_PREFIX", home.join("mamba")),
        ("CONDA_PKGS_DIRS", work.join("pkgs")),
    ]
}

/// `micromamba(.exe)` in the kit's `tools`, the one every environment is
/// made with.
pub(in crate::install) fn micromamba_path(kit: &Kit) -> PathBuf {
    kit.apps()
        .join("tools")
        .join(format!("micromamba{}", kit.os().exe_suffix()))
}

/// Creates the conda environment `prefix` with `specs`, by the micromamba
/// of `release` (`micromamba_release`), put into `tools`. Any old `prefix`
/// goes first. Its home, root prefix and package cache are in the work
/// folder (`mamba_envs`). The environment is created where it stays: conda
/// writes `prefix` into some of its files, so it cannot be moved (D-92).
/// For a component that cannot follow the kit its archives are then kept
/// in the kit with the explicit list of the line (`conda::keep`, D-118).
pub(in crate::install) fn micromamba_create(
    ctx: &Ctx,
    component: &Component,
    release: &Release,
    prefix: &Path,
    specs: &[&str],
    progress: &mut dyn Progress,
) -> Result<()> {
    let part = release
        .parts
        .first()
        .ok_or_else(|| anyhow!("{}: no micromamba to download", component.id))?;
    let guard = work_dir(ctx, component)?;
    let work = guard.path();
    let result = (|| -> Result<()> {
        let mm_new = fetch(
            ctx,
            &part.url,
            &part.file_name,
            &part.expect,
            work,
            progress,
        )?;
        // The kit's own micromamba goes into its `tools`; one for this PC
        // (`micromamba_runner`) is run from the work folder and left there.
        let mm = if micromamba_runner(ctx.kit.target()) == ctx.kit.target() {
            let mm = micromamba_path(ctx.kit);
            if let Some(tools) = mm.parent() {
                fs::create_dir_all(tools)?;
            }
            crate::net::archive::replace_file(&mm_new, &mm)?;
            mm
        } else {
            mm_new
        };
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(m) = fs::metadata(&mm) {
                let mut p = m.permissions();
                p.set_mode(p.mode() | 0o755);
                let _ = fs::set_permissions(&mm, p);
            }
        }
        let envs = mamba_envs(work);
        let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
        progress.note(&format!("micromamba: {}", specs.join(" ")));
        let platform = target_of(ctx, component).conda_subdir();
        let own = micromamba_runner(ctx.kit.target()).conda_subdir();
        let args = micromamba_create_args(prefix, specs, (platform != own).then_some(platform));
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        // The same line again (a patch): the old environment is set aside,
        // not deleted, and comes back if the new one is not created (no
        // network, solver, full disk). What an interrupted run set aside
        // next to it goes first.
        if let Some(parent) = prefix.parent() {
            crate::net::archive::clean_leftovers(parent);
        }
        crate::net::archive::rebuild_in_place(prefix, || {
            run(ctx, &mm, &args, &envs, &mut *progress)?;
            // The package cache is in the work folder; a `pkgs` left in the
            // environment is only downloaded archives.
            let _ = fs::remove_dir_all(prefix.join("pkgs"));
            Ok(())
        })?;
        if crate::install::conda::wanted(ctx, component) {
            crate::install::conda::keep(
                ctx,
                component,
                &release.version,
                &mm,
                &envs,
                prefix,
                &work.join("pkgs"),
                progress,
            );
        }
        Ok(())
    })();
    drop(guard);
    result
}

#[cfg(test)]
#[path = "../../../../test/rust/install/native/micromamba.rs"]
mod tests;
