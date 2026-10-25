//! Files and folders on Windows (`platform::files`, `platform::safe_path`):
//! links and junctions, protected DACLs, the renames another program holds
//! up.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};
use windows::core::HSTRING;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{SetFileSecurityW, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};

/// Windows has no executable bit: a program is one by its extension.
pub const HAS_EXECUTABLE_BIT: bool = false;

/// A symbolic link or a junction (reparse point), seen without following it.
pub fn is_reparse(meta: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
    meta.file_type().is_symlink() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

/// Protected DACL with full access for the owner only.
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    set_dacl(path, "D:P(A;;FA;;;OW)")
}

/// Protected DACL with full access for the owner only, inherited.
pub fn restrict_dir_to_owner(path: &Path) -> Result<()> {
    set_dacl(path, "D:P(A;OICI;FA;;;OW)")
}

/// No executable bit on Windows.
pub fn set_executable(_path: &Path) -> io::Result<()> {
    Ok(())
}

/// A new file, owner-only before a byte is written.
pub fn create_private(path: &Path) -> io::Result<fs::File> {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    if let Err(e) = restrict_to_owner(path) {
        crate::common::log::warn(&format!("{}: permissions: {e:#}", path.display()));
    }
    Ok(file)
}

/// What to say of a file another account made private.
pub fn denied_text(shown: &str, e: &io::Error) -> String {
    format!(
        "{shown}: access denied ({e}). It was made private by another Windows account, \
         perhaps on another PC (an NTFS disk keeps its permissions): delete it, or as \
         administrator run takeown /f \"{shown}\" and icacls \"{shown}\" /reset"
    )
}

/// Access denied (5) or sharing violation (32): another program holds the
/// target of the rename.
pub fn rename_busy(e: &io::Error) -> bool {
    matches!(e.raw_os_error(), Some(5 | 32))
}

pub(crate) fn set_dacl(path: &Path, sddl: &str) -> Result<()> {
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

/// The temporary folders are per user.
pub fn owner_only(_meta: &fs::Metadata) -> bool {
    true
}

pub fn safe_parent(_meta: &fs::Metadata) -> bool {
    true
}

/// A plain folder: the parent, the user's own, gives its permissions.
pub fn create_dir_owner_only(path: &Path) -> io::Result<()> {
    fs::DirBuilder::new().create(path)
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/files.rs"]
mod tests;
