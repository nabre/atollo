//! WebView2 without installing it (DECISIONS.md D-05).
//!
//! Order: the runtime of the PC; else a "Fixed Version" runtime extracted in
//! `data\apps\win32-<arch>\webview2\` (used through
//! `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`); else the caller shows a message
//! with the download link. Microsoft's bootstrapper is never started.
//!
//! macOS (WKWebView) and Linux (WebKitGTK) have the web view of the system:
//! always `Runtime::System`, never a fixed one.

use std::path::{Path, PathBuf};

use crate::kit::Kit;
use crate::os::current::webview as imp;

pub const DOWNLOAD_URL: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";

pub enum Runtime {
    System(String),
    Fixed { folder: PathBuf, version: String },
}

pub fn fixed_folder(kit: &Kit) -> PathBuf {
    kit.apps().join("webview2")
}

/// The WebKitGTK library Tauri 2 uses on Linux.
pub const WEBKITGTK: &str = "libwebkit2gtk-4.1.so.0";

/// The command that installs WebKitGTK 4.1 on the Linux described by
/// `os_release` (`/etc/os-release`: `ID`, `ID_LIKE`), if it is known.
pub(crate) fn webkit_install_command(os_release: &str) -> Option<&'static str> {
    let ids: Vec<String> = os_release
        .lines()
        .filter_map(|l| l.strip_prefix("ID=").or_else(|| l.strip_prefix("ID_LIKE=")))
        .flat_map(|v| {
            v.trim_matches('"')
                .split(' ')
                .map(str::to_ascii_lowercase)
                .collect::<Vec<_>>()
        })
        .collect();
    let has = |id: &str| ids.iter().any(|i| i == id);
    if has("debian") || has("ubuntu") {
        Some("sudo apt install libwebkit2gtk-4.1-0")
    } else if has("fedora") || has("rhel") {
        Some("sudo dnf install webkit2gtk4.1")
    } else if has("arch") {
        Some("sudo pacman -S webkit2gtk-4.1")
    } else if has("suse") || has("opensuse") {
        Some("sudo zypper install libwebkit2gtk-4_1-0")
    } else {
        None
    }
}

/// The message when the web view is missing: on Windows the one of
/// WebView2 (unchanged), on Linux what to install. `os_release` is the
/// text of `/etc/os-release` (Linux).
pub fn missing_message(kit: &Kit, os_release: Option<&str>, lang: &str) -> String {
    use crate::common::i18n::t;
    if kit.os().is_linux() {
        let install = match os_release.and_then(webkit_install_command) {
            Some(command) => t(lang, "Installalo con {0}, poi riavvia Atollo. In alternativa usa l'AppImage di Atollo, che lo contiene.", &[command]),
            None => t(lang, "Installa il pacchetto WebKitGTK 4.1 della tua distribuzione ({0}), poi riavvia Atollo. In alternativa usa l'AppImage di Atollo, che lo contiene.", &[WEBKITGTK]),
        };
        return format!(
            "{}\n\n{install}",
            t(
                lang,
                "Manca WebKitGTK 4.1, che serve per mostrare le finestre di Atollo.",
                &[]
            ),
        );
    }
    format!(
        "{}\n\n{}",
        t(lang, "Manca Microsoft Edge WebView2, che serve per mostrare le finestre di Atollo.", &[]),
        t(
            lang,
            "Scarica la versione \"Fixed Version\" per {0} da {1} ed estraila in {2}, poi riavvia Atollo. Non serve installare nulla.",
            &[kit.arch().as_str(), DOWNLOAD_URL, &fixed_folder(kit).display().to_string()]
        )
    )
}

/// The version of the web view: the system's (`folder` `None`), or the
/// fixed WebView2 runtime in `folder` (Windows only); `None` when there is
/// none. On Linux only when WebKitGTK loads (D-105).
fn version_in(folder: Option<&Path>) -> Option<String> {
    imp::version_in(folder)
}

/// The runtime in use, without changing anything (for the relations
/// matrix): the PC's, or the kit's fixed one.
pub fn found(kit: &Kit) -> Option<Runtime> {
    if let Some(version) = version_in(None) {
        return Some(Runtime::System(version));
    }
    let folder = fixed_folder(kit);
    if !folder.join("msedgewebview2.exe").is_file() {
        return None;
    }
    let version = version_in(Some(&folder))?;
    Some(Runtime::Fixed { folder, version })
}

/// Finds a usable runtime; sets `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` for the
/// fixed one. Must run before any thread starts (it sets a variable).
pub fn locate(kit: &Kit) -> Option<Runtime> {
    if let Some(version) = version_in(None) {
        return Some(Runtime::System(version));
    }
    let folder = fixed_folder(kit);
    if folder.join("msedgewebview2.exe").is_file() {
        if let Some(version) = version_in(Some(&folder)) {
            std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", &folder);
            return Some(Runtime::Fixed { folder, version });
        }
    }
    None
}

#[cfg(test)]
#[path = "../../../test/rust/platform/webview.rs"]
mod tests;
