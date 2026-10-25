//! The folder a new project goes in: what is there already, and merging
//! the framework's files into it.

use std::fs;
use std::path::Path;

use serde::Serialize;

use super::Problem;
use crate::kit::Kit;

/// What is already in `projects\<name>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Existing {
    /// No folder: the creation makes it.
    New,
    /// An empty folder: the framework goes in it.
    Empty,
    /// Files but no Git: the framework's files are added, the others stay.
    Files,
    /// A Git repository: never used.
    Git,
}

fn existing_at(dir: &Path) -> Existing {
    if !dir.exists() {
        Existing::New
    } else if dir.join(".git").exists() {
        Existing::Git
    } else if fs::read_dir(dir).is_ok_and(|mut d| d.next().is_none()) {
        Existing::Empty
    } else {
        Existing::Files
    }
}

/// A folder of `projects\` and what is in it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub name: String,
    pub state: Existing,
}

/// The projects' folders with their state, for the wizard's name.
pub fn targets(kit: &Kit) -> Vec<Target> {
    crate::projects::folders(kit)
        .iter()
        .filter_map(|d| {
            Some(Target {
                name: d.file_name()?.to_string_lossy().into_owned(),
                state: existing_at(d),
            })
        })
        .collect()
}

/// The state of `projects\<name>`; a Git repository or a link is refused.
pub fn existing(kit: &Kit, name: &str) -> Result<Existing, Problem> {
    crate::kit::validate_project_name(name)
        .map_err(|e| ("Nome non valido: {0}", vec![e.to_string()]))?;
    let dir = kit.projects().join(name);
    if dir.exists() && !crate::platform::safe_path::is_dir(&kit.projects(), &dir) {
        return Err((
            "{0} non è una cartella dei progetti.",
            vec![name.to_owned()],
        ));
    }
    match existing_at(&dir) {
        Existing::Git => Err((
            "{0} è già un repository Git: scegli un altro nome.",
            vec![name.to_owned()],
        )),
        state => Ok(state),
    }
}

/// Moves what is in `from` into `to`: what `to` already has stays as it is
/// (its path relative to `to` goes in `kept`), folders are merged.
pub fn merge_into(from: &Path, to: &Path, kept: &mut Vec<String>) -> std::io::Result<()> {
    merge_at(from, to, Path::new(""), kept)
}

fn merge_at(from: &Path, to: &Path, rel: &Path, kept: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        let rel = rel.join(entry.file_name());
        match fs::symlink_metadata(&dst) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => fs::rename(&src, &dst)?,
            Err(e) => return Err(e),
            Ok(m)
                if m.is_dir()
                    && !crate::platform::safe_path::is_reparse(&m)
                    && entry.file_type()?.is_dir() =>
            {
                merge_at(&src, &dst, &rel, kept)?;
            }
            Ok(_) => kept.push(rel.to_string_lossy().into_owned()),
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/frameworks/target.rs"]
mod tests;
