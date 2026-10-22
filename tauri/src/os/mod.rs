//! Windows and the app's processes: the Job Object every program of the kit
//! starts in (`jobs`, D-07), the processes of the PC by folder
//! (`processes`), the kit's open programs, other copies and "Rimuovi
//! l'unità" (`sessions`), one app per kit (`instance`, D-08), WebView2
//! (`webview`, D-05), reading inside a project without following links
//! (`safe_path`, D-77) and the build of Windows (here).
//!
//! macOS and Linux build the same modules with `#[cfg(unix)]` equivalents
//! (a process group for the job and a guard for a killed app, `flock` for
//! the locks, a socket for the instance, `/proc` or `libproc` for the
//! processes): the crate compiles there; Windows stays the platform the
//! kit is made for.

pub mod guard;
pub mod instance;
pub mod jobs;
pub mod memory;
pub mod processes;
pub mod safe_path;
pub mod sessions;
pub mod temp;
pub mod volume;
pub mod webview;

#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

/// `text` as one word for a POSIX shell, for the commands the user is told
/// to paste in a Terminal: always in single quotes, where nothing expands
/// (`$(...)`, backquotes), each `'` written as `'\''`. A volume label or a
/// bundle name is chosen by whoever made the disk.
pub fn sh_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}

/// Not Windows: no build to read.
#[cfg(not(windows))]
pub fn windows_build() -> Option<u32> {
    None
}

/// The build of Windows (`26200`); `None` when it cannot be read.
#[cfg(windows)]
pub fn windows_build() -> Option<u32> {
    let key = HSTRING::from(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion");
    let value = HSTRING::from("CurrentBuildNumber");
    let mut buf = [0u16; 32];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer and size describe the same memory; read-only query.
    let r = unsafe {
        RegGetValueW(
            HKEY_LOCAL_MACHINE,
            &key,
            &value,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    r.is_ok().then_some(())?;
    String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)])
        .parse()
        .ok()
}
