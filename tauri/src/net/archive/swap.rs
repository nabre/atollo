//! Folders and files put in place of the installed ones, set aside, removed, and their leftovers.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};

use super::copy::{check_fits, check_space, copy_tree};

fn stamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis().to_string())
        .unwrap_or_default()
}

#[derive(Debug, thiserror::Error)]
#[error("{0} is in use: close the programs of the kit and try again")]
pub struct InUse(pub PathBuf);

/// Puts `new` (a complete folder on the same disk) in place of `dest`. The
/// old folder is renamed first. Windows also refuses that when a program of
/// the PC only keeps one of its folders open (an indexer, an antivirus, File
/// Explorer): then, if no program runs from it, the old tree moves out entry
/// by entry, and a folder that cannot move is emptied and stays, empty, for
/// the new tree to fill. A program running from it, or a file locked, stops
/// it: everything goes back and `InUse` is returned.
pub fn replace_dir(new: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let old = sibling(dest, "old");
    let had_old = dest.exists();
    if had_old && fs::rename(dest, &old).is_err() {
        // A running program could be moved too (Windows allows renaming a
        // running exe), and would then load the new files with the old ones.
        if !crate::os::processes::running_from(&crate::os::processes::all(), dest).is_empty() {
            return Err(InUse(dest.to_path_buf()).into());
        }
        if let Err(e) = move_tree(dest, &old) {
            crate::common::log::info(&format!("{}: {e}", dest.display()));
            let _ = move_tree(&old, dest);
            let _ = fs::remove_dir_all(&old);
            return Err(InUse(dest.to_path_buf()).into());
        }
    }
    // `dest` is gone, or only empty folders held open are left in it.
    let moved = retry_denied(|| {
        if dest.exists() {
            move_tree(new, dest).map(|()| {
                let _ = fs::remove_dir_all(new);
            })
        } else {
            fs::rename(new, dest)
        }
    });
    if let Err(e) = moved {
        if had_old {
            let _ = fs::remove_dir_all(dest);
            if fs::rename(&old, dest).is_err() {
                let _ = move_tree(&old, dest);
            }
        }
        return Err(e).with_context(|| format!("moving into {}", dest.display()));
    }
    if had_old {
        let _ = fs::remove_dir_all(&old);
    }
    Ok(())
}

/// Puts `tree` (a complete folder, in the PC's temporary folder or on the
/// kit's disk) in place of `dest`. On the same disk it is `replace_dir`. From
/// another disk, where a rename fails, the tree is first checked against the
/// destination's disk (`check_fits`, `check_space`), then copied next to
/// `dest` into `<dest>.atollo-new`, which `replace_dir` swaps in: an
/// interrupted copy leaves only that folder, which `clean_leftovers` removes,
/// and the installed version is never half replaced.
pub fn stage_into(tree: &Path, dest: &Path) -> Result<()> {
    let parent = dest.parent().context("no parent folder")?;
    fs::create_dir_all(parent)?;
    let same = crate::os::volume::same_volume(tree, parent);
    stage_with(tree, dest, same)
}

/// `stage_into` with the disk decision given (tests cannot make a second disk).
fn stage_with(tree: &Path, dest: &Path, same_volume: bool) -> Result<()> {
    if same_volume {
        return replace_dir(tree, dest);
    }
    check_fits(tree, dest)?;
    check_space(tree, dest)?;
    let staged = staged_path(dest);
    let copied = retry_denied(|| {
        // Each attempt starts from empty: a leftover of an interrupted copy,
        // or what the antivirus let through before it said no.
        if staged.exists() {
            fs::remove_dir_all(&staged)?;
        }
        copy_tree(tree, &staged)
    });
    if let Err(e) = copied {
        let _ = fs::remove_dir_all(&staged);
        return Err(e).with_context(|| format!("copying into {}", staged.display()));
    }
    let swapped = replace_dir(&staged, dest);
    if swapped.is_err() {
        let _ = fs::remove_dir_all(&staged);
    }
    swapped
}

/// `<dest>.atollo-new`, where a tree from another disk is copied first.
fn staged_path(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!("{name}{STAGED}"))
}

const STAGED: &str = ".atollo-new";

/// Runs `op` again while Windows answers "access denied", up to 30 s: the
/// antivirus scans the files just extracted and, while it holds them, their
/// folder cannot move (Java and Kotlin failed so on their first install).
fn retry_denied(mut op: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    let start = Instant::now();
    loop {
        match op() {
            Err(e)
                if e.kind() == io::ErrorKind::PermissionDenied
                    && start.elapsed() < Duration::from_secs(30) =>
            {
                std::thread::sleep(Duration::from_millis(250));
            }
            other => return other,
        }
    }
}

/// Moves the content of `from` into `to` (created if needed), entry by
/// entry: a folder that cannot be renamed is entered and emptied, and stays
/// where it is if Windows does not let it go. A file that cannot move is an
/// error (in use).
fn move_tree(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let target = to.join(entry.file_name());
        let is_dir = entry.file_type()?.is_dir();
        if is_dir && target.is_dir() {
            move_tree(&source, &target)?;
            let _ = fs::remove_dir(&source);
            continue;
        }
        if let Err(e) = fs::rename(&source, &target) {
            if !is_dir {
                return Err(e);
            }
            move_tree(&source, &target)?;
            let _ = fs::remove_dir(&source);
        }
    }
    Ok(())
}

/// Puts `new` in place of the file `dest`. The old file is renamed to
/// `<name>.old` first: Windows allows renaming a running exe, so a program in
/// use can still be updated (`Replace-File` of the kit). Leftover `.old`
/// files are deleted when possible; one still running (the program updated
/// twice while it runs) stays, and the old file goes to `<name>.old-<ms>`
/// instead, deleted by a later replace or `clean_leftovers`.
pub fn replace_file(new: &Path, dest: &Path) -> Result<()> {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let old = dest.with_file_name(format!("{name}.old"));
    let _ = fs::remove_file(&old);
    remove_old_files(dest, &format!("{name}.old-"));
    let old = if fs::symlink_metadata(&old).is_ok() {
        sibling(dest, "old")
    } else {
        old
    };
    // Copied next to `dest` first, not renamed: `new` may be on another disk
    // (PC temp folder), and an interrupted copy leaves only `<name>.atollo-new`.
    let staged = staged_path(dest);
    retry_denied(|| fs::copy(new, &staged).map(|_| ()))
        .with_context(|| format!("writing {}", staged.display()))?;
    let had_old = dest.exists();
    if had_old && fs::rename(dest, &old).is_err() {
        let _ = fs::remove_file(&staged);
        return Err(InUse(dest.to_path_buf()).into());
    }
    if let Err(e) = fs::rename(&staged, dest) {
        let _ = fs::remove_file(&staged);
        if had_old {
            let _ = fs::rename(&old, dest);
        }
        return Err(e).with_context(|| format!("writing {}", dest.display()));
    }
    let _ = fs::remove_file(&old);
    Ok(())
}

/// Deletes the files next to `dest` whose name is `prefix` followed by a
/// stamp (`<name>.old-<ms>`), when they are no longer in use.
fn remove_old_files(dest: &Path, prefix: &str) {
    let Some(dir) = dest.parent() else {
        return;
    };
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let stamped = name
            .strip_prefix(prefix)
            .is_some_and(|ms| !ms.is_empty() && ms.bytes().all(|b| b.is_ascii_digit()));
        if stamped && entry.file_type().is_ok_and(|t| t.is_file()) {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Removes a component folder: renamed first, so a folder in use is left
/// untouched (`InUse`).
pub fn remove_dir(dest: &Path) -> Result<()> {
    if !dest.exists() {
        return Ok(());
    }
    let old = sibling(dest, "old");
    fs::rename(dest, &old).map_err(|_| InUse(dest.to_path_buf()))?;
    let _ = fs::remove_dir_all(&old);
    Ok(())
}

/// Runs `make`, which builds `dest` where it stays (a conda environment
/// writes its own folder into its files, so it cannot be built elsewhere and
/// swapped in). An existing `dest` is set aside first (`<name>.old-<ms>`,
/// `InUse` if it cannot move), deleted when `make` succeeds and put back in
/// place of what `make` left when it fails: an update that breaks halfway
/// leaves the old one working.
pub fn rebuild_in_place<T>(dest: &Path, make: impl FnOnce() -> Result<T>) -> Result<T> {
    let old = if dest.exists() {
        let old = sibling(dest, "old");
        fs::rename(dest, &old).map_err(|_| InUse(dest.to_path_buf()))?;
        Some(old)
    } else {
        None
    };
    match make() {
        Ok(v) => {
            if let Some(old) = old {
                let _ = fs::remove_dir_all(old);
            }
            Ok(v)
        }
        Err(e) => {
            // What `make` left halfway goes, so it is not taken for installed.
            if dest.exists() {
                let _ = fs::remove_dir_all(dest);
            }
            if let Some(old) = old {
                if let Err(back) = fs::rename(&old, dest) {
                    crate::common::log::warn(&format!(
                        "{} not put back from {}: {back}",
                        dest.display(),
                        old.display()
                    ));
                }
            }
            Err(e)
        }
    }
}

fn sibling(dest: &Path, tag: &str) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!("{name}.{tag}-{}", stamp()))
}

/// Deletes leftovers of interrupted swaps (`*.old-*`, `*.new-*`) and copies
/// (`*.atollo-new`, `stage_into`) in `dir`.
pub fn clean_leftovers(dir: &Path) {
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.contains(".old-") || name.contains(".new-") || name.ends_with(STAGED) {
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                let _ = fs::remove_dir_all(entry.path());
            } else {
                let _ = fs::remove_file(entry.path());
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/net/archive/swap.rs"]
mod tests;
