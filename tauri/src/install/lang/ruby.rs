//! Ruby, from RubyInstaller2's releases.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::install::{arch_of, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Ruby: RubyInstaller2 7z (without the MSYS2 devkit), highest version and
/// build among the non-prerelease releases.
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
/// among the releases that are not previews.
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
            let c = re.captures(name)?;
            Some((c[1].to_owned(), c[2].parse::<u64>().ok()?, a))
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

fn ruby_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "arm"
    } else {
        "x64"
    }
}

fn ruby_releases(ctx: &Ctx) -> Result<Vec<Value>> {
    ctx.net
        .json("https://api.github.com/repos/oneclick/rubyinstaller2/releases?per_page=30")
}

fn ruby_release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
    let arch = ruby_arch(ctx, component);
    let releases = ruby_releases(ctx)?;
    let (version, asset) = pick_ruby_line(&releases, arch, line)
        .ok_or_else(|| anyhow!("no RubyInstaller {} for {arch}", line.unwrap_or("")))?;
    let digest = asset
        .get("digest")
        .and_then(Value::as_str)
        .and_then(|d| d.strip_prefix("sha256:"))
        .ok_or_else(|| anyhow!("RubyInstaller: no SHA-256"))?;
    Ok(Release::single(
        version,
        s(asset, "browser_download_url")?.to_owned(),
        s(asset, "name")?.to_owned(),
        Expect::Sha256(digest.to_owned()),
        Kind::SevenZip,
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
        Ok(ruby_lines(&ruby_releases(ctx)?, ruby_arch(ctx, component)))
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
        for sub in [r"lib\ruby\gems", "bin"] {
            copy_missing(&old.join(sub), &tree.join(sub))?;
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
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ruby_highest_version_and_build() {
        let rel = |pre: bool, names: &[&str]| json!({"prerelease": pre, "assets": names.iter().map(|n| json!({"name": n})).collect::<Vec<_>>()});
        let releases = vec![
            rel(
                false,
                &[
                    "rubyinstaller-3.3.9-1-x64.7z",
                    "rubyinstaller-3.3.9-1-arm.7z",
                ],
            ),
            rel(
                false,
                &[
                    "rubyinstaller-3.4.5-1-x64.7z",
                    "rubyinstaller-3.4.5-2-x64.7z",
                ],
            ),
            rel(true, &["rubyinstaller-3.5.0-1-x64.7z"]),
        ];
        assert_eq!(
            pick_ruby_line(&releases, "x64", None).map(|(v, _)| v),
            Some("3.4.5-2".into())
        );
        assert_eq!(
            pick_ruby_line(&releases, "arm", None).map(|(v, _)| v),
            Some("3.3.9-1".into())
        );
    }
}
