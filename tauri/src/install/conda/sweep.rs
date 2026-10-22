//! What no list of an environment on disk names goes (`sweep`), the lists
//! of a removed component (`forget`), everything («Libera», `free`), and the
//! state of the kept packages for the Settings page.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;

use super::list::parse_explicit;
use super::{lock_name, locks_dir, pkgs_dir, root};
use crate::install::installer;
use crate::kit::Kit;
use crate::settings::Settings;
use crate::setup::Catalog;

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

/// The lists of component `id` on this target removed, those of every line
/// or of `line` only: the component (or the line) was removed, so nothing is
/// left to create again. The archives they named go with the next `sweep`,
/// which keeps those another list still names.
pub fn forget(kit: &Kit, id: &str, line: Option<&str>) {
    let stem = line.map(|l| lock_name(id, l));
    let prefix = format!("{id}@");
    for entry in fs::read_dir(locks_dir(kit)).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(own) = name
            .strip_suffix(".txt")
            .or_else(|| name.strip_suffix(".json"))
        else {
            continue;
        };
        let mine = match &stem {
            Some(stem) => own == stem,
            None => own.starts_with(&prefix),
        };
        if mine {
            let _ = fs::remove_file(entry.path());
        }
    }
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

#[cfg(test)]
#[path = "../../../../test/rust/install/conda/sweep.rs"]
mod tests;
