//! The JavaScript runtimes from GitHub releases: Bun and Deno.

use anyhow::Result;

use super::{github_line, github_stable, release_lines};
use crate::install::{arch_of, github_single, triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::setup::catalog::Component;

pub struct Bun;

const BUN_REPO: &str = "oven-sh/bun";

/// The Bun zip for this kit's platform.
fn bun_asset(ctx: &Ctx, component: &Component) -> String {
    let arch = if arch_of(ctx, component) == "arm64" {
        "aarch64"
    } else {
        "x64"
    };
    format!(r"^bun-windows-{arch}\.zip$")
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

/// The Deno zip for this kit's platform.
fn deno_asset(ctx: &Ctx, component: &Component) -> String {
    let arch = triple_arch(ctx, component);
    format!(r"^deno-{arch}-pc-windows-msvc\.zip$")
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
