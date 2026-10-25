//! Files and folders on macOS and Linux: links.

use std::fs;

/// A symbolic link (Unix has no junctions), seen without following it.
pub fn is_reparse(meta: &fs::Metadata) -> bool {
    meta.file_type().is_symlink()
}
