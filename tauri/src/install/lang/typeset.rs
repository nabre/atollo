//! Typesetting: Typst and TinyTeX.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::{github_line_kind, github_stable, release_lines};
use crate::fsx::join_rel;
use crate::install::{github_single, target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::{GhAsset, GhRelease};
use crate::setup::catalog::{translate_rel, Component};

pub struct Typst;

/// The Typst archive pattern and kind for `target`.
pub(crate) fn typst_spec(target: Target) -> (String, Kind) {
    let arch = target.gnu_arch();
    match target.os {
        Os::Win32 => (format!(r"^typst-{arch}-pc-windows-msvc\.zip$"), Kind::Zip),
        Os::Darwin => (
            format!(r"^typst-{arch}-apple-darwin\.tar\.xz$"),
            Kind::TarXz,
        ),
        Os::Linux => (
            format!(r"^typst-{arch}-unknown-linux-musl\.tar\.xz$"),
            Kind::TarXz,
        ),
    }
}

/// Typst's 0.x versions change the syntax: old documents need their own
/// (versioned, D-81; `typst.toml` declares the compiler).
impl Installer for Typst {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let (pat, kind) = typst_spec(target_of(ctx, component));
        github_single(ctx, "typst/typst", &pat, kind, Inner::Single)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let (pat, _) = typst_spec(target_of(ctx, component));
        Ok(release_lines(&github_stable(ctx, "typst/typst", &pat)?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let (pat, kind) = typst_spec(target_of(ctx, component));
        github_line_kind(ctx, "typst/typst", &pat, line, kind, Inner::Single)
    }
}

/// TinyTeX-1: a self-extracting 7z on Windows, `.tar.xz` on macOS and Linux.
pub struct TinyTex;

const TINYTEX_REPO: &str = "rstudio/tinytex-releases";

/// The TinyTeX-1 archives of `target`, the preferred first: the names
/// since v2026.04 (`TinyTeX-1-<os>-<arch>-v<version>.tar.xz`, checked on
/// rstudio/tinytex-releases up to v2026.10), then those before (`.tgz` on
/// macOS, `.tar.gz` on Linux), for a release that still has only those.
pub(crate) fn tinytex_specs(target: Target) -> Vec<(&'static str, Kind, Inner)> {
    match (target.os, target.arch) {
        (Os::Win32, _) => vec![(
            r"^TinyTeX-1-windows-v[\d.]+\.exe$",
            Kind::SevenZip,
            Inner::Named("TinyTeX".into()),
        )],
        (Os::Darwin, _) => vec![
            (
                r"^TinyTeX-1-darwin-v[\d.]+\.tar\.xz$",
                Kind::TarXz,
                Inner::Single,
            ),
            (r"^TinyTeX-1-v[\d.]+\.tgz$", Kind::TarGz, Inner::Single),
        ],
        (Os::Linux, Arch::X64) => vec![
            (
                r"^TinyTeX-1-linux-x86_64-v[\d.]+\.tar\.xz$",
                Kind::TarXz,
                Inner::Single,
            ),
            (
                r"^TinyTeX-1-(?:x86_64-linux-)?v[\d.]+\.tar\.gz$",
                Kind::TarGz,
                Inner::Single,
            ),
        ],
        (Os::Linux, Arch::Arm64) => vec![
            (
                r"^TinyTeX-1-linux-arm64-v[\d.]+\.tar\.xz$",
                Kind::TarXz,
                Inner::Single,
            ),
            (
                r"^TinyTeX-1-arm64-v[\d.]+\.tar\.gz$",
                Kind::TarGz,
                Inner::Single,
            ),
            (
                r"^TinyTeX-1-aarch64-linux-v[\d.]+\.tar\.xz$",
                Kind::TarXz,
                Inner::Single,
            ),
        ],
    }
}

/// The archive of `target` in `release`: the first of [`tinytex_specs`]
/// it has, with its kind and inner folder.
pub(crate) fn tinytex_pick(release: &GhRelease, target: Target) -> Option<(&GhAsset, Kind, Inner)> {
    tinytex_specs(target)
        .into_iter()
        .find_map(|(re, kind, inner)| {
            let re = regex::Regex::new(re).ok()?;
            release.asset(&re).map(|a| (a, kind, inner))
        })
}

/// `tlmgr` relative to the TinyTeX folder on `target`: the catalog's path
/// translated with its `tinytex/` (`bin/x86_64-linux/tlmgr`), then without it.
pub(crate) fn tlmgr_rel(target: Target) -> String {
    let rel = translate_rel(target, "tinytex/bin/windows/tlmgr.bat");
    rel.strip_prefix("tinytex/").unwrap_or(&rel).to_owned()
}

fn tlmgr_path(ctx: &Ctx, root: &Path) -> std::path::PathBuf {
    join_rel(root, &tlmgr_rel(ctx.kit.target()))
}

/// The packages a TinyTeX has (`tlmgr info --list --only-installed`).
fn tlmgr_packages(ctx: &Ctx, root: &Path) -> Option<Vec<String>> {
    if !crate::router::target::can_execute_on_host(ctx.kit.target()) {
        return None;
    }
    let tlmgr = tlmgr_path(ctx, root);
    if !tlmgr.is_file() {
        return None;
    }
    let out = crate::os::jobs::kit_command(&tlmgr, ctx.env)
        .args(["info", "--list", "--only-installed", "--data", "name"])
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::trim)
            // `xetex.windows`: the binaries of a package, which comes with it.
            .filter(|l| !l.is_empty() && !l.contains('.'))
            .map(str::to_owned)
            .collect(),
    )
}

/// The packages of the TinyTeX being replaced: TeX Live accepts updates only
/// from its own year, so the yearly release is a new TinyTeX, and the
/// packages added with tlmgr (or by R Markdown, latexmk…) come back.
fn tinytex_list(ctx: &Ctx) -> std::path::PathBuf {
    join_rel(ctx.kit.data(), "texlive/packages.txt")
}

impl Installer for TinyTex {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let rel = ctx.net.github_latest(TINYTEX_REPO)?;
        let (asset, kind, inner) = tinytex_pick(&rel, target)
            .ok_or_else(|| anyhow!("{TINYTEX_REPO} {}: no TinyTeX-1 for {target}", rel.tag_name))?;
        Ok(Release::single(
            rel.tag_name.trim_start_matches('v').to_owned(),
            asset.browser_download_url.clone(),
            asset.name.clone(),
            Expect::Sha256(ctx.net.release_sha256(&rel, asset)?),
            kind,
            inner,
        ))
    }

    fn before_swap(
        &self,
        ctx: &Ctx,
        _component: &Component,
        _tree: &Path,
        old: &Path,
    ) -> Result<()> {
        if let Some(list) = tlmgr_packages(ctx, old) {
            let file = tinytex_list(ctx);
            fs::create_dir_all(file.parent().unwrap_or(&file))?;
            crate::fsx::write_atomic(&file, list.join("\n").as_bytes())?;
        }
        Ok(())
    }

    /// The packages of the old TinyTeX it does not have, installed again; a
    /// failure (offline) only goes to the log: the list stays for next time.
    fn finish(&self, ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        if !crate::router::target::can_execute_on_host(ctx.kit.target()) {
            return Ok(());
        }
        let Ok(text) = fs::read_to_string(tinytex_list(ctx)) else {
            return Ok(());
        };
        let have = tlmgr_packages(ctx, dest).unwrap_or_default();
        let missing: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|p| !p.is_empty() && !have.iter().any(|h| h == p))
            .collect();
        if missing.is_empty() {
            return Ok(());
        }
        let ok = crate::os::jobs::kit_command(tlmgr_path(ctx, dest), ctx.env)
            .arg("install")
            .args(&missing)
            .stdin(std::process::Stdio::null())
            .output()
            .is_ok_and(|o| o.status.success());
        if ok {
            crate::log::info(&format!("TinyTeX: {} packages back", missing.len()));
        } else {
            crate::log::warn(&format!(
                "TinyTeX: packages not installed again ({}): tlmgr install them by hand",
                missing.join(" ")
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/typeset.rs"]
mod tests;
