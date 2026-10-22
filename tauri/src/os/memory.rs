//! The PC's memory: how much RAM it has, to tell whether a local model fits
//! (`llm`).

/// The physical memory of the PC, in bytes.
#[cfg(windows)]
pub fn total_ram() -> Option<u64> {
    use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
    let mut status = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    // SAFETY: the structure is ours, its length set as the call requires.
    unsafe { GlobalMemoryStatusEx(&mut status) }.ok()?;
    Some(status.ullTotalPhys)
}

/// The physical memory of the Mac, in bytes (`hw.memsize`).
#[cfg(target_os = "macos")]
pub fn total_ram() -> Option<u64> {
    let mut value: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let name = c"hw.memsize";
    // SAFETY: a u64 and its length, as the call expects for hw.memsize.
    let rc = unsafe {
        libc::sysctlbyname(
            name.as_ptr(),
            (&mut value as *mut u64).cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    (rc == 0).then_some(value)
}

/// The physical memory, in bytes (`MemTotal` of `/proc/meminfo`).
#[cfg(all(unix, not(target_os = "macos")))]
pub fn total_ram() -> Option<u64> {
    mem_total(&std::fs::read_to_string("/proc/meminfo").ok()?)
}

/// `MemTotal: <n> kB` of a `/proc/meminfo` text, in bytes.
#[cfg_attr(windows, allow(dead_code))]
pub(crate) fn mem_total(meminfo: &str) -> Option<u64> {
    meminfo.lines().find_map(|l| {
        let kb = l.strip_prefix("MemTotal:")?.trim().strip_suffix("kB")?;
        kb.trim().parse::<u64>().ok().map(|kb| kb * 1024)
    })
}

#[cfg(test)]
#[path = "../../../test/rust/os/memory.rs"]
mod tests;
