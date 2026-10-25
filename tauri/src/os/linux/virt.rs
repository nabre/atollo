//! What this Linux lacks for Docker Sandboxes (`platform::virt`): Ubuntu
//! 24.04 or later (Docker supports no derivative: `ID` only, not
//! `ID_LIKE`), and `/dev/kvm` the user can read and write (the `kvm`
//! group).

use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use crate::platform::virt::Gap;

const KVM: &str = "/dev/kvm";

pub fn gaps() -> Vec<Gap> {
    gaps_of(super::host::os_release().as_deref(), kvm())
}

/// What `/dev/kvm` is to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kvm {
    Missing,
    Denied,
    Ready,
}

/// The gaps of a Linux with this `/etc/os-release` and KVM.
pub(crate) fn gaps_of(os_release: Option<&str>, kvm: Kvm) -> Vec<Gap> {
    let field = |name: &str| {
        os_release?
            .lines()
            .find_map(|l| l.strip_prefix(name)?.strip_prefix('='))
            .map(|v| v.trim().trim_matches('"').to_owned())
    };
    let id = field("ID").unwrap_or_default();
    let version = field("VERSION_ID").unwrap_or_default();
    let (major, minor) = version.split_once('.').unwrap_or((&version, "0"));
    let new_enough = (major.parse().unwrap_or(0u32), minor.parse().unwrap_or(0u32)) >= (24, 4);
    if id != "ubuntu" || !new_enough {
        let name =
            field("PRETTY_NAME").unwrap_or_else(|| format!("{id} {version}").trim().to_owned());
        return vec![Gap::Distribution(name)];
    }
    match kvm {
        Kvm::Missing => vec![Gap::KvmMissing],
        Kvm::Denied => vec![Gap::KvmDenied],
        Kvm::Ready => Vec::new(),
    }
}

fn kvm() -> Kvm {
    let path = Path::new(KVM);
    if !path.exists() {
        return Kvm::Missing;
    }
    let Ok(c) = CString::new(path.as_os_str().as_bytes()) else {
        return Kvm::Denied;
    };
    // SAFETY: a NUL-terminated path; `access` only reads it.
    if unsafe { libc::access(c.as_ptr(), libc::R_OK | libc::W_OK) } == 0 {
        Kvm::Ready
    } else {
        Kvm::Denied
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/virt.rs"]
mod tests;
