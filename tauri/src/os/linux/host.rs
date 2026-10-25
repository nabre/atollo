//! Facts about the Linux the app runs on (`platform::host`).

use std::path::{Path, PathBuf};

pub use crate::os::unix::host::{arch, env_locale as locale, user_id, windows_build};

/// Every 127.x.x.x answers.
pub const LOOPBACK_RANGE: bool = true;

/// The text of `/etc/os-release` (or `/usr/lib/os-release`).
pub fn os_release() -> Option<String> {
    ["/etc/os-release", "/usr/lib/os-release"]
        .iter()
        .find_map(|p| std::fs::read_to_string(p).ok())
}

/// No Rosetta on Linux.
/// No Xcode here.
pub fn xcode_tools() -> Option<String> {
    None
}

pub fn rosetta_present() -> bool {
    false
}

/// `$APPIMAGE`, when `exe` is inside the image's mount (`$APPDIR`).
pub fn running_appimage(exe: &Path) -> Option<PathBuf> {
    appimage_of(
        exe,
        std::env::var_os("APPIMAGE").as_deref(),
        std::env::var_os("APPDIR").as_deref(),
    )
}

/// The AppImage the app runs from: `image`, when `exe` is inside the
/// image's mount (`appdir`). A plain binary started from a program that
/// itself runs from an AppImage inherits both variables, but is not inside
/// that mount.
pub(crate) fn appimage_of(
    exe: &Path,
    image: Option<&std::ffi::OsStr>,
    appdir: Option<&std::ffi::OsStr>,
) -> Option<PathBuf> {
    let (image, appdir) = (image?, appdir?);
    (!image.is_empty() && !appdir.is_empty() && exe.starts_with(appdir))
        .then(|| PathBuf::from(image))
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/host.rs"]
mod tests;
