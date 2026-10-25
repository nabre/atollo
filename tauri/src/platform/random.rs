//! Random bytes of the system, for tokens: `BCryptGenRandom` on Windows,
//! `/dev/urandom` on macOS and Linux.

use anyhow::Result;

/// Fills `bytes` with random bytes of the system.
pub fn fill(bytes: &mut [u8]) -> Result<()> {
    crate::os::current::random::fill(bytes)
}
