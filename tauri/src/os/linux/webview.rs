//! The web view of Linux (`platform::webview`): WebKitGTK 4.1, when the
//! system has it (D-105).

use std::path::Path;

/// The system's web view, only when WebKitGTK loads; there is no fixed
/// runtime in a folder.
pub fn version_in(folder: Option<&Path>) -> Option<String> {
    (folder.is_none() && webkit_present()).then(|| "WebKitGTK".to_owned())
}

/// WebKitGTK can be loaded (`platform::webview::WEBKITGTK`). A program
/// linked with it does not even start without it, so this matters when the
/// library is opened at run time (and for the launcher of D-113, which
/// shows `missing_message`).
fn webkit_present() -> bool {
    // SAFETY: a NUL-terminated name; the handle stays open on purpose
    // (the library is the app's own, loaded anyway).
    !unsafe { libc::dlopen(c"libwebkit2gtk-4.1.so.0".as_ptr(), libc::RTLD_LAZY) }.is_null()
}

/// Nothing to change in the system's web view.
pub fn quiet<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>, _what: &'static str) {}
