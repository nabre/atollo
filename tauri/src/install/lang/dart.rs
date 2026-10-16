//! Dart SDK, from Google's stable channel.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{is_release, s, version_key};
use crate::install::{arch_of, target_of, Ctx, Installer, Release};
use crate::kit::Os;
use crate::net::archive::{Inner, Kind};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Dart;

const DART_STABLE: &str = "https://storage.googleapis.com/dart-archive/channels/stable/release";
const DART_LIST: &str = "https://storage.googleapis.com/storage/v1/b/dart-archive/o?prefix=channels/stable/release/&delimiter=/&fields=prefixes,nextPageToken";

/// Every stable version of the archive, newest first (its folders, without
/// the old ones named by revision number).
fn dart_versions(ctx: &Ctx) -> Result<Vec<String>> {
    let mut out: Vec<String> = Vec::new();
    let mut token: Option<String> = None;
    loop {
        let url = match &token {
            Some(t) => format!("{DART_LIST}&pageToken={t}"),
            None => DART_LIST.to_owned(),
        };
        let page: Value = ctx.net.json(&url)?;
        out.extend(
            page.get("prefixes")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter_map(|p| p.trim_end_matches('/').rsplit('/').next())
                .filter(|v| is_release(v) && v.split('.').count() == 3)
                .map(str::to_owned),
        );
        token = page
            .get("nextPageToken")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if token.is_none() {
            break;
        }
    }
    out.sort_by_key(|v| std::cmp::Reverse(version_key(v)));
    Ok(out)
}

/// The SDK zip of version `v` with its SHA-256.
fn dart_release(ctx: &Ctx, component: &Component, v: &str) -> Result<Release> {
    let target = target_of(ctx, component);
    let os_tag = match target.os {
        Os::Win32 => "windows",
        Os::Darwin => "macos",
        Os::Linux => "linux",
    };
    let arch = arch_of(ctx, component);
    let name = format!("dartsdk-{os_tag}-{arch}-release.zip");
    let url = format!("{DART_STABLE}/{v}/sdk/{name}");
    let sha = first_sha256(&ctx.net.text(&format!("{url}.sha256sum"))?)
        .ok_or_else(|| anyhow!("Dart {v}: no SHA-256"))?;
    Ok(Release::single(
        v.to_owned(),
        url,
        name,
        Expect::Sha256(sha),
        Kind::Zip,
        Inner::Named("dart-sdk".into()),
    ))
}

impl Installer for Dart {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let info: Value = ctx.net.json(&format!("{DART_STABLE}/latest/VERSION"))?;
        dart_release(ctx, component, s(&info, "version")?)
    }

    fn has_lines(&self) -> bool {
        true
    }

    /// Dart 2 and later: the SDKs of Dart 1 are another language.
    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        let mut out: Vec<String> = Vec::new();
        for v in dart_versions(ctx)? {
            if let Some(line) = crate::versions::line_of(&v).filter(|l| !l.starts_with("1.")) {
                if !out.contains(&line) {
                    out.push(line);
                }
            }
        }
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let v = dart_versions(ctx)?
            .into_iter()
            .find(|v| crate::versions::line_of(v).as_deref() == Some(line))
            .ok_or_else(|| anyhow!("Dart {line} not available"))?;
        dart_release(ctx, component, &v)
    }
}
