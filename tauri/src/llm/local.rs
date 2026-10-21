//! The models on disk: `data/models/gguf/<owner>--<repo>/`, the GGUF files
//! of that repository side by side (all the parts of a model in several,
//! the `mmproj-*.gguf` of a model that reads images) and `atollo.json`,
//! what the kit downloaded there: each model's id, files, commit and
//! whether it is complete. A `.gguf` put there by hand (or in a folder of
//! its own) is a model too, named after its file.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::catalog::ModelFile;
use crate::kit::Kit;

/// The name of the manifest in a model's folder.
pub const MANIFEST: &str = "atollo.json";

/// One model the kit downloaded (or is downloading).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub repo: String,
    pub commit: String,
    pub files: Vec<ModelFile>,
    /// The catalog's model it is, if any.
    #[serde(default)]
    pub catalog: Option<String>,
    #[serde(default)]
    pub ctx: Option<u32>,
    /// Every file downloaded and checked.
    #[serde(default)]
    pub complete: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub models: Vec<Entry>,
}

/// A model of the kit, as the interface and the server see it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModel {
    pub id: String,
    pub name: String,
    /// `owner/repo`, empty for a file put there by hand outside a folder of
    /// Hugging Face's name.
    pub repo: String,
    /// The folder, relative to `data/models/gguf`.
    pub folder: String,
    /// Its GGUF files, the first part first.
    pub files: Vec<String>,
    /// The projector of a model that reads images, if any.
    pub mmproj: Option<String>,
    pub bytes: u64,
    /// Bytes on disk, for a download not finished.
    pub downloaded: u64,
    pub complete: bool,
    pub catalog: Option<String>,
    pub ctx: Option<u32>,
    /// Put there by hand: no checksum known to the kit.
    pub by_hand: bool,
}

/// `owner/repo` as a folder name: `owner--repo`.
pub fn folder_of(repo: &str) -> String {
    repo.replacen('/', "--", 1)
}

/// `owner/repo` of Hugging Face: two names of letters, digits, `.`, `_`
/// and `-`, neither `.` nor `..`.
pub fn valid_repo(repo: &str) -> bool {
    let name = |s: &str| {
        !s.is_empty()
            && s.len() <= 96
            && s.bytes().any(|b| b != b'.')
            && s.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
    };
    matches!(repo.split_once('/'), Some((o, r)) if name(o) && name(r) && !r.contains('/'))
}

/// The manifest of `dir`; empty without one.
pub fn read_manifest(dir: &Path) -> Result<Manifest> {
    Ok(crate::fsx::try_read_json(&dir.join(MANIFEST))?.unwrap_or_default())
}

pub fn write_manifest(dir: &Path, manifest: &Manifest) -> Result<()> {
    fs::create_dir_all(dir)?;
    crate::fsx::write_json(&dir.join(MANIFEST), manifest)
}

/// `-00001-of-00003` at the end of a stem: the part and the count.
pub fn part_of(stem: &str) -> Option<(u32, u32)> {
    let (rest, count) = stem.rsplit_once("-of-")?;
    let (_, part) = rest.rsplit_once('-')?;
    let number = |s: &str| {
        (s.len() == 5 && s.bytes().all(|b| b.is_ascii_digit()))
            .then(|| s.parse().ok())
            .flatten()
    };
    Some((number(part)?, number(count)?))
}

/// The stem of a GGUF file without its part number.
pub fn model_stem(file: &str) -> &str {
    let stem = file.strip_suffix(".gguf").unwrap_or(file);
    match stem.rsplit_once("-of-") {
        Some((rest, _)) if part_of(stem).is_some() => {
            rest.rsplit_once('-').map_or(rest, |(head, _)| head)
        }
        _ => stem,
    }
}

/// The projector of a model that reads images.
pub fn is_mmproj(file: &str) -> bool {
    file.to_ascii_lowercase().starts_with("mmproj")
}

fn size(path: &Path) -> u64 {
    fs::metadata(path).map_or(0, |m| m.len())
}

/// Every model of the kit, folder by folder (by name), then by id.
pub fn list(kit: &Kit) -> Vec<LocalModel> {
    let root = super::models_dir(kit);
    let Ok(entries) = fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .map(|e| e.path())
        .collect();
    dirs.sort();
    let mut out = Vec::new();
    let mut ids = BTreeSet::new();
    for dir in dirs {
        let folder = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        out.extend(in_folder(&dir, &folder, &mut ids));
    }
    out
}

/// The models of one folder: those of its manifest, then the files no
/// entry names (put there by hand).
fn in_folder(dir: &Path, folder: &str, ids: &mut BTreeSet<String>) -> Vec<LocalModel> {
    let manifest = read_manifest(dir).unwrap_or_else(|e| {
        crate::log::warn(&format!("{}: {e:#}", dir.display()));
        Manifest::default()
    });
    let mut files: Vec<String> = fs::read_dir(dir)
        .map(|d| {
            d.flatten()
                .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .filter(|n| n.to_ascii_lowercase().ends_with(".gguf"))
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    let mmproj = files.iter().find(|f| is_mmproj(f)).cloned();
    let mut named: BTreeSet<&str> = BTreeSet::new();
    let mut out = Vec::new();
    for e in &manifest.models {
        named.extend(e.files.iter().map(|f| f.name.as_str()));
        let downloaded = e
            .files
            .iter()
            .map(|f| {
                let file = dir.join(&f.name);
                if file.is_file() {
                    size(&file)
                } else {
                    crate::net::resume::downloaded(&file)
                }
            })
            .sum();
        let complete = e.complete && e.files.iter().all(|f| dir.join(&f.name).is_file());
        ids.insert(e.id.clone());
        out.push(LocalModel {
            id: e.id.clone(),
            name: e.name.clone(),
            repo: e.repo.clone(),
            folder: folder.to_owned(),
            files: e.files.iter().map(|f| f.name.clone()).collect(),
            mmproj: mmproj.clone(),
            bytes: e.files.iter().map(|f| f.bytes).sum(),
            downloaded,
            complete,
            catalog: e.catalog.clone(),
            ctx: e.ctx,
            by_hand: false,
        });
    }
    // The files of no entry: one model per stem, its parts together.
    let mut by_stem: Vec<(String, Vec<String>)> = Vec::new();
    for f in files
        .iter()
        .filter(|f| !is_mmproj(f) && !named.contains(f.as_str()))
    {
        let stem = model_stem(f).to_owned();
        match by_stem.iter_mut().find(|(s, _)| *s == stem) {
            Some((_, parts)) => parts.push(f.clone()),
            None => by_stem.push((stem, vec![f.clone()])),
        }
    }
    let repo = folder
        .split_once("--")
        .map(|(o, r)| format!("{o}/{r}"))
        .filter(|r| valid_repo(r))
        .unwrap_or_default();
    for (stem, parts) in by_stem {
        let mut id = super::slug(&stem);
        if id.is_empty() || ids.contains(&id) {
            id = super::slug(&format!("{stem}-{folder}"));
        }
        if id.is_empty() || !ids.insert(id.clone()) {
            continue;
        }
        let complete = parts.iter().all(|p| match part_of(model_stem_full(p)) {
            None => true,
            Some((_, count)) => parts.len() == count as usize,
        });
        out.push(LocalModel {
            id,
            name: stem.clone(),
            repo: repo.clone(),
            folder: folder.to_owned(),
            bytes: parts.iter().map(|p| size(&dir.join(p))).sum(),
            downloaded: 0,
            files: parts,
            mmproj: mmproj.clone(),
            complete,
            catalog: None,
            ctx: None,
            by_hand: true,
        });
    }
    out
}

/// The stem of a file name, part number included.
fn model_stem_full(file: &str) -> &str {
    file.strip_suffix(".gguf").unwrap_or(file)
}

/// The model `id` on disk.
pub fn find(kit: &Kit, id: &str) -> Option<LocalModel> {
    list(kit).into_iter().find(|m| m.id == id)
}

/// Removes model `id`: its files (and what is left of a download), its
/// entry, and its folder once empty. The server must not be using it
/// (on Windows a file it reads cannot be deleted).
pub fn remove(kit: &Kit, id: &str) -> Result<()> {
    let Some(model) = find(kit, id) else {
        bail!("model {id} not found");
    };
    let dir = super::models_dir(kit).join(&model.folder);
    for f in &model.files {
        let file = dir.join(f);
        for path in [file.clone(), crate::net::resume::part_of(&file)] {
            match fs::remove_file(&path) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    return Err(e).with_context(|| path.display().to_string())
                }
                _ => {}
            }
        }
    }
    let mut manifest = read_manifest(&dir)?;
    let before = manifest.models.len();
    manifest.models.retain(|e| e.id != id);
    if manifest.models.len() != before {
        write_manifest(&dir, &manifest)?;
    }
    // Nothing left but the manifest (and a projector no model uses).
    let left: Vec<String> = fs::read_dir(&dir)
        .map(|d| {
            d.flatten()
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    let only_ours = left
        .iter()
        .all(|n| n == MANIFEST || is_mmproj(n) && manifest.models.is_empty());
    if manifest.models.is_empty() && only_ours {
        let _ = fs::remove_dir_all(&dir);
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/llm/local.rs"]
mod tests;
