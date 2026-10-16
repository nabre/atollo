//! Go, from go.dev's list of releases.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{s, version_key};
use crate::install::{target_of, Ctx, Installer, Release};
use crate::kit::Os;
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Go;

/// The archive for `(os, arch)` of a release of go.dev's list.
pub(crate) fn go_file_for<'a>(rel: &'a Value, os: &str, arch: &str) -> Option<&'a Value> {
    rel.get("files")?.as_array()?.iter().find(|f| {
        f.get("os").and_then(Value::as_str) == Some(os)
            && f.get("arch").and_then(Value::as_str) == Some(arch)
            && f.get("kind").and_then(Value::as_str) == Some("archive")
    })
}

/// The stable lines of go.dev's full list (`include=all`) for Windows `arch`, newest first.
#[cfg(test)]
pub fn go_lines(list: &[Value], arch: &str) -> Vec<String> {
    go_lines_for(list, "windows", arch)
}

/// The stable lines of go.dev's full list (`include=all`) for `(os, arch)`, newest first.
pub fn go_lines_for(list: &[Value], os: &str, arch: &str) -> Vec<String> {
    let mut out: Vec<String> = list
        .iter()
        .filter(|r| {
            r.get("stable") == Some(&Value::Bool(true)) && go_file_for(r, os, arch).is_some()
        })
        .filter_map(|r| crate::versions::line_of(r.get("version")?.as_str()?))
        .collect();
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out.dedup();
    out
}

impl Go {
    fn release(ctx: &Ctx, component: &Component, line: Option<&str>) -> Result<Release> {
        let target = target_of(ctx, component);
        let os = target.go_os();
        let arch = target.go_arch();
        let kind = match target.os {
            Os::Win32 => Kind::Zip,
            Os::Darwin | Os::Linux => Kind::TarGz,
        };
        let url = if line.is_some() {
            "https://go.dev/dl/?mode=json&include=all"
        } else {
            "https://go.dev/dl/?mode=json"
        };
        let list: Vec<Value> = ctx.net.json(url)?;
        // Newest first in the list; the full list is checked anyway.
        let rel = list
            .iter()
            .filter(|r| {
                r.get("stable") == Some(&Value::Bool(true)) && go_file_for(r, os, arch).is_some()
            })
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
        let f =
            go_file_for(rel, os, arch).ok_or_else(|| anyhow!("no Go archive for {os}/{arch}"))?;
        let name = s(f, "filename")?;
        Ok(Release::single(
            s(rel, "version")?.trim_start_matches("go").to_owned(),
            format!("https://go.dev/dl/{name}"),
            name.to_owned(),
            Expect::Sha256(s(f, "sha256")?.to_owned()),
            kind,
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
        let target = target_of(ctx, component);
        let list: Vec<Value> = ctx.net.json("https://go.dev/dl/?mode=json&include=all")?;
        Ok(go_lines_for(&list, target.go_os(), target.go_arch()))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        Go::release(ctx, component, Some(line))
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/lang/go.rs"]
mod tests;
