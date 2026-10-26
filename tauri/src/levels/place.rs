//! Which project a folder belongs to (LEVELS.md §3.3): the rule of
//! `runtime/vm.ps1` (`$projects`, `$below`) and `ipc::project_dir`, in one
//! place. A project is a real folder right under `projects/`, reached
//! without a link or a junction on the way (D-77): a folder seen through a
//! link is no project's, whatever its name.

use std::ffi::OsStr;
use std::path::{Component, Path};

use crate::kit::Kit;
use crate::platform::safe_path;

use super::Place;

/// The project `dir` belongs to: the first folder under `kit.projects()`,
/// read without following links (`safe_path`); `Folder` otherwise. Names
/// are compared as the PC's file system does: without regard to case on
/// Windows and macOS, the project's name being the one on disk.
pub fn place_of(kit: &Kit, dir: &Path) -> Place {
    let dir = crate::kit::strip_verbatim(dir);
    let dir = safe_path::lexical(&dir).unwrap_or(dir);
    project_of(kit, &dir).unwrap_or(Place::Folder { dir })
}

fn project_of(kit: &Kit, dir: &Path) -> Option<Place> {
    if !dir.is_absolute() {
        return None;
    }
    let fold = folds_case(kit);
    let projects = kit.projects();
    let rest = under(dir, &projects, fold)?;
    let mut parts = rest.iter();
    let first = parts.next()?;
    let name = on_disk(&projects, first, fold)?;
    crate::kit::validate_project_name(&name).ok()?;
    let project = projects.join(&name);
    if !safe_path::is_dir(&projects, &project) {
        return None;
    }
    let below: Vec<String> = parts.map(|p| p.to_string_lossy().into_owned()).collect();
    let mut inside = project.clone();
    inside.extend(&below);
    if !safe_path::no_links(&projects, &inside) {
        return None;
    }
    Some(Place::Project {
        name,
        dir: project,
        below: (!below.is_empty()).then(|| below.join("/")),
    })
}

/// True where the file system ignores case by default: Windows, macOS.
fn folds_case(kit: &Kit) -> bool {
    kit.os().is_windows() || kit.os().is_macos()
}

/// The names of `dir` under `root`, compared one by one (`fold`: without
/// regard to case); `None` when `dir` is not under `root`.
fn under(dir: &Path, root: &Path, fold: bool) -> Option<Vec<std::ffi::OsString>> {
    let mut dir_parts = dir.components();
    for want in root.components() {
        let got = dir_parts.next()?;
        if !same(got.as_os_str(), want.as_os_str(), fold) {
            return None;
        }
    }
    dir_parts
        .map(|c| match c {
            Component::Normal(name) => Some(name.to_os_string()),
            _ => None,
        })
        .collect()
}

fn same(a: &OsStr, b: &OsStr, fold: bool) -> bool {
    a == b || (fold && a.to_string_lossy().to_lowercase() == b.to_string_lossy().to_lowercase())
}

/// The name of the folder `name` under `projects` as it is on disk: the
/// same name if there is one, else (`fold`) the one equal but for case.
fn on_disk(projects: &Path, name: &OsStr, fold: bool) -> Option<String> {
    let entries: Vec<String> = std::fs::read_dir(projects)
        .ok()?
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();
    let exact = entries.iter().find(|e| OsStr::new(e.as_str()) == name);
    exact
        .or_else(|| {
            fold.then(|| entries.iter().find(|e| same(e.as_ref(), name, true)))
                .flatten()
        })
        .cloned()
}

#[cfg(test)]
#[path = "../../../test/rust/levels/place.rs"]
mod tests;
