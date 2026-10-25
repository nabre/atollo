//! What this Windows lacks for Docker Sandboxes (`platform::virt`):
//! Windows 11 on an x64 processor, and Windows Hypervisor Platform on.
//! The feature is read without an administrator: `WinHvPlatform.dll` is
//! there only when it is on, and its `WHvGetCapability` says whether the
//! hypervisor runs (on, but not yet restarted: it does not).

use windows::core::{s, w, HRESULT};
use windows::Win32::Foundation::FreeLibrary;
use windows::Win32::System::LibraryLoader::{
    GetProcAddress, LoadLibraryExW, LOAD_LIBRARY_SEARCH_SYSTEM32,
};

use crate::kit::Arch;
use crate::platform::virt::Gap;

/// The first build of Windows 11.
const WINDOWS_11: u32 = 22000;

/// `WHvCapabilityCodeHypervisorPresent`.
const HYPERVISOR_PRESENT: i32 = 0;

type GetCapabilityFn =
    unsafe extern "system" fn(i32, *mut core::ffi::c_void, u32, *mut u32) -> HRESULT;

pub fn gaps() -> Vec<Gap> {
    gaps_of(
        super::host::windows_build(),
        super::host::arch(),
        hypervisor_runs,
    )
}

/// The gaps of a Windows `build` on `arch`, the hypervisor asked last.
pub(crate) fn gaps_of(build: Option<u32>, arch: Arch, hypervisor: impl Fn() -> bool) -> Vec<Gap> {
    if let Some(build) = build.filter(|b| *b < WINDOWS_11) {
        return vec![Gap::OldWindows(build)];
    }
    if arch == Arch::Arm64 {
        return vec![Gap::Processor];
    }
    if hypervisor() {
        Vec::new()
    } else {
        vec![Gap::HypervisorOff]
    }
}

/// Windows Hypervisor Platform is on and its hypervisor runs.
fn hypervisor_runs() -> bool {
    // SAFETY: a NUL-terminated name, looked for in System32 only; the
    // function has the signature of `WinHvPlatform.h`; the buffer is a
    // BOOL, its size given; the module is freed after the call.
    unsafe {
        let Ok(module) =
            LoadLibraryExW(w!("WinHvPlatform.dll"), None, LOAD_LIBRARY_SEARCH_SYSTEM32)
        else {
            return false;
        };
        let present = GetProcAddress(module, s!("WHvGetCapability")).is_some_and(|f| {
            let get =
                std::mem::transmute::<unsafe extern "system" fn() -> isize, GetCapabilityFn>(f);
            let mut value: i32 = 0;
            let mut written = 0u32;
            get(
                HYPERVISOR_PRESENT,
                (&raw mut value).cast(),
                std::mem::size_of::<i32>() as u32,
                &raw mut written,
            )
            .is_ok()
                && value != 0
        });
        let _ = FreeLibrary(module);
        present
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/virt.rs"]
mod tests;
