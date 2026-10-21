//! Installers made of processes: Rust (rustup), pnpm and yarn (npm), R
//! (micromamba). Port of nabre/dev-kit `_prepare-rust.ps1` and
//! `_languages.ps1:254-429`.
//!
//! Files: `mod` (the programs run with their output in the notes), `rust`
//! (rustup), `pnpm` (pnpm and yarn), `micromamba` (conda-forge versions and
//! environments, shared with Ruby, C/C++, MongoDB and SQLite), `r` (R).

mod micromamba;
mod pnpm;
mod r;
mod rust;

use std::path::Path;
use std::process::Stdio;

use anyhow::{bail, Context, Result};

use crate::install::{Ctx, Progress};
use crate::os::jobs;

pub use micromamba::conda_versions;
/// For the tests of Ruby (`lang/ruby.rs`).
#[cfg(test)]
pub use micromamba::micromamba_create_args;
pub(super) use micromamba::{
    conda_info, mamba_envs, micromamba_create, micromamba_path, micromamba_release,
};
pub use pnpm::Pnpm;
pub use r::R;
pub use rust::Rust;

/// Runs a kit program, sending its output lines to the progress notes.
pub(super) fn run(
    ctx: &Ctx,
    program: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
    progress: &mut dyn Progress,
) -> Result<()> {
    let mut cmd = jobs::kit_command(program, ctx.env);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .with_context(|| program.display().to_string())?;
    for line in String::from_utf8_lossy(&out.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&out.stderr).lines())
    {
        if !line.trim().is_empty() {
            progress.note(line.trim());
        }
    }
    if !out.status.success() {
        bail!(
            "{} failed ({})",
            program.file_name().unwrap_or_default().to_string_lossy(),
            out.status
        );
    }
    Ok(())
}

/// Runs a kit program and returns what it printed on its standard output;
/// its errors go to the progress notes only when it fails.
pub(super) fn run_output(
    ctx: &Ctx,
    program: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
) -> Result<String> {
    let mut cmd = jobs::kit_command(program, ctx.env);
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd
        .output()
        .with_context(|| program.display().to_string())?;
    if !out.status.success() {
        bail!(
            "{} failed ({}): {}",
            program.file_name().unwrap_or_default().to_string_lossy(),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}
