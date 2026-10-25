//! Windows: the Win32 side of every module of `crate::platform`.

pub mod files;
pub mod instance;
pub mod jobs;
pub mod lock;
pub mod memory;
pub mod processes;
pub mod sessions;
pub mod system;
pub mod temp;
pub mod volume;
pub mod webview;

use crate::kit::Os;

/// The system the app is built for.
pub const OS: Os = Os::Win32;

/// NTFS compares names without case.
pub const PATHS_IGNORE_CASE: bool = true;
