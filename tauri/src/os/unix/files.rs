//! Files and folders on macOS and Linux (`platform::files`,
//! `platform::safe_path`): links, modes and owners.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{Context, Result};

/// A program needs its executable bit.
pub const HAS_EXECUTABLE_BIT: bool = true;

/// A symbolic link (Unix has no junctions), seen without following it.
pub fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}

/// Mode `0o600`.
pub fn restrict_to_owner(path: &Path) -> Result<()> {
    set_mode(path, 0o600)
}

/// Mode `0o700`.
pub fn restrict_dir_to_owner(path: &Path) -> Result<()> {
    set_mode(path, 0o700)
}

/// Mode `0o755`.
pub fn set_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

/// A new file created with mode `0o600`.
pub fn create_private(path: &Path) -> io::Result<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

/// What to say of a file of another user.
pub fn denied_text(shown: &str, e: &io::Error) -> String {
    format!(
        "{shown}: permission denied ({e}). It belongs to another user, perhaps on another \
         computer: delete it, or give it back with chown"
    )
}

/// A rename never fails because another program holds the target.
pub fn rename_busy(_e: &io::Error) -> bool {
    false
}

fn set_mode(path: &Path, mode: u32) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .with_context(|| format!("chmod {mode:o} {}", path.display()))
}

/// Owned by this user, mode `0700` or tighter.
pub fn owner_only(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    meta.uid() == super::users::effective() && meta.mode() & 0o077 == 0
}

/// Writable only by this user, or sticky and of root or this user.
pub fn safe_parent(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    let me = super::users::effective();
    let sticky = meta.mode() & 0o1000 != 0;
    let owner_ok = meta.uid() == me || meta.uid() == 0;
    owner_ok && (sticky || (meta.uid() == me && meta.mode() & 0o022 == 0))
}

/// Mode `0700` from the start.
pub fn create_dir_owner_only(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(path)
}
