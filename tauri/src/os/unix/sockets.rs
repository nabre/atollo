//! Unix domain sockets (`platform::sockets`) on macOS and Linux.

use std::os::unix::fs::FileTypeExt;
use std::os::unix::net::UnixStream;
use std::path::Path;

pub fn answers(path: &Path) -> bool {
    UnixStream::connect(path).is_ok()
}

/// A socket in `dir` that a connect reaches.
pub fn live_socket_in(dir: &Path) -> Option<bool> {
    let alive = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .any(|f| f.file_type().is_ok_and(|t| t.is_socket()) && answers(&f.path()));
    Some(alive)
}
