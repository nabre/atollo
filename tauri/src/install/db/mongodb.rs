//! MongoDB: the server of a production line (MongoDB's archives, conda-forge
//! on Linux, D-112), with mongosh and the Database Tools in their own
//! folders.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use serde_json::Value;

use super::mongodb_tools::{host_os_release, mongodb_tools_for_distro};
use super::remove_debug_symbols;
use crate::install::lang::version_key;
use crate::install::native::{conda_info, conda_versions, micromamba_create, micromamba_release};
use crate::install::{pattern, target_of, Ctx, Dest, Installer, Part, Progress, Release};
use crate::kit::{Arch, Os, Target};
use crate::net::archive::{Inner, Kind};
use crate::net::github_digest;
use crate::net::verify::Expect;
use crate::setup::catalog::Component;

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

/// The server on Linux comes from conda-forge (D-112): MongoDB's own
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
        self.after_conda(ctx, component, target, progress)?;
        let work = crate::install::work_dir(ctx, component)?;
        let result = release
            .parts
            .iter()
            .skip(1)
            .enumerate()
            .try_for_each(|(i, part)| {
                crate::install::install_part(
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

    fn conda_prefix(&self, target: Target, dir: &Path) -> Option<PathBuf> {
        mongodb_from_conda(target).then(|| dir.to_path_buf())
    }

    fn conda_programs(&self, target: Target) -> Option<&'static str> {
        mongodb_from_conda(target).then_some("mongodb")
    }

    /// The check of the server and the launchers of `mongod` and `mongos`
    /// (D-125): its data is in `data/db`, never in the environment, and
    /// mongosh and the tools have their own folders, which follow the kit
    /// (D-118).
    fn after_conda(
        &self,
        ctx: &Ctx,
        component: &Component,
        dir: &Path,
        _progress: &mut dyn Progress,
    ) -> Result<()> {
        if !crate::versions::has_key_for(target_of(ctx, component), component, dir) {
            bail!("bin/mongod not found after the installation");
        }
        crate::install::conda::write_launchers(dir, "mongodb")
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
            crate::install::web::copy_vc_runtime(ctx, &tree.join("bin"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/install/db/mongodb.rs"]
mod tests;
