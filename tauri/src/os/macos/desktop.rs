//! The desktop around the app's windows on macOS (`platform::desktop`):
//! the dialogs of `os/unix/desktop.rs`, and the menu bar.

pub use crate::os::unix::desktop::{error_box, owner, pick_files, pick_folder, Owner};

/// The menu bar is always there.
pub fn tray_available() -> bool {
    true
}
