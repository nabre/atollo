//! Facts about the Mac the app runs on (`platform::host`).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

pub use crate::os::unix::host::{arch, windows_build};

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

/// Rosetta 2 runs x64 programs: `/usr/bin/arch -x86_64 /usr/bin/true`
/// exits with 0. Asked once per start.
pub fn rosetta_present() -> bool {
    static PRESENT: OnceLock<bool> = OnceLock::new();
    *PRESENT.get_or_init(|| {
        std::process::Command::new("/usr/bin/arch")
            .args(["-x86_64", "/usr/bin/true"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    })
}

/// No AppImage on macOS.
pub fn running_appimage(_exe: &Path) -> Option<PathBuf> {
    None
}

#[cfg(test)]
#[path = "../../../../test/rust/os/macos/host.rs"]
mod tests;
