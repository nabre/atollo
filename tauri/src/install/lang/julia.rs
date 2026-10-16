//! Julia, from julialang.org's list of versions.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{file_name_of, is_release, s, version_key};
use crate::install::{triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Julia;

const JULIA_VERSIONS: &str = "https://julialang-s3.julialang.org/bin/versions.json";

/// The Windows zip of `arch` of a version, if it has one.
fn julia_zip<'a>(version: &'a Value, arch: &str) -> Option<&'a Value> {
    version
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|f| {
            f.get("os").and_then(Value::as_str) == Some("winnt")
                && f.get("arch").and_then(Value::as_str) == Some(arch)
                && f.get("kind").and_then(Value::as_str) == Some("archive")
                && f.get("extension").and_then(Value::as_str) == Some("zip")
                && f.get("sha256").and_then(Value::as_str).is_some()
        })
}

/// The stable versions with a zip for `arch`, newest first.
fn julia_stable<'a>(versions: &'a Value, arch: &str) -> Result<Vec<(&'a str, &'a Value)>> {
    let obj = versions
        .as_object()
        .ok_or_else(|| anyhow!("Julia versions"))?;
    let mut out: Vec<(&str, &Value)> = obj
        .iter()
        .filter(|(k, v)| is_release(k) && v.get("stable") == Some(&Value::Bool(true)))
        .filter_map(|(k, v)| Some((k.as_str(), julia_zip(v, arch)?)))
        .collect();
    out.sort_by_key(|(k, _)| std::cmp::Reverse(version_key(k)));
    Ok(out)
}

/// The newest stable version (of `line`, when given).
fn julia_release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
    let arch = triple_arch(ctx, component);
    let versions: Value = ctx.net.json(JULIA_VERSIONS)?;
    let (v, f) = julia_stable(&versions, arch)?
        .into_iter()
        .find(|(k, _)| line.is_none_or(|l| crate::versions::line_of(k).as_deref() == Some(l)))
        .ok_or_else(|| anyhow!("no stable Julia {} for {arch}", line.unwrap_or("")))?;
    let url = s(f, "url")?.to_owned();
    Ok(Release::single(
        v.to_owned(),
        url.clone(),
        file_name_of(&url),
        Expect::Sha256(s(f, "sha256")?.to_owned()),
        Kind::Zip,
        Inner::Single,
    ))
}

impl Installer for Julia {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        julia_release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let versions: Value = ctx.net.json(JULIA_VERSIONS)?;
        let mut out: Vec<String> = Vec::new();
        for (k, _) in julia_stable(&versions, triple_arch(ctx, component))? {
            if let Some(line) = crate::versions::line_of(k) {
                if !out.contains(&line) {
                    out.push(line);
                }
            }
        }
        Ok(out)
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        julia_release(ctx, component, Some(line))
    }
}
