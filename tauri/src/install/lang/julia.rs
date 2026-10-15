//! Julia, from julialang.org's list of versions.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{file_name_of, is_release, s, version_key};
use crate::install::{triple_arch, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Julia;

impl Installer for Julia {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = triple_arch(ctx, component);
        let versions: Value = ctx
            .net
            .json("https://julialang-s3.julialang.org/bin/versions.json")?;
        let obj = versions
            .as_object()
            .ok_or_else(|| anyhow!("Julia versions"))?;
        let v = obj
            .iter()
            .filter(|(k, v)| is_release(k) && v.get("stable") == Some(&Value::Bool(true)))
            .map(|(k, _)| k)
            .max_by_key(|k| version_key(k))
            .ok_or_else(|| anyhow!("no stable Julia"))?;
        let f = obj[v]
            .get("files")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .find(|f| {
                f.get("os").and_then(Value::as_str) == Some("winnt")
                    && f.get("arch").and_then(Value::as_str) == Some(arch)
                    && f.get("kind").and_then(Value::as_str) == Some("archive")
                    && f.get("extension").and_then(Value::as_str) == Some("zip")
            })
            .ok_or_else(|| anyhow!("Julia {v}: no zip for {arch}"))?;
        let url = s(f, "url")?.to_owned();
        Ok(Release::single(
            v.clone(),
            url.clone(),
            file_name_of(&url),
            Expect::Sha256(s(f, "sha256")?.to_owned()),
            Kind::Zip,
            Inner::Single,
        ))
    }
}
