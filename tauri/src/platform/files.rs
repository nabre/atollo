//! Files with permissions: owner-only files and folders (a protected DACL
//! on Windows, a mode on macOS and Linux), the executable bit, atomic
//! writes of private files, and what to say when another account's file
//! is in the way. Each system sets the permissions its way
//! (`os/*/files.rs`).

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};

use crate::common::fsx::write_atomic;
use crate::kit::Target;
use crate::os::current::files as imp;

/// Protected DACL with full access for the owner only on Windows
/// (`D:P(A;;FA;;;OW)`), or mode `0o600` on Unix.
/// Fails on filesystems without permissions (FAT32/exFAT): callers log a warning.
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    imp::restrict_to_owner(path)
}

/// Owner-only directory permissions: inherited DACL `D:P(A;OICI;FA;;;OW)` on
/// Windows, mode `0o700` on Unix.
pub fn restrict_dir_to_owner(path: &Path) -> Result<()> {
    imp::restrict_dir_to_owner(path)
}

/// Marks `path` executable (`0o755`) on Unix; no-op on Windows.
pub fn set_executable(path: &Path) -> io::Result<()> {
    imp::set_executable(path)
}

/// Adds the execute bits (`0o755`) to the mode of `path` on Unix, keeping
/// the others; no-op on Windows.
pub fn add_executable(path: &Path) -> io::Result<()> {
    imp::add_executable(path)
}

/// `path` has every execute bit on Unix (always on Windows).
pub fn is_executable(path: &Path) -> io::Result<bool> {
    imp::is_executable(path)
}

/// Writes `bytes` atomically to `path` (creating parent folders) and marks the
/// file executable on Unix (`0o755`).
pub fn write_executable(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_atomic(path, bytes)?;
    set_executable(path)
}

/// Writes `text` atomically in a folder made owner-only first, then sets the
/// file's own owner-only permissions.
pub fn write_private(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        if let Err(e) = restrict_dir_to_owner(parent) {
            crate::common::log::warn(&format!("{}: permissions: {e:#}", parent.display()));
        }
    }
    write_in_private(path, text)
}

/// Writes `text` atomically in an already-private folder and sets the file's
/// own owner-only permissions. A file this account may not replace (made
/// private by another account, or on another PC: an NTFS disk keeps its
/// DACL) is deleted and written again; failing that, the error names the
/// file and what to do (PORT-V6-01).
pub fn write_in_private(path: &Path, text: &str) -> Result<()> {
    if let Err(e) = write_atomic(path, text.as_bytes()) {
        if e.kind() != io::ErrorKind::PermissionDenied {
            return Err(e).with_context(|| path.display().to_string());
        }
        let _ = fs::remove_file(path);
        write_atomic(path, text.as_bytes()).map_err(|e| access_error(path, e))?;
    }
    if let Err(e) = restrict_to_owner(path) {
        crate::common::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
    Ok(())
}

/// Writes a file that holds a secret in clear (`text`), atomically: on
/// macOS and Linux owner-only (600, in a 700 folder: [`write_private`]); on
/// Windows a plain file, in the user's own folder of the kit.
pub fn write_secret(path: &Path, text: &str) -> Result<()> {
    imp::write_secret(path, text)
}

/// The permissions a file in `dir` really has after being set to 600: the
/// mode where the disk keeps one (not exFAT, FAT), `None` there and on
/// Windows.
pub fn mode_after_600(dir: &Path) -> Option<u32> {
    imp::mode_after_600(dir)
}

/// Replaces `path` with a new owner-only file holding `text`: the old one is
/// deleted first (never rewritten: one made private by another account
/// keeps that account's DACL, PORT-V6-01), the new one is created owner-only
/// before a byte is written (BUG-V6-33). For secrets that live one session.
pub fn replace_private(path: &Path, text: &str) -> Result<()> {
    use std::io::Write;
    match fs::remove_file(path) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(access_error(path, e)),
        _ => {}
    }
    let mut file = imp::create_private(path).map_err(|e| access_error(path, e))?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| access_error(path, e))
}

/// `e` on `path`, named; "access denied" also says why it happens to a kit
/// carried to another PC or account and what to do (PORT-V6-01).
pub fn access_error(path: &Path, e: io::Error) -> anyhow::Error {
    let shown = path.display();
    if e.kind() != io::ErrorKind::PermissionDenied {
        return anyhow::Error::new(e).context(shown.to_string());
    }
    anyhow::anyhow!(imp::denied_text(&shown.to_string(), &e))
}

/// Ensures extracted Unix binaries (`bin/`, `sbin/`) have the executable
/// bit set when running on a Unix host (on Windows nothing is set).
pub fn ensure_tree_executables(target: Target, tree: &Path) {
    if !target.is_unix() || !imp::HAS_EXECUTABLE_BIT {
        return;
    }
    for sub in ["bin", "sbin"] {
        let dir = tree.join(sub);
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.file_type().is_ok_and(|t| t.is_file()) {
                    let _ = set_executable(&entry.path());
                }
            }
        }
    }
}

/// A folder only its owner can enter, as the metadata (of the folder
/// itself, not followed) says: on macOS and Linux owned by this user with
/// no right for group and others (mode `0700` or tighter); on Windows,
/// whose temporary folders are per user and get an owner-only DACL, always.
pub fn owner_only(meta: &fs::Metadata) -> bool {
    imp::owner_only(meta)
}

/// A folder other users cannot replace what this user makes in it: on
/// macOS and Linux one only this user can write in, or a sticky one
/// (`/tmp`: nobody renames another's entries) of root or this user; on
/// Windows always.
pub fn safe_parent(meta: &fs::Metadata) -> bool {
    imp::safe_parent(meta)
}

/// Makes the folder `path` (its parent must exist, and `path` must not):
/// mode `0700` from the start on macOS and Linux; on Windows it gets the
/// permissions of its parent, the user's own temporary folder.
pub fn create_dir_owner_only(path: &Path) -> io::Result<()> {
    imp::create_dir_owner_only(path)
}

/// `a` and `b` are one entry on disk: a folder renamed only in case on a
/// file system that ignores case (macOS, exFAT) already "exists" under its
/// new name. Always false on Windows.
pub fn same_entry(a: &Path, b: &Path) -> bool {
    imp::same_entry(a, b)
}

/// The file has an execute bit (macOS, Linux); false on Windows.
pub fn any_exec_bit(meta: &fs::Metadata) -> bool {
    imp::any_exec_bit(meta)
}

/// The file version in the resources of the Windows program `exe`
/// (`a.b.c.d`); an error elsewhere.
pub fn program_version(exe: &Path) -> Result<String> {
    imp::program_version(exe)
}

/// The bytes the files under `path` take on the disk, links not followed:
/// a sparse file (a VM's disk, 20 GB of which a few are written) counts
/// what is written, not its size.
pub fn allocated_size(path: &Path) -> u64 {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return 0;
    };
    if meta.is_file() {
        return imp::allocated(path, &meta);
    }
    if !meta.is_dir() {
        return 0;
    }
    fs::read_dir(path)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| allocated_size(&e.path()))
        .sum()
}

/// The files of the Windows Installer package `msi`, extracted into `dest`
/// without installing it (Docker Sandboxes, D-167); an error elsewhere.
pub fn extract_msi(msi: &Path, dest: &Path) -> Result<()> {
    imp::extract_msi(msi, dest)
}

/// A name of an archive's entry that means more than a file name on this
/// system: on Windows one with `:` (a drive, `C:x`, an alternate data
/// stream).
pub fn special_in_name(name: &str) -> bool {
    imp::special_in_name(name)
}

/// A rename refused for a moment because another program holds the target
/// without sharing its deletion (Windows: access denied or sharing
/// violation): worth trying again. Never on macOS and Linux, where a rename
/// does not fail this way.
pub fn rename_busy(e: &io::Error) -> bool {
    imp::rename_busy(e)
}

#[cfg(test)]
#[path = "../../../test/rust/platform/files.rs"]
mod tests;
