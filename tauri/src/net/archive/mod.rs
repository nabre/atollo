//! Archives and folder swaps for the installer (port of nabre/atollo
//! `Expand-Package`, `Replace-Dir` and `Replace-File`).
//!
//! Extraction refuses entries that would land outside the target (absolute
//! paths, `..`, drive prefixes) and entries whose way from the target crosses
//! a symbolic link; a file is written only as a new file (an existing one is
//! removed first, never truncated through a link). Links are never created on
//! Windows. On Unix symbolic links are created last, after every file and
//! folder (as GNU tar does), and only if the target is relative, has no `\`
//! or `:`, and resolves inside the target folder through real folders only
//! (so a chain of links cannot climb out); a hard link is created only to a
//! regular file already extracted there. Other links are skipped. On a disk
//! without links (exFAT, FAT) the target is copied in place of the link. File
//! times are not restored (exFAT rejects them; `tar -m` in the kit). Swaps are atomic from the user's point of view: the
//! new folder is complete before the old one moves, and a failure puts the
//! old one back.
//!
//! Files: `mod` (the kinds, safe names, `extract`, `pick_inner`), `formats`
//! (zip, tar and 7z), `links` (symbolic and hard links, copies where the
//! disk has none), `copy` (a tree to another disk and the checks before
//! it), `swap` (folders and files put in place, set aside, removed).

mod copy;
mod formats;
mod links;
mod swap;

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

pub use copy::{check_fits, check_space, copy_tree};
use formats::{extract_7z, extract_tar_gz, extract_tar_xz, extract_zip};
pub(crate) use links::links_unsupported;
pub use swap::{clean_leftovers, rebuild_in_place, remove_dir, replace_file, stage_into, InUse};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Zip,
    TarGz,
    TarXz,
    /// A 7z archive, also inside a self-extracting `.7z.exe`.
    SevenZip,
    /// Not an archive: the download is the file itself (an exe), copied
    /// into the target folder under its name.
    File,
}

/// Which part of the extracted tree becomes the component folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inner {
    /// The archive root.
    Root,
    /// Its only top-level folder (error if there is not exactly one).
    Single,
    /// The only top-level folder if there is exactly one, else the root.
    Auto,
    /// A named subfolder.
    Named(String),
    /// The only folder inside a named subfolder (`package/vendor/<target>`).
    SingleUnder(String),
    /// Its only top-level folder, followed by a relative path inside it
    /// (e.g. `Contents/Home` for macOS Temurin JDK `.tar.gz`).
    SingleThen(&'static str),
}

/// `base` + an archive entry name, or `None` if the name would escape `base`.
/// Splits on both `'/'` and `'\\'` so paths work identically on Windows,
/// macOS, and Linux without creating literal backslashes on Unix. A `:` is
/// refused on Windows only (a drive, `C:rel`, or an alternate data stream);
/// on macOS and Linux it is an ordinary character of a file name, which
/// tar archives for those systems may have.
pub fn safe_join(base: &Path, name: &str) -> Option<PathBuf> {
    if name.starts_with(['/', '\\']) || (cfg!(windows) && name.contains(':')) {
        return None;
    }
    let mut out = base.to_path_buf();
    let mut depth = 0usize;
    for seg in name.split(['/', '\\']) {
        match seg {
            "" | "." => {}
            ".." => return None,
            part => {
                out.push(part);
                depth += 1;
            }
        }
    }
    (depth > 0).then_some(out)
}

#[cfg(unix)]
fn set_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o755);
        let _ = fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) {}

pub fn extract(archive: &Path, kind: Kind, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    match kind {
        Kind::Zip => extract_zip(archive, dest),
        Kind::TarGz => extract_tar_gz(archive, dest),
        Kind::TarXz => extract_tar_xz(archive, dest),
        Kind::SevenZip => extract_7z(archive, dest),
        Kind::File => {
            let name = archive.file_name().context("file name")?;
            let out = dest.join(name);
            fs::copy(archive, &out)?;
            set_executable(&out);
            Ok(())
        }
    }
    .with_context(|| format!("extracting {}", archive.display()))
}

/// Refuses `path` if it, or a folder between `dest` and it, is a symbolic
/// link (or another link Windows reports as one): nothing is ever written
/// through a link, wherever it came from.
fn no_link_on_the_way(dest: &Path, path: &Path) -> Result<()> {
    let rel = path
        .strip_prefix(dest)
        .with_context(|| format!("{} is outside {}", path.display(), dest.display()))?;
    let mut cur = dest.to_path_buf();
    for part in rel.components() {
        cur.push(part);
        match fs::symlink_metadata(&cur) {
            Ok(meta) if meta.file_type().is_symlink() => {
                bail!("unsafe path in archive: {} is a link", cur.display())
            }
            Ok(_) => {}
            Err(_) => break, // Not there yet: nothing below it exists either.
        }
    }
    Ok(())
}

/// Creates the folder `dir` inside `dest`, never through a link.
fn prepare_dir(dest: &Path, dir: &Path) -> Result<()> {
    no_link_on_the_way(dest, dir)?;
    fs::create_dir_all(dir)?;
    Ok(())
}

/// A new file at `path` inside `dest`: what was there (a file of an
/// earlier entry, or a link) is removed, never truncated or followed.
fn create_file(dest: &Path, path: &Path) -> Result<File> {
    if let Some(dir) = path.parent() {
        prepare_dir(dest, dir)?;
    }
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.is_dir() => {
            bail!("unsafe path in archive: {} is a folder", path.display())
        }
        Ok(_) => fs::remove_file(path)?,
        Err(_) => {}
    }
    Ok(fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?)
}

/// The folder that `inner` designates inside an extracted tree.
pub fn pick_inner(extracted: &Path, inner: &Inner) -> Result<PathBuf> {
    let dirs = || -> Result<(Vec<PathBuf>, usize)> {
        let mut dirs = Vec::new();
        let mut total = 0;
        for entry in fs::read_dir(extracted)? {
            let entry = entry?;
            total += 1;
            if entry.file_type()?.is_dir() {
                dirs.push(entry.path());
            }
        }
        Ok((dirs, total))
    };
    match inner {
        Inner::Root => Ok(extracted.to_path_buf()),
        Inner::Named(name) => {
            let path = crate::fsx::join_rel(extracted, name);
            if !path.is_dir() {
                bail!("the archive has no folder {name}");
            }
            Ok(path)
        }
        Inner::SingleUnder(name) => {
            let sub = crate::fsx::join_rel(extracted, name);
            pick_inner(&sub, &Inner::Single).with_context(|| format!("in {name}"))
        }
        Inner::SingleThen(sub) => {
            let top = pick_inner(extracted, &Inner::Single)?;
            let path = crate::fsx::join_rel(&top, sub);
            if !path.is_dir() {
                bail!("the archive has no folder {sub} inside {}", top.display());
            }
            Ok(path)
        }
        Inner::Single => match dirs()? {
            (dirs, 1) if dirs.len() == 1 => Ok(dirs.into_iter().next().unwrap_or_default()),
            (_, n) => bail!("expected one folder at the top of the archive, found {n} items"),
        },
        Inner::Auto => match dirs()? {
            (dirs, 1) if dirs.len() == 1 => Ok(dirs.into_iter().next().unwrap_or_default()),
            _ => Ok(extracted.to_path_buf()),
        },
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/net/archive/mod.rs"]
mod tests;
