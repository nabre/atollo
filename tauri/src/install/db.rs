//! Database servers and tools of the kit (DECISIONS.md D-73): MariaDB
//! (compatible with MySQL), MongoDB with mongosh, SQLite. Sources and
//! checks in docs/app/CATALOG.md §2.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, Result};
use serde_json::Value;

use super::{arch_of, pattern, target_of, Ctx, Dest, Installer, Part, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::github_digest;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

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

fn mariadb_release(ctx: &Ctx, target: Target, major: &str) -> Result<Release> {
    let latest = ctx
        .net
        .json::<Value>(&format!("{MARIADB_API}/{major}/latest/"))?;
    let (version, name, sha, kind) = mariadb_archive_for(target, &latest)?;
    Ok(Release::single(
        version.clone(),
        // The API lists http:// links: the same file over https.
        format!("{MARIADB_API}/{version}/{name}"),
        name,
        Expect::Sha256(sha),
        kind,
        Inner::Single,
    ))
}

/// Version, file name and SHA-256 of the Windows x64 zip in the answer of
/// `/<major>/latest/` (debug symbols excluded).
#[cfg(test)]
fn mariadb_zip(latest: &Value) -> Result<(String, String, String)> {
    let (v, n, s, _) = mariadb_archive_for(Target::WIN32_X64, latest)?;
    Ok((v, n, s))
}

/// Version, file name, SHA-256 and archive kind for `target` in the answer
/// of `/<major>/latest/`.
pub(crate) fn mariadb_archive_for(
    target: Target,
    latest: &Value,
) -> Result<(String, String, String, Kind)> {
    let releases = latest["releases"]
        .as_object()
        .ok_or_else(|| anyhow!("MariaDB: no releases"))?;
    let (version, release) = releases
        .iter()
        .max_by_key(|(v, _)| version_key(v))
        .ok_or_else(|| anyhow!("MariaDB: no releases"))?;
    let mut files = release["files"].as_array().into_iter().flatten();
    let (file, kind) = match target.os {
        Os::Win32 => {
            let f = files
                .find(|f| {
                    f["os"] == "Windows"
                        && f["cpu"] == "x86_64"
                        && f["package_type"] == "ZIP file"
                        && f["file_name"]
                            .as_str()
                            .is_some_and(|n| n.ends_with("-winx64.zip"))
                })
                .ok_or_else(|| anyhow!("MariaDB {version}: no Windows zip"))?;
            (f, Kind::Zip)
        }
        Os::Linux => {
            // Only Linux files (never another system's), the
            // `linux-systemd` build first (glibc, the one MariaDB
            // publishes today), debug symbols excluded (BUG-V3-04).
            let cpu = target.gnu_arch();
            let tarballs: Vec<&Value> = files
                .filter(|f| {
                    f["os"] == "Linux"
                        && f["cpu"] == cpu
                        && f["package_type"] == "gzipped tar file"
                        && f["file_name"]
                            .as_str()
                            .is_some_and(|n| n.ends_with(".tar.gz") && !n.contains("debugsymbols"))
                })
                .collect();
            let f = tarballs
                .iter()
                .find(|f| {
                    f["file_name"]
                        .as_str()
                        .is_some_and(|n| n.contains("-linux-systemd-"))
                })
                .or(tarballs.first())
                .copied()
                .ok_or_else(|| anyhow!("MariaDB {version}: no Linux {cpu} tar.gz"))?;
            (f, Kind::TarGz)
        }
        // The REST API lists Windows, Linux and source files only: no macOS
        // build (the catalog leaves MariaDB out of darwin targets).
        Os::Darwin => return Err(anyhow!("MariaDB {version}: no macOS build")),
    };
    let name = file["file_name"].as_str().unwrap_or_default().to_owned();
    let sha = file["checksum"]["sha256sum"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MariaDB {version}: no SHA-256"))?;
    Ok((version.clone(), name, sha.to_owned(), kind))
}

pub struct MariaDb;

impl Installer for MariaDb {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let major = mariadb_major(&ctx.net.json::<Value>(&format!("{MARIADB_API}/"))?)?;
        mariadb_release(ctx, target, &major)
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, _component: &Component) -> Result<Vec<String>> {
        Ok(mariadb_lines(
            &ctx.net.json::<Value>(&format!("{MARIADB_API}/"))?,
        ))
    }

    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        mariadb_release(ctx, target, line)
    }

    fn before_swap(&self, _ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        remove_debug_symbols(tree)
    }
}

/// The newest production release of a major version (`x.0.y`: minor
/// releases are rapid releases), its Windows x64 zip and SHA-256.
#[cfg(test)]
fn mongodb_zip(current: &Value, line: Option<&str>) -> Result<(String, String, String)> {
    let (v, u, s, _) = mongodb_archive_for(Target::WIN32_X64, current, line)?;
    Ok((v, u, s))
}

/// The newest production release of a major version for `target`, its
/// archive URL, SHA-256 and `Kind`.
pub(crate) fn mongodb_archive_for(
    target: Target,
    current: &Value,
    line: Option<&str>,
) -> Result<(String, String, String, Kind)> {
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
    let mut downloads = version["downloads"].as_array().into_iter().flatten();
    let (archive, kind) = match target.os {
        Os::Win32 => {
            let d = downloads
                .find(|d| {
                    d["target"] == "windows" && d["arch"] == "x86_64" && d["edition"] == "base"
                })
                .ok_or_else(|| anyhow!("MongoDB {number}: no Windows zip"))?;
            (&d["archive"], Kind::Zip)
        }
        Os::Darwin => {
            let arch = match target.arch {
                Arch::Arm64 => "arm64",
                Arch::X64 => "x86_64",
            };
            let d = downloads
                .find(|d| d["target"] == "macos" && d["arch"] == arch && d["edition"] == "base")
                .ok_or_else(|| anyhow!("MongoDB {number}: no macOS {arch} archive"))?;
            (&d["archive"], Kind::TarGz)
        }
        Os::Linux => {
            let arch = target.gnu_arch();
            let d = downloads
                .find(|d| {
                    d["edition"] == "base"
                        && d["arch"] == arch
                        && d["target"].as_str().is_some_and(|t| {
                            t == "ubuntu2204" || t == "ubuntu2404" || t.starts_with("ubuntu")
                        })
                })
                .ok_or_else(|| anyhow!("MongoDB {number}: no Linux {arch} archive"))?;
            (&d["archive"], Kind::TarGz)
        }
    };
    let url = archive["url"]
        .as_str()
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| anyhow!("MongoDB {number}: no https link"))?;
    let sha = archive["sha256"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MongoDB {number}: no SHA-256"))?;
    Ok((number, url.to_owned(), sha.to_owned(), kind))
}

/// The newest MongoDB Database Tools for Windows x64 from `release.json`
/// (version, https link, SHA-256).
#[cfg(test)]
fn mongodb_tools_zip(release: &Value) -> Result<(String, String, String)> {
    let (v, u, s, _) = mongodb_tools_for(Target::WIN32_X64, release)?;
    Ok((v, u, s))
}

/// The newest MongoDB Database Tools for `target` from `release.json`.
pub(crate) fn mongodb_tools_for(
    target: Target,
    release: &Value,
) -> Result<(String, String, String, Kind)> {
    let v = release["versions"]
        .as_array()
        .and_then(|a| a.first())
        .ok_or_else(|| anyhow!("MongoDB tools: no versions"))?;
    let number = v["version"].as_str().unwrap_or_default().to_owned();
    let mut downloads = v["downloads"].as_array().into_iter().flatten();
    let (archive, kind) = match target.os {
        Os::Win32 => {
            let d = downloads
                .find(|d| d["name"] == "windows" && d["arch"] == "x86_64")
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Windows zip"))?;
            (&d["archive"], Kind::Zip)
        }
        Os::Darwin => {
            let arch = match target.arch {
                Arch::Arm64 => "arm64",
                Arch::X64 => "x86_64",
            };
            let d = downloads
                .find(|d| d["name"] == "macos" && d["arch"] == arch)
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no macOS {arch} zip"))?;
            (&d["archive"], Kind::Zip)
        }
        Os::Linux => {
            let arch = match target.arch {
                Arch::Arm64 => "arm64",
                Arch::X64 => "x86_64",
            };
            let d = downloads
                .find(|d| {
                    (d["arch"] == arch || d["arch"] == target.gnu_arch())
                        && d["name"].as_str().is_some_and(|n| {
                            n == "ubuntu2204" || n == "ubuntu2404" || n.starts_with("ubuntu")
                        })
                })
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Linux {arch} archive"))?;
            (&d["archive"], Kind::TarGz)
        }
    };
    let url = archive["url"]
        .as_str()
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no https archive"))?;
    let sha = archive["sha256"]
        .as_str()
        .filter(|s| s.len() == 64)
        .ok_or_else(|| anyhow!("MongoDB tools {number}: no SHA-256"))?;
    Ok((number, url.to_owned(), sha.to_owned(), kind))
}

fn mongosh_asset_pattern(target: Target) -> (&'static str, Kind) {
    match (target.os, target.arch) {
        (Os::Win32, _) => (r"^mongosh-[\d.]+-win32-x64\.zip$", Kind::Zip),
        (Os::Darwin, Arch::Arm64) => (r"^mongosh-[\d.]+-darwin-arm64\.zip$", Kind::Zip),
        (Os::Darwin, Arch::X64) => (r"^mongosh-[\d.]+-darwin-x64\.zip$", Kind::Zip),
        (Os::Linux, Arch::X64) => (r"^mongosh-[\d.]+-linux-x64\.tgz$", Kind::TarGz),
        (Os::Linux, Arch::Arm64) => (r"^mongosh-[\d.]+-linux-arm64\.tgz$", Kind::TarGz),
    }
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
    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        let (version, url, sha, kind) =
            mongodb_archive_for(target, &ctx.net.json::<Value>(MONGODB_CURRENT)?, Some(line))?;
        let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
        Ok(Release {
            version,
            parts: vec![Part {
                url,
                file_name,
                expect: Expect::Sha256(sha),
                kind,
                inner: Inner::Single,
                dest: Dest::Own,
                sign: None,
            }],
        })
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let current = ctx.net.json::<Value>(MONGODB_CURRENT)?;
        let (version, url, sha, server_kind) = mongodb_archive_for(target, &current, None)?;
        let shell = ctx.net.github_latest("mongodb-js/mongosh")?;
        let (shell_pat, shell_kind) = mongosh_asset_pattern(target);
        let asset = shell
            .asset(&pattern(shell_pat)?)
            .ok_or_else(|| anyhow!("mongosh {}: no archive for {target}", shell.tag_name))?;
        let shell_version = shell.tag_name.trim_start_matches('v').to_owned();
        let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
        // mongodump and mongorestore: export and import of a project's
        // database (D-78).
        let (tools_version, tools_url, tools_sha, tools_kind) = mongodb_tools_for(
            target,
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
                    kind: server_kind,
                    inner: Inner::Single,
                    dest: Dest::Own,
                    sign: None,
                },
                Part {
                    url: asset.browser_download_url.clone(),
                    file_name: asset.name.clone(),
                    expect: Expect::Sha256(github_digest(asset)?),
                    kind: shell_kind,
                    inner: Inner::Single,
                    dest: Dest::Folder("mongosh"),
                    sign: None,
                },
                Part {
                    url: tools_url,
                    file_name: tools_file,
                    expect: Expect::Sha256(tools_sha),
                    kind: tools_kind,
                    inner: Inner::Single,
                    dest: Dest::Folder("mongodb-tools"),
                    sign: None,
                },
            ],
        })
    }

    /// mongod needs the VC++ runtime on Windows (the zip only has its installer):
    /// the kit's copy, as for Apache; no debug symbols, no installer.
    fn before_swap(&self, ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        remove_debug_symbols(tree)?;
        let _ = fs::remove_file(tree.join("bin").join("vc_redist.x64.exe"));
        if ctx.kit.os().is_windows() {
            super::web::copy_vc_runtime(ctx, &tree.join("bin"))?;
        }
        Ok(())
    }
}

/// From the `PRODUCT,<version>,<path>,<size>,<sha3-256>` lines of
/// sqlite.org's download page: the command-line tools for Windows `arch`.
#[cfg(test)]
fn sqlite_tools(page: &str, arch: &str) -> Result<(String, String, String)> {
    sqlite_tools_for(page, "win", arch)
}

/// From the `PRODUCT,<version>,<path>,<size>,<sha3-256>` lines of
/// sqlite.org's download page: the command-line tools for `os_tag` (`"win"`,
/// `"osx"`, `"linux"`) and `arch` (`"x64"`, `"arm64"`).
pub(crate) fn sqlite_tools_for(
    page: &str,
    os_tag: &str,
    arch: &str,
) -> Result<(String, String, String)> {
    let re = pattern(&format!(
        r"(?m)^PRODUCT,([\d.]+),(\d{{4}}/sqlite-tools-{os_tag}-{arch}-\d+\.zip),\d+,([0-9a-f]{{64}})\s*$"
    ))?;
    let c = re
        .captures(page)
        .ok_or_else(|| anyhow!("SQLite: no tools for {os_tag}-{arch} on the download page"))?;
    Ok((c[1].to_owned(), c[2].to_owned(), c[3].to_owned()))
}

pub struct Sqlite;

impl Installer for Sqlite {
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let os_tag = match target.os {
            Os::Win32 => "win",
            Os::Darwin => "osx",
            Os::Linux => "linux",
        };
        let arch = arch_of(ctx, component);
        let page = ctx.net.text("https://sqlite.org/download.html")?;
        let (version, path, sha3) = sqlite_tools_for(&page, os_tag, arch)
            .or_else(|_| sqlite_tools_for(&page, os_tag, "x64"))?;
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

/// phpMyAdmin's list of its releases.
const PMA_VERSIONS: &str = "https://www.phpmyadmin.net/home_page/version.json";

/// `(version, zip)` of the newest phpMyAdmin from `version.json`: the
/// build with every language (the interface follows the app's).
pub(crate) fn phpmyadmin_zip(versions: &Value) -> Result<(String, String)> {
    let version = versions
        .get("version")
        .and_then(Value::as_str)
        .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit() || c == '.'))
        .ok_or_else(|| anyhow!("phpMyAdmin: no version in version.json"))?;
    Ok((
        version.to_owned(),
        format!(
            "https://files.phpmyadmin.net/phpMyAdmin/{version}/phpMyAdmin-{version}-all-languages.zip"
        ),
    ))
}

/// phpMyAdmin (D-107): the release with every language, SHA-256 from the
/// `.sha256` file next to it; the folder is `phpmyadmin\<line>`.
pub struct PhpMyAdmin;

impl Installer for PhpMyAdmin {
    fn latest(&self, ctx: &Ctx, _component: &Component) -> Result<Release> {
        let (version, url) = phpmyadmin_zip(&ctx.net.json(PMA_VERSIONS)?)?;
        let name = url
            .rsplit('/')
            .next()
            .unwrap_or("phpMyAdmin.zip")
            .to_owned();
        let sha = crate::net::hash_in_list(&ctx.net.text(&format!("{url}.sha256"))?, &name)
            .ok_or_else(|| anyhow!("phpMyAdmin {version}: no SHA-256"))?;
        Ok(Release::single(
            version,
            url,
            name,
            Expect::Sha256(sha),
            Kind::Zip,
            Inner::Single,
        ))
    }

    /// Without its setup pages (the kit writes `config.inc.php`).
    fn before_swap(&self, _ctx: &Ctx, _c: &Component, tree: &Path, _old: &Path) -> Result<()> {
        let setup = tree.join("setup");
        if setup.is_dir() {
            fs::remove_dir_all(&setup)?;
        }
        let _ = fs::remove_file(tree.join("config.sample.inc.php"));
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/db.rs"]
mod tests;
