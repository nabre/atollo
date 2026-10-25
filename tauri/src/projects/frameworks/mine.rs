//! «Salva come base» (report v6/04, N-10, DECISIONS.md D-162): a project's
//! files as a base of the wizard, under «Le mie basi».
//!
//! - the files: those Git would commit (`git ls-files -co --exclude-standard`,
//!   so `.gitignore` is respected) in a repository, otherwise every file but
//!   the folders that hold no sources (`node_modules`, `vendor`, `.venv`…);
//!   never `.env` (passwords) nor `.git`; the top folders and files the user
//!   leaves out are skipped too;
//! - copied to `data\frameworks\mine\<id>`, and a base `<id>` with
//!   `template` in `data\app\frameworks.json`: creating a project from it
//!   copies the folder instead of running commands;
//! - a base of the same name is replaced; one removed from the bases' editor
//!   has its folder removed at the next save ([`prune`]).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use super::{Kind, Piece};
use crate::env::KitEnv;
use crate::kit::Kit;

/// The category of these bases in the wizard.
pub const CATEGORY: &str = "mine";

pub fn mine_dir(kit: &Kit) -> PathBuf {
    kit.data().join("frameworks").join("mine")
}

/// The folder of base `template`, when its name is one Atollo made.
pub fn template_dir(kit: &Kit, template: &str) -> Option<PathBuf> {
    super::validate::valid_id(template).then(|| mine_dir(kit).join(template))
}

/// `.env` and its copies hold passwords: never in a base (`.env.example`
/// does not).
fn secret(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    name == ".env" || (name.starts_with(".env.") && name != ".env.example")
}

/// The files of `dir` a base takes, as `/` paths, sorted.
pub fn files_of(kit: &Kit, env: &KitEnv, dir: &Path) -> Vec<String> {
    let listed = if dir.join(".git").exists() {
        git_files(kit, env, dir)
    } else {
        None
    };
    let mut files = listed.unwrap_or_else(|| walk(dir));
    files.retain(|rel| {
        !secret(rel)
            && !rel.starts_with(".git/")
            && crate::platform::safe_path::is_file(dir, &dir.join(rel))
    });
    files.sort();
    files.dedup();
    files
}

/// What Git would commit: the tracked files and the new ones `.gitignore`
/// does not leave out. `None` when Git is missing or fails. The project's
/// `.git/config` cannot start a program of its own through
/// `core.fsmonitor` (D-167): the app runs this, not the user.
fn git_files(kit: &Kit, env: &KitEnv, dir: &Path) -> Option<Vec<String>> {
    let git = crate::setup::kit_path(kit, r"git\cmd\git.exe");
    if !git.is_file() {
        return None;
    }
    let out = crate::platform::jobs::kit_command(&git, env)
        .args([
            "-c",
            "core.fsmonitor=false",
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()?;
    out.status.success().then(|| {
        String::from_utf8_lossy(&out.stdout)
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect()
    })
}

/// Every file but the folders that hold no sources, without Git.
fn walk(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let Ok(entries) = fs::read_dir(dir.join(&rel)) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else {
                continue;
            };
            let name = e.file_name().to_string_lossy().into_owned();
            let child = rel.join(&name);
            if kind.is_dir() {
                if !crate::detect::IGNORED.contains(&name.as_str()) {
                    stack.push(child);
                }
            } else if kind.is_file() {
                out.push(child.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out
}

/// A top folder or file of the project, with what a base would take of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub dir: bool,
    pub files: usize,
    pub bytes: u64,
}

/// What «Salva come base» would copy, by top folder and file.
pub fn preview(kit: &Kit, env: &KitEnv, dir: &Path) -> Vec<Entry> {
    let mut top: BTreeMap<String, Entry> = BTreeMap::new();
    for rel in files_of(kit, env, dir) {
        let (name, is_dir) = match rel.split_once('/') {
            Some((first, _)) => (first.to_owned(), true),
            None => (rel.clone(), false),
        };
        let bytes = fs::metadata(dir.join(&rel)).map_or(0, |m| m.len());
        let e = top.entry(name.clone()).or_insert(Entry {
            name,
            dir: is_dir,
            files: 0,
            bytes: 0,
        });
        e.files += 1;
        e.bytes += bytes;
    }
    top.into_values().collect()
}

/// The id of a base named `name`: `mine-` and its letters and digits.
pub fn id_of(name: &str) -> Option<String> {
    let mut slug = String::new();
    for c in name.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let slug = slug.trim_end_matches('-');
    let id = format!("{CATEGORY}-{}", &slug[..slug.len().min(30)]);
    let id = id.trim_end_matches('-').to_owned();
    (!slug.is_empty() && super::validate::valid_id(&id)).then_some(id)
}

/// Copies `dir`'s files (but the top entries in `skip`) as base `name` and
/// adds it to the bases. Returns its id.
pub fn save(
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    name: &str,
    desc: &str,
    skip: &[String],
) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 60 {
        bail!("name: 1 to 60 characters");
    }
    let id = id_of(name).context("name: letters or digits needed")?;
    let files: Vec<String> = files_of(kit, env, dir)
        .into_iter()
        .filter(|rel| {
            let top = rel.split('/').next().unwrap_or(rel);
            !skip.iter().any(|s| s == top)
        })
        .collect();
    if files.is_empty() {
        bail!("no file to save");
    }
    // Made aside, then put in place of the old one of the same name.
    let root = mine_dir(kit);
    let staging = root.join(format!(".{id}.new"));
    let _ = fs::remove_dir_all(&staging);
    for rel in &files {
        let to = staging.join(rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(dir.join(rel), &to).with_context(|| rel.clone())?;
    }
    let target = root.join(&id);
    if target.exists() {
        fs::remove_dir_all(&target).with_context(|| target.display().to_string())?;
    }
    fs::rename(&staging, &target)?;
    let mut pieces = super::load(kit).pieces;
    pieces.retain(|p| p.id != id);
    pieces.push(Piece {
        id: id.clone(),
        kind: Kind::Base,
        name: name.to_owned(),
        desc: desc.trim().to_owned(),
        home: String::new(),
        slot: None,
        category: Some(CATEGORY.into()),
        needs: Vec::new(),
        create: Vec::new(),
        template: Some(id.clone()),
        steps: Vec::new(),
        actions: Vec::new(),
        env: Vec::new(),
        database: None,
        note: None,
        on: BTreeMap::new(),
    });
    super::save(kit, pieces).map_err(|(text, args)| anyhow::anyhow!("{text} {args:?}"))?;
    Ok(id)
}

/// Removes the folders of bases no longer in `pieces` (removed from the
/// bases' editor), and what a save left halfway.
pub fn prune(kit: &Kit, pieces: &[Piece]) {
    let Ok(entries) = fs::read_dir(mine_dir(kit)) else {
        return;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let used = pieces
            .iter()
            .any(|p| p.template.as_deref() == Some(name.as_str()));
        if !used && e.file_type().is_ok_and(|k| k.is_dir()) {
            let _ = fs::remove_dir_all(e.path());
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/frameworks/mine.rs"]
mod tests;
