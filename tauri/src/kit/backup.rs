//! A copy of the kit and back (report v6/04, N-12, DECISIONS.md D-163):
//! the README's backup rule done by the app.
//!
//! - **Export**: `projects\` and `data\` but what is downloaded again or
//!   belongs to this PC ([`left_out`]: the programs, the caches, the local
//!   models, the WebView's profile, the logs), into a folder or a zip, with
//!   `atollo-backup.json`; the database servers are stopped first, so their
//!   files are a coherent copy, and started again after;
//! - **Import**: the copy is checked and put in `.atollo-import\` of the
//!   kit, then applied at the next start before anything is opened
//!   ([`apply_pending`]): what it replaces goes to
//!   `.atollo-before-import-<stamp>\`, never deleted; the programs this kit
//!   already has stay, the missing ones are downloaded again; a copy of a
//!   kit elsewhere is moved as any moved kit (`Kit::check_moved`).

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const MARKER: &str = "atollo-backup.json";
const STAGING: &str = ".atollo-import";
/// In the staging folder once the whole copy is there.
const READY: &str = ".ready";
const ASIDE: &str = ".atollo-before-import-";

/// The folders of `data\` that stay this PC's: the programs, the caches,
/// the local models, the temporary files, the WebView's profile.
const LOCAL: [&str; 6] = ["apps", "cache", "models", "tmp", "webview2", "downloads"];

/// A path from the kit's root (`/`) that is not copied. Also the state of
/// the kit's Docker Sandboxes daemon (`…/state/sandboxd` in the kit's
/// home: the VMs' images and sparse disks, tens of GB once copied): a
/// sandbox missing there is made again from its kit (D-175); its settings
/// and sign-in beside it are copied.
pub fn left_out(rel: &str) -> bool {
    if rel.starts_with("data/home/") && rel.ends_with("/state/sandboxd") {
        return true;
    }
    let mut parts = rel.split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("data"), Some(dir), _) if LOCAL.contains(&dir) => true,
        (Some("data"), Some("app"), Some("logs" | "diagnostics" | "conda")) => true,
        _ => false,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// The version of Atollo that made it.
    pub atollo: String,
    /// When (Unix seconds).
    pub created: u64,
    /// The kit it comes from.
    pub root: String,
    pub files: usize,
    pub bytes: u64,
}

use crate::common::time::now_secs as now;

/// Every file of `projects\` and `data\` a copy takes (links skipped), as
/// `/` paths from the root, sorted.
pub fn files(root: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let mut stack: Vec<String> = vec!["projects".into(), "data".into()];
    while let Some(rel) = stack.pop() {
        let Ok(entries) = fs::read_dir(root.join(&rel)) else {
            continue;
        };
        for e in entries.flatten() {
            let Ok(kind) = e.file_type() else {
                continue;
            };
            let child = format!("{rel}/{}", e.file_name().to_string_lossy());
            if left_out(&child) {
                continue;
            }
            if kind.is_dir() {
                stack.push(child);
            } else if kind.is_file() {
                out.push((child, e.path()));
            }
        }
    }
    out.sort();
    out
}

/// What an export made.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Exported {
    pub path: String,
    pub files: usize,
    pub bytes: u64,
}

/// Copies the kit at `root` into `dest` (a folder `Atollo-<stamp>`, or
/// `Atollo-<stamp>.zip`). The database servers must be stopped.
pub fn export(root: &Path, dest: &Path, zip: bool) -> Result<Exported> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let dest_real = fs::canonicalize(dest).with_context(|| dest.display().to_string())?;
    if dest_real.starts_with(&root) {
        bail!("the copy cannot go inside the kit");
    }
    let list = files(&root);
    let bytes: u64 = list
        .iter()
        .map(|(_, p)| fs::metadata(p).map_or(0, |m| m.len()))
        .sum();
    let manifest = Manifest {
        atollo: env!("CARGO_PKG_VERSION").into(),
        created: now(),
        root: root.display().to_string(),
        files: list.len(),
        bytes,
    };
    let marker = serde_json::to_string_pretty(&manifest)? + "\n";
    let name = format!("Atollo-{}", crate::agents::sync::stamp());
    let path = if zip {
        let path = dest.join(format!("{name}.zip"));
        let part = dest.join(format!("{name}.zip.part"));
        let file = fs::File::create(&part).with_context(|| part.display().to_string())?;
        let mut w = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .large_file(true);
        w.start_file(MARKER, options)?;
        w.write_all(marker.as_bytes())?;
        let mut buf = vec![0u8; 1024 * 1024];
        for (rel, p) in &list {
            w.start_file(rel.as_str(), options)?;
            let mut f = fs::File::open(p).with_context(|| rel.clone())?;
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                w.write_all(&buf[..n])?;
            }
        }
        w.finish()?;
        fs::rename(&part, &path)?;
        path
    } else {
        let path = dest.join(&name);
        let part = dest.join(format!("{name}.part"));
        for (rel, p) in &list {
            let to = part.join(rel);
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(p, &to).with_context(|| rel.clone())?;
        }
        fs::create_dir_all(&part)?;
        fs::write(part.join(MARKER), &marker)?;
        fs::rename(&part, &path)?;
        path
    };
    Ok(Exported {
        path: path.display().to_string(),
        files: manifest.files,
        bytes,
    })
}

fn staging(root: &Path) -> PathBuf {
    root.join(STAGING)
}

/// A copy waits to be applied at the next start.
pub fn pending(root: &Path) -> bool {
    staging(root).join(READY).is_file()
}

/// The copy waiting is given up.
pub fn cancel(root: &Path) -> Result<()> {
    let s = staging(root);
    if s.exists() {
        fs::remove_dir_all(&s)?;
    }
    Ok(())
}

/// Checks the copy at `source` (a folder or a zip made by [`export`]) and
/// puts it in the kit's staging folder; applied at the next start.
pub fn stage(root: &Path, source: &Path) -> Result<Manifest> {
    let real = fs::canonicalize(source).with_context(|| source.display().to_string())?;
    let root_real = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    if real.starts_with(&root_real) {
        bail!("the copy is inside this kit");
    }
    cancel(root)?;
    let to = staging(root);
    fs::create_dir_all(&to)?;
    let result = if real.is_dir() {
        stage_folder(&real, &to)
    } else {
        stage_zip(&real, &to)
    };
    match result {
        Ok(m) => {
            fs::write(to.join(READY), "")?;
            Ok(m)
        }
        Err(e) => {
            let _ = cancel(root);
            Err(e)
        }
    }
}

fn read_marker(text: &str) -> Result<Manifest> {
    serde_json::from_str(text).context("not a copy of Atollo: atollo-backup.json is not valid")
}

/// A path of the copy that may be applied: under `projects/` or `data/`,
/// no `..`, not left out.
fn allowed(rel: &str) -> bool {
    (rel.starts_with("projects/") || rel.starts_with("data/"))
        && !rel
            .split('/')
            .any(|p| p.is_empty() || p == "." || p == "..")
        && !left_out(rel)
}

fn stage_folder(from: &Path, to: &Path) -> Result<Manifest> {
    let text = fs::read_to_string(from.join(MARKER))
        .context("not a copy of Atollo: atollo-backup.json is missing")?;
    let manifest = read_marker(&text)?;
    for (rel, p) in files(from) {
        if !allowed(&rel) {
            continue;
        }
        let dst = to.join(&rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&p, &dst).with_context(|| rel.clone())?;
    }
    Ok(manifest)
}

fn stage_zip(from: &Path, to: &Path) -> Result<Manifest> {
    let file = fs::File::open(from).with_context(|| from.display().to_string())?;
    let mut zip = zip::ZipArchive::new(file).context("not a zip")?;
    let manifest = {
        let mut m = zip
            .by_name(MARKER)
            .context("not a copy of Atollo: atollo-backup.json is missing")?;
        let mut text = String::new();
        m.read_to_string(&mut text)?;
        read_marker(&text)?
    };
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let rel = entry.name().replace('\\', "/");
        if entry.is_dir() || !allowed(&rel) || entry.enclosed_name().is_none() {
            continue;
        }
        let dst = to.join(&rel);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&dst).with_context(|| rel.clone())?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(manifest)
}

/// The moves of the swap, in order: what replaces is aside first, then the
/// copy in its place.
fn swap_moves(root: &Path, from: &Path, aside: &Path) -> Vec<(PathBuf, PathBuf)> {
    let data = root.join("data");
    let mut moves = Vec::new();
    if root.join("projects").exists() {
        moves.push((root.join("projects"), aside.join("projects")));
    }
    for e in fs::read_dir(&data).into_iter().flatten().flatten() {
        let name = e.file_name();
        if !LOCAL.contains(&name.to_string_lossy().as_ref()) {
            moves.push((e.path(), aside.join("data").join(&name)));
        }
    }
    if from.join("projects").exists() {
        moves.push((from.join("projects"), root.join("projects")));
    }
    for e in fs::read_dir(from.join("data"))
        .into_iter()
        .flatten()
        .flatten()
    {
        moves.push((e.path(), data.join(e.file_name())));
    }
    moves
}

/// At the start, before anything is opened: the copy waiting takes the
/// place of `projects\` and of `data\` but [`LOCAL`], which go to
/// `.atollo-before-import-<stamp>\`. All or nothing: a move that fails (a
/// file held open) puts back those done, the copy keeps waiting for the next
/// start. Returns that folder.
pub fn apply_pending(root: &Path) -> Result<Option<PathBuf>> {
    if !pending(root) {
        return Ok(None);
    }
    let from = staging(root);
    let aside = root.join(format!("{ASIDE}{}", crate::agents::sync::stamp()));
    fs::create_dir_all(aside.join("data"))?;
    fs::create_dir_all(root.join("data"))?;
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
    for (src, dst) in swap_moves(root, &from, &aside) {
        if let Err(e) = fs::rename(&src, &dst) {
            for (back_src, back_dst) in done.iter().rev() {
                if let Err(undo) = fs::rename(back_dst, back_src) {
                    crate::common::log::warn(&format!(
                        "import: {} not put back: {undo}",
                        back_dst.display()
                    ));
                }
            }
            let _ = fs::remove_dir_all(&aside);
            return Err(e).with_context(|| src.display().to_string());
        }
        done.push((src, dst));
    }
    fs::create_dir_all(root.join("projects"))?;
    fs::remove_dir_all(&from)?;
    Ok(Some(aside))
}

#[cfg(test)]
#[path = "../../../test/rust/kit/backup.rs"]
mod tests;
