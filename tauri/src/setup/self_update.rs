//! New versions of the app (DECISIONS.md D-20), from the `app-v*` releases
//! of nabre/atollo2.
//!
//! - check: the newest `app-vX.Y.Z` release (no drafts or pre-releases)
//!   newer than this app, with this system's asset ([`asset_name`]);
//! - stage: download, SHA-256 from GitHub (the asset digest, or
//!   `<asset>.sha256` in the release), then `update.json` with the version:
//!   only a complete, verified copy is staged;
//! - swap, when the app closes ([`SwapPlan`]): the running copy moves to its
//!   `old` name, the staged one takes its place; the old one is deleted at
//!   the next start. A failed move puts the old one back.
//!
//! What is staged and swapped, per system (D-113):
//!
//! - Windows: `data\app\update\Atollo.exe`; `Atollo.exe` → `Atollo.old.exe`
//!   (Windows lets a running exe be renamed);
//! - macOS: the `.app` of the zip, extracted next to the running bundle as
//!   `Atollo.app.new`; `Atollo.app` → `Atollo.app.old`;
//! - Linux: the AppImage as `Atollo.AppImage.new` (mode 755) next to the
//!   running one; `Atollo.AppImage` → `Atollo.AppImage.old`.
//!
//! The release names carry the version (`Atollo-0.3.0-win32-x64.exe`,
//! [`asset_name`]). A running copy with such a name takes the new
//! version's name at the swap (`Atollo-0.2.0-…` → `Atollo-0.3.0-…`); its
//! `old` name has no version (`Atollo-linux-x64.AppImage.old`), so the
//! next start finds it. Any other name stays: a renamed bundle or AppImage
//! keeps its name. A plain binary on macOS or Linux (`cargo run`, the
//! tar.gz) does not replace itself: the check says so.
//!
//! No installer, no admin rights: only files next to the app.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, PoisonError};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::common::fsx::{read_text, write_atomic};
use crate::kit::{Kit, Os};
use crate::net::archive::{self, Kind};
use crate::net::verify::{self, Expect};
use crate::net::{GhAsset, Net};

pub const REPO: &str = "nabre/atollo";
pub use crate::kit::release::parse_version;
use crate::kit::release::{release_rest, TAG_PREFIX};

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
    /// `latest` is newer, but this copy cannot replace itself (a plain
    /// binary off Windows: the tar.gz, `cargo run`): download it from the
    /// release by hand. Not an error.
    pub manual: bool,
}

/// What a check found.
#[derive(Debug, PartialEq, Eq)]
enum Found {
    /// Nothing newer than this version (the version itself).
    UpToDate(String),
    /// Newer, downloaded and verified: swapped at exit.
    Staged(String),
    /// Newer, to download by hand (no copy that replaces itself).
    Manual(String),
}

/// The status after a check, from what it found or why it failed.
fn record(s: &mut Status, result: Result<Found>) {
    s.checking = false;
    match result {
        Ok(found) => {
            s.error = None;
            s.manual = matches!(found, Found::Manual(_));
            let (Found::UpToDate(v) | Found::Staged(v) | Found::Manual(v)) = &found;
            s.latest = Some(v.clone());
            if let Found::Staged(v) = found {
                s.staged = Some(v);
            }
        }
        Err(e) => {
            crate::common::log::warn(&format!("update check: {e:#}"));
            s.error = Some(format!("{e:#}"));
        }
    }
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
    /// SHA-256 of the staged file, or [`tree_sha256`] of a staged bundle.
    sha256: String,
    /// macOS and Linux: the staged copy's name in the kit root
    /// (`Atollo.app.new`). Absent on Windows (and in the files of earlier
    /// versions): `data\app\update\Atollo.exe`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

/// The release asset of this kit's system for `version`:
/// `Atollo-<version>-<target>.exe` (Windows), `.app.zip` (macOS),
/// `.AppImage` (Linux). The `.tar.gz` of Linux, named the same way, is
/// downloaded by hand.
pub fn asset_name(kit: &Kit, version: &str) -> String {
    let ext = match kit.os() {
        Os::Win32 => "exe",
        Os::Darwin => "app.zip",
        Os::Linux => "AppImage",
    };
    format!("Atollo-{version}-{}.{ext}", kit.target().as_str())
}

/// `name` without its version (`Atollo-linux-x64.AppImage`); any other
/// name as it is.
fn unversioned(name: &str) -> String {
    release_rest(name).map_or_else(|| name.to_owned(), |rest| format!("Atollo-{rest}"))
}

/// Where the new copy goes at the swap: the running copy's place, under
/// `version`'s name when the running copy has a release name, unless a
/// file of that name is already there.
fn destination(current: &Path, version: &str) -> PathBuf {
    let renamed = current
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(release_rest)
        .map(|rest| current.with_file_name(format!("Atollo-{version}-{rest}")));
    match renamed {
        Some(p) if p != current && fs::symlink_metadata(&p).is_err() => p,
        _ => current.to_path_buf(),
    }
}

/// What the app is made of on disk, i.e. what an update replaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// `Atollo.exe` (Windows).
    Exe,
    /// `Atollo.app`, a folder (macOS).
    Bundle,
    /// `Atollo.AppImage`, one file (Linux).
    AppImage,
}

/// How a staged update replaces the running app when it closes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwapPlan {
    pub form: Form,
    /// The running copy: the exe, the bundle or the AppImage.
    pub current: PathBuf,
    /// The verified new copy, ready before the app closes.
    pub staged: PathBuf,
    /// Where `current` moves; deleted at the next start.
    pub old: PathBuf,
}

/// The swap for the program at `exe` on `os` (pure: the tests cover the
/// three systems from any of them). `image` is the AppImage the app runs
/// from on Linux (`$APPIMAGE`, [`crate::kit::app_exe`]); `update_dir` holds
/// the staged exe on Windows. `None`: this copy cannot replace itself (a
/// plain binary on macOS or Linux).
pub fn swap_plan(os: Os, exe: &Path, image: Option<&Path>, update_dir: &Path) -> Option<SwapPlan> {
    let beside = |current: &Path, form: Form| {
        let name = current.file_name()?.to_str()?;
        Some(SwapPlan {
            form,
            current: current.to_path_buf(),
            staged: current.with_file_name(format!("{name}.new")),
            old: current.with_file_name(format!("{}.old", unversioned(name))),
        })
    };
    match os {
        Os::Win32 => Some(SwapPlan {
            form: Form::Exe,
            current: exe.to_path_buf(),
            staged: update_dir.join("Atollo.exe"),
            old: exe.with_file_name("Atollo.old.exe"),
        }),
        Os::Darwin => beside(app_bundle(exe)?, Form::Bundle),
        Os::Linux => beside(image.filter(|i| i.has_root())?, Form::AppImage),
    }
}

/// The `*.app` bundle whose program is `exe` (`X.app/Contents/MacOS/x`),
/// the same rule as the kit root's (`kit::root_from`).
fn app_bundle(exe: &Path) -> Option<&Path> {
    let macos = exe.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    let named = |p: &Path, n: &str| p.file_name().is_some_and(|f| f == n);
    let is_app = bundle
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("app"));
    (named(macos, "MacOS") && named(contents, "Contents") && is_app).then_some(bundle)
}

/// The AppImage the program at `exe` runs from (Linux): `$APPIMAGE`, when
/// `exe` is inside the image's mount (`$APPDIR`), the rule of the kit root.
fn running_image(exe: &Path) -> Option<PathBuf> {
    if !Os::current().is_linux() {
        return None;
    }
    let image = std::env::var_os("APPIMAGE")?;
    let appdir = std::env::var_os("APPDIR")?;
    (!image.is_empty() && !appdir.is_empty() && exe.starts_with(&appdir))
        .then(|| PathBuf::from(image))
}

/// The plan for this kit's system and the program at `exe`.
fn plan_for(kit: &Kit, exe: &Path) -> Option<SwapPlan> {
    swap_plan(
        kit.os(),
        exe,
        running_image(exe).as_deref(),
        &update_dir(kit),
    )
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

/// The staged copy named by `info`: only a plain `*.new` name in the kit
/// root is ever used (the file could have been edited).
fn staged_path(kit: &Kit, info: &Staged) -> Option<PathBuf> {
    match info.file.as_deref() {
        None => Some(staged_exe(kit)),
        Some(name) => {
            let plain = Path::new(name).file_name().is_some_and(|f| f == name)
                && !name.contains(['/', '\\'])
                && name.len() > ".new".len()
                && name.ends_with(".new");
            plain.then(|| kit.root().join(name))
        }
    }
}

fn read_staged(kit: &Kit) -> Option<(Staged, PathBuf)> {
    let info: Staged = crate::common::fsx::read_json(&staged_info(kit))?;
    let path = staged_path(kit, &info)?;
    let there = fs::symlink_metadata(&path)
        .is_ok_and(|m| !m.file_type().is_symlink() && (m.is_file() || m.is_dir()));
    there.then_some((info, path))
}

/// The newest release newer than `current`, with this kit's asset
/// (`asset` names it for a release's version).
fn newest<'a>(
    releases: &'a [Release],
    current: &str,
    asset: impl Fn(&str) -> String,
) -> Option<&'a Release> {
    let now = parse_version(current)?;
    releases
        .iter()
        .filter(|r| !r.draft && !r.prerelease && r.tag_name.starts_with(TAG_PREFIX))
        .filter_map(|r| parse_version(&r.tag_name).map(|v| (v, r)))
        .filter(|(v, r)| {
            let name = asset(r.tag_name.trim_start_matches(TAG_PREFIX));
            *v > now && r.assets.iter().any(|a| a.name == name)
        })
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

/// SHA-256 of a folder tree: every entry, sorted by path, with its kind,
/// its path (`/`-separated), the executable bit (Unix) and its content or
/// link target. A staged bundle is checked with it right before the swap.
/// The Finder's and macOS's own sidecar files are left out: `.DS_Store`,
/// and the `._name` files that hold extended attributes on exFAT and FAT,
/// which macOS may add between staging and swap.
pub fn tree_sha256(root: &Path) -> Result<String> {
    fn walk(root: &Path, dir: &Path, hasher: &mut Sha256) -> Result<()> {
        let mut entries = fs::read_dir(dir)
            .with_context(|| dir.display().to_string())?
            .collect::<std::io::Result<Vec<_>>>()?;
        entries.retain(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            !(name.starts_with("._") || name == ".DS_Store")
        });
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let path = entry.path();
            let rel = path
                .strip_prefix(root)?
                .components()
                .map(|c| c.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let meta = fs::symlink_metadata(&path)?;
            let kind = meta.file_type();
            if kind.is_symlink() {
                let target = fs::read_link(&path)?;
                hasher.update(format!("L {rel}\0{}\0", target.to_string_lossy()).as_bytes());
            } else if kind.is_dir() {
                hasher.update(format!("D {rel}\0").as_bytes());
                walk(root, &path, hasher)?;
            } else {
                let content = verify::sha256_hex(&path)?;
                hasher.update(format!("F {rel}\0{}\0{content}\0", executable(&meta)).as_bytes());
            }
        }
        Ok(())
    }
    let mut hasher = Sha256::new();
    walk(root, root, &mut hasher)?;
    Ok(crate::common::hex(&hasher.finalize()))
}

/// 1 when the file has an execute bit (macOS, Linux), 0 on Windows.
fn executable(meta: &fs::Metadata) -> u8 {
    u8::from(crate::platform::files::any_exec_bit(meta))
}

/// The digest recorded for a staged copy and checked again at the swap.
fn staged_digest(form: Form, path: &Path) -> Result<String> {
    match form {
        Form::Bundle => tree_sha256(path),
        Form::Exe | Form::AppImage => verify::sha256_hex(path),
    }
}

/// `rwxr-xr-x`: the AppImage must run (Linux). Nothing on Windows.
fn set_mode_755(path: &Path) -> Result<()> {
    crate::platform::files::set_executable(path).with_context(|| path.display().to_string())
}

/// Removes a file or a folder (never through a link: a link itself goes).
fn remove_any(path: &Path) -> std::io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(m) if m.is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

/// The only `*.app` folder at the top of an extracted zip, with its
/// `Contents/MacOS` program folder.
fn bundle_in(dir: &Path) -> Result<PathBuf> {
    let mut apps = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        let is_app = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("app"));
        if is_app && fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir()) {
            apps.push(path);
        }
    }
    match apps.as_slice() {
        [app] if app.join("Contents").join("MacOS").is_dir() => Ok(app.clone()),
        [app] => bail!("{}: no Contents/MacOS", app.display()),
        _ => bail!("the update zip holds {} .app bundles, not one", apps.len()),
    }
}

/// Turns the verified download into the staged copy of `plan` and returns
/// its digest: the exe stays where it was downloaded (Windows), the
/// AppImage moves next to the running one with mode 755 (Linux), the zip's
/// bundle is extracted next to the running one (macOS).
fn stage(plan: &SwapPlan, download: &Path, work: &Path) -> Result<String> {
    match plan.form {
        Form::Exe => {}
        Form::AppImage => {
            remove_any(&plan.staged)?;
            fs::rename(download, &plan.staged)
                .or_else(|_| fs::copy(download, &plan.staged).map(|_| ()))
                .with_context(|| plan.staged.display().to_string())?;
            let _ = fs::remove_file(download);
            set_mode_755(&plan.staged)?;
        }
        Form::Bundle => {
            let _ = remove_any(work);
            archive::extract(download, Kind::Zip, work)?;
            let app = bundle_in(work)?;
            remove_any(&plan.staged)?;
            if fs::rename(&app, &plan.staged).is_err() {
                archive::copy_tree(&app, &plan.staged)
                    .with_context(|| plan.staged.display().to_string())?;
            }
            let _ = remove_any(work);
            let _ = fs::remove_file(download);
        }
    }
    staged_digest(plan.form, &plan.staged)
}

pub struct Updater {
    kit: Kit,
    status: Mutex<Status>,
}

impl Updater {
    pub fn new(kit: Kit) -> Updater {
        let staged = read_staged(&kit)
            .map(|(s, _)| s.version)
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
        // A panic is a failed check: `checking` must not stay set.
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.check_and_stage(net)))
                .unwrap_or_else(|p| Err(anyhow!(super::task::panic_text(&*p))));
        self.set(|s| record(s, result));
        self.status()
    }

    fn check_and_stage(&self, net: &Net) -> Result<Found> {
        let releases: Vec<Release> = net.json(&format!(
            "https://api.github.com/repos/{REPO}/releases?per_page=30"
        ))?;
        let current = env!("CARGO_PKG_VERSION");
        let Some(release) = newest(&releases, current, |v| asset_name(&self.kit, v)) else {
            return Ok(Found::UpToDate(current.to_owned()));
        };
        let version = release.tag_name.trim_start_matches(TAG_PREFIX).to_owned();
        let asset_name = asset_name(&self.kit, &version);
        if read_staged(&self.kit).is_some_and(|(s, _)| s.version == version) {
            return Ok(Found::Staged(version));
        }
        let exe = std::env::current_exe().context("the app's own path")?;
        // Only Atollo.app (macOS) and the AppImage (Linux) replace themselves
        // off Windows: a plain binary learns of the version all the same.
        let Some(plan) = plan_for(&self.kit, &exe) else {
            crate::common::log::info(&format!(
                "update {version} is out: this copy does not replace itself, download it"
            ));
            return Ok(Found::Manual(version));
        };
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| anyhow!("{asset_name} missing"))?;
        let hash = expected_hash(net, release, asset)?;
        let dir = update_dir(&self.kit);
        discard(&self.kit);
        fs::create_dir_all(&dir)?;
        // Windows downloads straight to the staged exe, as before.
        let download = match plan.form {
            Form::Exe => plan.staged.clone(),
            Form::Bundle | Form::AppImage => dir.join(&asset_name),
        };
        let cancel = AtomicBool::new(false);
        net.download(
            &asset.browser_download_url,
            &download,
            &cancel,
            &mut |_, _| {},
        )?;
        verify::check(&download, &Expect::Sha256(hash.clone()))?;
        let digest = stage(&plan, &download, &dir.join("unpacked"))?;
        let file = match plan.form {
            Form::Exe => None,
            Form::Bundle | Form::AppImage => plan
                .staged
                .file_name()
                .map(|n| n.to_string_lossy().into_owned()),
        };
        let info = Staged {
            version: version.clone(),
            sha256: digest,
            file,
        };
        crate::common::fsx::write_json(&staged_info(&self.kit), &info)?;
        crate::common::log::info(&format!("update {version} staged"));
        Ok(Found::Staged(version))
    }
}

/// When the app closes: the staged copy takes the place of the app at
/// `exe` (its bundle on macOS, its AppImage on Linux). Checked again
/// (hash) right before, so a copy changed meanwhile is never used.
pub fn apply_staged(kit: &Kit, exe: &Path) -> Result<bool> {
    match plan_for(kit, exe) {
        Some(plan) => apply_plan(kit, &plan),
        None => {
            if read_staged(kit).is_some() {
                discard(kit);
            }
            Ok(false)
        }
    }
}

/// [`apply_staged`] with the plan given.
fn apply_plan(kit: &Kit, plan: &SwapPlan) -> Result<bool> {
    let Some((info, staged)) = read_staged(kit) else {
        return Ok(false);
    };
    if parse_version(&info.version) <= parse_version(env!("CARGO_PKG_VERSION")) {
        discard(kit);
        return Ok(false);
    }
    // Staged for another copy (the bundle or AppImage renamed since).
    if staged != plan.staged {
        discard(kit);
        return Ok(false);
    }
    if !staged_digest(plan.form, &staged)?.eq_ignore_ascii_case(&info.sha256) {
        discard(kit);
        bail!("staged update changed on disk: discarded");
    }
    let (current, old) = (&plan.current, &plan.old);
    let dest = destination(current, &info.version);
    // The copy of an earlier update still running (a `sync` or `keyring`
    // of that version) cannot be deleted: it moves to a name of its own,
    // deleted at a later start, and `old` is free for this one.
    if remove_any(old).is_err() {
        let _ = fs::rename(old, stamped_old(old, &now_ms()));
    }
    fs::rename(current, old)
        .with_context(|| format!("{} → {}", current.display(), old.display()))?;
    let moved = fs::rename(&staged, &dest).or_else(|e| match plan.form {
        Form::Bundle => Err(e),
        Form::Exe | Form::AppImage => fs::copy(&staged, &dest).map(|_| ()),
    });
    if let Err(e) = moved {
        // Never leave the kit without its app.
        let _ = remove_any(&dest);
        let _ = fs::rename(old, current);
        return Err(e).with_context(|| dest.display().to_string());
    }
    if plan.form == Form::AppImage {
        set_mode_755(&dest)?;
    }
    discard(kit);
    crate::common::log::info(&format!("updated to {} ({})", info.version, dest.display()));
    Ok(true)
}

/// Deletes the staged copy and its `update.json`.
fn discard(kit: &Kit) {
    if let Some(info) = crate::common::fsx::read_json::<Staged>(&staged_info(kit)) {
        if let Some(path) = staged_path(kit, &info) {
            let _ = remove_any(&path);
        }
    }
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
            crate::common::log::warn(&format!("{}: {e}", path.display()));
        }
    }
}

/// At start: the copy replaced by the last update. Still in use (a `sync`
/// or `keyring` of the old version running): tried again next time.
pub fn remove_old(exe: &Path) {
    let image = running_image(exe);
    if let Some(plan) = swap_plan(Os::current(), exe, image.as_deref(), Path::new("")) {
        remove_old_of(&plan);
    }
}

fn remove_old_of(plan: &SwapPlan) {
    if fs::symlink_metadata(&plan.old).is_ok() {
        if let Err(e) = remove_any(&plan.old) {
            crate::common::log::warn(&format!("{}: {e}", plan.old.display()));
        }
    }
    // Those set aside by a swap while an older one still ran.
    let Some(dir) = plan.old.parent() else {
        return;
    };
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_stamped_old(&plan.old, &name) {
            let _ = remove_any(&entry.path());
        }
    }
}

fn now_ms() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_default()
}

/// `old` with a stamp after its `.old`: `Atollo.old-<ms>.exe`,
/// `Atollo.app.old-<ms>`.
fn stamped_old(old: &Path, stamp: &str) -> PathBuf {
    let name = old
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let (head, tail) = split_old(&name);
    old.with_file_name(format!("{head}.old-{stamp}{tail}"))
}

/// `name` is `old`'s name with a stamp ([`stamped_old`]).
fn is_stamped_old(old: &Path, name: &str) -> bool {
    let Some(old_name) = old.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return false;
    };
    let (head, tail) = split_old(&old_name);
    name.strip_prefix(&format!("{head}.old-"))
        .and_then(|rest| rest.strip_suffix(tail))
        .is_some_and(|ms| !ms.is_empty() && ms.bytes().all(|b| b.is_ascii_digit()))
}

/// The name around its last `.old`: (`Atollo`, `.exe`) for `Atollo.old.exe`.
fn split_old(name: &str) -> (&str, &str) {
    match name.rfind(".old") {
        Some(i) => (&name[..i], &name[i + 4..]),
        None => (name, ""),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/setup/self_update.rs"]
mod tests;
