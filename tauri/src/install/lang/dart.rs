//! Dart SDK, from Google's stable channel.

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::s;
use crate::install::{arch_of, Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

pub struct Dart;

impl Installer for Dart {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let arch = arch_of(ctx, component);
        let info: Value = ctx.net.json(
            "https://storage.googleapis.com/dart-archive/channels/stable/release/latest/VERSION",
        )?;
        let v = s(&info, "version")?.to_owned();
        let name = format!("dartsdk-windows-{arch}-release.zip");
        let url = format!(
            "https://storage.googleapis.com/dart-archive/channels/stable/release/{v}/sdk/{name}"
        );
        let sha = first_sha256(&ctx.net.text(&format!("{url}.sha256sum"))?)
            .ok_or_else(|| anyhow!("Dart: no SHA-256"))?;
        Ok(Release::single(
            v,
            url,
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Named("dart-sdk".into()),
        ))
    }
}
