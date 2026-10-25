//! macOS: what only macOS has, and the modules of `os/unix/` it shares
//! with Linux, under the names of `crate::platform`.

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
    console, files, instance, jobs, links, local_time, lock, pty, random, registry, sessions,
    signature, sockets, temp,
};

use crate::kit::Os;

/// What separates the folders of `PATH`.
pub const PATH_SEPARATOR: char = ':';

/// The system the app is built for.
pub const OS: Os = Os::Darwin;

/// APFS (by default) compares names without case.
pub const PATHS_IGNORE_CASE: bool = true;

/// What opens a folder or a web address with the system's handler.
pub const OPENER: &str = "open";
