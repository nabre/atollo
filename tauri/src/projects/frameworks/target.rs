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
    if dir.exists() && !crate::os::safe_path::is_dir(&kit.projects(), &dir) {
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
                    && !crate::os::safe_path::is_reparse(&m)
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn existing_folders() {
        let tmp = TempDir::new("frameworks-targets");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let p = kit.projects();
        fs::create_dir_all(p.join("empty")).unwrap();
        fs::create_dir_all(p.join("files")).unwrap();
        fs::write(p.join("files").join("notes.txt"), "x").unwrap();
        fs::create_dir_all(p.join("repo").join(".git")).unwrap();
        let states: Vec<(String, Existing)> = targets(&kit)
            .into_iter()
            .map(|t| (t.name, t.state))
            .collect();
        assert_eq!(
            states,
            [
                ("empty".to_owned(), Existing::Empty),
                ("files".to_owned(), Existing::Files),
                ("repo".to_owned(), Existing::Git)
            ]
        );
        assert_eq!(existing(&kit, "new").unwrap(), Existing::New);
        assert_eq!(existing(&kit, "files").unwrap(), Existing::Files);
        assert!(existing(&kit, "repo").is_err());
        assert!(existing(&kit, "..").is_err());
    }

    #[test]
    fn merging_keeps_what_is_there() {
        let tmp = TempDir::new("frameworks-merge");
        let (from, to) = (tmp.0.join("from"), tmp.0.join("to"));
        fs::create_dir_all(from.join("src")).unwrap();
        fs::create_dir_all(to.join("src")).unwrap();
        fs::write(from.join("package.json"), "new").unwrap();
        fs::write(from.join("README.md"), "new").unwrap();
        fs::write(from.join("src").join("main.ts"), "new").unwrap();
        fs::write(from.join("src").join("app.css"), "new").unwrap();
        fs::write(to.join("README.md"), "mine").unwrap();
        fs::write(to.join("src").join("app.css"), "mine").unwrap();
        let mut kept = Vec::new();
        merge_into(&from, &to, &mut kept).unwrap();
        kept.sort();
        assert_eq!(kept, ["README.md", r"src\app.css"]);
        assert_eq!(fs::read_to_string(to.join("README.md")).unwrap(), "mine");
        assert_eq!(fs::read_to_string(to.join("package.json")).unwrap(), "new");
        assert_eq!(
            fs::read_to_string(to.join("src").join("main.ts")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(to.join("src").join("app.css")).unwrap(),
            "mine"
        );
    }
}
