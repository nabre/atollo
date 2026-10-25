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

/// The effective user id.
pub fn user_id() -> Option<u32> {
    Some(super::users::effective())
}

/// Not Windows: no build to read.
pub fn windows_build() -> Option<u32> {
    None
}

/// `gethostname`.
pub fn host_name() -> String {
    let mut buf = [0u8; 256];
    // SAFETY: the buffer and its length; the name is NUL-terminated within.
    let ok = unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } == 0;
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    if ok {
        String::from_utf8_lossy(&buf[..end]).into_owned()
    } else {
        String::new()
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/host.rs"]
mod tests;
