//! The web view of macOS (`platform::webview`): WKWebView, always there.

use std::path::Path;

/// The system's web view; there is no fixed runtime in a folder.
pub fn version_in(folder: Option<&Path>) -> Option<String> {
    folder.is_none().then(|| "WKWebView".to_owned())
}

/// Nothing to change in the system's web view.
pub fn quiet<R: tauri::Runtime>(_window: &tauri::WebviewWindow<R>, _what: &'static str) {}
