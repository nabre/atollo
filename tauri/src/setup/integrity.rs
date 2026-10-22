//! The integrity of the installed components (report v6/04, N-5,
//! DECISIONS.md D-159): USB sticks get damaged, and `check` only sees
//! whether a program answers, not whether one of its libraries is broken.
//!
//! - at the end of an installation, `.atollo-manifest` in the component's
//!   folder (the line's, for one with a folder per version): every file
//!   with its size and SHA-256 (the folder swap of the next installation
//!   replaces it with the folder);
//! - «Verifica i file» compares: a file missing or different is damage; a
//!   file added is not (the programs write theirs: `pip install`, gems);
//! - left out, the files the kit or the programs change on purpose
//!   ([`changes_on_purpose`]) and the components whose folder is theirs to
//!   change (Rust's toolchains, Android's packages, TinyTeX's);
//! - a component installed before this version has no manifest: it is
//!   listed as not verifiable until its next installation.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::kit::Kit;
use crate::setup::catalog::{Catalog, Component};

pub const MANIFEST: &str = ".atollo-manifest";
const HEADER: &str = "# Atollo manifest 1: sha256 size path";

/// Their folder changes by design: `rustup`/`cargo install`, `sdkmanager`,
/// `tlmgr install` write inside it.
const VOLATILE: &[&str] = &["rust", "android", "tinytex"];

/// At most this many paths per list in a report.
const SHOWN: usize = 20;

/// A path (with `/`) the kit or the programs change after the installation:
/// configuration rewritten by the kit (Apache's httpd.conf, phpMyAdmin's
/// config.inc.php, R's Makeconf, Git's system gitconfig, Composer's
/// self-update), the packages installed into Python and Ruby, compiled
/// Python, logs, leftovers of a swap, the kit's own marks.
pub fn changes_on_purpose(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let inside = |dir: &str| rel.starts_with(dir) || rel.contains(&format!("/{dir}"));
    matches!(
        name,
        MANIFEST
            | ".atollo-version"
            | ".DS_Store"
            | "httpd.conf"
            | "config.inc.php"
            | "Makeconf"
            | "gitconfig"
            | "composer.phar"
            | "gemrc"
    ) || name.starts_with("._")
        || name.ends_with(".pyc")
        || name.ends_with(".log")
        || name.ends_with(".pid")
        || name.ends_with(".old")
        || name.contains(".old-")
        || name.contains(".new-")
        || name.ends_with(".atollo-new")
        || inside("__pycache__/")
        || inside("site-packages/")
        || inside("Scripts/")
        || inside("gems/")
        || inside("conda-meta/")
        // .NET's workloads (`dotnet workload install`) update these.
        || inside("sdk-manifests/")
        || inside("metadata/")
}

/// Every file under `dir` (links not followed), as `/` paths, sorted.
fn files(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else {
                continue;
            };
            let path = e.path();
            if kind.is_dir() {
                stack.push(path);
            } else if kind.is_file() {
                let rel = path
                    .strip_prefix(dir)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .replace('\\', "/");
                if !changes_on_purpose(&rel) {
                    out.push((rel, path));
                }
            }
        }
    }
    out.sort();
    out
}

/// `@` and the path from the kit's root, for a file outside the folder.
fn from_root(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    Some(format!("@{}", rel.to_string_lossy().replace('\\', "/")))
}

/// Writes `dir`'s manifest: its files, and `outside` (the component's parts
/// in shared folders, `tools\gh.exe`) as `@` paths from `root`.
pub fn write(root: &Path, dir: &Path, outside: &[PathBuf]) -> Result<()> {
    let mut text = format!("{HEADER}\n");
    let extra = outside
        .iter()
        .filter(|p| p.is_file() && !p.starts_with(dir))
        .filter_map(|p| Some((from_root(root, p)?, p.clone())));
    for (rel, path) in files(dir).into_iter().chain(extra) {
        let size = fs::metadata(&path)?.len();
        let hash = crate::net::verify::sha256_hex(&path)?;
        text.push_str(&format!("{hash} {size} {rel}\n"));
    }
    let file = dir.join(MANIFEST);
    let mut f = fs::File::create(&file).with_context(|| file.display().to_string())?;
    f.write_all(text.as_bytes())?;
    Ok(())
}

/// After an installation into `dir`: the manifest, unless the component's
/// folder is its own to change; a failure is only logged. `default`: the
/// installation of the default version, which also puts the parts outside
/// its folder (`keys`).
pub fn after_install(kit: &Kit, c: &Component, dir: &Path, default: bool) {
    if VOLATILE.contains(&c.id.as_str()) || !dir.is_dir() {
        return;
    }
    let outside = if default {
        c.key_paths(kit)
    } else {
        Vec::new()
    };
    if let Err(e) = write(kit.root(), dir, &outside) {
        crate::log::warn(&format!("{}: manifest: {e:#}", c.id));
    }
}

/// What differs from the manifest.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Damage {
    pub checked: usize,
    pub missing: Vec<String>,
    pub changed: Vec<String>,
}

impl Damage {
    pub fn is_damaged(&self) -> bool {
        !self.missing.is_empty() || !self.changed.is_empty()
    }
}

/// `dir` against its manifest (`@` paths from `root`); `None` without one.
pub fn verify(root: &Path, dir: &Path) -> Option<Result<Damage>> {
    let text = fs::read_to_string(dir.join(MANIFEST)).ok()?;
    Some(compare(root, dir, &text))
}

fn compare(root: &Path, dir: &Path, text: &str) -> Result<Damage> {
    let mut damage = Damage::default();
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let mut it = line.splitn(3, ' ');
        let (Some(hash), Some(size), Some(rel)) = (it.next(), it.next(), it.next()) else {
            anyhow::bail!("{MANIFEST}: damaged line: {line}");
        };
        let size: u64 = size.parse().context(MANIFEST)?;
        damage.checked += 1;
        let path = match rel.strip_prefix('@') {
            Some(from_root) => root.join(from_root),
            None => dir.join(rel),
        };
        match fs::metadata(&path) {
            Ok(m) if m.is_file() => {
                if m.len() != size
                    || crate::net::verify::sha256_hex(&path).ok().as_deref() != Some(hash)
                {
                    damage.changed.push(rel.to_owned());
                }
            }
            _ => damage.missing.push(rel.to_owned()),
        }
    }
    Ok(damage)
}

/// One installed folder of a component, verified.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Integrity {
    pub id: String,
    pub name: String,
    /// The version's folder, for a component with one per version.
    pub line: Option<String>,
    /// Whether it is the default version (the others are saved lines).
    pub default: bool,
    /// `None`: no manifest (installed before Atollo kept one).
    pub damage: Option<Damage>,
    /// The manifest could not be read.
    pub error: Option<String>,
}

/// The folders of `c` on disk: its line folders, or its own folder.
fn folders(kit: &Kit, c: &Component) -> Vec<(Option<String>, bool, PathBuf)> {
    if !c.per_line() {
        return c
            .folder_path(kit)
            .filter(|d| d.is_dir())
            .map(|d| vec![(None, true, d)])
            .unwrap_or_default();
    }
    let mut out = Vec::new();
    if let Some(line) = crate::versions::default_line(kit, c) {
        let dir = crate::versions::default_dir(kit, c);
        if dir.is_dir() {
            out.push((Some(line), true, dir));
        }
    }
    for l in crate::versions::saved(kit, c) {
        let dir = crate::versions::saved_dir(kit, c, &l.line);
        if dir.is_dir() {
            out.push((Some(l.line), false, dir));
        }
    }
    out
}

/// Every installed component of `kit` against its manifests, in the
/// catalog's order. The lists of paths are cut at [`SHOWN`].
pub fn check_all(kit: &Kit) -> Vec<Integrity> {
    let mut out = Vec::new();
    for c in Catalog::get().all() {
        if VOLATILE.contains(&c.id.as_str()) || !c.is_present(kit) {
            continue;
        }
        for (line, default, dir) in folders(kit, c) {
            let (damage, error) = match verify(kit.root(), &dir) {
                None => (None, None),
                Some(Ok(mut d)) => {
                    d.missing.truncate(SHOWN);
                    d.changed.truncate(SHOWN);
                    (Some(d), None)
                }
                Some(Err(e)) => (None, Some(format!("{e:#}"))),
            };
            out.push(Integrity {
                id: c.id.clone(),
                name: c.name.clone(),
                line,
                default,
                damage,
                error,
            });
        }
    }
    out
}

/// Marks the damaged folders to be installed again at their version (the
/// marks of D-92): the next preparation of those components reinstalls
/// them. Returns the components to prepare.
pub fn mark_for_repair(kit: &Kit, items: &[(String, Option<String>)]) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for (id, line) in items {
        let c = Catalog::get()
            .find(id)
            .with_context(|| format!("unknown component: {id}"))?;
        let saved = line
            .as_deref()
            .filter(|l| crate::versions::default_line(kit, c).as_deref() != Some(*l));
        match saved {
            Some(l) => c.mark_line_reinstall(kit, l, "integrity")?,
            None => c.mark_reinstall(kit, "integrity")?,
        }
        if !ids.contains(&c.id) {
            ids.push(c.id.clone());
        }
    }
    Ok(ids)
}

#[cfg(test)]
#[path = "../../../test/rust/setup/integrity.rs"]
mod tests;
