//! Languages installed from archives (sources and checks in
//! docs/app/CATALOG.md §2).
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

pub(crate) use cc::xcode_selected;
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
use crate::net::{verifiable, GhRelease};

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
        .filter(|r| !r.prerelease && is_release(tag_version(&r.tag_name)))
        // Only what can be checked: GitHub's digest or a checksum file.
        .filter(|r| r.asset(&re).is_some_and(|a| verifiable(r, a)))
        .collect();
    out.sort_by_key(|r| std::cmp::Reverse(version_key(tag_version(&r.tag_name))));
    Ok(out)
}

/// The version of a release tag: `v2.1.0`, `bun-v1.2.0` → `2.1.0`, `1.2.0`.
fn tag_version(tag: &str) -> &str {
    tag.trim_start_matches("bun-").trim_start_matches('v')
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
    github_line_kind(ctx, repo, asset, line, Kind::Zip, inner)
}

fn github_line_kind(
    ctx: &Ctx,
    repo: &str,
    asset: &str,
    line: &str,
    kind: Kind,
    inner: Inner,
) -> Result<Release> {
    let releases = github_stable(ctx, repo, asset)?;
    let rel = releases
        .iter()
        .find(|r| crate::versions::line_of(&r.tag_name).as_deref() == Some(line))
        .ok_or_else(|| anyhow!("{repo} {line} not available"))?;
    let file = rel
        .asset(&pattern(asset)?)
        .ok_or_else(|| anyhow!("{repo} {}: no asset", rel.tag_name))?;
    Ok(Release::single(
        tag_version(&rel.tag_name).to_owned(),
        file.browser_download_url.clone(),
        file.name.clone(),
        Expect::Sha256(ctx.net.release_sha256(rel, file)?),
        kind,
        inner,
    ))
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/mod.rs"]
mod tests;
