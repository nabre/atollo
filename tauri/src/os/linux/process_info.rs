//! What Linux tells of a process (`/proc/<pid>/stat`) and the closing of
//! descriptors, for the guard and the stop of the kit's programs
//! (`os/unix/guard.rs`, `os/unix/sessions.rs`). No allocation: the guard
//! calls them in the child of a fork.

/// The start time of a Linux process from `/proc/<pid>/stat` (field 22,
/// after the name in parentheses, which may hold spaces and `)`).
pub(crate) fn stat_start_time(stat: &[u8]) -> Option<u64> {
    let close = stat.iter().rposition(|&b| b == b')')?;
    // Fields 3.. after ") "; the start time is field 22.
    let mut field = 2;
    let mut value: Option<u64> = None;
    for &b in stat.get(close + 1..)? {
        if b == b' ' {
            if field == 22 {
                break;
            }
            field += 1;
            value = None;
            continue;
        }
        if field == 22 {
            let digit = u64::from(b.checked_sub(b'0').filter(|d| *d <= 9)?);
            value = Some(value.unwrap_or(0).wrapping_mul(10).wrapping_add(digit));
        }
    }
    value.filter(|_| field == 22)
}

/// When `pid` started, to tell it from a later process with the same pid.
/// No allocation: also called by the guard.
pub(crate) fn start_time(pid: i32) -> Option<u64> {
    // "/proc/<pid>/stat\0" in a buffer on the stack.
    let mut path = [0u8; 32];
    let mut len = 0;
    let mut put = |bytes: &[u8]| {
        for &b in bytes {
            if let Some(slot) = path.get_mut(len) {
                *slot = b;
                len += 1;
            }
        }
    };
    put(b"/proc/");
    let mut digits = [0u8; 10];
    let mut n = u32::try_from(pid).ok()?;
    let mut count = 0;
    loop {
        *digits.get_mut(count)? = b'0' + (n % 10) as u8;
        count += 1;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    for i in (0..count).rev() {
        put(&[digits[i]]);
    }
    put(b"/stat\0");
    let mut buf = [0u8; 1024];
    // SAFETY: a NUL-terminated path on the stack; the descriptor is closed.
    let read = unsafe {
        let fd = libc::open(path.as_ptr().cast(), libc::O_RDONLY | libc::O_CLOEXEC);
        if fd < 0 {
            return None;
        }
        let n = libc::read(fd, buf.as_mut_ptr().cast(), buf.len());
        libc::close(fd);
        n
    };
    stat_start_time(buf.get(..usize::try_from(read).ok()?)?)
}

/// Closes every descriptor from `first` on.
pub(crate) unsafe fn close_from(first: libc::c_uint) {
    if libc::syscall(libc::SYS_close_range, first, libc::c_uint::MAX, 0) == 0 {
        return;
    }
    crate::os::unix::guard::close_loop(first);
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/process_info.rs"]
mod tests;
