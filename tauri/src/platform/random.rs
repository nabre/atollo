//! Random bytes of the system, for tokens: `BCryptGenRandom` on Windows,
//! `/dev/urandom` on macOS and Linux.

use anyhow::Result;

/// 32 random bytes of the system as 64 hex digits: the tokens of the
/// editor, the instance, the web tools.
pub fn token() -> Result<String> {
    let mut bytes = [0u8; 32];
    fill(&mut bytes)?;
    Ok(crate::common::hex(&bytes))
}

/// Fills `bytes` with random bytes of the system.
pub fn fill(bytes: &mut [u8]) -> Result<()> {
    crate::os::current::random::fill(bytes)
}
