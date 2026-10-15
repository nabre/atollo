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

/// The stable releases with long-term support (`12.3`, `11.8`, `11.4`…),
/// newest first: the lines a project can follow (D-81).
pub fn mariadb_lines(index: &Value) -> Vec<String> {
    let mut out: Vec<String> = index["major_releases"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["release_status"] == "Stable")
        .filter(|m| m["release_support_type"] == "Long Term Support")
        .filter_map(|m| m["release_id"].as_str())
        .filter(|id| crate::versions::valid_line(id))
        .map(str::to_owned)
        .collect();
    out.sort_by_key(|id| std::cmp::Reverse(version_key(id)));
    out
}

/// The newest major release that is stable and has long-term support.
fn mariadb_major(index: &Value) -> Result<String> {
    mariadb_lines(index)
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("MariaDB: no stable long-term release"))
}

fn mariadb_release(ctx: &Ctx, major: &str) -> Result<Release> {
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
        mariadb_release(ctx, &major)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(mariadb_lines(
            &ctx.net.json::<Value>(&format!("{MARIADB_API}/"))?,
        ))
    }

    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        mariadb_release(ctx, line)
    }

    fn before_swap(&self, _ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        remove_debug_symbols(tree)
    }
}

/// The newest production release of a major version (`x.0.y`: minor
/// releases are rapid releases), its Windows x64 zip and SHA-256.
fn mongodb_zip(current: &Value, line: Option<&str>) -> Result<(String, String, String)> {
    let version = current["versions"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|v| v["production_release"] == true)
        .find(|v| {
            v["version"].as_str().is_some_and(|s| {
                s.split('.').nth(1) == Some("0")
                    && line.is_none_or(|l| crate::versions::line_of(s).as_deref() == Some(l))
            })
        })
        .ok_or_else(|| anyhow!("MongoDB {}: no production release", line.unwrap_or("")))?;
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

/// The newest MongoDB Database Tools for Windows x64 from `release.json`
/// (version, https link, SHA-256).
fn mongodb_tools_zip(release: &Value) -> Result<(String, String, String)> {
    let v = release["versions"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| anyhow!("MongoDB tools: no versions"))?;
    let number = v["version"].as_str().unwrap_or_default().to_owned();
    let archive = v["downloads"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|d| d["name"] == "windows" && d["arch"] == "x86_64")
        .map(|d| &d["archive"])
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no Windows zip"))?;
    let url = archive["url"]
        .as_str()
        .filter(|u| u.starts_with("https://") && u.ends_with(".zip"))
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no https zip"))?;
    let sha = archive["sha256"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no SHA-256"))?;
    Ok((number, url.to_owned(), sha.to_owned()))
}

/// The major versions in production (`9.0`, `8.0`, `7.0`), newest first:
/// the lines a project can follow (D-81). Rapid releases (`8.3`) are not.
pub fn mongodb_lines(current: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in current["versions"].as_array().into_iter().flatten() {
        let Some(number) = v["version"].as_str() else {
            continue;
        };
        if v["production_release"] != true || number.split('.').nth(1) != Some("0") {
            continue;
        }
        if let Some(line) = crate::versions::line_of(number) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out.sort_by_key(|l| std::cmp::Reverse(version_key(l)));
    out
}

const MONGODB_CURRENT: &str = "https://downloads.mongodb.org/current.json";

pub struct MongoDb;

impl Installer for MongoDb {
    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(mongodb_lines(&ctx.net.json::<Value>(MONGODB_CURRENT)?))
    }

    /// The server of `line` only: mongosh and the tools are shared, at their
    /// latest (they support every server in production).
    fn release_line(&self, ctx: &Ctx, _component: &Component, line: &str) -> Result<Release> {
        let (version, url, sha) =
            mongodb_zip(&ctx.net.json::<Value>(MONGODB_CURRENT)?, Some(line))?;
        let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
        Ok(Release {
            version,
            parts: vec![Part {
                url,
                file_name,
                expect: Expect::Sha256(sha),
                kind: Kind::Zip,
                inner: Inner::Single,
                dest: Dest::Own,
                sign: None,
            }],
        })
    }

    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let current = ctx.net.json::<Value>(MONGODB_CURRENT)?;
        let (version, url, sha) = mongodb_zip(&current, None)?;
        let shell = ctx.net.github_latest("mongodb-js/mongosh")?;
        let asset = shell
            .asset(&pattern(r"^mongosh-[\d.]+-win32-x64\.zip$")?)
            .ok_or_else(|| anyhow!("mongosh {}: no Windows zip", shell.tag_name))?;
        let shell_version = shell.tag_name.trim_start_matches('v').to_owned();
        let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
        // mongodump and mongorestore: export and import of a project's
        // database (D-78).
        let (tools_version, tools_url, tools_sha) = mongodb_tools_zip(
            &ctx.net
                .json::<Value>("https://downloads.mongodb.org/tools/db/release.json")?,
        )?;
        let tools_file = tools_url
            .rsplit('/')
            .next()
            .unwrap_or("mongodb-tools.zip")
            .to_owned();
        Ok(Release {
            version: format!("{version}+mongosh{shell_version}+tools{tools_version}"),
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
                Part {
                    url: tools_url,
                    file_name: tools_file,
                    expect: Expect::Sha256(tools_sha),
                    kind: Kind::Zip,
                    inner: Inner::Single,
                    dest: Dest::Folder("mongodb-tools"),
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
        let (v, url, sha) = mongodb_zip(&current, None).unwrap();
        assert_eq!(v, "9.0.4");
        assert!(url.ends_with("mongodb-windows-x86_64-9.0.4.zip"));
        assert_eq!(sha, "b".repeat(64));
        assert_eq!(mongodb_zip(&current, Some("9.0")).unwrap().0, "9.0.4");
        assert!(
            mongodb_zip(&current, Some("8.0")).is_err(),
            "8.3 is a rapid release"
        );
        assert_eq!(mongodb_lines(&current), ["9.0"]);
    }

    #[test]
    fn mariadb_lines_are_the_stable_lts() {
        let index = serde_json::json!({"major_releases": [
            {"release_id": "13.0", "release_status": "Stable", "release_support_type": "Short Term Support"},
            {"release_id": "12.3", "release_status": "Stable", "release_support_type": "Long Term Support"},
            {"release_id": "10.11", "release_status": "Stable", "release_support_type": "Long Term Support"},
            {"release_id": "11.8", "release_status": "Stable", "release_support_type": "Long Term Support"},
            {"release_id": "13.1", "release_status": "RC", "release_support_type": "Long Term Support"}
        ]});
        assert_eq!(mariadb_lines(&index), ["12.3", "11.8", "10.11"]);
    }

    #[test]
    fn mongodb_tools_from_release_json() {
        let j: Value = serde_json::from_str(&format!(
            r#"{{"versions":[{{"version":"100.19.1","downloads":[
              {{"name":"ubuntu2204","arch":"x86_64","archive":{{"url":"https://x/u.tgz","sha256":"{u}"}}}},
              {{"name":"windows","arch":"x86_64","archive":{{"url":"https://fastdl.mongodb.org/tools/db/mongodb-database-tools-windows-x86_64-100.19.1.zip","sha256":"{w}"}}}}]}}]}}"#,
            u = "u".repeat(64),
            w = "w".repeat(64)
        ))
        .unwrap();
        let (v, url, sha) = mongodb_tools_zip(&j).unwrap();
        assert_eq!(v, "100.19.1");
        assert!(url.ends_with("windows-x86_64-100.19.1.zip"));
        assert_eq!(sha, "w".repeat(64));
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
