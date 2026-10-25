//! The PC's processes (`platform::processes`): what macOS and Linux share.

/// True if a process with this id runs (signal 0 checks without sending;
/// EPERM: it runs under another user).
pub fn exists(pid: u32) -> bool {
    let Ok(pid @ 1..) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // SAFETY: signal 0 sends nothing.
    let sent = unsafe { libc::kill(pid, 0) } == 0;
    sent || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}
