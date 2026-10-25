//! The web view of macOS (`platform::webview`): WKWebView, always there.

use std::path::Path;

/// The system's web view; there is no fixed runtime in a folder.
pub fn version_in(folder: Option<&Path>) -> Option<String> {
    folder.is_none().then(|| "WKWebView".to_owned())
}
