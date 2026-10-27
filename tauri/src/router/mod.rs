//! Central multi-platform Router of Atollo (`WIN / LIN / MAC` ↔ `ROUTER` ↔
//! `ATOLLO 2`): target capabilities (`target`), OS-aware binary and launcher
//! resolution (`bins`), multi-target disk and health probing (`probe`). The
//! private and executable files of this PC are `platform::files`.

pub mod availability;
pub mod bins;
pub mod bootstrap;
pub mod probe;
pub mod target;

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::setup::checkup::Check;

/// Router bound to a [`Kit`] root and its active target.
#[derive(Debug, Clone)]
pub struct Router {
    kit: Kit,
}

impl Router {
    pub fn new(kit: Kit) -> Router {
        Router { kit }
    }

    /// Health check of installed tools on this router's target (live version
    /// commands when executable on host, static key/version check otherwise).
    pub fn probe_health(&self, env: &KitEnv) -> Vec<Check> {
        probe::probe_health(&self.kit, env)
    }
}

#[cfg(test)]
#[path = "../../../test/rust/router/mod.rs"]
mod tests;
