//! Project skills: `<project>\.agents\skills` (Codex, Antigravity) <->
//! `<project>\.claude\skills` (Claude Code), port of `project.js`. Nothing is
//! created in a project that has neither folder; a link or junction in place
//! of these folders stops the sync (a cloned repository must not make the kit
//! read or write elsewhere). The state stays in the kit.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use sha2::{Digest, Sha256};

use super::tree::{is_reparse, sync_tree};
use super::{Paths, Report};

/// The closest folder with `.git` above `dir` (or `dir`); inside `projects\`
/// the search stops there.
fn project_root(p: &Paths, dir: &Path) -> PathBuf {
    let in_projects = dir.starts_with(&p.projects) && dir != p.projects;
    let mut d = dir.to_path_buf();
    loop {
        if in_projects && d == p.projects {
            return dir.to_path_buf();
        }
        if d.join(".git").exists() {
            return d;
        }
        match d.parent() {
            Some(parent) => d = parent.to_path_buf(),
            None => return dir.to_path_buf(),
        }
    }
}

pub fn sync(p: &Paths, dir: &Path, report: &mut Report) -> Result<()> {
    if !dir.exists() {
        return Ok(());
    }
    let root = project_root(p, dir);
    let shared = root.join(r".agents\skills");
    let claude = root.join(r".claude\skills");
    if !shared.exists() && !claude.exists() {
        return Ok(());
    }
    for d in [
        root.join(".agents"),
        shared.clone(),
        root.join(".claude"),
        claude.clone(),
    ] {
        if let Ok(m) = fs::symlink_metadata(&d) {
            if is_reparse(&d) || !m.is_dir() {
                report.warn(format!(
                    "{} is a link or not a folder: project skills not synced",
                    d.display()
                ));
                return Ok(());
            }
        }
    }
    // Same key wherever the kit is plugged in, for projects inside it.
    let id = root
        .strip_prefix(&p.root)
        .map_or_else(|_| root.clone(), Path::to_path_buf);
    let id = id.display().to_string().to_lowercase().replace('\\', "/");
    let key: String = Sha256::digest(id.as_bytes())
        .iter()
        .take(8)
        .map(|b| format!("{b:02x}"))
        .collect();
    sync_tree(
        p,
        &shared,
        &claude,
        &format!("projects/{key}"),
        true,
        ".agents/skills",
        report,
    )
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/project.rs"]
mod tests;
