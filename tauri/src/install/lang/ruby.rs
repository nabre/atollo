//! Ruby, from RubyInstaller2's releases.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::fsx::join_rel;
use crate::install::{arch_of, target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Ruby: RubyInstaller2 7z on Windows (without the MSYS2 devkit), or
/// ruby/ruby-builder portable tar.gz on macOS and Linux.
pub struct Ruby;

/// The best `rubyinstaller-X.Y.Z-N-<arch>.7z` among release assets, of
/// `line` (`3.4`) or of any line: (version `X.Y.Z-N`, asset object).
pub fn pick_ruby_line<'a>(
    releases: &'a [Value],
    arch: &str,
    line: Option<&str>,
) -> Option<(String, &'a Value)> {
    ruby_assets(releases, arch)
        .into_iter()
        .filter(|(v, _, _)| line.is_none_or(|l| crate::versions::line_of(v).as_deref() == Some(l)))
        .max_by(|a, b| {
            version_key(&a.0)
                .cmp(&version_key(&b.0))
                .then(a.1.cmp(&b.1))
        })
        .map(|(v, n, a)| (format!("{v}-{n}"), a))
}

/// `(version, build, asset)` of every `rubyinstaller-X.Y.Z-N-<arch>.7z`
/// among the releases that are not previews, with GitHub's SHA-256 (the
/// older ones have none: they could not be checked).
fn ruby_assets<'a>(releases: &'a [Value], arch: &str) -> Vec<(String, u64, &'a Value)> {
    let Ok(re) = regex::Regex::new(&format!(
        r"^rubyinstaller-(\d+\.\d+\.\d+)-(\d+)-{arch}\.7z$"
    )) else {
        return Vec::new();
    };
    releases
        .iter()
        .filter(|r| {
            r.get("prerelease") != Some(&Value::Bool(true))
                && r.get("draft") != Some(&Value::Bool(true))
        })
        .flat_map(|r| {
            r.get("assets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?;
            a.get("digest")?.as_str()?.strip_prefix("sha256:")?;
            let c = re.captures(name)?;
            Some((c[1].to_owned(), c[2].parse::<u64>().ok()?, a))
        })
        .collect()
}

pub(crate) fn ruby_unix_suffix(target: Target) -> &'static str {
    match (target.os, target.arch) {
        (Os::Darwin, Arch::Arm64) => r"darwin-arm64\.tar\.gz",
        (Os::Darwin, Arch::X64) => r"darwin-x64\.tar\.gz",
        (Os::Linux, Arch::X64) => r"ubuntu-(?:22\.04|24\.04)(?:-x64)?\.tar\.gz",
        (Os::Linux, Arch::Arm64) => r"ubuntu-(?:22\.04|24\.04)-arm64\.tar\.gz",
        (Os::Win32, _) => "",
    }
}

fn ruby_unix_assets(releases: &[Value], target: Target) -> Vec<(String, &Value)> {
    let suffix = ruby_unix_suffix(target);
    let Ok(re) = regex::Regex::new(&format!(r"^ruby-(\d+\.\d+\.\d+)-{suffix}$")) else {
        return Vec::new();
    };
    releases
        .iter()
        .filter(|r| {
            r.get("prerelease") != Some(&Value::Bool(true))
                && r.get("draft") != Some(&Value::Bool(true))
        })
        .flat_map(|r| {
            r.get("assets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?;
            a.get("digest")?.as_str()?.strip_prefix("sha256:")?;
            let c = re.captures(name)?;
            Some((c[1].to_owned(), a))
        })
        .collect()
}

/// The minor lines RubyInstaller has for `arch`, newest first (3.3 has no
/// ARM64 build).
pub fn ruby_lines(releases: &[Value], arch: &str) -> Vec<String> {
    let mut out: Vec<String> = ruby_assets(releases, arch)
        .into_iter()
        .filter_map(|(v, _, _)| crate::versions::line_of(&v))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

pub fn ruby_unix_lines(releases: &[Value], target: Target) -> Vec<String> {
    let mut out: Vec<String> = ruby_unix_assets(releases, target)
        .into_iter()
        .filter_map(|(v, _)| crate::versions::line_of(&v))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

fn ruby_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "arm"
    } else {
        "x64"
    }
}

fn ruby_releases(ctx: &Ctx, target: Target) -> Result<Vec<Value>> {
    let url = match target.os {
        Os::Win32 => "https://api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30",
        Os::Darwin | Os::Linux => {
            "https://api.github.com/repos/ruby/ruby-builder/releases?per_page=30"
        }
    };
    ctx.net.json(url)
}

fn ruby_release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
    let target = target_of(ctx, component);
    let releases = ruby_releases(ctx, target)?;
    if target.os == Os::Win32 {
        let arch = ruby_arch(ctx, component);
        let (version, asset) = pick_ruby_line(&releases, arch, line)
            .ok_or_else(|| anyhow!("no RubyInstaller {} for {arch}", line.unwrap_or("")))?;
        let digest = asset
            .get("digest")
            .and_then(Value::as_str)
            .and_then(|d| d.strip_prefix("sha256:"))
            .ok_or_else(|| anyhow!("RubyInstaller: no SHA-256"))?;
        return Ok(Release::single(
            version,
            s(asset, "browser_download_url")?.to_owned(),
            s(asset, "name")?.to_owned(),
            Expect::Sha256(digest.to_owned()),
            Kind::SevenZip,
            Inner::Single,
        ));
    }
    let (version, asset) = ruby_unix_assets(&releases, target)
        .into_iter()
        .filter(|(v, _)| line.is_none_or(|l| crate::versions::line_of(v).as_deref() == Some(l)))
        .max_by(|a, b| version_key(&a.0).cmp(&version_key(&b.0)))
        .ok_or_else(|| anyhow!("no Ruby {} for {target}", line.unwrap_or("")))?;
    let digest = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or_else(|| anyhow!("Ruby: no SHA-256"))?;
    Ok(Release::single(
        version,
        s(asset, "browser_download_url")?.to_owned(),
        s(asset, "name")?.to_owned(),
        Expect::Sha256(digest.to_owned()),
        Kind::TarGz,
        Inner::Single,
    ))
}

/// Gems live in the folder of their Ruby (`lib\ruby\gems\3.4.0`, commands
/// in `bin\`), one set per minor version (D-81): no GEM_HOME shared by all.
impl Installer for Ruby {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        ruby_release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        let releases = ruby_releases(ctx, target)?;
        if target.os == Os::Win32 {
            Ok(ruby_lines(&releases, ruby_arch(ctx, component)))
        } else {
            Ok(ruby_unix_lines(&releases, target))
        }
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        ruby_release(ctx, component, Some(line))
    }

    /// A new patch of the same line replaces the folder: the gems installed
    /// in the old one (and their commands) move to the new tree.
    fn before_swap(
        &self,
        _ctx: &Ctx,
        _component: &Component,
        tree: &Path,
        old: &Path,
    ) -> Result<()> {
        for sub in ["lib/ruby/gems", "bin"] {
            copy_missing(&join_rel(old, sub), &join_rel(tree, sub))?;
        }
        Ok(())
    }
}

/// Files and folders of `from` that `to` does not have, copied.
fn copy_missing(from: &Path, to: &Path) -> Result<()> {
    let Ok(entries) = fs::read_dir(from) else {
        return Ok(());
    };
    fs::create_dir_all(to)?;
    for entry in entries.flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_missing(&entry.path(), &target)?;
        } else if !target.exists() {
            fs::copy(entry.path(), &target).with_context(|| target.display().to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/ruby.rs"]
mod tests;
