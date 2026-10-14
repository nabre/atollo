//! Database servers and tools of the kit (DECISIONS.md D-73): MariaDB
//! (compatible with MySQL), MongoDB with mongosh, SQLite. Sources and
//! checks in docs/app/CATALOG.md §2.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{arch_of, pattern, Ctx, Dest, Installer, Part, Release};
use crate::archive::{Inner, Kind};
use crate::catalog::Component;
use crate::net::github_digest;
use crate::verify::Expect;

use super::lang::version_key;

/// Debug symbols (`.pdb`) are most of the archives (2.2 GB of MongoDB's
/// 2.5): removed from the new tree before the swap.
fn remove_debug_symbols(tree: &Path) -> Result<()> {
    for entry in fs::read_dir(tree)?.flatten() {
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            remove_debug_symbols(&path)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdb"))
        {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

const MARIADB_API: &str = "https://downloads.mariadb.org/rest-api/mariadb";

/// The newest major release that is stable and has long-term support.
fn mariadb_major(index: &Value) -> Result<String> {
    index["major_releases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["release_status"] == "Stable")
        .filter(|m| m["release_support_type"] == "Long Term Support")
        .filter_map(|m| m["release_id"].as_str())
        .max_by_key(|id| version_key(id))
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("MariaDB: no stable long-term release"))
}

/// Version, file name and SHA-256 of the Windows x64 zip in the answer of
/// `/<major>/latest/` (debug symbols excluded).
fn mariadb_zip(latest: &Value) -> Result<(String, String, String)> {
    let releases = latest["releases"]
        .as_object()
        .ok_or_else(|| anyhow!("MariaDB: no releases"))?;
    let (version, release) = releases
        .iter()
        .max_by_key(|(v, _)| version_key(v))
        .ok_or_else(|| anyhow!("MariaDB: no releases"))?;
    let file = release["files"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|f| {
            f["os"] == "Windows"
                && f["cpu"] == "x86_64"
                && f["package_type"] == "ZIP file"
                && f["file_name"]
                    .as_str()
                    .is_some_and(|n| n.ends_with("-winx64.zip"))
        })
        .ok_or_else(|| anyhow!("MariaDB {version}: no Windows zip"))?;
    let name = file["file_name"].as_str().unwrap_or_default().to_owned();
    let sha = file["checksum"]["sha256sum"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MariaDB {version}: no SHA-256"))?;
    Ok((version.clone(), name, sha.to_owned()))
}

pub struct MariaDb;

impl Installer for MariaDb {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let major = mariadb_major(&ctx.net.json::<Value>(&format!("{MARIADB_API}/"))?)?;
        let latest = ctx
            .net
            .json::<Value>(&format!("{MARIADB_API}/{major}/latest/"))?;
        let (version, name, sha) = mariadb_zip(&latest)?;
        Ok(Release::single(
            version.clone(),
            // The API lists http:// links: the same file over https.
            format!("{MARIADB_API}/{version}/{name}"),
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Single,
        ))
    }

    fn before_swap(&self, _ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        remove_debug_symbols(tree)
    }
}

/// The newest production release of a major version (`x.0.y`: minor
/// releases are rapid releases), its Windows x64 zip and SHA-256.
fn mongodb_zip(current: &Value) -> Result<(String, String, String)> {
    let version = current["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["production_release"] == true)
        .find(|v| {
            v["version"]
                .as_str()
                .is_some_and(|s| s.split('.').nth(1) == Some("0"))
        })
        .ok_or_else(|| anyhow!("MongoDB: no production release"))?;
    let number = version["version"].as_str().unwrap_or_default().to_owned();
    let archive = version["downloads"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|d| d["target"] == "windows" && d["arch"] == "x86_64" && d["edition"] == "base")
        .map(|d| &d["archive"])
        .ok_or_else(|| anyhow!("MongoDB {number}: no Windows zip"))?;
    let url = archive["url"]
        .as_str()
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| anyhow!("MongoDB {number}: no https link"))?;
    let sha = archive["sha256"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MongoDB {number}: no SHA-256"))?;
    Ok((number, url.to_owned(), sha.to_owned()))
}

pub struct MongoDb;

impl Installer for MongoDb {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let current = ctx
            .net
            .json::<Value>("https://downloads.mongodb.org/current.json")?;
        let (version, url, sha) = mongodb_zip(&current)?;
        let shell = ctx.net.github_latest("mongodb-js/mongosh")?;
        let asset = shell
            .asset(&pattern(r"^mongosh-[\d.]+-win32-x64\.zip$")?)
            .ok_or_else(|| anyhow!("mongosh {}: no Windows zip", shell.tag_name))?;
        let shell_version = shell.tag_name.trim_start_matches('v').to_owned();
        let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
        Ok(Release {
            version: format!("{version}+mongosh{shell_version}"),
            parts: vec![
                Part {
                    url,
                    file_name,
                    expect: Expect::Sha256(sha),
                    kind: Kind::Zip,
                    inner: Inner::Single,
                    dest: Dest::Own,
                    sign: None,
                },
                Part {
                    url: asset.browser_download_url.clone(),
                    file_name: asset.name.clone(),
                    expect: Expect::Sha256(github_digest(asset)?),
                    kind: Kind::Zip,
                    inner: Inner::Single,
                    dest: Dest::Folder("mongosh"),
                    sign: None,
                },
            ],
        })
    }

    /// mongod needs the VC++ runtime (the zip only has its installer):
    /// the kit's copy, as for Apache; no debug symbols, no installer.
    fn before_swap(&self, ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        remove_debug_symbols(tree)?;
        let _ = fs::remove_file(tree.join("bin").join("vc_redist.x64.exe"));
        super::web::copy_vc_runtime(ctx, &tree.join("bin"))
    }
}

/// From the `PRODUCT,<version>,<path>,<size>,<sha3-256>` lines of
/// sqlite.org's download page: the command-line tools for `arch`.
fn sqlite_tools(page: &str, arch: &str) -> Result<(String, String, String)> {
    let re = pattern(&format!(
        r"(?m)^PRODUCT,([\d.]+),(\d{{4}}/sqlite-tools-win-{arch}-\d+\.zip),\d+,([0-9a-f]{{64}})\s*$"
    ))?;
    let c = re
        .captures(page)
        .ok_or_else(|| anyhow!("SQLite: no tools for win-{arch} on the download page"))?;
    Ok((c[1].to_owned(), c[2].to_owned(), c[3].to_owned()))
}

pub struct Sqlite;

impl Installer for Sqlite {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let page = ctx.net.text("https://sqlite.org/download.html")?;
        let (version, path, sha3) = sqlite_tools(&page, arch_of(ctx, component))?;
        let name = path.rsplit('/').next().unwrap_or("sqlite.zip").to_owned();
        Ok(Release::single(
            version,
            format!("https://sqlite.org/{path}"),
            name,
            Expect::Sha3_256(sha3),
            Kind::Zip,
            Inner::Auto,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mariadb_newest_lts_and_its_zip() {
        let index: Value = serde_json::from_str(
            r#"{"major_releases":[
              {"release_id":"13.0","release_status":"Stable","release_support_type":"Rolling"},
              {"release_id":"12.3","release_status":"Stable","release_support_type":"Long Term Support"},
              {"release_id":"11.8","release_status":"Stable","release_support_type":"Long Term Support"},
              {"release_id":"13.2","release_status":"Preview","release_support_type":null}]}"#,
        )
        .unwrap();
        assert_eq!(mariadb_major(&index).unwrap(), "12.3");
        let latest: Value = serde_json::from_str(&format!(
            r#"{{"releases":{{"12.3.4":{{"files":[
              {{"file_name":"repo/","os":null,"cpu":null,"package_type":null,"checksum":{{}}}},
              {{"file_name":"mariadb-12.3.4-winx64-debugsymbols.zip","os":"Windows","cpu":"x86_64","package_type":"ZIP file","checksum":{{"sha256sum":"{a}"}}}},
              {{"file_name":"mariadb-12.3.4-winx64.zip","os":"Windows","cpu":"x86_64","package_type":"ZIP file","checksum":{{"sha256sum":"{b}"}}}}]}}}}}}"#,
            a = "a".repeat(64),
            b = "b".repeat(64)
        ))
        .unwrap();
        let (v, name, sha) = mariadb_zip(&latest).unwrap();
        assert_eq!(v, "12.3.4");
        assert_eq!(name, "mariadb-12.3.4-winx64.zip");
        assert_eq!(sha, "b".repeat(64));
    }

    #[test]
    fn mongodb_major_release_only() {
        let current: Value = serde_json::from_str(&format!(
            r#"{{"versions":[
              {{"version":"9.1.0-rc0","production_release":false,"downloads":[]}},
              {{"version":"8.3.13","production_release":true,"downloads":[]}},
              {{"version":"9.0.4","production_release":true,"downloads":[
                {{"target":"windows","arch":"x86_64","edition":"enterprise","archive":{{"url":"https://x/e.zip","sha256":"{e}"}}}},
                {{"target":"windows","arch":"x86_64","edition":"base","archive":{{"url":"https://fastdl.mongodb.org/windows/mongodb-windows-x86_64-9.0.4.zip","sha256":"{b}"}}}}]}}]}}"#,
            e = "e".repeat(64),
            b = "b".repeat(64)
        ))
        .unwrap();
        let (v, url, sha) = mongodb_zip(&current).unwrap();
        assert_eq!(v, "9.0.4");
        assert!(url.ends_with("mongodb-windows-x86_64-9.0.4.zip"));
        assert_eq!(sha, "b".repeat(64));
    }

    #[test]
    fn sqlite_tools_for_each_arch() {
        let page = format!(
            "<!--\nPRODUCT,3.54.0,2026/sqlite-dll-win-x64-3540000.zip,1450308,{d}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-arm64-3540000.zip,5619849,{a}\n\
             PRODUCT,3.54.0,2026/sqlite-tools-win-x64-3540000.zip,6982216,{x}\n-->",
            d = "d".repeat(64),
            a = "a".repeat(64),
            x = "c".repeat(64)
        );
        let (v, path, sha) = sqlite_tools(&page, "x64").unwrap();
        assert_eq!(v, "3.54.0");
        assert_eq!(path, "2026/sqlite-tools-win-x64-3540000.zip");
        assert_eq!(sha, "c".repeat(64));
        assert_eq!(sqlite_tools(&page, "arm64").unwrap().2, "a".repeat(64));
    }
}
