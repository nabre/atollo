//! Installers made of processes: Rust (rustup), pnpm and yarn (npm), R
//! (micromamba).
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

/// A kit program with `args` and `envs` ([`Ctx::command`]: on this PC or
/// in the tools sandbox).
fn command(
    ctx: &Ctx,
    program: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
) -> Result<std::process::Command> {
    let args: Vec<String> = args.iter().map(|a| (*a).to_owned()).collect();
    let envs: Vec<(String, String)> = envs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.display().to_string()))
        .collect();
    ctx.command(program, &args, &envs, None, false)
}

/// Runs a kit program, sending its output lines to the progress notes.
pub(super) fn run(
    ctx: &Ctx,
    program: &Path,
    args: &[&str],
    envs: &[(&str, &Path)],
    progress: &mut dyn Progress,
) -> Result<()> {
    let mut cmd = command(ctx, program, args, envs)?;
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
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
    let mut cmd = command(ctx, program, args, envs)?;
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
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
