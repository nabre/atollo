//! A tree copied to another disk (`copy_tree`) and the checks before it: file size, free space.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use super::links::finish_links_here;

/// Copies the folder `from` into `to` (created); hard links become separate
/// files. On Unix a symbolic link is made again with the same relative
/// target when that stays inside the copy (the rules of the extraction,
/// [`finish_links`]): `bin/python3 -> python3.12` of a tree installed from
/// a temporary folder on another disk. A link that leaves the tree is
/// skipped with a warning; on a disk without links the target is copied.
/// On Windows links are skipped (sources made here have none).
pub fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    let mut links = Vec::new();
    copy_entries(from, to, &mut links)?;
    finish_links_here(to, links).map_err(io::Error::other)?;
    Ok(())
}

/// The files and folders of `copy_tree`; the links to make in `links`
/// (none on Windows: `platform::links::link_to_copy`).
fn copy_entries(from: &Path, to: &Path, links: &mut Vec<(PathBuf, String)>) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_symlink() {
            if let Some(t) = crate::platform::links::link_to_copy(&entry.path())? {
                match t.to_str() {
                    Some(t) => links.push((target, t.to_owned())),
                    None => crate::common::log::warn(&format!(
                        "link not copied (target not UTF-8): {}",
                        entry.path().display()
                    )),
                }
            }
            continue;
        }
        if kind.is_dir() {
            copy_entries(&entry.path(), &target, links)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// The files of `tree` (a folder or one file) with their sizes; links are
/// not counted (`copy_tree` makes them again on Unix and skips them on
/// Windows). On a disk without links their targets are copied too, so the
/// count is short there by what they point at.
fn sizes(tree: &Path) -> io::Result<Vec<(PathBuf, u64)>> {
    let meta = fs::symlink_metadata(tree)?;
    if !meta.is_dir() {
        return Ok(vec![(tree.to_path_buf(), meta.len())]);
    }
    let mut out = Vec::new();
    let mut stack = vec![tree.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                stack.push(entry.path());
            } else {
                out.push((entry.path(), entry.metadata()?.len()));
            }
        }
    }
    Ok(out)
}

/// The nearest folder of `path` that exists: the disk of a destination not
/// created yet.
fn existing_ancestor(path: &Path) -> &Path {
    path.ancestors().find(|p| p.is_dir()).unwrap_or(path)
}

/// `1.5 GiB`, `20 MiB`, `300 KiB`, `12 bytes`.
fn human(bytes: u64) -> String {
    const UNITS: [(&str, u64); 3] = [("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)];
    for (unit, size) in UNITS {
        if bytes >= size {
            return format!("{:.1} {unit}", bytes as f64 / size as f64);
        }
    }
    format!("{bytes} bytes")
}

/// Every file of `tree` fits on the disk of `dest`: FAT32 holds no file of
/// 4 GiB or more. Checked before the installed version is touched. When the
/// disk cannot be read, nothing is refused (a copy that fails still does).
pub fn check_fits(tree: &Path, dest: &Path) -> Result<()> {
    match crate::platform::volume::info(existing_ancestor(dest)) {
        Ok(v) => fits(tree, v.fs.max_file_size(), v.fs.as_str()),
        Err(e) => {
            crate::common::log::warn(&format!("{}: {e}", dest.display()));
            Ok(())
        }
    }
}

/// `check_fits` with the disk's limit given.
fn fits(tree: &Path, limit: Option<u64>, fs_name: &str) -> Result<()> {
    let Some(limit) = limit else {
        return Ok(());
    };
    if let Some((file, size)) = sizes(tree)?.into_iter().find(|(_, s)| *s > limit) {
        let name = file.strip_prefix(tree).unwrap_or(&file);
        let name = if name.as_os_str().is_empty() {
            &file
        } else {
            name
        };
        bail!(
            "{} is {} ({size} bytes): too large for the kit's disk ({fs_name}, at most {} per file)",
            name.display(),
            human(size),
            human(limit)
        );
    }
    Ok(())
}

/// The disk of `dest` has room for a copy of `tree`. When the disk cannot be
/// read, nothing is refused (a copy that fails still does).
pub fn check_space(tree: &Path, dest: &Path) -> Result<()> {
    match crate::platform::volume::free_space(existing_ancestor(dest)) {
        Ok(free) => has_space(tree, free, dest),
        Err(e) => {
            crate::common::log::warn(&format!("{}: {e}", dest.display()));
            Ok(())
        }
    }
}

/// `check_space` with the free bytes given.
fn has_space(tree: &Path, free: u64, dest: &Path) -> Result<()> {
    let needed: u64 = sizes(tree)?.iter().map(|(_, s)| s).sum();
    if needed > free {
        bail!(
            "not enough space for {}: {} needed, {} free",
            dest.display(),
            human(needed),
            human(free)
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/net/archive/copy.rs"]
mod tests;
