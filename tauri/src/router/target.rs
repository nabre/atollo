//! Target capabilities and cross-target execution matrix for the 4 supported
//! OS/architecture targets (`win32-x64`, `darwin-arm64`, `linux-x64`,
//! `linux-arm64`, D-176).
//!
//! The rules are pure functions of the host and the target, so the tests
//! cover every pair from any system.

use serde::Serialize;

use crate::kit::{Arch, Os, Target};
use crate::setup::Component;

/// Capabilities of one [`Target`] relative to the current host machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetProfile {
    pub id: &'static str,
    pub os: Os,
    pub arch: Arch,
    pub is_host: bool,
    pub executable_on_host: bool,
}

/// True when binaries of `target` can be executed on the current host OS and CPU.
pub fn can_execute_on_host(target: Target) -> bool {
    can_execute_between(Target::detect(), target)
}

/// Pure capability rule between `host` and `target`: the same system on the
/// same CPU. No translation layer is used, so an ARM64 Linux or Mac does
/// not run x64 programs.
pub fn can_execute_between(host: Target, target: Target) -> bool {
    host == target
}

/// `component` can be installed for `target` from this host: the catalog
/// has a build for it ([`Component::is_available_on`]).
pub fn installable_on(component: &Component, target: Target) -> bool {
    component.is_available_on(target)
}

/// Capability profile of `target` on the current host.
pub fn profile(target: Target) -> TargetProfile {
    TargetProfile {
        id: target.as_str(),
        os: target.os,
        arch: target.arch,
        is_host: Target::detect() == target,
        executable_on_host: can_execute_on_host(target),
    }
}

/// Capability profiles of the 4 supported targets (`Target::ALL`), in
/// the order of the grid of the Tools page.
pub fn all_profiles() -> [TargetProfile; 4] {
    Target::ALL.map(profile)
}
