//! Unix domain sockets the kit's servers listen on (Caddy's admin endpoint,
//! php-fpm; D-111): whether one still answers. The app talks to them on
//! macOS and Linux (`os/unix/sockets.rs`); on Windows std has no Unix
//! sockets, so the app cannot ask (`os/windows/sockets.rs`).

use std::path::Path;

use crate::os::current::sockets as imp;

/// A server takes a connection on the socket `path` now; on Windows,
/// whether the file is there.
pub fn answers(path: &Path) -> bool {
    imp::answers(path)
}

/// Whether a server still listens on one of the sockets in `dir`: `None`
/// where the app cannot tell (Windows).
pub fn live_socket_in(dir: &Path) -> Option<bool> {
    imp::live_socket_in(dir)
}
