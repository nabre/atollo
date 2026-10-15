//! Typesetting: Typst and TinyTeX.

use std::fs;
use std::path::Path;

use anyhow::Result;

use super::{github_line, github_stable, release_lines};
use crate::install::{github_single, triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::setup::catalog::Component;

pub struct Typst;

/// The Typst zip for this kit's platform.
fn typst_asset(ctx: &Ctx, component: &Component) -> String {
    let arch = triple_arch(ctx, component);
    format!(r"^typst-{arch}-pc-windows-msvc\.zip$")
}

/// Typst's 0.x versions change the syntax: old documents need their own
/// (versioned, D-81; `typst.toml` declares the compiler).
impl Installer for Typst {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
            Kind::Zip,
            Inner::Single,
        )
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        Ok(release_lines(&github_stable(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
        )?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        github_line(
            ctx,
            "typst/typst",
            &typst_asset(ctx, component),
            line,
            Inner::Single,
        )
    }
}

/// TinyTeX-1 (x64 build, also on ARM64): a self-extracting 7z, read directly.
pub struct TinyTex;

/// The packages a TinyTeX has (`tlmgr info --list --only-installed`).
fn tlmgr_packages(ctx: &Ctx, root: &Path) -> Option<Vec<String>> {
    let tlmgr = root.join(r"bin\windows\tlmgr.bat");
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
    ctx.kit.data().join(r"texlive\packages.txt")
}

impl Installer for TinyTex {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        github_single(
            ctx,
            "rstudio/tinytex-releases",
            r"^TinyTeX-1-windows-v[\d.]+\.exe$",
            Kind::SevenZip,
            Inner::Named("TinyTeX".into()),
        )
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
        let ok = crate::os::jobs::kit_command(dest.join(r"bin\windows\tlmgr.bat"), ctx.env)
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
