//! Unix domain sockets (`platform::sockets`) on Windows: std cannot
//! connect to them; php-fpm runs on macOS and Linux only (php-cgi listens on
//! TCP), Caddy's admin socket is only written to.

use std::path::Path;

pub fn answers(path: &Path) -> bool {
    path.exists()
}

/// Cannot tell.
pub fn live_socket_in(_dir: &Path) -> Option<bool> {
    None
}
