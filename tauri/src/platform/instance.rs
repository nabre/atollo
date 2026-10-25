//! One running app per kit (DECISIONS.md D-08).
//!
//! The first start of a kit holds a lock for as long as it runs; a second
//! start of the same kit sees it, sends a message to the first (for now
//! only "focus") and exits. Two different kits (two USB sticks) have
//! different ids and run side by side. Each system does it its own way:
//!
//! - Windows (`os/windows/instance.rs`): a named mutex
//!   `Local\Atollo-App-<kit id>` marks the running app, the message goes
//!   through the pipe of the same name, which rejects remote clients;
//! - macOS and Linux (`os/unix/instance.rs`): an exclusive `flock` on
//!   `data/app/instance.lock`, the message through a socket never inside
//!   the kit (an abstract one on Linux, one in the user's temporary folder
//!   on macOS), or a TCP port on 127.0.0.1 with a token (D-105).

use anyhow::Result;

use crate::kit::Kit;
use crate::os::current::instance as imp;

/// While it lives, this process is the app of the kit.
pub use imp::MutexGuard;

pub enum Instance {
    /// This process is the app of the kit; keep the guard alive.
    Primary(MutexGuard),
    /// Another process already is.
    Secondary,
}

/// Whether this process is the app of `kit` or another one already is.
pub fn acquire(kit: &Kit) -> Result<Instance> {
    imp::acquire(kit)
}

/// Sends `message` to the running app of the kit (retries while it starts).
pub fn notify_primary(kit: &Kit, message: &str) -> Result<()> {
    imp::notify_primary(kit, message)
}

/// Serves the messages of later starts on a background thread;
/// `on_message` gets each one. Called by the primary only.
pub fn listen(kit: &Kit, on_message: impl Fn(String) + Send + 'static) {
    imp::listen(kit, on_message)
}

#[cfg(test)]
#[path = "../../../test/rust/platform/instance.rs"]
mod tests;
