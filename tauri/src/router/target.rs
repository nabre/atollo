//! Target capabilities and cross-target execution matrix for the 6 supported
//! OS/architecture targets (`win32-x64`, `win32-arm64`, `darwin-arm64`,
//! `darwin-x64`, `linux-x64`, `linux-arm64`).

use serde::Serialize;

use crate::kit::{Arch, Os, Target};
use crate::setup::catalog::Component;

/// Capabilities of one [`Target`] relative to the current host machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetProfile {
    pub id: &'static str,
    pub os: Os,
    pub arch: Arch,
    pub is_host: bool,
    pub executable_on_host: bool,
    pub emulated_on_host: bool,
}

/// True when binaries of `target` can be executed on the current host OS and CPU
/// (either natively, or via WOW64 on `win32-arm64` / Rosetta 2 on `darwin-arm64`).
pub fn can_execute_on_host(target: Target) -> bool {
    can_execute_between(Target::detect(), target)
}

/// Pure capability rule between `host` and `target`.
pub const fn can_execute_between(host: Target, target: Target) -> bool {
    if host.os as u8 != target.os as u8 {
        return false;
    }
    if host.arch as u8 == target.arch as u8 {
        return true;
    }
    matches!(
        (host.os, host.arch, target.arch),
        (Os::Win32 | Os::Darwin, Arch::Arm64, Arch::X64)
    )
}

/// True when `target` runs on `host` through binary translation (WOW64 or Rosetta 2).
pub fn is_emulated_on_host(target: Target) -> bool {
    let host = Target::detect();
    can_execute_between(host, target) && host.arch != target.arch
}

/// Effective [`Target`] used when downloading `component` for `target`,
/// honouring `x64_only` emulation on `win32-arm64`.
pub fn effective_target(target: Target, component: &Component) -> Target {
    if component.is_emulated_on(target) {
        target.with_arch(Arch::X64)
    } else {
        target
    }
}

/// Capability profile of `target` on the current host.
pub fn profile(target: Target) -> TargetProfile {
    let host = Target::detect();
    let executable = can_execute_between(host, target);
    TargetProfile {
        id: target.as_str(),
        os: target.os,
        arch: target.arch,
        is_host: host == target,
        executable_on_host: executable,
        emulated_on_host: is_emulated_on_host(target),
    }
}

/// Capability profiles of all 6 supported targets (`Target::ALL`).
#[allow(dead_code)]
pub fn all_profiles() -> [TargetProfile; 6] {
    Target::ALL.map(profile)
}
