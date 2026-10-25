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

/// The bytes the file takes on the disk: its blocks of 512 bytes, so a
/// sparse file counts what is written.
pub fn allocated(_path: &Path, meta: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    meta.blocks() * 512
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

/// The mode with `0o755` added.
pub fn add_executable(path: &Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(path, perms)
}

/// Every execute bit (`0o111`).
pub fn is_executable(path: &Path) -> io::Result<bool> {
    use std::os::unix::fs::PermissionsExt;
    Ok(fs::metadata(path)?.permissions().mode() & 0o111 == 0o111)
}

/// Only `/` separates.
pub fn special_in_name(_name: &str) -> bool {
    false
}

/// The same device and inode.
pub fn same_entry(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(x), Ok(y)) => x.dev() == y.dev() && x.ino() == y.ino(),
        _ => false,
    }
}

/// Some execute bit.
pub fn any_exec_bit(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    meta.permissions().mode() & 0o111 != 0
}

/// Only Windows programs carry a version resource.
pub fn program_version(exe: &Path) -> Result<String> {
    anyhow::bail!("{}: only on Windows", exe.display())
}

/// Windows Installer packages are Windows only.
pub fn extract_msi(msi: &Path, _dest: &Path) -> Result<()> {
    anyhow::bail!("{}: an MSI package opens on Windows only", msi.display())
}

/// 600 in a 700 folder.
pub fn write_secret(path: &Path, text: &str) -> Result<()> {
    crate::platform::files::write_private(path, text)
}

/// Sets 600 on a probe and reads it back.
pub fn mode_after_600(dir: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    fs::create_dir_all(dir).ok()?;
    let probe = dir.join(".mode-probe");
    fs::write(&probe, b"").ok()?;
    let _ = fs::set_permissions(&probe, fs::Permissions::from_mode(0o600));
    let mode = fs::metadata(&probe).ok().map(|m| m.permissions().mode());
    let _ = fs::remove_file(&probe);
    mode
}
