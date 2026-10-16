//! Go, from go.dev's list of releases.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::install::{arch_of, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Go;

/// The Windows archive for `arch` of a release of go.dev's list.
fn go_file<'a>(rel: &'a Value, arch: &str) -> Option<&'a Value> {
    rel.get("files")?.as_array()?.iter().find(|f| {
        f.get("os").and_then(Value::as_str) == Some("windows")
            && f.get("arch").and_then(Value::as_str) == Some(arch)
            && f.get("kind").and_then(Value::as_str) == Some("archive")
    })
}

fn go_arch(ctx: &Ctx, component: &Component) -> &'static str {
    if arch_of(ctx, component) == "arm64" {
        "arm64"
    } else {
        "amd64"
    }
}

/// The stable lines of go.dev's full list (`include=all`), newest first.
pub fn go_lines(list: &[Value], arch: &str) -> Vec<String> {
    let mut out: Vec<String> = list
        .iter()
        .filter(|r| r.get("stable") == Some(&Value::Bool(true)) && go_file(r, arch).is_some())
        .filter_map(|r| crate::versions::line_of(r.get("version")?.as_str()?))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

impl Go {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let arch = go_arch(ctx, component);
        let url = if line.is_some() {
            "https://go.dev/dl/?mode=json&include=all"
        } else {
            "https://go.dev/dl/?mode=json"
        };
        let list: Vec<Value> = ctx.net.json(url)?;
        // Newest first in the list; the full list is checked anyway.
        let rel = list
            .iter()
            .filter(|r| r.get("stable") == Some(&Value::Bool(true)) && go_file(r, arch).is_some())
            .filter(|r| {
                line.is_none_or(|l| {
                    r.get("version")
                        .and_then(Value::as_str)
                        .and_then(crate::versions::line_of)
                        .as_deref()
                        == Some(l)
                })
            })
            .max_by_key(|r| version_key(r.get("version").and_then(Value::as_str).unwrap_or("")))
            .ok_or_else(|| anyhow!("no stable Go {}", line.unwrap_or("")))?;
        let f = go_file(rel, arch).ok_or_else(|| anyhow!("no Go archive for windows/{arch}"))?;
        let name = s(f, "filename")?;
        Ok(Release::single(
            s(rel, "version")?.trim_start_matches("go").to_owned(),
            format!("https://go.dev/dl/{name}"),
            name.to_owned(),
            Expect::Sha256(s(f, "sha256")?.to_owned()),
            Kind::Zip,
            Inner::Named("go".into()),
        ))
    }
}

impl Installer for Go {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        Go::release(ctx, component, None)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let list: Vec<Value> = ctx.net.json("https://go.dev/dl/?mode=json&include=all")?;
        Ok(go_lines(&list, go_arch(ctx, component)))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Go::release(ctx, component, Some(line))
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/go.rs"]
mod tests;
