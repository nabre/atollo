//! Central multi-platform Router of Atollo (`WIN / LIN / MAC` ↔ `ROUTER` ↔
//! `ATOLLO 2`): target capabilities (`target`), OS-aware binary and launcher
//! resolution (`bins`), multi-target disk and health probing (`probe`), and
//! OS-aware private/executable file operations (`fs`).

pub mod availability;
pub mod bins;
pub mod bootstrap;
pub mod fs;
pub mod probe;
pub mod target;

// The private and executable files, by a short path
// (`crate::router::write_private`).
pub use fs::{
    restrict_dir_to_owner, restrict_to_owner, set_executable, write_executable, write_private,
};

use crate::env::KitEnv;
use crate::kit::{Kit, Target};
use crate::setup::checkup::Check;

/// Multi-target router bound to a [`Kit`] root and active [`Target`].
#[derive(Debug, Clone)]
pub struct Router {
    kit: Kit,
}

#[allow(dead_code)]
impl Router {
    pub fn new(kit: Kit) -> Router {
        Router { kit }
    }

    pub fn kit(&self) -> &Kit {
        &self.kit
    }

    pub fn target(&self) -> Target {
        self.kit.target()
    }

    /// Returns a [`Router`] scoped to `target` on the same kit root.
    pub fn for_target(&self, target: Target) -> Router {
        Router {
            kit: self.kit.for_target(target),
        }
    }

    /// True when binaries of this router's target can run on the current host.
    pub fn can_execute_on_host(&self) -> bool {
        target::can_execute_on_host(self.kit.target())
    }

    /// Static disk probe of this router's target, reasons in `lang`.
    pub fn probe_target(&self, lang: &str) -> probe::TargetSummary {
        probe::probe_target(&self.kit, self.kit.target(), lang)
    }

    /// Static disk probe of all 6 supported targets (`Target::ALL`).
    pub fn probe_all_targets(&self, lang: &str) -> Vec<probe::TargetSummary> {
        probe::probe_all_targets(&self.kit, lang)
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
