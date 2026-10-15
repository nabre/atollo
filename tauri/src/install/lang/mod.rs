//! Languages installed from archives (port of nabre/atollo
//! `_languages.ps1:77-340`; sources and checks in docs/app/CATALOG.md §2).
//! One file per language or family; here the helpers they share.

mod cc;
mod dart;
mod dotnet;
mod go;
mod js;
mod julia;
mod jvm;
mod ruby;
mod typeset;
mod zig;

use anyhow::{anyhow, Result};
use serde_json::Value;

pub use cc::{make_launchers, Cc, Cmake};
pub use dart::Dart;
pub use dotnet::Dotnet;
pub use go::Go;
pub use js::{Bun, Deno};
pub use julia::Julia;
pub use jvm::{Java, Kotlin};
pub use ruby::Ruby;
pub use typeset::{TinyTex, Typst};
pub use zig::Zig;

use super::{pattern, Ctx, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::net::{github_digest, GhRelease};

/// Numeric parts of a version, for "newest" comparisons (`1.10.2` > `1.9`).
pub fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

/// A version made only of numbers and dots (no betas, no `master`).
pub fn is_release(v: &str) -> bool {
    !v.is_empty()
        && v.split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn s<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    v.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing {key}"))
}

fn file_name_of(url: &str) -> String {
    url.rsplit('/')
        .next()
        .unwrap_or("download")
        .split('?')
        .next()
        .unwrap_or("download")
        .to_owned()
}

/// The stable releases of `repo` with an asset matching `asset`, newest
/// first (its last 100 releases: betas and RCs left out). For the versions
/// of tools published on GitHub (Kotlin, Typst).
fn github_stable(ctx: &Ctx, repo: &str, asset: &str) -> Result<Vec<GhRelease>> {
    let all: Vec<GhRelease> = ctx.net.json(&format!(
        "https://api.github.com/repos/{repo}/releases?per_page=100"
    ))?;
    let re = pattern(asset)?;
    let mut out: Vec<GhRelease> = all
        .into_iter()
        .filter(|r| !r.prerelease && is_release(r.tag_name.trim_start_matches('v')))
        .filter(|r| r.asset(&re).is_some())
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(version_key(r.tag_name.trim_start_matches('v'))));
    Ok(out)
}

/// The minor lines of `releases` (newest first, each once).
pub fn release_lines(releases: &[GhRelease]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for r in releases {
        if let Some(line) = crate::versions::line_of(&r.tag_name) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out
}

/// The newest release of `line` among the stable ones of `repo`, as one
/// archive (GitHub's SHA-256 digest).
fn github_line(ctx: &Ctx, repo: &str, asset: &str, line: &str, inner: Inner) -> Result<Release> {
    let releases = github_stable(ctx, repo, asset)?;
    let rel = releases
        .iter()
        .find(|r| crate::versions::line_of(&r.tag_name).as_deref() == Some(line))
        .ok_or_else(|| anyhow!("{repo} {line} not available"))?;
    let file = rel
        .asset(&pattern(asset)?)
        .ok_or_else(|| anyhow!("{repo} {}: no asset", rel.tag_name))?;
    Ok(Release::single(
        rel.tag_name.trim_start_matches('v').to_owned(),
        file.browser_download_url.clone(),
        file.name.clone(),
        Expect::Sha256(github_digest(file)?),
        Kind::Zip,
        inner,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kotlin_lines_from_releases() {
        let rel = |tag: &str| GhRelease {
            tag_name: tag.into(),
            assets: Vec::new(),
            prerelease: false,
        };
        let list = [
            rel("v2.2.21"),
            rel("v2.2.20"),
            rel("v2.1.21"),
            rel("v2.0.0"),
        ];
        assert_eq!(release_lines(&list), ["2.2", "2.1", "2.0"]);
    }

    #[test]
    fn versions() {
        assert!(version_key("1.10.2") > version_key("1.9.9"));
        assert!(
            is_release("0.14.1")
                && !is_release("master")
                && !is_release("4.0.0-beta-1")
                && !is_release("")
        );
    }
}
