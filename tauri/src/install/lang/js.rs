//! The JavaScript runtimes from GitHub releases: Bun and Deno.

use anyhow::Result;

use crate::install::{arch_of, github_single, triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::setup::catalog::Component;

pub struct Bun;

impl Installer for Bun {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = if arch_of(ctx, component) == "arm64" {
            "aarch64"
        } else {
            "x64"
        };
        github_single(
            ctx,
            "oven-sh/bun",
            &format!(r"^bun-windows-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )
    }
}

pub struct Deno;

impl Installer for Deno {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        github_single(
            ctx,
            "denoland/deno",
            &format!(r"^deno-{arch}-pc-windows-msvc\.zip$"),
            Kind::Zip,
            Inner::Root,
        )
    }
}
