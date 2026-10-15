//! The C and C++ toolchain: CMake (with Ninja) and llvm-mingw.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::install::{
    arch_of, github_single, pattern, triple_arch, Ctx, Dest, Installer, Part, Release,
};
use crate::net::archive::{Inner, Kind};
use crate::net::github_digest;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// CMake, with Ninja in `tools\`.
pub struct Cmake;

impl Installer for Cmake {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arm = arch_of(ctx, component) == "arm64";
        let arch = if arm { "arm64" } else { "x86_64" };
        let mut release = github_single(
            ctx,
            "Kitware/CMake",
            &format!(r"^cmake-[\d.]+-windows-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )?;
        let ninja = ctx.net.github_latest("ninja-build/ninja")?;
        let name = if arm {
            "ninja-winarm64.zip"
        } else {
            "ninja-win.zip"
        };
        let asset = ninja
            .assets
            .iter()
            .find(|a| a.name == name)
            .ok_or_else(|| anyhow!("no {name}"))?;
        release.version = format!(
            "{} (Ninja {})",
            release.version,
            ninja.tag_name.trim_start_matches('v')
        );
        release.parts.push(Part {
            url: asset.browser_download_url.clone(),
            file_name: asset.name.clone(),
            expect: Expect::Sha256(github_digest(asset)?),
            kind: Kind::Zip,
            inner: Inner::Root,
            dest: Dest::Tools(vec!["ninja.exe"]),
            sign: None,
        });
        Ok(release)
    }
}

/// C and C++: llvm-mingw (UCRT). The cross `*-w64-mingw32-gcc/g++` are
/// removed: on the PATH they would make Rust's `gnu` toolchain use them as
/// linker and fail (`_languages.ps1:302-316`).
pub struct Cc;

impl Installer for Cc {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        github_single(
            ctx,
            "mstorsjo/llvm-mingw",
            &format!(r"^llvm-mingw-\d+-ucrt-{arch}\.zip$"),
            Kind::Zip,
            Inner::Single,
        )
    }

    fn finish(&self, _ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let re = pattern(r"(?i)^\w+-w64-mingw32-(gcc|g\+\+)\.exe$")?;
        for entry in fs::read_dir(dest.join("bin"))?.flatten() {
            if re.is_match(&entry.file_name().to_string_lossy()) {
                fs::remove_file(entry.path())?;
            }
        }
        make_launchers(&dest.join("bin"))
    }
}

/// `make` is mingw32-make in llvm-mingw (make.exe is only in busybox, not
/// on the PATH): Makefiles, Ruby's native gems and R's source packages
/// call `make`. Also at start, for a C/C++ installed before (D-81).
pub fn make_launchers(bin: &Path) -> Result<()> {
    if !bin.join("mingw32-make.exe").is_file() {
        return Ok(());
    }
    let files: [(&str, &[u8]); 2] = [
        ("make.cmd", b"@\"%~dp0mingw32-make.exe\" %*\r\n"),
        (
            "make",
            b"#!/bin/sh\nexec \"$(dirname \"$0\")/mingw32-make.exe\" \"$@\"\n",
        ),
    ];
    for (name, content) in files {
        let path = bin.join(name);
        if fs::read(&path).ok().as_deref() != Some(content) {
            crate::fsx::write_atomic(&path, content)?;
        }
    }
    Ok(())
}
