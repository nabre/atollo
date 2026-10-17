//! New versions of `Atollo.exe` (DECISIONS.md D-20), from the `app-v*`
//! releases of nabre/atollo.
//!
//! - check: the newest `app-vX.Y.Z` release (no drafts or pre-releases)
//!   newer than this exe, with the asset `Atollo-<x64|arm64>.exe`;
//! - stage: download to `data\app\update\Atollo.exe`, SHA-256 from GitHub
//!   (the asset digest, or `Atollo-<arch>.exe.sha256` in the release), then
//!   `update.json` with the version: only a complete, verified file is staged;
//! - swap, when the app closes: `Atollo.exe` → `Atollo.old.exe` (Windows
//!   lets a running exe be renamed), the staged file in its place; the old
//!   one is deleted at the next start. A failed move puts the old one back.
//!
//! No installer, no admin rights: only files next to the exe.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, PoisonError};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::fsx::{read_text, write_atomic};
use crate::kit::Kit;
use crate::net::verify::{self, Expect};
use crate::net::{GhAsset, Net};

pub const REPO: &str = "nabre/atollo";
const TAG_PREFIX: &str = "app-v";

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub current: String,
    /// Newest version published, once checked.
    pub latest: Option<String>,
    /// Downloaded and verified: swapped when the app closes.
    pub staged: Option<String>,
    pub checking: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Staged {
    version: String,
    sha256: String,
}

/// `1.2.3` from `app-v1.2.3` (or `1.2.3`), as numbers for comparing.
pub fn parse_version(text: &str) -> Option<(u64, u64, u64)> {
    let v = text.strip_prefix(TAG_PREFIX).unwrap_or(text);
    let mut parts = v.split('.');
    let n = |p: Option<&str>| p.and_then(|s| s.parse().ok());
    let out = (n(parts.next())?, n(parts.next())?, n(parts.next())?);
    parts.next().is_none().then_some(out)
}

pub fn asset_name(kit: &Kit) -> String {
    match kit.os() {
        crate::kit::Os::Win32 => format!("Atollo-{}.exe", kit.arch().as_str()),
        crate::kit::Os::Darwin => format!("Atollo-{}.app.tar.gz", kit.target().as_str()),
        crate::kit::Os::Linux => format!("Atollo-{}.AppImage", kit.target().as_str()),
    }
}

fn update_dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("update")
}

fn staged_exe(kit: &Kit) -> PathBuf {
    update_dir(kit).join("Atollo.exe")
}

fn staged_info(kit: &Kit) -> PathBuf {
    update_dir(kit).join("update.json")
}

fn read_staged(kit: &Kit) -> Option<Staged> {
    let info: Staged = crate::fsx::read_json(&staged_info(kit))?;
    staged_exe(kit).is_file().then_some(info)
}

/// The newest release newer than `current`, with this kit's exe.
fn newest<'a>(releases: &'a [Release], current: &str, asset: &str) -> Option<&'a Release> {
    let now = parse_version(current)?;
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease && r.tag_name.starts_with(TAG_PREFIX))
        .filter_map(|r| parse_version(&r.tag_name).map(|v| (v, r)))
        .filter(|(v, r)| *v > now && r.assets.iter().any(|a| a.name == asset))
        .max_by_key(|(v, _)| *v)
        .map(|(_, r)| r)
}

/// SHA-256 of the asset: GitHub's digest, or the `.sha256` file next to it.
fn expected_hash(net: &Net, release: &Release, asset: &GhAsset) -> Result<String> {
    if let Ok(hash) = crate::net::github_digest(asset) {
        return Ok(hash);
    }
    let sums = format!("{}.sha256", asset.name);
    let file = release
        .assets
        .iter()
        .find(|a| a.name == sums)
        .ok_or_else(|| anyhow!("the release has no SHA-256 for {}", asset.name))?;
    let text = net.text(&file.browser_download_url)?;
    crate::net::first_sha256(&text).ok_or_else(|| anyhow!("{sums}: no SHA-256 inside"))
}

pub struct Updater {
    kit: Kit,
    status: Mutex<Status>,
}

impl Updater {
    pub fn new(kit: Kit) -> Updater {
        let staged = read_staged(&kit)
            .map(|s| s.version)
            .filter(|v| parse_version(v) > parse_version(env!("CARGO_PKG_VERSION")));
        Updater {
            kit,
            status: Mutex::new(Status {
                current: env!("CARGO_PKG_VERSION").to_owned(),
                staged,
                ..Status::default()
            }),
        }
    }

    pub fn status(&self) -> Status {
        self.status
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn set(&self, f: impl FnOnce(&mut Status)) {
        f(&mut self.status.lock().unwrap_or_else(PoisonError::into_inner));
    }

    /// Looks for a newer release and stages it. Blocking (network).
    pub fn check(&self, net: &Net) -> Status {
        {
            let mut s = self.status.lock().unwrap_or_else(PoisonError::into_inner);
            if s.checking {
                return s.clone();
            }
            s.checking = true;
            s.error = None;
        }
        let result = self.check_and_stage(net);
        self.set(|s| {
            s.checking = false;
            match result {
                Ok((latest, staged)) => {
                    s.latest = Some(latest);
                    if staged.is_some() {
                        s.staged = staged;
                    }
                }
                Err(e) => {
                    crate::log::warn(&format!("update check: {e:#}"));
                    s.error = Some(format!("{e:#}"));
                }
            }
        });
        self.status()
    }

    /// (newest version published, version staged now if any).
    fn check_and_stage(&self, net: &Net) -> Result<(String, Option<String>)> {
        let releases: Vec<Release> = net.json(&format!(
            "https://api.github.com/repos/{REPO}/releases?per_page=30"
        ))?;
        let current = env!("CARGO_PKG_VERSION");
        let asset_name = asset_name(&self.kit);
        let Some(release) = newest(&releases, current, &asset_name) else {
            return Ok((current.to_owned(), None));
        };
        let version = release.tag_name.trim_start_matches(TAG_PREFIX).to_owned();
        if read_staged(&self.kit).is_some_and(|s| s.version == version) {
            return Ok((version.clone(), Some(version)));
        }
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| anyhow!("{asset_name} missing"))?;
        let hash = expected_hash(net, release, asset)?;
        let dir = update_dir(&self.kit);
        let _ = fs::remove_file(staged_info(&self.kit));
        fs::create_dir_all(&dir)?;
        let exe = staged_exe(&self.kit);
        let cancel = AtomicBool::new(false);
        net.download(&asset.browser_download_url, &exe, &cancel, &mut |_, _| {})?;
        verify::check(&exe, &Expect::Sha256(hash.clone()))?;
        let info = Staged {
            version: version.clone(),
            sha256: hash,
        };
        crate::fsx::write_json(&staged_info(&self.kit), &info)?;
        crate::log::info(&format!("update {version} staged"));
        Ok((version.clone(), Some(version)))
    }
}

/// When the app closes: the staged exe takes the place of `exe`. Checked
/// again (hash) right before, so a file changed meanwhile is never used.
pub fn apply_staged(kit: &Kit, exe: &Path) -> Result<bool> {
    let Some(info) = read_staged(kit) else {
        return Ok(false);
    };
    if parse_version(&info.version) <= parse_version(env!("CARGO_PKG_VERSION")) {
        discard(kit);
        return Ok(false);
    }
    let staged = staged_exe(kit);
    if !verify::sha256_hex(&staged)?.eq_ignore_ascii_case(&info.sha256) {
        discard(kit);
        bail!("staged update changed on disk: discarded");
    }
    let old = exe.with_file_name("Atollo.old.exe");
    let _ = fs::remove_file(&old);
    fs::rename(exe, &old).with_context(|| format!("{} → {}", exe.display(), old.display()))?;
    if let Err(e) = fs::rename(&staged, exe).or_else(|_| fs::copy(&staged, exe).map(|_| ())) {
        // Never leave the kit without its exe.
        let _ = fs::rename(&old, exe);
        return Err(e).with_context(|| exe.display().to_string());
    }
    discard(kit);
    crate::log::info(&format!("updated to {}", info.version));
    Ok(true)
}

fn discard(kit: &Kit) {
    let _ = fs::remove_file(staged_exe(kit));
    let _ = fs::remove_file(staged_info(kit));
}

/// The app's version among the components' ones (`data\app\versions\app.txt`).
pub fn record_version(kit: &Kit) {
    let path = kit.versions_root().join("app.txt");
    if read_text(&path).ok().as_deref().map(str::trim) != Some(env!("CARGO_PKG_VERSION")) {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        if let Err(e) = write_atomic(&path, env!("CARGO_PKG_VERSION").as_bytes()) {
            crate::log::warn(&format!("{}: {e}", path.display()));
        }
    }
}

/// At start: the exe replaced by the last update. Still in use (a `sync` or
/// `keyring` of the old version running): tried again next time.
pub fn remove_old(exe: &Path) {
    let old = exe.with_file_name("Atollo.old.exe");
    if old.exists() {
        if let Err(e) = fs::remove_file(&old) {
            crate::log::warn(&format!("{}: {e}", old.display()));
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/self_update.rs"]
mod tests;
