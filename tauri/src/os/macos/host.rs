//! Facts about the Mac the app runs on (`platform::host`).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub use crate::os::unix::host::{arch, host_name, user_id, windows_build};

/// macOS answers only on 127.0.0.1 unless an alias is added.
pub const LOOPBACK_RANGE: bool = false;

/// The first of the languages chosen in System Settings, else the
/// environment's: an app opened from the Finder gets no `LANG` (D-105).
pub fn locale() -> Option<String> {
    apple_language().or_else(crate::os::unix::host::env_locale)
}

/// `defaults read -g AppleLanguages`, read once: the first call comes at
/// start, before the app points `HOME` into the kit.
fn apple_language() -> Option<String> {
    static LANG: OnceLock<Option<String>> = OnceLock::new();
    LANG.get_or_init(|| {
        let out = std::process::Command::new("/usr/bin/defaults")
            .args(["read", "-g", "AppleLanguages"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        first_apple_language(&String::from_utf8_lossy(&out.stdout))
    })
    .clone()
}

/// The first language of the list `defaults` prints
/// (`(\n    "it-IT",\n    en\n)`).
pub(crate) fn first_apple_language(output: &str) -> Option<String> {
    output
        .lines()
        .map(|l| l.trim().trim_end_matches(',').trim().trim_matches('"'))
        .find(|l| !l.is_empty() && *l != "(" && *l != ")")
        .map(str::to_owned)
}

/// No `/etc/os-release` on macOS.
pub fn os_release() -> Option<String> {
    None
}

/// No AppImage on macOS.
pub fn running_appimage(_exe: &Path) -> Option<PathBuf> {
    None
}

/// The folder of the active developer tools (`xcode-select -p`). `-p` only
/// prints: it never opens the installation
/// dialog. A folder found is kept until the app closes; "not installed" is
/// asked again after [`XCODE_RECHECK`], so tools installed meanwhile with
/// `xcode-select --install` are seen without a restart.
pub fn xcode_tools() -> Option<String> {
    use std::sync::Mutex;
    use std::time::Instant;
    static KEPT: Mutex<Option<(Instant, Option<String>)>> = Mutex::new(None);
    let mut kept = KEPT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some((at, answer)) = kept.as_ref() {
        if answer.is_some() || at.elapsed() < XCODE_RECHECK {
            return answer.clone();
        }
    }
    let answer = std::process::Command::new("/usr/bin/xcode-select")
        .arg("-p")
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|path| std::path::Path::new(path).is_dir());
    *kept = Some((Instant::now(), answer.clone()));
    answer
}

/// How long "no Command Line Tools" is believed before asking again.
const XCODE_RECHECK: std::time::Duration = std::time::Duration::from_secs(60);

/// The folders `xcode-select -p` prints when Xcode Command Line Tools, or
/// Xcode itself, are installed, each with the `clang` of its own.
pub(crate) const XCODE_DEVELOPER_DIRS: [&str; 2] = [
    "/Library/Developer/CommandLineTools",
    "/Applications/Xcode.app/Contents/Developer",
];

/// A `clang` in one of [`XCODE_DEVELOPER_DIRS`]; looked up once.
pub fn apple_clang_present() -> bool {
    static PRESENT: OnceLock<bool> = OnceLock::new();
    *PRESENT.get_or_init(|| {
        XCODE_DEVELOPER_DIRS
            .iter()
            .any(|d| crate::common::fsx::join_rel(Path::new(d), "usr/bin/clang").is_file())
    })
}

/// .NET finds ICU on a Mac.
pub fn libicu_present() -> bool {
    true
}

/// `kern.uuid` (the hardware's IOPlatformUUID, the same `ioreg` prints),
/// fixed for the life of the Mac.
pub fn machine_id() -> Option<String> {
    let mut buf = [0u8; 64];
    let mut len = buf.len();
    // SAFETY: a NUL-terminated name, the buffer and its length; nothing is
    // written (new value null).
    let ok = unsafe {
        libc::sysctlbyname(
            c"kern.uuid".as_ptr(),
            buf.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    } == 0;
    let end = buf[..len.min(buf.len())]
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(len.min(buf.len()));
    let id = std::str::from_utf8(&buf[..end]).ok()?.trim().to_owned();
    (ok && !id.is_empty()).then_some(id)
}

#[cfg(test)]
#[path = "../../../../test/rust/os/macos/host.rs"]
mod tests;
