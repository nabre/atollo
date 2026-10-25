//! The kit's links on macOS and Linux (`platform::links`): symbolic links,
//! for folders and files alike.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn create_dir_link(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

/// Removes the link only, never what it points to.
pub fn delete_dir_link(link: &Path) -> io::Result<()> {
    fs::remove_file(link)
}

pub fn dir_link_target(link: &Path) -> io::Result<PathBuf> {
    fs::read_link(link)
}

pub fn is_dir_link(path: &Path) -> bool {
    path.is_symlink()
}

pub fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

pub fn link_to_copy(path: &Path) -> io::Result<Option<PathBuf>> {
    fs::read_link(path).map(Some)
}

pub fn refused(e: &io::Error) -> bool {
    if let Some(code) = e.raw_os_error() {
        return code == libc::EPERM || code == libc::ENOTSUP || code == libc::EOPNOTSUPP;
    }
    e.kind() == io::ErrorKind::Unsupported
}

pub fn remove_symlink(path: &Path) -> Option<io::Result<()>> {
    fs::symlink_metadata(path)
        .is_ok_and(|m| m.file_type().is_symlink())
        .then(|| fs::remove_file(path))
}
