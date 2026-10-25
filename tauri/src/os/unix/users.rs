//! The user the app runs as, on macOS and Linux.

/// The real user id (`getuid`).
pub fn real() -> u32 {
    // SAFETY: getuid has no preconditions and cannot fail.
    unsafe { libc::getuid() }
}

/// The effective user id (`geteuid`): the owner of what the app creates.
pub fn effective() -> u32 {
    // SAFETY: geteuid has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}
