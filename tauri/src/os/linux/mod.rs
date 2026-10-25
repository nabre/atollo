//! Linux: what only Linux has, and the modules of `os/unix/` it shares
//! with macOS, under the names of `crate::platform`.

pub mod child;
pub mod desktop;
pub mod host;
pub mod local_socket;
pub mod memory;
pub mod process_info;
pub mod processes;
pub mod proxy;
pub mod volume;
pub mod webview;

pub use super::unix::{
    console, files, instance, jobs, lock, pty, random, sessions, signature, sockets, temp,
};

use crate::kit::Os;

/// The system the app is built for.
pub const OS: Os = Os::Linux;

/// The file systems of Linux compare names exactly: `VSCODIUM-SERVER` and
/// `vscodium-server` are two folders (D-123).
pub const PATHS_IGNORE_CASE: bool = false;

/// What opens a folder or a web address with the system's handler.
pub const OPENER: &str = "xdg-open";
