//! What macOS tells of a process (`proc_pidinfo`) and the closing of
//! descriptors, for the guard and the stop of the kit's programs
//! (`os/unix/guard.rs`, `os/unix/sessions.rs`). No allocation: the guard
//! calls them in the child of a fork.

/// When `pid` started, to tell it from a later process with the same pid.
pub(crate) fn start_time(pid: i32) -> Option<u64> {
    // SAFETY: a zeroed proc_bsdinfo is valid; the call fills at most its size.
    unsafe {
        let mut info: libc::proc_bsdinfo = std::mem::zeroed();
        let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
        let got = libc::proc_pidinfo(
            pid,
            libc::PROC_PIDTBSDINFO,
            0,
            std::ptr::from_mut(&mut info).cast(),
            size,
        );
        (got == size).then(|| {
            info.pbi_start_tvsec
                .wrapping_mul(1_000_000)
                .wrapping_add(info.pbi_start_tvusec)
        })
    }
}

/// Closes every descriptor from `first` on.
pub(crate) unsafe fn close_from(first: libc::c_uint) {
    crate::os::unix::guard::close_loop(first);
}
