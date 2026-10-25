//! Random bytes (`platform::random`) of `BCryptGenRandom`.

use anyhow::{Context, Result};
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

pub fn fill(bytes: &mut [u8]) -> Result<()> {
    // SAFETY: the buffer is valid for its length; system RNG, no algorithm handle.
    unsafe { BCryptGenRandom(None, bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .context("BCryptGenRandom")
}
