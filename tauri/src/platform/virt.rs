//! What the PC lacks to run Docker Sandboxes' microVMs (`isolation`,
//! D-167): the system and the processor Docker supports, and the
//! hypervisor the sandboxes need (Windows Hypervisor Platform, KVM; the
//! Mac's Virtualization framework is always there). Each system finds them
//! its own way (`os/*/virt.rs`), without an administrator.

use std::path::{Path, PathBuf};

use crate::os::current::virt as imp;

/// Something the PC lacks, in the order the user fixes them.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // each system makes its own
pub enum Gap {
    /// Windows before 11 (its build).
    OldWindows(u32),
    /// macOS before 14 (its version).
    OldMacos(String),
    /// A Linux other than Ubuntu 24.04 or later (its name).
    Distribution(String),
    /// Windows Hypervisor Platform is off.
    HypervisorOff,
    /// No `/dev/kvm`: virtualization off in the firmware, or no KVM module.
    KvmMissing,
    /// `/dev/kvm` is there but the user is not in the `kvm` group.
    KvmDenied,
    /// Ubuntu keeps unprivileged user namespaces from programs without an
    /// AppArmor profile, and the microVM's shim (the path) has none.
    AppArmor(PathBuf),
}

/// What the PC lacks for the sandboxes; empty when it has all. A system
/// Docker does not support stops the list there: the rest would not help.
/// `shim`: the microVM's shim of the kit's Docker Sandboxes, when it is
/// installed (Linux checks its AppArmor profile).
pub fn gaps(shim: Option<&Path>) -> Vec<Gap> {
    imp::gaps(shim)
}

#[cfg(test)]
#[path = "../../../test/rust/platform/virt.rs"]
mod tests;
