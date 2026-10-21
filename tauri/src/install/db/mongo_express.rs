//! mongo-express (D-147): the kit's own build, a zip of pure JavaScript
//! for every system, from a GitHub release of this repository.

use anyhow::{anyhow, bail, Result};

use crate::install::{Ctx, Installer, Release};
use crate::net::archive::{Inner, Kind};
use crate::net::first_sha256;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

/// Releases `mongo-express-v<version>` of this repository
/// (`.github/workflows/mongo-express.yml`, `scripts/build-mongo-express`).
const RELEASES: &str = "https://github.com/nabre/atollo/releases/download";

/// The zip of the kit's build `version` (`1.1.0-rc.4-1`: upstream's version
/// and the kit's build number).
pub fn mongo_express_file(version: &str) -> String {
    format!("mongo-express-{version}.zip")
}

/// The address of the zip of `version` in its release.
pub fn mongo_express_url(version: &str) -> String {
    format!(
        "{RELEASES}/mongo-express-v{version}/{}",
        mongo_express_file(version)
    )
}

/// A version the catalog may name: digits and dots, `-rc.<n>` maybe, then
/// the build number (`1.1.0-rc.4-1`, `1.2.0-1`); nothing else reaches the
/// address.
pub fn valid_version(version: &str) -> bool {
    let Some((upstream, build)) = version.rsplit_once('-') else {
        return false;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let numbers = |s: &str| s.split('.').count() == 3 && s.split('.').all(digits);
    let upstream_ok = match upstream.split_once("-rc.") {
        Some((v, rc)) => numbers(v) && digits(rc),
        None => numbers(upstream),
    };
    upstream_ok && digits(build)
}

fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// The release of `version` checked against `sha`: the SHA-256 fixed in
/// the catalog, or the one published next to the zip.
pub fn mongo_express_release(version: &str, sha: &str) -> Result<Release> {
    if !valid_version(version) {
        bail!("mongo-express: {version:?} is not a version of the kit's build");
    }
    if !is_sha256(sha) {
        bail!("mongo-express {version}: no valid SHA-256");
    }
    Ok(Release::single(
        version.to_owned(),
        mongo_express_url(version),
        mongo_express_file(version),
        Expect::Sha256(sha.to_ascii_lowercase()),
        Kind::Zip,
        Inner::Root,
    ))
}

/// mongo-express (D-147): always the version the catalog fixes for this
/// release of Atollo (`version` of `mongo-express`), like the editor. Its
/// SHA-256 is the catalog's (`sha256`) once the release is published and
/// pinned; before that, the `.sha256` the workflow publishes next to the
/// zip. Never a download without one. The folder is `mongo-express\<line>`.
pub struct MongoExpress;

impl Installer for MongoExpress {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let version = component
            .version
            .as_deref()
            .ok_or_else(|| anyhow!("mongo-express: no version fixed in the catalog"))?;
        if !valid_version(version) {
            bail!("mongo-express: {version:?} is not a version of the kit's build");
        }
        if let Some(sha) = component.sha256.as_deref() {
            return mongo_express_release(version, sha);
        }
        let file = mongo_express_file(version);
        let text = ctx
            .net
            .text(&format!("{}.sha256", mongo_express_url(version)))?;
        let sha = crate::net::hash_in_list(&text, &file)
            .or_else(|| first_sha256(&text))
            .ok_or_else(|| anyhow!("mongo-express {version}: no SHA-256 in {file}.sha256"))?;
        mongo_express_release(version, &sha)
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/mongo_express.rs"]
mod tests;
