//! Database servers and tools of the kit (DECISIONS.md D-73): MariaDB
//! (compatible with MySQL), MongoDB with mongosh, SQLite. Sources and
//! checks in docs/app/CATALOG.md §2.

use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use super::native::{conda_info, conda_versions, micromamba_create, micromamba_release};
use super::{arch_of, pattern, target_of, Ctx, Dest, Installer, Part, Progress, Release};
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

/// The newest MongoDB Database Tools for `target` from `release.json`; on
/// Linux the build of the distribution of this PC, unknown here
/// (`mongodb_tools_linux_name` with no `os-release`).
#[cfg(test)]
pub(crate) fn mongodb_tools_for(
    target: Target,
    release: &Value,
) -> Result<(String, String, String, Kind)> {
    mongodb_tools_for_distro(target, release, None)
}

/// A Linux distribution as MongoDB names its builds: family and version
/// (`ubuntu` 24.04 → `["ubuntu", [24, 4]]`), from the text of
/// `/etc/os-release`. Derivatives follow their base: Linux Mint and Pop!_OS
/// by `UBUNTU_CODENAME`, Rocky, Alma, CentOS and Oracle Linux as RHEL,
/// Fedora as the newest RHEL, openSUSE Leap as SUSE. `None` for the others.
pub fn linux_distro(os_release: &str) -> Option<(&'static str, Vec<u64>)> {
    let field = |name: &str| {
        os_release.lines().find_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == name).then(|| v.trim().trim_matches(['"', '\'']).to_owned())
        })
    };
    let id = field("ID").unwrap_or_default().to_ascii_lowercase();
    let like = field("ID_LIKE").unwrap_or_default().to_ascii_lowercase();
    let like: Vec<&str> = like.split_whitespace().collect();
    let version = field("VERSION_ID")
        .map(|v| version_key(&v))
        .filter(|v| !v.is_empty());
    let ubuntu_codename = |name: &str| -> Option<Vec<u64>> {
        Some(match name {
            "focal" => vec![20, 4],
            "jammy" => vec![22, 4],
            "noble" => vec![24, 4],
            "resolute" => vec![26, 4],
            _ => return None,
        })
    };
    match id.as_str() {
        "ubuntu" => return Some(("ubuntu", version?)),
        "debian" => return Some(("debian", version?)),
        "rhel" | "centos" | "rocky" | "almalinux" | "ol" => return Some(("rhel", version?)),
        "fedora" => return Some(("rhel", vec![u64::MAX])),
        "amzn" => return Some(("amazon", version?)),
        "sles" | "sled" | "opensuse-leap" => return Some(("suse", version?)),
        _ => {}
    }
    if like.contains(&"ubuntu") {
        let codename = field("UBUNTU_CODENAME").unwrap_or_default();
        return ubuntu_codename(&codename).map(|v| ("ubuntu", v));
    }
    if like
        .iter()
        .any(|l| ["rhel", "centos", "fedora"].contains(l))
    {
        return Some(("rhel", version?));
    }
    None
}

/// The family and version of a MongoDB build name (`ubuntu2404` →
/// `("ubuntu", [24, 4])`, `debian92` → `[9, 2]`, `rhel88` → `[8, 8]`,
/// `rhel10` → `[10]`, `amazon` → `[1]`).
fn mongodb_build_version(name: &str) -> Option<(&'static str, Vec<u64>)> {
    for family in ["ubuntu", "debian", "rhel", "amazon", "suse"] {
        let Some(digits) = name.strip_prefix(family) else {
            continue;
        };
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let n: u64 = digits.parse().unwrap_or(1);
        let v = match family {
            "ubuntu" if digits.len() == 4 => vec![n / 100, n % 100],
            // `debian92`, `rhel88`; `debian12` and `rhel10` are majors.
            "debian" | "rhel" if digits.len() == 2 && n > 13 => vec![n / 10, n % 10],
            _ => vec![n],
        };
        return Some((family, v));
    }
    None
}

/// The build of the MongoDB Database Tools (a name of `release.json`:
/// `ubuntu2404`, `debian12`, `rhel93`...) for the distribution described by
/// `os_release`, among `names` (those of the right CPU): the newest of its
/// family not newer than it. When the distribution is unknown, or not
/// readable (a kit prepared from another system), the oldest Ubuntu build
/// from 20.04 on: built on the oldest glibc, it runs on the newer
/// distributions too (the tools are Go programs that need only glibc and
/// Kerberos, verified with `ldd` on 100.19.1).
pub fn mongodb_tools_linux_name<'a>(
    os_release: Option<&str>,
    names: &[&'a str],
) -> Option<&'a str> {
    let builds: Vec<(&str, &'static str, Vec<u64>)> = names
        .iter()
        .filter_map(|n| mongodb_build_version(n).map(|(f, v)| (*n, f, v)))
        .collect();
    if let Some((family, version)) = os_release.and_then(linux_distro) {
        let best = builds
            .iter()
            .filter(|(_, f, v)| *f == family && *v <= version)
            .max_by(|a, b| a.2.cmp(&b.2));
        if let Some((name, _, _)) = best {
            return Some(name);
        }
    }
    builds
        .iter()
        .filter(|(_, f, v)| *f == "ubuntu" && *v >= vec![20, 4])
        .min_by(|a, b| a.2.cmp(&b.2))
        .map(|(name, _, _)| *name)
}

/// The text of `/etc/os-release` (or `/usr/lib/os-release`) of this PC when
/// it runs the kit's Linux target; `None` elsewhere.
fn host_os_release(target: Target) -> Option<String> {
    if target.os != Os::Linux || !cfg!(target_os = "linux") {
        return None;
    }
    ["/etc/os-release", "/usr/lib/os-release"]
        .iter()
        .find_map(|p| fs::read_to_string(p).ok())
}

/// The newest MongoDB Database Tools for `target` from `release.json`, on
/// Linux for the distribution `os_release` (`mongodb_tools_linux_name`).
pub(crate) fn mongodb_tools_for_distro(
    target: Target,
    release: &Value,
    os_release: Option<&str>,
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
            // MongoDB says `arm64` for Ubuntu, `aarch64` for the others.
            let ours: Vec<&Value> = downloads
                .filter(|d| d["arch"] == arch || d["arch"] == target.gnu_arch())
                .collect();
            let names: Vec<&str> = ours.iter().filter_map(|d| d["name"].as_str()).collect();
            let name = mongodb_tools_linux_name(os_release, &names)
                .ok_or_else(|| anyhow!("MongoDB tools {number}: no Linux {arch} archive"))?;
            let d = ours
                .iter()
                .find(|d| d["name"] == name)
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

/// The mongosh archive for `target`, the download target of MongoDB
/// (`target_of`): x64 on Windows ARM64, where mongosh has no build and the
/// x64 one runs emulated (`x64Only`), like the server.
fn mongosh_asset_pattern(target: Target) -> (&'static str, Kind) {
    match (target.os, target.arch) {
        (Os::Win32, Arch::X64) => (r"^mongosh-[\d.]+-win32-x64\.zip$", Kind::Zip),
        (Os::Win32, Arch::Arm64) => (r"^mongosh-[\d.]+-win32-arm64\.zip$", Kind::Zip),
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

/// The production lines (`8.0`, `7.0`: minor 0, as `mongodb_lines`) of
/// conda-forge `mongodb` versions (newest first), each once; rapid
/// releases (`8.3.7`) are left out.
pub fn mongodb_conda_lines(versions: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for v in versions.iter().filter(|v| v.split('.').nth(1) == Some("0")) {
        if let Some(line) = crate::versions::line_of(v) {
            if !out.contains(&line) {
                out.push(line);
            }
        }
    }
    out
}

/// The newest conda-forge `mongodb` of a production line (of `line`, or of
/// any) among `versions` (newest first).
pub fn mongodb_conda_pick<'a>(versions: &'a [String], line: Option<&str>) -> Option<&'a String> {
    versions.iter().find(|v| {
        v.split('.').nth(1) == Some("0")
            && line.is_none_or(|l| crate::versions::line_of(v).as_deref() == Some(l))
    })
}

/// The server's version in a release version of MongoDB
/// (`8.0.23+mongosh2.5.9+tools100.19.1` → `8.0.23`).
pub fn mongodb_server_version(release_version: &str) -> &str {
    release_version.split('+').next().unwrap_or(release_version)
}

/// The server on Linux comes from conda-forge (WP-2.5): MongoDB's own
/// archives are built for one distribution each.
fn mongodb_from_conda(target: Target) -> bool {
    target.os == Os::Linux
}

/// The conda-forge server of `line` (or the newest) for `target`, as a
/// micromamba release.
fn mongodb_conda_release(ctx: &Ctx, target: Target, line: Option<&str>) -> Result<Release> {
    let subdir = target.conda_subdir();
    let versions = conda_versions(&conda_info(ctx, "mongodb")?, subdir);
    let version = mongodb_conda_pick(&versions, line)
        .ok_or_else(|| anyhow!("no conda-forge MongoDB {} for {subdir}", line.unwrap_or("")))?;
    micromamba_release(ctx, version)
}

/// mongosh (the generic build of each system: on Linux independent of the
/// distribution) and the Database Tools (on Linux the build of this PC's
/// distribution, `mongodb_tools_linux_name`), each into its own folder,
/// with the `+mongosh<v>+tools<v>` of the release version.
fn mongodb_shared_parts(ctx: &Ctx, target: Target) -> Result<(Vec<Part>, String)> {
    let shell = ctx.net.github_latest("mongodb-js/mongosh")?;
    let (shell_pat, shell_kind) = mongosh_asset_pattern(target);
    let asset = shell
        .asset(&pattern(shell_pat)?)
        .ok_or_else(|| anyhow!("mongosh {}: no archive for {target}", shell.tag_name))?;
    let shell_version = shell.tag_name.trim_start_matches('v').to_owned();
    // mongodump and mongorestore: export and import of a project's
    // database (D-78).
    let (tools_version, tools_url, tools_sha, tools_kind) = mongodb_tools_for_distro(
        target,
        &ctx.net
            .json::<Value>("https://downloads.mongodb.org/tools/db/release.json")?,
        host_os_release(target).as_deref(),
    )?;
    let tools_file = tools_url
        .rsplit('/')
        .next()
        .unwrap_or("mongodb-tools.zip")
        .to_owned();
    let parts = vec![
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
    ];
    Ok((
        parts,
        format!("+mongosh{shell_version}+tools{tools_version}"),
    ))
}

pub struct MongoDb;

impl Installer for MongoDb {
    /// micromamba for the server on Linux, the shared path elsewhere.
    fn custom(&self, target: Target) -> bool {
        mongodb_from_conda(target)
    }

    /// A conda environment cannot follow the kit (D-92).
    fn relocatable(&self, target: Target) -> bool {
        !mongodb_from_conda(target)
    }

    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
        self.install_custom_at(ctx, component, release, &dir, progress)
    }

    /// The server's environment in the folder of its line (`bin/mongod`,
    /// the catalog's key), then mongosh and the tools (the other parts of a
    /// release of `latest`; a saved line usually has only the server).
    fn install_custom_at(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        target: &Path,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let spec = format!("mongodb={}", mongodb_server_version(&release.version));
        micromamba_create(ctx, component, release, target, &[&spec], progress)?;
        if !crate::versions::has_key_for(target_of(ctx, component), component, target) {
            bail!("bin/mongod not found after the installation");
        }
        let work = super::work_dir(ctx, component)?;
        let result = release
            .parts
            .iter()
            .skip(1)
            .enumerate()
            .try_for_each(|(i, part)| {
                super::install_part(
                    ctx,
                    self,
                    component,
                    part,
                    Some(target),
                    &work.path().join(i.to_string()),
                    progress,
                )
            });
        drop(work);
        result
    }

    fn has_lines(&self) -> bool {
        true
    }

    fn lines(&self, ctx: &Ctx, component: &Component) -> Result<Vec<String>> {
        let target = target_of(ctx, component);
        if mongodb_from_conda(target) {
            let versions = conda_versions(&conda_info(ctx, "mongodb")?, target.conda_subdir());
            return Ok(mongodb_conda_lines(&versions));
        }
        Ok(mongodb_lines(&ctx.net.json::<Value>(MONGODB_CURRENT)?))
    }

    /// The server of `line` only: mongosh and the tools are shared, at their
    /// latest (they support every server in production). On Linux, where
    /// `install::release_at` does not add them to a custom installer's
    /// release, they come with it while they are not on disk (a first
    /// installation at a line).
    fn release_line(&self, ctx: &Ctx, component: &Component, line: &str) -> Result<Release> {
        let target = target_of(ctx, component);
        if mongodb_from_conda(target) {
            let mut release = mongodb_conda_release(ctx, target, Some(line))?;
            let shared_missing = component
                .keys_for(target)
                .iter()
                .any(|k| !crate::fsx::join_rel(ctx.kit.apps(), k).is_file());
            if shared_missing {
                release.parts.extend(mongodb_shared_parts(ctx, target)?.0);
            }
            return Ok(release);
        }
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

    /// The server, mongosh (the generic build of each system, also on
    /// Linux) and the Database Tools (on Linux the build of this PC's
    /// distribution, `mongodb_tools_linux_name`). On Linux the server's part
    /// is micromamba, which creates its conda-forge environment.
    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        let (version, server) = if mongodb_from_conda(target) {
            let mut conda = mongodb_conda_release(ctx, target, None)?;
            (conda.version, conda.parts.remove(0))
        } else {
            let current = ctx.net.json::<Value>(MONGODB_CURRENT)?;
            let (version, url, sha, server_kind) = mongodb_archive_for(target, &current, None)?;
            let file_name = url.rsplit('/').next().unwrap_or("mongodb.zip").to_owned();
            let part = Part {
                url,
                file_name,
                expect: Expect::Sha256(sha),
                kind: server_kind,
                inner: Inner::Single,
                dest: Dest::Own,
                sign: None,
            };
            (version, part)
        };
        let (shared, suffix) = mongodb_shared_parts(ctx, target)?;
        let mut parts = vec![server];
        parts.extend(shared);
        Ok(Release {
            version: format!("{version}{suffix}"),
            parts,
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

/// The other build of the SQLite tools to take when sqlite.org has none
/// for `target`: x64 on Windows ARM64, which runs it emulated. Nowhere
/// else: an x64 binary does not start on Linux ARM64 (no silent fallback,
/// D-108; SQLite comes from conda-forge there).
pub(crate) fn sqlite_fallback_arch(target: Target) -> Option<&'static str> {
    (target == Target::WIN32_ARM64).then_some("x64")
}

/// sqlite.org publishes no command-line tools for Linux ARM64: there SQLite
/// is conda-forge `sqlite` (WP-2.5).
fn sqlite_from_conda(target: Target) -> bool {
    target == Target::LINUX_ARM64
}

/// The catalog's key of SQLite off Windows is `sqlite/sqlite3`, the file
/// of sqlite.org's zip; in a conda environment the program is
/// `bin/sqlite3`, which finds its libraries from there (`$ORIGIN/../lib`):
/// this launcher at the key starts it.
pub const SQLITE_CONDA_LAUNCHER: &str =
    "#!/bin/sh\n# Dev-Kit: sqlite3 of the conda-forge environment next to this file\nexec \"$(dirname \"$0\")/bin/sqlite3\" \"$@\"\n";

pub struct Sqlite;

impl Installer for Sqlite {
    /// micromamba on Linux ARM64, the shared path elsewhere.
    fn custom(&self, target: Target) -> bool {
        sqlite_from_conda(target)
    }

    /// A conda environment cannot follow the kit (D-92).
    fn relocatable(&self, target: Target) -> bool {
        !sqlite_from_conda(target)
    }

    /// The environment in the folder of its line, and the launcher at the
    /// catalog's key.
    fn install_custom(
        &self,
        ctx: &Ctx,
        component: &Component,
        release: &Release,
        progress: &mut dyn Progress,
    ) -> Result<()> {
        let dir = crate::versions::install_dir(ctx.kit, component, &release.version);
        let spec = format!("sqlite={}", release.version);
        micromamba_create(ctx, component, release, &dir, &[&spec], progress)?;
        if !dir.join("bin").join("sqlite3").is_file() {
            bail!("bin/sqlite3 not found after the installation");
        }
        let launcher = dir.join("sqlite3");
        crate::fsx::write_atomic(&launcher, SQLITE_CONDA_LAUNCHER.as_bytes())?;
        crate::router::fs::set_executable(&launcher)?;
        Ok(())
    }

    fn latest(&self, ctx: &Ctx, component: &Component) -> Result<Release> {
        let target = target_of(ctx, component);
        if sqlite_from_conda(target) {
            let subdir = target.conda_subdir();
            let versions = conda_versions(&conda_info(ctx, "sqlite")?, subdir);
            let version = versions
                .first()
                .ok_or_else(|| anyhow!("no conda-forge sqlite for {subdir}"))?;
            return micromamba_release(ctx, version);
        }
        let os_tag = match target.os {
            Os::Win32 => "win",
            Os::Darwin => "osx",
            Os::Linux => "linux",
        };
        let arch = arch_of(ctx, component);
        let page = ctx.net.text("https://sqlite.org/download.html")?;
        let (version, path, sha3) = sqlite_tools_for(&page, os_tag, arch).or_else(|e| {
            match sqlite_fallback_arch(target) {
                Some(other) => sqlite_tools_for(&page, os_tag, other),
                None => Err(e),
            }
        })?;
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
