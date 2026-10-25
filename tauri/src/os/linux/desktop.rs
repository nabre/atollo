//! The desktop around the app's windows on Linux (`platform::desktop`):
//! the dialogs of `os/unix/desktop.rs`, and the StatusNotifier tray.

pub use crate::os::unix::desktop::{error_box, owner, pick_files, pick_folder, Owner};

/// The icon of Linux needs libayatana-appindicator (or libappindicator),
/// loaded at run time: without it the icon would end the app.
pub fn tray_available() -> bool {
    [
        c"libayatana-appindicator3.so.1",
        c"libappindicator3.so.1",
        c"libayatana-appindicator3.so",
        c"libappindicator3.so",
    ]
    .iter()
    .any(|name| {
        // SAFETY: a NUL-terminated name; the handle is closed at once (the
        // tray opens the library again).
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY) };
        if handle.is_null() {
            return false;
        }
        // SAFETY: a handle dlopen just returned.
        unsafe { libc::dlclose(handle) };
        true
    })
}
