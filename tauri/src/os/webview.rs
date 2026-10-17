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

#[cfg(windows)]
use windows::core::{HSTRING, PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::System::Com::CoTaskMemFree;

use crate::kit::Kit;

pub const DOWNLOAD_URL: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";

pub enum Runtime {
    System(String),
    Fixed { folder: PathBuf, version: String },
}

pub fn fixed_folder(kit: &Kit) -> PathBuf {
    kit.apps().join("webview2")
}

/// The system's web view; there is no fixed runtime in a folder. On Linux
/// only when WebKitGTK loads (T1.8.2).
#[cfg(not(windows))]
fn version_in(folder: Option<&Path>) -> Option<String> {
    let name = if cfg!(target_os = "macos") {
        "WKWebView"
    } else {
        "WebKitGTK"
    };
    (folder.is_none() && webkit_present()).then(|| name.to_owned())
}

/// The WebKitGTK library Tauri 2 uses on Linux.
pub const WEBKITGTK: &str = "libwebkit2gtk-4.1.so.0";

/// WebKitGTK can be loaded. A program linked with it does not even start
/// without it, so this matters when the library is opened at run time
/// (and for the launcher of WP-5.3, which shows [`missing_message`]).
#[cfg(target_os = "linux")]
fn webkit_present() -> bool {
    // SAFETY: a NUL-terminated name; the handle stays open on purpose
    // (the library is the app's own, loaded anyway).
    !unsafe { libc::dlopen(c"libwebkit2gtk-4.1.so.0".as_ptr(), libc::RTLD_LAZY) }.is_null()
}

#[cfg(all(unix, not(target_os = "linux")))]
fn webkit_present() -> bool {
    true
}

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
#[allow(dead_code)] // called by main.rs at start (WP-1.8)
pub fn missing_message(kit: &Kit, os_release: Option<&str>, lang: &str) -> String {
    use crate::i18n::t;
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

#[cfg(windows)]
fn version_in(folder: Option<&Path>) -> Option<String> {
    let folder = folder.map(|f| HSTRING::from(f.as_os_str()));
    let folder_ptr = folder
        .as_ref()
        .map(|h| PCWSTR(h.as_ptr()))
        .unwrap_or(PCWSTR::null());
    let mut info = PWSTR::null();
    // SAFETY: `folder_ptr` is null or a valid string that outlives the call;
    // `info` receives a CoTaskMem string that is freed below.
    let result = unsafe {
        webview2_com::Microsoft::Web::WebView2::Win32::GetAvailableCoreWebView2BrowserVersionString(
            folder_ptr, &mut info,
        )
    };
    if result.is_err() || info.is_null() {
        return None;
    }
    // SAFETY: `info` is a valid null-terminated string from WebView2.
    let version = unsafe { info.to_string() }.ok();
    // SAFETY: allocated by WebView2 with CoTaskMemAlloc.
    unsafe { CoTaskMemFree(Some(info.0 as *const _)) };
    version.filter(|v| !v.is_empty())
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
#[path = "../../../test/rust/os/webview.rs"]
mod tests;
