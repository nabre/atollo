//! The end of the kit's programs on Windows (`platform::sessions::stop`):
//! `TerminateProcess`, at once.

use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

/// Ends `tree`, each process at once.
pub fn stop_tree(tree: &[u32]) {
    for &pid in tree {
        terminate(pid);
    }
}

fn terminate(pid: u32) {
    // SAFETY: terminate-only handle, closed right after.
    unsafe {
        if let Ok(h) = OpenProcess(PROCESS_TERMINATE, false, pid) {
            let _ = TerminateProcess(h, 1);
            let _ = CloseHandle(h);
        }
    }
}
