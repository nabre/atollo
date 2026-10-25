//! The kit's lock around the moves (`platform::lock`): the named mutex
//! `Local\Atollo-agy-keyring-<kit id>` on Windows, an exclusive `flock` on
//! `data/antigravity/keyring.lock` elsewhere, waited for at most 30 s.

use std::time::Duration;

use anyhow::Result;

use crate::kit::Kit;
use crate::platform::lock::{self, KitLock};

/// Held while it lives; empty when another process kept the lock for
/// 30 s (the move goes on without it, as the kit's own scripts do).
pub(super) struct Lock(#[allow(dead_code)] Option<KitLock>);

impl Lock {
    pub(super) fn take(kit: &Kit) -> Result<Lock> {
        let name = format!("Atollo-agy-keyring-{}", kit.id());
        let file = kit.data().join("antigravity").join("keyring.lock");
        let held = lock::wait(&name, &file, Duration::from_secs(30))?;
        if held.is_none() {
            crate::common::log::warn("antigravity keyring: lock busy for 30 s, going on");
        }
        Ok(Lock(held))
    }
}
