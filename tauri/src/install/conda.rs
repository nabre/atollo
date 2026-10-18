//! The packages of the conda environments that cannot follow the kit
//! (D-92: R, Ruby off Windows, MongoDB on Linux, SQLite on Linux ARM64,
//! make of C/C++ off Windows), kept in the kit so that a move creates them
//! again without the network (D-118).
//!
//! `data/app/conda/<target>/`, one folder per target, never mixed:
//! - `pkgs/<file>`: the archives (`.conda`, `.tar.bz2`) as conda-forge
//!   publishes them, one copy shared by every list of the target;
//! - `locks/<id>@<line>.txt`: the explicit list of the line's environment
//!   (`@EXPLICIT`, then `url#sha256` in micromamba's order);
//! - `locks/<id>@<line>.json`: version, date, micromamba, file, size and
//!   SHA-256 of every archive.
//!
//! An installation from the network (`keep`) copies the archives out of
//! micromamba's cache in the work folder, each checked against the SHA-256
//! of `conda-meta` (conda-forge's metadata, which micromamba checked when it
//! downloaded). After a move (`reinstall`) every archive is checked again
//! against the list, and the list against its record; only then, with copies
//! in the work folder checked once more, `micromamba create --offline`
//! makes the environment again in its folder, followed by the component's
//! own steps (`Installer::after_conda`). Anything missing or different:
//! the installation from the network as before, with a warning. What no
//! list names any more goes (`sweep`).

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::native::{mamba_envs, micromamba_path, run, run_output};
use super::{installer, work_dir, Ctx, Installer, Progress};
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::net::verify::{self, Expect};
use crate::settings::Settings;
use crate::setup::catalog::{Catalog, Component};

/// The only channel of the kit's environments (`micromamba_create_args`).
const CHANNEL: &str = "https://conda.anaconda.org/conda-forge/";

/// `data/app/conda/<target>`: the kept packages and lists of the kit's
/// target.
pub fn root(kit: &Kit) -> PathBuf {
    kit.app_dir().join("conda").join(kit.target().as_str())
}

fn pkgs_dir(kit: &Kit) -> PathBuf {
    root(kit).join("pkgs")
}

fn locks_dir(kit: &Kit) -> PathBuf {
    root(kit).join("locks")
}

/// `<id>@<line>`: the name of the lists of `line` of component `id`.
pub fn lock_name(id: &str, line: &str) -> String {
    format!("{id}@{line}")
}

/// One archive of an explicit list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archive {
    pub url: String,
    /// Its name in `pkgs/`, the last part of the URL.
    pub file: String,
    /// Lowercase hex.
    pub sha256: String,
}

impl Archive {
    /// The conda subdir of its URL (`linux-64`, `noarch`).
    pub fn subdir(&self) -> Option<&str> {
        self.url.strip_prefix(CHANNEL)?.split('/').next()
    }
}

/// The record of a list (`locks/<id>@<line>.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Record {
    /// The component's version, as saved.
    pub version: String,
    pub target: String,
    /// Unix seconds.
    pub created: u64,
    /// What `micromamba --version` said, for the diagnosis only: the
    /// explicit format does not change with it.
    pub micromamba: Option<String>,
    pub packages: Vec<Package>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    pub file: String,
    pub sha256: String,
    pub size: u64,
}

fn is_sha256(hash: &str) -> bool {
    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A file name of conda-forge that can only be a file of `pkgs/`: no
/// separator, not hidden, an archive's extension.
pub fn safe_file(name: &str) -> bool {
    !name.starts_with('.')
        && (name.ends_with(".conda") || name.ends_with(".tar.bz2"))
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._+-!".contains(c))
}

fn safe_subdir(subdir: &str) -> bool {
    !subdir.is_empty()
        && subdir
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
}

/// The archives of an explicit list: `@EXPLICIT`, then one `url#sha256`
/// per line; comments (`#`) and blank lines are skipped. Only conda-forge's
/// https URLs and plain file names are taken, and only SHA-256: an MD5
/// (what `micromamba env export` writes) is refused, since the list is what
/// the archives are checked against.
pub fn parse_explicit(text: &str) -> Result<Vec<Archive>> {
    let mut explicit = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "@EXPLICIT" {
            explicit = true;
            continue;
        }
        if !explicit {
            bail!("not an explicit list: {line} before @EXPLICIT");
        }
        let (url, hash) = line
            .split_once('#')
            .ok_or_else(|| anyhow!("{line}: no hash"))?;
        let hash = hash.strip_prefix("sha256:").unwrap_or(hash);
        if !is_sha256(hash) {
            bail!("{url}: not a SHA-256");
        }
        let (subdir, file) = url
            .strip_prefix(CHANNEL)
            .and_then(|rest| rest.split_once('/'))
            .ok_or_else(|| anyhow!("{url}: not a package of conda-forge"))?;
        if !safe_subdir(subdir) || !safe_file(file) {
            bail!("{url}: not a package of conda-forge");
        }
        out.push(Archive {
            url: url.to_owned(),
            file: file.to_owned(),
            sha256: hash.to_ascii_lowercase(),
        });
    }
    if !explicit {
        bail!("not an explicit list: no @EXPLICIT");
    }
    if out.is_empty() {
        bail!("an explicit list without packages");
    }
    Ok(out)
}

/// The text of an explicit list of `archives` for `subdir`.
pub fn explicit_text(archives: &[Archive], subdir: &str) -> String {
    let mut out = format!(
        "# Atollo: the packages of a conda environment, kept to reinstall it without the network (D-118)\n# platform: {subdir}\n@EXPLICIT\n"
    );
    for a in archives {
        out.push_str(&format!("{}#{}\n", a.url, a.sha256));
    }
    out
}

/// Why an environment is installed from the network after a move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallback {
    /// No list for the line: installed before D-118, freed, or kept
    /// packages turned off.
    NoList,
    /// The list or its record cannot be read.
    Unreadable(String),
    /// The list and its record name another archive or hash.
    Disagree(String),
    /// An archive of another system.
    OtherTarget(String),
    /// An archive is not there, or has another size.
    Missing(String),
    /// An archive has another SHA-256: never used.
    Changed(String),
    /// micromamba is not in `tools`.
    NoMicromamba,
}

impl fmt::Display for Fallback {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fallback::NoList => write!(f, "no kept packages"),
            Fallback::Unreadable(e) => {
                write!(f, "the list of the kept packages cannot be read ({e})")
            }
            Fallback::Disagree(file) => write!(f, "the list and its record disagree on {file}"),
            Fallback::OtherTarget(file) => write!(f, "{file} is a package of another system"),
            Fallback::Missing(file) => write!(f, "{file} is missing from the kept packages"),
            Fallback::Changed(file) => write!(f, "{file} does not have the SHA-256 of the list"),
            Fallback::NoMicromamba => write!(f, "micromamba is not in the kit"),
        }
    }
}

/// The decision before an installation without the network: the archives
/// of `list` when it and its `record` agree, every archive is for `subdir`
/// (or `noarch`) and every file of `pkgs` has the size and the SHA-256 of
/// the list; otherwise why not. The reference is the list, whose hashes
/// came from conda-forge's metadata; the record must say the same, so a
/// record changed alone does not let another archive through.
pub fn check(
    list: Option<&str>,
    record: Option<&str>,
    subdir: &str,
    pkgs: &Path,
) -> Result<Vec<Archive>, Fallback> {
    let (Some(list), Some(record)) = (list, record) else {
        return Err(Fallback::NoList);
    };
    let archives = parse_explicit(list).map_err(|e| Fallback::Unreadable(format!("{e:#}")))?;
    let record: Record =
        serde_json::from_str(record).map_err(|e| Fallback::Unreadable(format!("record: {e}")))?;
    if let Some(a) = archives
        .iter()
        .find(|a| !matches!(a.subdir(), Some(s) if s == subdir || s == "noarch"))
    {
        return Err(Fallback::OtherTarget(a.file.clone()));
    }
    if record.packages.len() != archives.len() {
        return Err(Fallback::Disagree(format!(
            "{} packages instead of {}",
            record.packages.len(),
            archives.len()
        )));
    }
    let mut sizes = Vec::with_capacity(archives.len());
    for a in &archives {
        let package = record
            .packages
            .iter()
            .find(|p| p.file == a.file)
            .filter(|p| p.sha256.eq_ignore_ascii_case(&a.sha256))
            .ok_or_else(|| Fallback::Disagree(a.file.clone()))?;
        sizes.push(package.size);
    }
    for (a, size) in archives.iter().zip(sizes) {
        match fs::metadata(pkgs.join(&a.file)) {
            Ok(m) if m.is_file() && m.len() == size => {}
            _ => return Err(Fallback::Missing(a.file.clone())),
        }
    }
    for a in &archives {
        let path = pkgs.join(&a.file);
        if !verify::matches(&path, &Expect::Sha256(a.sha256.clone())).unwrap_or(false) {
            return Err(Fallback::Changed(a.file.clone()));
        }
    }
    Ok(archives)
}

/// A `file://` URL of `path` for an explicit list: forward slashes, every
/// byte but the unreserved ones and `/` `:` percent-encoded (spaces and
/// parentheses of a kit's folder).
pub fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let mut out = String::from("file://");
    if !text.starts_with('/') {
        out.push('/');
    }
    for b in text.bytes() {
        if b.is_ascii_alphanumeric() || b"/:-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// The packages of `component` are kept: it cannot follow the kit on this
/// target (D-92) and the setting is on (the default).
pub(super) fn wanted(ctx: &Ctx, component: &Component) -> bool {
    let target = ctx.kit.target();
    installer(&component.id).is_some_and(|i| !i.relocatable(target))
        && Settings::load(ctx.kit).settings.keep_conda_packages
}

/// After `micromamba create` from the network: the archives of the
/// environment at `prefix`, found in micromamba's package cache `cache`,
/// copied into the kit with the lists of the line of `version`. A failure
/// leaves the environment as it is, without the copy: a warning only.
#[allow(clippy::too_many_arguments)]
pub(super) fn keep(
    ctx: &Ctx,
    component: &Component,
    version: &str,
    mm: &Path,
    envs: &[(&str, &Path)],
    prefix: &Path,
    cache: &Path,
    progress: &mut dyn Progress,
) {
    match keep_lists(ctx, component, version, mm, envs, prefix, cache) {
        Ok(bytes) => progress.note(&format!(
            "packages kept to reinstall without the network: {} MB",
            bytes.div_ceil(1 << 20)
        )),
        Err(e) => {
            let text = format!(
                "{}: packages not kept, a move of the kit will need the network ({e:#})",
                component.id
            );
            crate::log::warn(&text);
            progress.note(&text);
        }
    }
}

/// What `conda-meta` says of each package of an environment, by URL:
/// file name, SHA-256 and size, from conda-forge's metadata.
fn conda_meta(prefix: &Path) -> Result<BTreeMap<String, Package>> {
    let dir = prefix.join("conda-meta");
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(&dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(meta) = serde_json::from_slice::<Value>(&fs::read(&path)?) else {
            continue;
        };
        let text = |k: &str| meta.get(k).and_then(Value::as_str).map(str::to_owned);
        let (Some(url), Some(file), Some(sha256)) = (text("url"), text("fn"), text("sha256"))
        else {
            continue;
        };
        let size = meta.get("size").and_then(Value::as_u64).unwrap_or(0);
        out.insert(url, Package { file, sha256, size });
    }
    Ok(out)
}

/// `src` into `dest` of `pkgs/`, unless it is there already with `sha256`;
/// the copy is checked before it takes the name.
fn store(src: &Path, dest: &Path, sha256: &str) -> Result<()> {
    let expect = Expect::Sha256(sha256.to_owned());
    if dest.is_file() && verify::matches(dest, &expect)? {
        return Ok(());
    }
    let part = dest.with_extension("part");
    fs::copy(src, &part).with_context(|| part.display().to_string())?;
    verify::check(&part, &expect)?;
    fs::rename(&part, dest).with_context(|| dest.display().to_string())?;
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn keep_lists(
    ctx: &Ctx,
    component: &Component,
    version: &str,
    mm: &Path,
    envs: &[(&str, &Path)],
    prefix: &Path,
    cache: &Path,
) -> Result<u64> {
    let prefix_arg = prefix.display().to_string();
    // micromamba's order, the one it installs an explicit list in.
    let exported = run_output(
        ctx,
        mm,
        &["env", "export", "--no-rc", "--explicit", "-p", &prefix_arg],
        envs,
    )?;
    let urls: Vec<&str> = exported
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with("https://"))
        .map(|l| l.split('#').next().unwrap_or(l))
        .collect();
    if urls.is_empty() {
        bail!("micromamba listed no packages");
    }
    let meta = conda_meta(prefix)?;
    let pkgs = pkgs_dir(ctx.kit);
    fs::create_dir_all(&pkgs).with_context(|| pkgs.display().to_string())?;
    let mut archives = Vec::with_capacity(urls.len());
    let mut packages = Vec::with_capacity(urls.len());
    let mut total = 0;
    for url in urls {
        let m = meta
            .get(url)
            .ok_or_else(|| anyhow!("{url}: not in conda-meta"))?;
        if !safe_file(&m.file) || !url.ends_with(&format!("/{}", m.file)) || !is_sha256(&m.sha256) {
            bail!("{url}: unexpected file or hash in conda-meta");
        }
        let found = super::find_file(cache, &m.file)
            .ok_or_else(|| anyhow!("{}: not in micromamba's cache", m.file))?;
        let dest = pkgs.join(&m.file);
        store(&found, &dest, &m.sha256)?;
        let size = fs::metadata(&dest)?.len();
        if m.size != 0 && m.size != size {
            bail!("{}: {size} bytes, conda-meta says {}", m.file, m.size);
        }
        total += size;
        archives.push(Archive {
            url: url.to_owned(),
            file: m.file.clone(),
            sha256: m.sha256.to_ascii_lowercase(),
        });
        packages.push(Package {
            file: m.file.clone(),
            sha256: m.sha256.to_ascii_lowercase(),
            size,
        });
    }
    let target = ctx.kit.target();
    let text = explicit_text(&archives, target.conda_subdir());
    // The same rules as the reading: a list the kit would refuse is not kept.
    parse_explicit(&text)?;
    let record = Record {
        version: version.to_owned(),
        target: target.as_str().to_owned(),
        created: now(),
        micromamba: run_output(ctx, mm, &["--version"], envs)
            .ok()
            .map(|v| v.trim().to_owned()),
        packages,
    };
    let locks = locks_dir(ctx.kit);
    fs::create_dir_all(&locks).with_context(|| locks.display().to_string())?;
    let name = lock_name(&component.id, &crate::versions::dir_name(version));
    write_atomic(
        &locks.join(format!("{name}.json")),
        serde_json::to_string_pretty(&record)?.as_bytes(),
    )?;
    write_atomic(&locks.join(format!("{name}.txt")), text.as_bytes())?;
    Ok(total)
}

/// The version, folder and line of `line` of `c` (`None`: the default).
fn located(kit: &Kit, c: &Component, line: Option<&str>) -> Option<(String, PathBuf, String)> {
    match line {
        None => {
            let version = c.installed_version(kit)?;
            let dir = if c.per_line() {
                crate::versions::install_dir(kit, c, &version)
            } else {
                c.folder_path(kit)?
            };
            let line = crate::versions::dir_name(&version);
            Some((version, dir, line))
        }
        Some(line) => {
            let version = crate::versions::saved(kit, c)
                .into_iter()
                .find(|l| l.line == line)?
                .version?;
            Some((
                version,
                crate::versions::saved_dir(kit, c, line),
                line.to_owned(),
            ))
        }
    }
}

/// After a move: the environment of `line` of `component` (`None`: the
/// default folder) created again from the kept packages, without the
/// network; its mark is removed. Returns the version, or `None` when it
/// must be installed from the network as before (the reason in the log and
/// the notes).
pub fn reinstall(
    ctx: &Ctx,
    component: &Component,
    line: Option<&str>,
    progress: &mut dyn Progress,
) -> Option<String> {
    let target = ctx.kit.target();
    let inst = installer(&component.id).filter(|i| !i.relocatable(target))?;
    let (version, dir, line_name) = located(ctx.kit, component, line)?;
    let prefix = inst.conda_prefix(target, &dir)?;
    let started = std::time::Instant::now();
    match restore(
        ctx, inst, component, &version, &dir, &prefix, &line_name, progress,
    ) {
        Ok(Ok(())) => {
            match line {
                None => component.clear_reinstall(ctx.kit),
                Some(l) => component.clear_line_reinstall(ctx.kit, l),
            }
            let text = format!(
                "{} {line_name}: created again from the kept packages, without the network ({} s)",
                component.id,
                started.elapsed().as_secs()
            );
            crate::log::info(&text);
            progress.note(&text);
            Some(version)
        }
        Ok(Err(fallback)) => {
            let text = format!(
                "{} {line_name}: installed again from the network: {fallback}",
                component.id
            );
            crate::log::warn(&text);
            progress.note(&text);
            None
        }
        Err(e) => {
            let text = format!(
                "{} {line_name}: not created from the kept packages ({e:#}), installed again from the network",
                component.id
            );
            crate::log::warn(&text);
            progress.note(&text);
            None
        }
    }
}

/// `prefix` renamed to `<name>.old-<ms>` next to it, put back by the
/// caller when the new one fails; `None` when there is none.
fn set_aside(prefix: &Path) -> Result<Option<PathBuf>> {
    if !prefix.exists() {
        return Ok(None);
    }
    let name = prefix
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let old = prefix.with_file_name(format!("{name}.old-{ms}"));
    fs::rename(prefix, &old).map_err(|_| crate::net::archive::InUse(prefix.to_path_buf()))?;
    Ok(Some(old))
}

#[allow(clippy::too_many_arguments)]
fn restore(
    ctx: &Ctx,
    inst: &dyn Installer,
    component: &Component,
    version: &str,
    dir: &Path,
    prefix: &Path,
    line: &str,
    progress: &mut dyn Progress,
) -> Result<Result<(), Fallback>> {
    let kit = ctx.kit;
    let mm = micromamba_path(kit);
    if !mm.is_file() {
        return Ok(Err(Fallback::NoMicromamba));
    }
    let name = lock_name(&component.id, line);
    let locks = locks_dir(kit);
    let list = fs::read_to_string(locks.join(format!("{name}.txt"))).ok();
    let record = fs::read_to_string(locks.join(format!("{name}.json"))).ok();
    let pkgs = pkgs_dir(kit);
    let archives = match check(
        list.as_deref(),
        record.as_deref(),
        kit.target().conda_subdir(),
        &pkgs,
    ) {
        Ok(a) => a,
        Err(f) => return Ok(Err(f)),
    };
    progress.note(&format!(
        "{} kept packages checked (SHA-256 of the list)",
        archives.len()
    ));
    let guard = work_dir(ctx, component)?;
    let work = guard.path();
    // Copied out of the kit and checked again: micromamba reads the copies,
    // so a file changed on the kit's disk after the check is never used.
    let input = work.join("kept");
    fs::create_dir_all(&input)?;
    let mut explicit = String::from("@EXPLICIT\n");
    for a in &archives {
        let copy = input.join(&a.file);
        fs::copy(pkgs.join(&a.file), &copy).with_context(|| copy.display().to_string())?;
        if !verify::matches(&copy, &Expect::Sha256(a.sha256.clone()))? {
            return Ok(Err(Fallback::Changed(a.file.clone())));
        }
        explicit.push_str(&format!("{}#{}\n", file_url(&copy), a.sha256));
    }
    let list_file = work.join("explicit.txt");
    fs::write(&list_file, explicit)?;
    let envs = mamba_envs(work);
    let envs: Vec<(&str, &Path)> = envs.iter().map(|(k, v)| (*k, v.as_path())).collect();
    // What an interrupted run set aside next to it goes first.
    if let Some(parent) = prefix.parent() {
        crate::net::archive::clean_leftovers(parent);
    }
    let old = set_aside(prefix)?;
    let created = (|| -> Result<()> {
        let prefix_arg = prefix.display().to_string();
        let list_arg = list_file.display().to_string();
        progress.note(&format!("micromamba --offline: {name}"));
        run(
            ctx,
            &mm,
            &[
                "create",
                "--no-rc",
                "--offline",
                "-y",
                "-q",
                "-p",
                &prefix_arg,
                "-f",
                &list_arg,
            ],
            &envs,
            progress,
        )?;
        let _ = fs::remove_dir_all(prefix.join("pkgs"));
        inst.after_conda(ctx, component, dir, progress)?;
        if component.per_line() {
            crate::versions::write_version(dir, version)?;
        }
        Ok(())
    })();
    drop(guard);
    match created {
        Ok(()) => {
            if let Some(old) = old {
                let _ = fs::remove_dir_all(old);
            }
            Ok(Ok(()))
        }
        Err(e) => {
            let _ = fs::remove_dir_all(prefix);
            if let Some(old) = old {
                let _ = fs::rename(&old, prefix);
            }
            Err(e)
        }
    }
}

/// The environment the list `<id>@<line>` was written for is still on
/// disk on the kit's target (moved or not: a moved one is what it is for).
fn lock_in_use(kit: &Kit, stem: &str) -> bool {
    let target = kit.target();
    let Some((id, line)) = stem.split_once('@') else {
        return false;
    };
    let Some(c) = Catalog::get().find(id) else {
        return false;
    };
    let Some(inst) = installer(id).filter(|i| !i.relocatable(target)) else {
        return false;
    };
    let dir = if c.per_line() {
        crate::versions::saved_dir(kit, c, line)
    } else {
        let current = c
            .installed_version(kit)
            .map(|v| crate::versions::dir_name(&v));
        match c.folder_path(kit) {
            Some(folder) if current.as_deref() == Some(line) => folder,
            _ => return false,
        }
    };
    inst.conda_prefix(target, &dir)
        .is_some_and(|p| p.join("conda-meta").is_dir())
}

fn tree_size(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return 0;
    };
    if !meta.is_dir() {
        return meta.len();
    }
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| tree_size(&e.path()))
        .sum()
}

fn remove_any(path: &Path) -> bool {
    if path.is_dir() {
        fs::remove_dir_all(path).is_ok()
    } else {
        fs::remove_file(path).is_ok()
    }
}

/// What `sweep` removed.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Swept {
    /// The names of the lists removed (`<id>@<line>`).
    pub lists: Vec<String>,
    pub archives: usize,
    pub bytes: u64,
}

/// At start and after every preparation: the lists whose environment is
/// no longer on disk (a line removed, a component uninstalled, another
/// line after an update) and the archives no list names go, with what an
/// interrupted copy left. With the setting off everything goes. Only this
/// target's folder.
pub fn sweep(kit: &Kit) -> Swept {
    let mut swept = Swept::default();
    if !root(kit).exists() {
        return swept;
    }
    if !Settings::load(kit).settings.keep_conda_packages {
        if let Ok(bytes) = free(kit) {
            swept.bytes = bytes;
            crate::log::info(&format!(
                "conda: kept packages off, {} MB freed",
                bytes.div_ceil(1 << 20)
            ));
        }
        return swept;
    }
    let locks = locks_dir(kit);
    let mut stems = BTreeSet::new();
    for entry in fs::read_dir(&locks).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        match name
            .strip_suffix(".txt")
            .or_else(|| name.strip_suffix(".json"))
        {
            Some(stem) if path.is_file() => {
                stems.insert(stem.to_owned());
            }
            _ => {
                remove_any(&path);
            }
        }
    }
    let mut referenced = BTreeSet::new();
    for stem in stems {
        let txt = locks.join(format!("{stem}.txt"));
        let json = locks.join(format!("{stem}.json"));
        let files = (json.is_file() && lock_in_use(kit, &stem))
            .then(|| fs::read_to_string(&txt).ok())
            .flatten()
            .and_then(|t| parse_explicit(&t).ok());
        match files {
            Some(archives) => referenced.extend(archives.into_iter().map(|a| a.file)),
            None => {
                let _ = fs::remove_file(&txt);
                let _ = fs::remove_file(&json);
                swept.lists.push(stem);
            }
        }
    }
    for entry in fs::read_dir(pkgs_dir(kit)).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_file() && referenced.contains(&name) {
            continue;
        }
        let size = tree_size(&path);
        if remove_any(&path) {
            swept.archives += 1;
            swept.bytes += size;
        }
    }
    if !swept.lists.is_empty() || swept.archives > 0 {
        crate::log::info(&format!(
            "conda: lists removed [{}], {} archives no list names removed ({} MB)",
            swept.lists.join(", "),
            swept.archives,
            swept.bytes.div_ceil(1 << 20)
        ));
    }
    swept
}

/// Everything kept for this target removed («Libera»): the next move
/// downloads again. Returns the bytes freed.
pub fn free(kit: &Kit) -> Result<u64> {
    let dir = root(kit);
    let bytes = tree_size(&dir);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| dir.display().to_string())?;
    }
    Ok(bytes)
}

/// The kept packages of this target, for the Settings page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Some component of the catalog cannot follow the kit on this target:
    /// the setting means something here.
    pub applies: bool,
    pub bytes: u64,
    /// The lines with a list (`<id>@<line>`).
    pub lists: Vec<String>,
}

pub fn status(kit: &Kit) -> Status {
    let target = kit.target();
    let mut lists: Vec<String> = fs::read_dir(locks_dir(kit))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_string_lossy()
                .strip_suffix(".txt")
                .map(str::to_owned)
        })
        .collect();
    lists.sort();
    Status {
        applies: Catalog::get()
            .all()
            .iter()
            .any(|c| installer(&c.id).is_some_and(|i| !i.relocatable(target))),
        bytes: tree_size(&root(kit)),
        lists,
    }
}

/// The folder of a conda environment with the launchers of the
/// component's own programs (D-125): on the PATH in place of `bin`, which
/// also holds those of every dependency (`openssl`, `tput`, `kinit`,
/// `envsubst`... 30 programs for Ruby's 16, 56 for MongoDB's 2, 156 for
/// R's 2).
pub const LAUNCHERS: &str = "kit-bin";

/// The programs of `bin/` that `package` put in the environment `prefix`,
/// from its record in `conda-meta` (`files`), sorted.
pub fn package_programs(prefix: &Path, package: &str) -> Result<Vec<String>> {
    let dir = prefix.join("conda-meta");
    for entry in fs::read_dir(&dir).with_context(|| dir.display().to_string())? {
        let path = entry?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // `<name>-<version>-<build>.json`: only the records that can be it.
        let Some(rest) = name.strip_prefix(package).and_then(|r| r.strip_prefix('-')) else {
            continue;
        };
        if !rest.starts_with(|c: char| c.is_ascii_digit()) || !name.ends_with(".json") {
            continue;
        }
        let meta: Value = serde_json::from_slice(&fs::read(&path)?)
            .with_context(|| path.display().to_string())?;
        if meta.get("name").and_then(Value::as_str) != Some(package) {
            continue;
        }
        let mut programs: Vec<String> = meta
            .get("files")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter_map(|f| f.strip_prefix("bin/"))
            .filter(|p| !p.is_empty() && !p.contains('/'))
            .map(str::to_owned)
            .collect();
        programs.sort();
        return Ok(programs);
    }
    bail!("{}: no record of {package}", dir.display())
}

/// The launcher of `program` in `kit-bin`: the program of `bin` next to
/// it, with that `bin` first in the PATH of the program only (its own
/// tools, as when the whole `bin` was on the kit's PATH).
pub fn launcher_text(program: &str) -> String {
    format!(
        "#!/bin/sh\n# Dev-Kit: {program} of the conda-forge environment, with its bin in its PATH only\nd=$(cd \"$(dirname \"$0\")/..\" && pwd)\nPATH=\"$d/bin:$PATH\" exec \"$d/bin/{program}\" \"$@\"\n"
    )
}

/// `kit-bin` of `prefix` made again with a launcher for each program of
/// `package`; nothing else in it.
pub fn write_launchers(prefix: &Path, package: &str) -> Result<()> {
    let programs = package_programs(prefix, package)?;
    if programs.is_empty() {
        bail!("{package}: no program in bin");
    }
    let dir = prefix.join(LAUNCHERS);
    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| dir.display().to_string())?;
    }
    for program in &programs {
        let path = dir.join(program);
        crate::router::fs::write_executable(&path, launcher_text(program).as_bytes())
            .with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// The launchers of the environments installed before the kit wrote them
/// (at start, `versions::side_by_side::refresh`): every line of every
/// component with [`Installer::conda_programs`], where `kit-bin` is missing.
pub fn launchers_of_older_installations(kit: &Kit) {
    let target = kit.target();
    for c in Catalog::get().all() {
        let Some(inst) = installer(&c.id) else {
            continue;
        };
        let Some(package) = inst.conda_programs(target) else {
            continue;
        };
        let lines = crate::versions::default_line(kit, c)
            .into_iter()
            .chain(crate::versions::saved(kit, c).into_iter().map(|l| l.line));
        for line in lines {
            let Some(prefix) = crate::versions::dir_for(kit, c, Some(&line))
                .and_then(|dir| inst.conda_prefix(target, &dir))
            else {
                continue;
            };
            if prefix.join(LAUNCHERS).is_dir() || !prefix.join("conda-meta").is_dir() {
                continue;
            }
            if let Err(e) = write_launchers(&prefix, package) {
                crate::log::warn(&format!("{}@{line}: launchers: {e:#}", c.id));
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/install/conda.rs"]
mod tests;
