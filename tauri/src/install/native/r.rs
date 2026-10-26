//! R from conda-forge (OpenBLAS) by micromamba, one folder per line, with
//! its launchers, its Makeconf set to the folder of each line and the CRAN
//! mirror.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};

use super::{conda_info, conda_versions, micromamba_create, micromamba_release};
use crate::common::fsx::{join_rel, write_atomic};
use crate::install::{target_of, Ctx, Installer, Progress, Release};
use crate::kit::{Kit, Target};
use crate::setup::Component;

/// R from conda-forge (OpenBLAS), installed by micromamba in the work
/// folder and moved into the kit; launchers `bin/R(.cmd)`, `bin/Rscript(.cmd)`
/// put R's libraries on the PATH of R only.
pub struct R;

/// `Makeconf` with its `LOCAL_SOFT` lines set to `<prefix>/Library`
/// (forward slashes, as conda writes it); `None` when they already are.
pub(crate) fn relocated_makeconf(text: &str, prefix: &Path) -> Option<String> {
    let library = format!(
        "{}/Library",
        prefix
            .to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches('/')
    );
    let mut changed = false;
    let mut out = String::with_capacity(text.len() + library.len());
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        let ending = &line[body.len()..];
        let value = body
            .strip_prefix("LOCAL_SOFT")
            .map(str::trim_start)
            .and_then(|rest| rest.strip_prefix("?=").or_else(|| rest.strip_prefix('=')));
        match value {
            Some(old) if old.trim() != library => {
                let op = if body.contains("?=") { "?=" } else { "=" };
                out.push_str(&format!("LOCAL_SOFT {op} {library}{ending}"));
                changed = true;
            }
            _ => out.push_str(line),
        }
    }
    changed.then_some(out)
}

/// conda-forge's `bin/R` writes `SED=<prefix>/bin/sed` and uses `${SED}`
/// without quotes everywhere (also `rtags`, through the export): in a folder
/// with a space the shell runs the rest of the path as a command and `R`
/// fails (`Rscript` is a binary and works). Quoting the line is not enough;
/// the system's `sed` instead, POSIX on every Unix, as R's scripts expect
/// (the BSD one on macOS too).
pub(crate) fn plain_r_sed(text: &str) -> Option<String> {
    let mut changed = false;
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        let ending = &line[body.len()..];
        match body.strip_prefix("SED=") {
            Some(value) if value != "sed" => {
                out.push_str(&format!("SED=sed{ending}"));
                changed = true;
            }
            _ => out.push_str(line),
        }
    }
    changed.then_some(out)
}

/// Packages compiled for one minor version of R do not load in another:
/// one folder per line (versioned, D-81), `renv.lock` declares R's version.
impl Installer for R {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let subdir = target_of(ctx, component).conda_subdir();
        let versions = conda_versions(&conda_info(ctx, "r-base")?, subdir);
        let version = versions
            .first()
            .ok_or_else(|| anyhow!("r-base: no version for {subdir}"))?;
        micromamba_release(ctx, version)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let subdir = target_of(ctx, component).conda_subdir();
        let mut lines: Vec<String> = Vec::new();
        for v in conda_versions(&conda_info(ctx, "r-base")?, subdir) {
            if let Some(line) = crate::versions::line_of(&v) {
                if !lines.contains(&line) {
                    lines.push(line);
                }
            }
        }
        Ok(lines)
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let subdir = target_of(ctx, component).conda_subdir();
        let versions = conda_versions(&conda_info(ctx, "r-base")?, subdir);
        let version = versions
            .iter()
            .find(|v| crate::versions::line_of(v).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("R {line} not available for {subdir}"))?;
        micromamba_release(ctx, version)
    }

    fn custom(&self, _target: Target) -> bool {
        true
    }

    /// Off Windows the moved environment loads its libraries and finds its
    /// files at the old folder: installed again (D-92). On Windows R runs
    /// from its launchers and only building packages from source reads the
    /// old folder; reinstalling 1 GB at every new drive letter of a stick
    /// would cost more.
    fn relocatable(&self, target: Target) -> bool {
        target.os.is_windows()
    }

    /// On Windows conda writes the environment's folder into
    /// `lib/R/etc/x64/Makeconf` (`LOCAL_SOFT`, where a package built from
    /// source finds the headers and libraries of `Library`): set again to
    /// the folder of each line, which moves with the kit (BUG-V3-06).
    fn relocate(&self, kit: &Kit, component: &Component) -> Result<()> {
        if !kit.os().is_windows() {
            return Ok(());
        }
        let Some(folder) = component.folder_path(kit) else {
            return Ok(());
        };
        for entry in fs::read_dir(&folder).into_iter().flatten().flatten() {
            let dir = entry.path();
            let makeconf = join_rel(&dir, "lib/R/etc/x64/Makeconf");
            let Ok(text) = fs::read_to_string(&makeconf) else {
                continue;
            };
            if let Some(new) = relocated_makeconf(&text, &dir) {
                write_atomic(&makeconf, new.as_bytes())
                    .with_context(|| makeconf.display().to_string())?;
            }
        }
        Ok(())
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let target = crate::versions::install_dir(ctx.kit, component, &release.version);
        self.install_custom_at(ctx, component, release, &target, progress)
    }

    /// The conda environment is created where it stays: conda writes its
    /// folder into some files (Makeconf, pkgconfig), so a moved one points at
    /// a folder that no longer exists.
    fn install_custom_at(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        target: &Path,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let spec = format!("r-base={}", release.version);
        micromamba_create(
            ctx,
            component,
            release,
            target,
            &[&spec, "libblas=*=*openblas"],
            progress,
        )?;
        self.after_conda(ctx, component, target, progress)
    }

    fn conda_prefix(&self, _target: Target, dir: &Path) -> Option<PathBuf> {
        Some(dir.to_path_buf())
    }

    /// Windows has its own launchers in `bin` (`after_conda`).
    fn conda_programs(&self, target: Target) -> Option<&'static str> {
        (!target.is_windows()).then_some("r-base")
    }

    /// The launchers on Windows, the plain `SED` of `bin/R` and the
    /// launchers of `kit-bin` elsewhere, and the CRAN mirror in
    /// `Rprofile.site`, the
    /// same after `micromamba create` from the network or from the kept
    /// packages (D-118). Makeconf's folder is conda's own here; `relocate`
    /// sets it again on Windows after a move (D-104).
    fn after_conda(
        &self,
        ctx: &Ctx,
        _component: &Component,
        target: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        if ctx.kit.os().is_windows() {
            if !join_rel(target, "lib/R/bin/x64/R.exe").is_file() {
                bail!("R.exe not found after the installation");
            }
            let bin = target.join("bin");
            fs::create_dir_all(&bin)?;
            for exe in ["R", "Rscript"] {
                let cmd = format!(
                        "@echo off\r\nrem Atollo: R with its DLLs (Library\\bin) in the PATH of R only\r\nsetlocal\r\nset \"PATH=%~dp0..\\lib\\R\\bin\\x64;%~dp0..\\Library\\bin;%PATH%\"\r\n\"%~dp0..\\lib\\R\\bin\\x64\\{exe}.exe\" %*\r\nexit /b %ERRORLEVEL%\r\n"
                    );
                write_atomic(&bin.join(format!("{exe}.cmd")), cmd.as_bytes())?;
                let sh = format!(
                        "#!/bin/sh\n# Atollo: R with its DLLs (Library/bin) in the PATH of R only\nd=$(dirname \"$0\")/..\nPATH=\"$d/lib/R/bin/x64:$d/Library/bin:$PATH\" exec \"$d/lib/R/bin/x64/{exe}.exe\" \"$@\"\n"
                    );
                write_atomic(&bin.join(exe), sh.as_bytes())?;
            }
        } else {
            if !join_rel(target, "bin/R").is_file() {
                bail!("bin/R not found after the installation");
            }
            // `bin/R` is a copy of `lib/R/bin/R`, not a link: both.
            for script in ["bin/R", "lib/R/bin/R"] {
                let path = join_rel(target, script);
                let Ok(text) = fs::read_to_string(&path) else {
                    continue;
                };
                if let Some(new) = plain_r_sed(&text) {
                    crate::platform::files::write_executable(&path, new.as_bytes())
                        .with_context(|| path.display().to_string())?;
                }
            }
            // `R` and `Rscript` on the PATH, not the 150 programs of the
            // environment's `bin` (D-125).
            crate::install::conda::write_launchers(target, "r-base")?;
        }
        let site = join_rel(target, "lib/R/etc/Rprofile.site");
        let mut text = fs::read_to_string(&site).unwrap_or_default();
        text.push_str("\n# Atollo: CRAN without asking for a mirror\nlocal({ r <- getOption('repos'); if (is.null(r) || identical(unname(r['CRAN']), '@CRAN@')) options(repos = c(CRAN = 'https://cloud.r-project.org')) })\n");
        fs::write(&site, text)?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/native/r.rs"]
mod tests;
