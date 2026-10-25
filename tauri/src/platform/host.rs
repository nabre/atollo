//! What the app knows of the machine it runs on: its processor, the user's
//! language, the build of Windows, the Linux distribution, Rosetta 2 on a
//! Mac, the AppImage the app runs from, which loopback addresses answer.
//! Each system finds them its own way (`os/*/host.rs`).

use std::path::{Path, PathBuf};

use crate::kit::Arch;
use crate::os::current::host as imp;

/// The processor programs run on: on Windows from `PROCESSOR_ARCHITECTURE`
/// and its WOW64 host (ARM64 when either says so, as `_env.cmd`), on macOS
/// and Linux the one the app is built for.
pub fn arch() -> Arch {
    imp::arch()
}

/// The user's locale (`it-IT`): Windows's default locale name; on macOS the
/// first of the languages chosen in System Settings (an app opened from
/// the Finder gets no `LANG`, D-105), then, as on Linux, `LC_ALL`,
/// `LC_MESSAGES` or `LANG` (`C` and `POSIX` are none).
pub fn locale() -> Option<String> {
    imp::locale()
}

/// The build of Windows (`26200`); `None` on macOS and Linux, and when it
/// cannot be read.
pub fn windows_build() -> Option<u32> {
    imp::windows_build()
}

/// The text of `/etc/os-release` (or `/usr/lib/os-release`) on Linux;
/// `None` elsewhere.
pub fn os_release() -> Option<String> {
    imp::os_release()
}

/// Rosetta 2 runs x64 programs on this Mac (asked once per start:
/// installing it needs an administrator, and the app is started again
/// after it); false on every other system.
pub fn rosetta_present() -> bool {
    imp::rosetta_present()
}

/// The folder of the Mac's active developer tools (`xcode-select -p`):
/// Clang and the SDK the Apple frameworks need; asked again a minute after
/// "not installed", so tools installed meanwhile are seen. `None`
/// elsewhere.
pub fn xcode_tools() -> Option<String> {
    imp::xcode_tools()
}

/// The Xcode Command Line Tools (or Xcode) on this Mac, with their
/// `clang`; looked up once. False elsewhere.
pub fn apple_clang_present() -> bool {
    imp::apple_clang_present()
}

/// ICU's library (`libicuuc.so.<n>`, which .NET loads) on this Linux PC;
/// looked up once. True elsewhere.
pub fn libicu_present() -> bool {
    imp::libicu_present()
}

/// The AppImage the app runs from, when `exe` is inside the image's mount
/// (Linux); `None` elsewhere.
pub fn running_appimage(exe: &Path) -> Option<PathBuf> {
    imp::running_appimage(exe)
}

/// The user the app runs as (its effective id) on macOS and Linux; `None`
/// on Windows.
pub fn user_id() -> Option<u32> {
    imp::user_id()
}

/// Every address of 127.0.0.0/8 answers without an alias: true on Windows
/// and Linux; macOS answers only on 127.0.0.1.
pub const LOOPBACK_RANGE: bool = imp::LOOPBACK_RANGE;
