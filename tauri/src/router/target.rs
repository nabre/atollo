//! Target capabilities and cross-target execution matrix for the 6 supported
//! OS/architecture targets (`win32-x64`, `win32-arm64`, `darwin-arm64`,
//! `darwin-x64`, `linux-x64`, `linux-arm64`).
//!
//! The rules are pure functions of the host, the target and (on a Mac with
//! Apple Silicon) whether Rosetta 2 is installed, so the tests cover every
//! pair from any system; only [`rosetta_present`] asks the machine.

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
    let host = Target::detect();
    can_execute_between(host, target, host_rosetta(host, target))
}

/// Pure capability rule between `host` and `target`: the same system, and
/// the same CPU or x64 programs on ARM64 where the system translates them
/// (always on Windows 11 ARM64, on macOS only with Rosetta 2, `rosetta`).
/// Linux on ARM64 has no such layer.
pub const fn can_execute_between(host: Target, target: Target, rosetta: bool) -> bool {
    if host.os as u8 != target.os as u8 {
        return false;
    }
    if host.arch as u8 == target.arch as u8 {
        return true;
    }
    match (host.os, host.arch, target.arch) {
        (Os::Win32, Arch::Arm64, Arch::X64) => true,
        (Os::Darwin, Arch::Arm64, Arch::X64) => rosetta,
        _ => false,
    }
}

/// Whether Rosetta 2 matters for running `target` on `host`, and is there:
/// probed only on a Mac with Apple Silicon asked about x64 programs.
fn host_rosetta(host: Target, target: Target) -> bool {
    host == Target::DARWIN_ARM64 && target == Target::DARWIN_X64 && rosetta_present()
}

/// Rosetta 2 runs x64 programs on this Mac: `/usr/bin/arch -x86_64
/// /usr/bin/true` exits with 0. Asked once per start (installing Rosetta
/// needs an administrator, and the app is started again after it); false
/// on every other system.
pub fn rosetta_present() -> bool {
    #[cfg(target_os = "macos")]
    {
        static PRESENT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        *PRESENT.get_or_init(|| {
            std::process::Command::new("/usr/bin/arch")
                .args(["-x86_64", "/usr/bin/true"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|s| s.success())
        })
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

/// Pure rule: `component` runs on `target` only as an x64 build under
/// Rosetta 2 (`rosetta` in the catalog), this host is that Mac, and
/// Rosetta is not installed (`rosetta`). A kit prepared for `darwin-arm64`
/// from another system cannot know: there it stays available, and the Mac
/// says it when the kit starts there.
pub fn rosetta_missing(host: Target, target: Target, component: &Component, rosetta: bool) -> bool {
    host == target && target == Target::DARWIN_ARM64 && component.is_emulated_on(target) && !rosetta
}

/// [`rosetta_missing`] on this host (Rosetta probed only when it matters).
pub fn rosetta_missing_on_host(target: Target, component: &Component) -> bool {
    let host = Target::detect();
    rosetta_missing(host, target, component, false) && !rosetta_present()
}

/// `component` can be installed for `target` from this host: the catalog
/// has a build for it ([`Component::is_available_on`]) and, for an x64
/// build on a Mac with Apple Silicon, Rosetta 2 is there.
pub fn installable_on(component: &Component, target: Target) -> bool {
    component.is_available_on(target) && !rosetta_missing_on_host(target, component)
}

/// True when `target` runs on `host` through binary translation (WOW64 or Rosetta 2).
pub fn is_emulated_on_host(target: Target) -> bool {
    let host = Target::detect();
    can_execute_on_host(target) && host.arch != target.arch
}

/// Effective [`Target`] used when downloading `component` for `target`:
/// the x64 build where it runs emulated (`x64_only` on `win32-arm64`,
/// `rosetta` on `darwin-arm64`).
pub fn effective_target(target: Target, component: &Component) -> Target {
    if component.is_emulated_on(target) {
        target.with_arch(Arch::X64)
    } else {
        target
    }
}

/// Capability profile of `target` on the current host.
pub fn profile(target: Target) -> TargetProfile {
    TargetProfile {
        id: target.as_str(),
        os: target.os,
        arch: target.arch,
        is_host: Target::detect() == target,
        executable_on_host: can_execute_on_host(target),
        emulated_on_host: is_emulated_on_host(target),
    }
}

/// Capability profiles of all 6 supported targets (`Target::ALL`).
#[allow(dead_code)]
pub fn all_profiles() -> [TargetProfile; 6] {
    Target::ALL.map(profile)
}
