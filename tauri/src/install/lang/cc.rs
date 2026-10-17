//! The C and C++ toolchain: CMake (with Ninja), llvm-mingw on Windows and
//! `zig cc` on macOS and Linux.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};

use crate::fsx::join_rel;
use crate::install::native::{conda_info, conda_versions, micromamba_create, micromamba_release};
use crate::install::{
    github_single, pattern, target_of, triple_arch, Ctx, Dest, Installer, Part, Progress, Release,
};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::github_digest;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

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

/// C and C++: llvm-mingw (UCRT) on Windows; on macOS and Linux shims over
/// the kit's Zig (`zig cc`, `zig c++`, `zig ar`, `zig ranlib`) with GNU make
/// from conda-forge (WP-2.3). Zig brings the headers and libraries of the C
/// library for every target, so neither `build-essential` nor Xcode Command
/// Line Tools are needed; only the Apple frameworks need those on macOS.
/// On Windows the cross `*-w64-mingw32-gcc/g++` are removed: on the PATH they
/// would make Rust's `gnu` toolchain use them as linker and fail (`_languages.ps1:302-316`).
pub struct Cc;

/// The version saved for C/C++ on macOS and Linux: the shims follow the
/// kit's Zig, so only make has a version of its own.
pub(crate) fn unix_version(make: &str) -> String {
    format!("zig cc, make {make}")
}

/// The version of make in a version saved by [`unix_version`].
pub(crate) fn make_version(version: &str) -> Option<&str> {
    version
        .strip_prefix("zig cc, make ")
        .filter(|v| super::is_release(v))
}

/// A compiler shim: `zig <cmd>` with the target triples of Clang and Rust
/// (`--target=x86_64-unknown-linux-gnu`, `-target arm64-apple-macosx14.0`,
/// as cc-rs and rustc pass them) turned into Zig's (`x86_64-linux-gnu`,
/// `aarch64-macos`), which Zig requires. Zig is the kit's, from the PATH:
/// the shims keep working when the kit moves or another version of Zig is
/// chosen for a project. Without one on the PATH (`CC` called from an
/// environment built before Zig was installed, or outside the kit's
/// shells) a line of the kit's Zig next to `cc/` is taken ([`FIND_ZIG`]).
const COMPILER_SHIM: &str = r#"#!/bin/sh
# Dev-Kit: @CMD@ of the kit is `zig @CMD@` (the kit's Zig on the PATH).
# Target triples of Clang and Rust (x86_64-unknown-linux-gnu,
# arm64-apple-macosx14.0) become Zig's (x86_64-linux-gnu, aarch64-macos).
zt() {
  arch=${1%%-*}
  [ "$arch" = arm64 ] && arch=aarch64
  case $1 in
    *-linux-*) echo "$arch-linux-${1##*-linux-}" ;;
    *-linux) echo "$arch-linux" ;;
    *-apple-*) echo "$arch-macos" ;;
    *) echo "$1" ;;
  esac
}
n=$#
t=
while [ "$n" -gt 0 ]; do
  a=$1
  shift
  n=$((n - 1))
  if [ -n "$t" ]; then
    set -- "$@" "$(zt "$a")"
    t=
    continue
  fi
  case $a in
    --target=*) set -- "$@" -target "$(zt "${a#--target=}")" ;;
    -target | --target) set -- "$@" -target; t=1 ;;
    *) set -- "$@" "$a" ;;
  esac
done
@ZIG@exec zig @CMD@ "$@"
"#;

/// A tool shim: `zig ar`, `zig ranlib`.
const TOOL_SHIM: &str = r#"#!/bin/sh
# Dev-Kit: @CMD@ of the kit is `zig @CMD@` (the kit's Zig on the PATH).
@ZIG@exec zig @CMD@ "$@"
"#;

/// The lines of the shims that find Zig: the PATH's, else the kit's newest
/// line (`data/apps/<target>/zig/<line>/zig`, two folders above `cc/bin`;
/// the line the kit uses, versions/choice.rs `kit_choice`), compared as
/// numbers (0.14 after 0.9), else a Zig of an older layout (`zig/zig`),
/// else a clear error. Shell built-ins only, so it works with any PATH:
/// `${0%/*}` rather than `dirname`, no `sort`.
const FIND_ZIG: &str = r#"if ! command -v zig >/dev/null 2>&1; then
  d=${0%/*}/../../zig
  best=
  bm=-1
  bn=-1
  for z in "$d"/*/zig; do
    [ -x "$z" ] || continue
    l=${z%/zig}
    l=${l##*/}
    m=${l%%.*}
    n=${l#*.}
    n=${n%%.*}
    case $m.$n in *[!0-9.]* | .* | *.) continue ;; esac
    if [ "$m" -gt "$bm" ] || { [ "$m" -eq "$bm" ] && [ "$n" -gt "$bn" ]; }; then
      best=${z%/zig}
      bm=$m
      bn=$n
    fi
  done
  [ -z "$best" ] && [ -x "$d/zig" ] && best=$d
  [ -n "$best" ] && PATH="$best:$PATH"
  command -v zig >/dev/null 2>&1 || { echo "Dev-Kit: @CMD@ needs Zig, install it from Tools" >&2; exit 127; }
fi
"#;

/// GNU make of the conda environment in `cc/make`, next to `cc/bin`.
const MAKE_SHIM: &str = r#"#!/bin/sh
# Dev-Kit: GNU make from conda-forge, in the folder of C and C++.
exec "$(dirname "$0")/../make/bin/make" "$@"
"#;

/// The scripts of `cc/bin` on macOS and Linux, by file name: the compilers
/// under the names build systems look for, `ar`, `ranlib` and `make`.
pub(crate) fn zig_shims() -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    for (names, cmd) in [
        (["cc", "gcc", "clang"], "cc"),
        (["c++", "g++", "clang++"], "c++"),
    ] {
        for name in names {
            out.push((
                name,
                COMPILER_SHIM
                    .replace("@ZIG@", FIND_ZIG)
                    .replace("@CMD@", cmd),
            ));
        }
    }
    for tool in ["ar", "ranlib"] {
        out.push((
            tool,
            TOOL_SHIM.replace("@ZIG@", FIND_ZIG).replace("@CMD@", tool),
        ));
    }
    out.push(("make", MAKE_SHIM.to_owned()));
    out
}

/// Writes [`zig_shims`] into `bin` (LF, 755).
pub(crate) fn write_zig_shims(bin: &Path) -> Result<()> {
    fs::create_dir_all(bin)?;
    for (name, script) in zig_shims() {
        let path = bin.join(name);
        crate::fsx::write_atomic(&path, script.as_bytes())?;
        crate::router::fs::set_executable(&path)?;
    }
    Ok(())
}

/// What to say about Xcode Command Line Tools on `target`, given the
/// folder `xcode-select -p` printed (`None`: not installed). Only macOS
/// needs them, and only for the Apple frameworks (T2.3.4).
pub(crate) fn xcode_note(target: Target, selected: Option<&str>) -> Option<&'static str> {
    if target.os != Os::Darwin || selected.is_some_and(|s| !s.trim().is_empty()) {
        return None;
    }
    Some("Xcode Command Line Tools not found: C and C++ work, programs that use the Apple frameworks need them (xcode-select --install, as an administrator)")
}

/// The folder of the active developer tools (`xcode-select -p`), on a Mac.
#[cfg(target_os = "macos")]
pub(crate) fn xcode_selected() -> Option<String> {
    let out = std::process::Command::new("/usr/bin/xcode-select")
        .arg("-p")
        .stdin(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let path = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    Path::new(&path).is_dir().then_some(path)
}

#[cfg(not(target_os = "macos"))]
pub(crate) fn xcode_selected() -> Option<String> {
    None
}

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
            _ => {
                let subdir = target.conda_subdir();
                let make = conda_versions(&conda_info(ctx, "make")?, subdir)
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow!("make: no version for {subdir}"))?;
                let mut release = micromamba_release(ctx, &make)?;
                release.version = unix_version(&make);
                Ok(release)
            }
        }
    }

    /// Shims and a conda environment for make on macOS and Linux.
    fn custom(&self, target: Target) -> bool {
        !target.is_windows()
    }

    /// llvm-mingw moves with the kit; make's conda environment has its
    /// folder written inside (D-92), so off Windows the component is
    /// installed again when the kit moves (the shims alone would not care).
    fn relocatable(&self, target: Target) -> bool {
        target.is_windows()
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let target = ctx.kit.target();
        if target.is_windows() {
            bail!("no custom installer for C and C++ on Windows");
        }
        let folder = component
            .folder_path(ctx.kit)
            .ok_or_else(|| anyhow!("cc has no folder"))?;
        let make = make_version(&release.version)
            .ok_or_else(|| anyhow!("cc: version {}", release.version))?;
        // A whole new folder: the LLVM of an older kit goes too.
        if folder.exists() {
            crate::net::archive::remove_dir(&folder)?;
        }
        let spec = format!("make={make}");
        micromamba_create(
            ctx,
            component,
            release,
            &folder.join("make"),
            &[&spec],
            progress,
        )?;
        if !join_rel(&folder, "make/bin/make").is_file() {
            bail!("make not found after the installation");
        }
        write_zig_shims(&folder.join("bin"))?;
        if !crate::setup::catalog::kit_path(ctx.kit, "zig/zig.exe").is_file() {
            progress.note(
                "Zig is not installed: cc, c++, ar and ranlib need it, and stay off the PATH until it is",
            );
        }
        // Only a Mac can say: a darwin kit prepared on another system would
        // always hear that the tools are missing.
        if cfg!(target_os = "macos") {
            if let Some(note) = xcode_note(target, xcode_selected().as_deref()) {
                progress.note(note);
            }
        }
        Ok(())
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
        Ok(())
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

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/cc.rs"]
mod tests;
