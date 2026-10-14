//! WebView2 without installing it (DECISIONS.md D-05).
//!
//! Order: the runtime of the PC; else a "Fixed Version" runtime extracted in
//! `data\apps\win32-<arch>\webview2\` (used through
//! `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`); else the caller shows a message
//! with the download link. Microsoft's bootstrapper is never started.

use std::path::{Path, PathBuf};

use windows::core::{HSTRING, PCWSTR, PWSTR};
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
