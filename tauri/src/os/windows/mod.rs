//! Windows: the Win32 side of every module of `crate::platform`.

pub mod agy_store;
pub mod conpty;
pub mod console;
pub mod desktop;
pub mod files;
pub mod host;
pub mod instance;
pub mod jobs;
pub mod links;
pub mod local_time;
pub mod lock;
pub mod memory;
pub mod processes;
pub mod proxy;
pub mod pty;
pub mod random;
pub mod registry;
pub mod sessions;
pub mod signature;
pub mod sockets;
pub mod temp;
pub mod volume;
pub mod webview;

use crate::kit::Os;

/// The system the app is built for.
pub const OS: Os = Os::Win32;

/// What separates the folders of `PATH`.
pub const PATH_SEPARATOR: char = ';';

/// NTFS compares names without case.
pub const PATHS_IGNORE_CASE: bool = true;
