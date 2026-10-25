//! The Mac's memory (`platform::memory`): `hw.memsize`.

/// The physical memory of the Mac, in bytes (`hw.memsize`).
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
