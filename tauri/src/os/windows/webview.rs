//! WebView2 (`platform::webview`): the version of the runtime of the PC or
//! of a fixed one in a folder.

use std::path::Path;

use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::System::Com::CoTaskMemFree;

pub fn version_in(folder: Option<&Path>) -> Option<String> {
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
