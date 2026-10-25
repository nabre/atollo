//! Facts about the macOS or Linux the app runs on (`platform::host`), the
//! part the two share.

use crate::kit::Arch;

/// The processor the app is built for.
pub fn arch() -> Arch {
    if cfg!(target_arch = "aarch64") {
        Arch::Arm64
    } else {
        Arch::X64
    }
}

/// The locale of `LC_ALL`, `LC_MESSAGES` or `LANG` (`it_IT.UTF-8` gives
/// `it-IT`); `C` and `POSIX` are none.
pub fn env_locale() -> Option<String> {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .iter()
        .filter_map(|v| std::env::var(v).ok())
        .find(|v| !v.is_empty())
        .and_then(|v| posix_locale(&v))
}

/// `it_IT.UTF-8@euro` → `it-IT`; `C` and `POSIX` → None.
pub(crate) fn posix_locale(value: &str) -> Option<String> {
    let name = value.split(['.', '@']).next().unwrap_or_default();
    if name.is_empty() || name == "C" || name == "POSIX" {
        return None;
    }
    Some(name.replace('_', "-"))
}

/// Not Windows: no build to read.
pub fn windows_build() -> Option<u32> {
    None
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/host.rs"]
mod tests;
