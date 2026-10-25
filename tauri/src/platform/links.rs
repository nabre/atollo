//! Links the kit makes: links to folders (junctions on Windows, which need
//! no privilege, symbolic links on macOS and Linux: the shared skills of
//! the agents, the .NET versions side by side) and symbolic links to files
//! of the archives of macOS and Linux (`python3 -> python3.13`), which
//! Windows cannot make without a privilege. Each system makes them its own
//! way (`os/*/links.rs`); a disk without links (FAT32, exFAT) refuses them
//! ([`refused`]) and the callers copy instead.

use std::io;
use std::path::{Path, PathBuf};

use crate::os::current::links as imp;

/// Makes `link`, a link to the folder `target`.
pub fn create_dir_link(target: &Path, link: &Path) -> io::Result<()> {
    imp::create_dir_link(target, link)
}

/// Removes the link `link` only, never what it points to (on Windows the
/// junction's folder itself may be left, empty).
pub fn delete_dir_link(link: &Path) -> io::Result<()> {
    imp::delete_dir_link(link)
}

/// Where the link to a folder `link` points, as it is written (a symbolic
/// link may hold a relative path).
pub fn dir_link_target(link: &Path) -> io::Result<PathBuf> {
    imp::dir_link_target(link)
}

/// `path` is a link to a folder of the kind [`create_dir_link`] makes.
pub fn is_dir_link(path: &Path) -> bool {
    imp::is_dir_link(path)
}

/// Makes `link`, a symbolic link to `target` (a file or a folder, as
/// written in an archive): `Unsupported` on Windows.
pub fn symlink(target: &Path, link: &Path) -> io::Result<()> {
    imp::symlink(target, link)
}

/// The target of the symbolic link `path` that a copy of its folder makes
/// again (`net::archive::copy_tree`); `None` on Windows, where the copy
/// skips links (the trees made here have none).
pub fn link_to_copy(path: &Path) -> io::Result<Option<PathBuf>> {
    imp::link_to_copy(path)
}

/// The file system refuses links: exFAT and FAT answer `EPERM` (Linux) or
/// `ENOTSUP`/`EOPNOTSUPP` (macOS), Windows `Unsupported`. Any other error
/// is a real one.
pub fn refused(e: &io::Error) -> bool {
    imp::refused(e)
}

/// A symbolic link at `path` removed as a link, never what it points to,
/// also when that went first (npm's `bin/pnpm` on macOS and Linux):
/// `None` when `path` is none of them, and always on Windows, where the
/// kit's links are junctions, removed as folders.
pub fn remove_symlink(path: &Path) -> Option<io::Result<()>> {
    imp::remove_symlink(path)
}
