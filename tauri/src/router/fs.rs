//! OS-aware file reading, atomic writing, owner-only permissions (DACL on
//! Windows, `chmod` on Unix), and executable bit management.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::Foundation::{LocalFree, HLOCAL};
#[cfg(windows)]
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
#[cfg(windows)]
use windows::Win32::Security::{SetFileSecurityW, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};

use crate::fsx::write_atomic;
use crate::kit::Target;

/// Protected DACL with full access for the owner only on Windows
/// (`D:P(A;;FA;;;OW)`), or mode `0o600` on Unix.
/// Fails on filesystems without permissions (FAT32/exFAT): callers log a warning.
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    #[cfg(windows)]
    return set_dacl(path, "D:P(A;;FA;;;OW)");
    #[cfg(unix)]
    return set_mode(path, 0o600);
}

/// Owner-only directory permissions: inherited DACL `D:P(A;OICI;FA;;;OW)` on
/// Windows, mode `0o700` on Unix.
pub fn restrict_dir_to_owner(path: &Path) -> Result<()> {
    #[cfg(windows)]
    return set_dacl(path, "D:P(A;OICI;FA;;;OW)");
    #[cfg(unix)]
    return set_mode(path, 0o700);
}

/// Marks `path` executable (`0o755`) on Unix; no-op on Windows.
pub fn set_executable(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
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
            crate::log::warn(&format!("{}: permissions: {e:#}", parent.display()));
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
        crate::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
    Ok(())
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
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|e| access_error(path, e))?;
    #[cfg(windows)]
    if let Err(e) = restrict_to_owner(path) {
        crate::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
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
    #[cfg(windows)]
    let what = format!(
        "{shown}: access denied ({e}). It was made private by another Windows account, \
         perhaps on another PC (an NTFS disk keeps its permissions): delete it, or as \
         administrator run takeown /f \"{shown}\" and icacls \"{shown}\" /reset"
    );
    #[cfg(unix)]
    let what = format!(
        "{shown}: permission denied ({e}). It belongs to another user, perhaps on another \
         computer: delete it, or give it back with chown"
    );
    anyhow::anyhow!(what)
}

/// Ensures extracted Unix binaries (`bin/`, `sbin/`, and root executables)
/// have the executable bit set when running on a Unix host.
pub fn ensure_tree_executables(target: Target, tree: &Path) {
    if !target.is_unix() {
        return;
    }
    #[cfg(unix)]
    {
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
    #[cfg(not(unix))]
    let _ = tree;
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("chmod {mode:o} {}", path.display()))
}

#[cfg(windows)]
fn set_dacl(path: &Path, sddl: &str) -> Result<()> {
    let sddl = HSTRING::from(sddl);
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: valid SDDL string; `sd` receives a LocalAlloc'd descriptor freed below.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None)
    }
    .context("security descriptor")?;
    let file = HSTRING::from(path.as_os_str());
    // SAFETY: `sd` is a valid descriptor for the duration of the call.
    let result = unsafe { SetFileSecurityW(&file, DACL_SECURITY_INFORMATION, sd) };
    // SAFETY: frees the descriptor allocated above, once.
    unsafe { LocalFree(Some(HLOCAL(sd.0))) };
    result.ok().context("SetFileSecurityW")
}

#[cfg(test)]
#[path = "../../../test/rust/router/fs.rs"]
mod tests;
