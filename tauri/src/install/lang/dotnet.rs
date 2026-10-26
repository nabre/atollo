//! .NET SDK, from Microsoft's release index.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::install::{target_of, Ctx, Installer, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::Component;

/// .NET SDK of the active LTS channel.
pub struct Dotnet;

const DOTNET_INDEX: &str =
    "https://builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json";

fn dotnet_channels(index: &Value) -> impl Iterator<Item = &Value> {
    index
        .get("releases-index")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
}

/// The channels of .NET that are not previews (`10.0`, `9.0`...), newest first.
pub fn dotnet_lines(index: &Value) -> Vec<String> {
    let mut out: Vec<String> = dotnet_channels(index)
        .filter(|c| {
            !matches!(
                c.get("support-phase").and_then(Value::as_str),
                Some("preview" | "go-live")
            )
        })
        .filter_map(|c| c.get("channel-version")?.as_str().map(str::to_owned))
        .filter(|v| crate::versions::valid_line(v))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out
}

pub(crate) fn dotnet_rid_spec(target: Target) -> (&'static str, &'static str, Kind) {
    match (target.os, target.arch) {
        (Os::Win32, _) => ("win-x64", ".zip", Kind::Zip),
        (Os::Darwin, _) => ("osx-arm64", ".tar.gz", Kind::TarGz),
        (Os::Linux, Arch::X64) => ("linux-x64", ".tar.gz", Kind::TarGz),
        (Os::Linux, Arch::Arm64) => ("linux-arm64", ".tar.gz", Kind::TarGz),
    }
}

impl Installer for Dotnet {
    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(dotnet_lines(&ctx.net.json(DOTNET_INDEX)?))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let index: Value = ctx.net.json(DOTNET_INDEX)?;
        let channel = dotnet_channels(&index)
            .find(|c| c.get("channel-version").and_then(Value::as_str) == Some(line))
            .ok_or_else(|| anyhow!(".NET {line}: no channel"))?;
        dotnet_release(ctx, component, channel)
    }

    /// The newest channel that is not a preview (the line marked "latest").
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let index: Value = ctx.net.json(DOTNET_INDEX)?;
        let newest = dotnet_lines(&index)
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!(".NET: no channel"))?;
        let channel = dotnet_channels(&index)
            .find(|c| c.get("channel-version").and_then(Value::as_str) == Some(newest.as_str()))
            .ok_or_else(|| anyhow!(".NET {newest}: no channel"))?;
        dotnet_release(ctx, component, channel)
    }
}

/// The newest SDK of a channel of `releases-index.json`.
fn dotnet_release(ctx: &Ctx, component: &Component, channel: &Value) -> Result<Release> {
    let target = target_of(ctx, component);
    let (rid, ext, kind) = dotnet_rid_spec(target);
    let sdk = s(channel, "latest-sdk")?.to_owned();
    let releases: Value = ctx.net.json(s(channel, "releases.json")?)?;
    let file = releases
        .get("releases")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .flat_map(|r| {
            r.get("sdks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .chain(r.get("sdk"))
        })
        .filter(|x| x.get("version").and_then(Value::as_str) == Some(sdk.as_str()))
        .flat_map(|x| {
            x.get("files")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
        })
        .find(|f| {
            f.get("rid").and_then(Value::as_str) == Some(rid)
                && f.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|n| n.ends_with(ext))
        })
        .ok_or_else(|| anyhow!(".NET SDK {sdk}: no {ext} for {rid}"))?;
    Ok(Release::single(
        sdk.clone(),
        s(file, "url")?.to_owned(),
        format!("dotnet-sdk-{sdk}-{rid}{ext}"),
        Expect::Sha512(s(file, "hash")?.to_owned()),
        kind,
        Inner::Root,
    ))
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/dotnet.rs"]
mod tests;
