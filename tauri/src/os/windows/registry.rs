//! The current user's registry (`platform::registry`).

use anyhow::Result;
use windows::core::HSTRING;
use windows::Win32::System::Registry::{
    RegDeleteTreeW, RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ,
};

pub fn user_value(key: &str, name: &str) -> Option<String> {
    let key = HSTRING::from(key);
    let value = HSTRING::from(name);
    let mut buf = vec![0u16; 2048];
    let mut size = (buf.len() * 2) as u32;
    // SAFETY: buffer and size describe the same memory; read-only query.
    let r = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            &key,
            &value,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut size),
        )
    };
    r.is_ok()
        .then(|| String::from_utf16_lossy(&buf[..(size as usize / 2).saturating_sub(1)]))
}

pub fn delete_user_key(key: &str) -> Result<()> {
    // SAFETY: deletes a subkey of the current user by its name.
    unsafe { RegDeleteTreeW(HKEY_CURRENT_USER, &HSTRING::from(key)) }.ok()?;
    Ok(())
}

pub fn set_user_value(key: &str, name: &str, value: &str) -> Result<()> {
    let wide: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    // SAFETY: REG_SZ data with its terminating NUL, size in bytes.
    unsafe {
        RegSetKeyValueW(
            HKEY_CURRENT_USER,
            &HSTRING::from(key),
            &HSTRING::from(name),
            REG_SZ.0,
            Some(wide.as_ptr().cast()),
            (wide.len() * 2) as u32,
        )
    }
    .ok()?;
    Ok(())
}
