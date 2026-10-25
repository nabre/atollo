//! Facts about the Windows the app runs on.

use windows::core::HSTRING;
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ};

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
