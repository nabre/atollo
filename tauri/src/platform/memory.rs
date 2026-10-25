//! The PC's memory: how much RAM it has, to tell whether a local model fits
//! (`llm`).

use crate::os::current::memory as imp;

/// The physical memory of the PC, in bytes.
pub fn total_ram() -> Option<u64> {
    imp::total_ram()
}

#[cfg(test)]
#[path = "../../../test/rust/platform/memory.rs"]
mod tests;
