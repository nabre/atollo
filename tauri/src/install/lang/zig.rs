//! Zig, from ziglang.org's index.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{file_name_of, is_release, s, version_key};
use crate::install::{triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Zig;

/// The release lines of Zig's index with a build for `key`, newest first.
pub fn zig_lines(index: &Value, key: &str) -> Vec<String> {
    let mut out: Vec<String> = index
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(k, v)| is_release(k) && v.get(key).is_some())
        .filter_map(|(k, _)| crate::versions::line_of(k))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

impl Installer for Zig {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Zig::release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let key = format!("{}-windows", triple_arch(ctx, component));
        let index: Value = ctx.net.json("https://ziglang.org/download/index.json")?;
        Ok(zig_lines(&index, &key))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Zig::release(ctx, component, Some(line))
    }
}

impl Zig {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let key = format!("{}-windows", triple_arch(ctx, component));
        let index: Value = ctx.net.json("https://ziglang.org/download/index.json")?;
        let obj = index.as_object().ok_or_else(|| anyhow!("Zig index"))?;
        let v = obj
            .keys()
            .filter(|k| is_release(k))
            .filter(|k| line.is_none_or(|l| crate::versions::line_of(k).as_deref() == Some(l)))
            .max_by_key(|k| version_key(k))
            .ok_or_else(|| anyhow!("no Zig release {}", line.unwrap_or("")))?;
        let f = obj[v]
            .get(&key)
            .ok_or_else(|| anyhow!("Zig {v}: no {key}"))?;
        let url = s(f, "tarball")?.to_owned();
        Ok(Release::single(
            v.clone(),
            url.clone(),
            file_name_of(&url),
            Expect::Sha256(s(f, "shasum")?.to_owned()),
            Kind::Zip,
            Inner::Single,
        ))
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/zig.rs"]
mod tests;
