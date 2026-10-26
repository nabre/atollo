//! Facts about the Windows the app runs on (`platform::host`).

use std::path::{Path, PathBuf};

use windows::core::HSTRING;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

use crate::kit::Arch;

/// Every 127.x.x.x answers.
pub const LOOPBACK_RANGE: bool = true;

/// The PC's processor, also when this exe runs translated: same rule as
/// `_env.cmd`, ARM64 when `PROCESSOR_ARCHITECTURE` or
/// `PROCESSOR_ARCHITEW6432` says so, otherwise x64.
pub fn arch() -> Arch {
    arch_from_env(
        std::env::var("PROCESSOR_ARCHITECTURE").ok().as_deref(),
        std::env::var("PROCESSOR_ARCHITEW6432").ok().as_deref(),
    )
}

pub(crate) fn arch_from_env(arch: Option<&str>, native: Option<&str>) -> Arch {
    let is_arm = |v: Option<&str>| v.is_some_and(|v| v.eq_ignore_ascii_case("ARM64"));
    if is_arm(arch) || is_arm(native) {
        Arch::Arm64
    } else {
        Arch::X64
    }
}

/// The user's default locale name from Windows, e.g. `it-IT`.
pub fn locale() -> Option<String> {
    use windows::Win32::Globalization::GetUserDefaultLocaleName;
    let mut buf = [0u16; 85];
    // SAFETY: the buffer is LOCALE_NAME_MAX_LENGTH (85) wide chars, as required.
    let len = unsafe { GetUserDefaultLocaleName(&mut buf) };
    (len > 1).then(|| String::from_utf16_lossy(&buf[..(len as usize - 1)]))
}

/// The build of Windows (`26200`); `None` when it cannot be read.
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

pub fn user_id() -> Option<u32> {
    None
}

pub fn os_release() -> Option<String> {
    None
}

/// No Xcode here.
pub fn xcode_tools() -> Option<String> {
    None
}

pub fn running_appimage(_exe: &Path) -> Option<PathBuf> {
    None
}

pub fn apple_clang_present() -> bool {
    false
}

/// .NET carries ICU on Windows.
pub fn libicu_present() -> bool {
    true
}

/// Not read on Windows.
pub fn host_name() -> String {
    String::new()
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/host.rs"]
mod tests;
