//! The kit's links on Windows (`platform::links`): junctions for folders,
//! no symbolic links (they need a privilege).

use std::io;
use std::path::{Path, PathBuf};

pub fn create_dir_link(target: &Path, link: &Path) -> io::Result<()> {
    junction::create(target, link)
}

pub fn delete_dir_link(link: &Path) -> io::Result<()> {
    junction::delete(link)
}

pub fn dir_link_target(link: &Path) -> io::Result<PathBuf> {
    junction::get_target(link)
}

pub fn is_dir_link(path: &Path) -> bool {
    junction::exists(path).unwrap_or(false)
}

/// A symbolic link needs a privilege: the callers copy instead.
pub fn symlink(_target: &Path, _link: &Path) -> io::Result<()> {
    Err(io::Error::from(io::ErrorKind::Unsupported))
}

/// Links are skipped by a copy.
pub fn link_to_copy(_path: &Path) -> io::Result<Option<PathBuf>> {
    Ok(None)
}

pub fn refused(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::Unsupported
}

/// The kit's links are junctions, removed as folders.
pub fn remove_symlink(_path: &Path) -> Option<io::Result<()>> {
    None
}
