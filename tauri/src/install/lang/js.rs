//! The JavaScript runtimes from GitHub releases: Bun and Deno.

use anyhow::Result;

use super::{github_line, github_stable, release_lines};
use crate::install::{github_single, target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::setup::catalog::Component;

pub struct Bun;

const BUN_REPO: &str = "oven-sh/bun";

/// The Bun zip for `target`; with `baseline`, on x64 only, the build for
/// CPUs without AVX2 (`bun-<os>-x64-baseline.zip`), which the other one
/// stops on with "Illegal instruction" (D-112).
pub(crate) fn bun_asset_for(target: Target, baseline: bool) -> String {
    let os = match target.os {
        Os::Win32 => "windows",
        Os::Darwin => "darwin",
        Os::Linux => "linux",
    };
    let arch = match target.arch {
        Arch::Arm64 => "aarch64",
        Arch::X64 if baseline => "x64-baseline",
        Arch::X64 => "x64",
    };
    format!(r"^bun-{os}-{arch}\.zip$")
}

/// The baseline build of Bun for `target`: an x64 kit prepared on this PC
/// (its CPU is the one Bun runs on) when the CPU has no AVX2. A kit
/// prepared for another system gets the usual build.
fn bun_baseline(target: Target) -> bool {
    target.arch == Arch::X64 && target == Target::detect() && !cpu_has_avx2()
}

#[cfg(target_arch = "x86_64")]
fn cpu_has_avx2() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
fn cpu_has_avx2() -> bool {
    true
}

fn bun_asset(ctx: &Ctx, component: &Component) -> String {
    let target = target_of(ctx, component);
    bun_asset_for(target, bun_baseline(target))
}

impl Installer for Bun {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        github_single(
            ctx,
            BUN_REPO,
            &bun_asset(ctx, component),
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
            BUN_REPO,
            &bun_asset(ctx, component),
        )?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        github_line(
            ctx,
            BUN_REPO,
            &bun_asset(ctx, component),
            line,
            Inner::Single,
        )
    }
}

pub struct Deno;

const DENO_REPO: &str = "denoland/deno";

/// The Deno zip for `target`.
pub(crate) fn deno_asset_for(target: Target) -> String {
    format!(r"^deno-{}\.zip$", target.rust_triple())
}

fn deno_asset(ctx: &Ctx, component: &Component) -> String {
    deno_asset_for(target_of(ctx, component))
}

impl Installer for Deno {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        github_single(
            ctx,
            DENO_REPO,
            &deno_asset(ctx, component),
            Kind::Zip,
            Inner::Root,
        )
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        Ok(release_lines(&github_stable(
            ctx,
            DENO_REPO,
            &deno_asset(ctx, component),
        )?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        github_line(
            ctx,
            DENO_REPO,
            &deno_asset(ctx, component),
            line,
            Inner::Root,
        )
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/js.rs"]
mod tests;
