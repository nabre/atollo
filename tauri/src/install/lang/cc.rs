//! The C and C++ toolchain: CMake (with Ninja) and llvm-mingw.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::install::{
    github_single, pattern, target_of, triple_arch, Ctx, Dest, Installer, Part, Release,
};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::{github_digest, verifiable, GhAsset, GhRelease};
use crate::setup::catalog::Component;

use super::version_key;

/// CMake, with Ninja in `tools\`.
pub struct Cmake;

pub(crate) fn cmake_spec(
    target: Target,
) -> (&'static str, Kind, Inner, &'static str, &'static str) {
    match (target.os, target.arch) {
        (Os::Win32, Arch::X64) => (
            r"^cmake-[\d.]+-windows-x86_64\.zip$",
            Kind::Zip,
            Inner::Single,
            "ninja-win.zip",
            "ninja.exe",
        ),
        (Os::Win32, Arch::Arm64) => (
            r"^cmake-[\d.]+-windows-arm64\.zip$",
            Kind::Zip,
            Inner::Single,
            "ninja-winarm64.zip",
            "ninja.exe",
        ),
        (Os::Darwin, _) => (
            r"^cmake-[\d.]+-macos(10\.10)?-universal\.tar\.gz$",
            Kind::TarGz,
            Inner::SingleThen("CMake.app/Contents"),
            "ninja-mac.zip",
            "ninja",
        ),
        (Os::Linux, Arch::X64) => (
            r"^cmake-[\d.]+-linux-x86_64\.tar\.gz$",
            Kind::TarGz,
            Inner::Single,
            "ninja-linux.zip",
            "ninja",
        ),
        (Os::Linux, Arch::Arm64) => (
            r"^cmake-[\d.]+-linux-aarch64\.tar\.gz$",
            Kind::TarGz,
            Inner::Single,
            "ninja-linux-aarch64.zip",
            "ninja",
        ),
    }
}

impl Installer for Cmake {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let (cmake_pat, cmake_kind, cmake_inner, ninja_zip, ninja_bin) = cmake_spec(target);
        let mut release = github_single(ctx, "Kitware/CMake", cmake_pat, cmake_kind, cmake_inner)?;
        let ninja = ctx.net.github_latest("ninja-build/ninja")?;
        let asset = ninja
            .assets
            .iter()
            .find(|a| a.name == ninja_zip)
            .ok_or_else(|| anyhow!("no {ninja_zip}"))?;
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
            dest: Dest::Tools(vec![ninja_bin]),
            sign: None,
        });
        Ok(release)
    }
}

/// C and C++: llvm-mingw (UCRT) on Windows, LLVM/Clang on macOS and Linux.
/// On Windows the cross `*-w64-mingw32-gcc/g++` are removed: on the PATH they
/// would make Rust's `gnu` toolchain use them as linker and fail (`_languages.ps1:302-316`).
pub struct Cc;

impl Installer for Cc {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        match (target.os, target.arch) {
            (Os::Win32, _) => {
                let arch = triple_arch(ctx, component);
                github_single(
                    ctx,
                    "mstorsjo/llvm-mingw",
                    &format!(r"^llvm-mingw-\d+-ucrt-{arch}\.zip$"),
                    Kind::Zip,
                    Inner::Single,
                )
            }
            _ => llvm_release(ctx, llvm_asset(target)),
        }
    }

    fn finish(&self, _ctx: &Ctx, _component: &Component, dest: &Path) -> Result<()> {
        let bin = dest.join("bin");
        let re = pattern(r"(?i)^\w+-w64-mingw32-(gcc|g\+\+)\.exe$")?;
        for entry in fs::read_dir(&bin)?.flatten() {
            if re.is_match(&entry.file_name().to_string_lossy()) {
                fs::remove_file(entry.path())?;
            }
        }
        make_launchers(&bin)?;
        if bin.join("clang").is_file() {
            for (alias, target_bin) in [("gcc", "clang"), ("g++", "clang++")] {
                let p = bin.join(alias);
                if !p.exists() {
                    let script =
                        format!("#!/bin/sh\nexec \"$(dirname \"$0\")/{target_bin}\" \"$@\"\n");
                    crate::fsx::write_atomic(&p, script.as_bytes())?;
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        let _ = fs::set_permissions(&p, fs::Permissions::from_mode(0o755));
                    }
                }
            }
        }
        Ok(())
    }
}

const LLVM_REPO: &str = "llvm/llvm-project";

/// The LLVM/Clang archive for `target` among the assets of a release of
/// llvm/llvm-project (macOS and Linux; Windows takes llvm-mingw). Clang
/// only: the headers and libraries of the C library come from the system
/// (Xcode Command Line Tools on macOS, libc headers and `make` on Linux).
pub(crate) fn llvm_asset(target: Target) -> &'static str {
    match (target.os, target.arch) {
        (Os::Darwin, Arch::Arm64) => {
            r"^(?:LLVM-[\d.]+-macOS-ARM64|clang\+llvm-[\d.]+-arm64-apple-darwin[^/]*)\.tar\.xz$"
        }
        (Os::Darwin, Arch::X64) => {
            r"^(?:LLVM-[\d.]+-macOS-X64|clang\+llvm-[\d.]+-x86_64-apple-darwin[^/]*)\.tar\.xz$"
        }
        (_, Arch::X64) => {
            r"^(?:LLVM-[\d.]+-Linux-X64|clang\+llvm-[\d.]+-x86_64-linux-gnu[^/]*)\.tar\.xz$"
        }
        (_, Arch::Arm64) => {
            r"^(?:LLVM-[\d.]+-Linux-ARM64|clang\+llvm-[\d.]+-aarch64-linux-gnu[^/]*)\.tar\.xz$"
        }
    }
}

/// The newest stable release with a checkable archive matching `re`.
/// llvm/llvm-project does not build every system for every release: some
/// have no macOS ARM64 archive, none after 20.1.7 has a macOS x64 one
/// (BUG-V3-05).
pub(crate) fn newest_llvm<'a>(
    releases: &'a [GhRelease],
    re: &regex::Regex,
) -> Option<(&'a GhRelease, &'a GhAsset)> {
    releases
        .iter()
        .filter(|r| !r.prerelease)
        .filter_map(|r| r.asset(re).filter(|a| verifiable(r, a)).map(|a| (r, a)))
        .max_by_key(|(r, _)| version_key(&r.tag_name))
}

/// The latest release of LLVM when it has the archive of `asset`, else the
/// newest of the last 100 that has it.
fn llvm_release(ctx: &Ctx, asset: &str) -> Result<Release> {
    let re = pattern(asset)?;
    let latest = ctx.net.github_latest(LLVM_REPO)?;
    let older: Vec<GhRelease>;
    let (rel, file) = match latest.asset(&re) {
        Some(file) => (&latest, file),
        None => {
            older = ctx.net.json(&format!(
                "https://api.github.com/repos/{LLVM_REPO}/releases?per_page=100"
            ))?;
            newest_llvm(&older, &re)
                .ok_or_else(|| anyhow!("{LLVM_REPO}: no release with an asset matching {asset}"))?
        }
    };
    Ok(Release::single(
        rel.tag_name.clone(),
        file.browser_download_url.clone(),
        file.name.clone(),
        Expect::Sha256(ctx.net.release_sha256(rel, file)?),
        Kind::TarXz,
        Inner::Single,
    ))
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

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/cc.rs"]
mod tests;
