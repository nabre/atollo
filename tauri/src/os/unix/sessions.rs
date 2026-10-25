//! The end of the kit's programs on macOS and Linux
//! (`platform::sessions::stop`): `SIGTERM` first, `SIGKILL` to what is left.

use crate::os::current::process_info::start_time;

/// Ends `tree` (each process before its children): `SIGTERM`, a wait of up
/// to 5 s, then `SIGKILL`.
pub fn stop_tree(tree: &[u32]) {
    stop_gently(
        tree,
        std::time::Duration::from_secs(5),
        std::time::Duration::from_millis(100),
        signal,
        identity,
    );
}

/// A signal of [`stop_gently`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Signal {
    Term,
    Kill,
}

/// `SIGTERM` to every process, a wait of up to `grace` (checked every
/// `step`) for them to end, then `SIGKILL` to those left (D-105): the
/// databases write their files to disk on `SIGTERM`. `identity` is when a
/// running process started (`None`: it has ended): a pid that ends and is
/// taken by another program during the wait no longer matches, and that
/// program, outside the kit, gets no signal.
pub(crate) fn stop_gently(
    pids: &[u32],
    grace: std::time::Duration,
    step: std::time::Duration,
    signal: impl Fn(u32, Signal),
    identity: impl Fn(u32) -> Option<u64>,
) {
    let mut left: Vec<(u32, u64)> = pids
        .iter()
        .filter_map(|&pid| Some((pid, identity(pid)?)))
        .collect();
    for &(pid, _) in &left {
        signal(pid, Signal::Term);
    }
    let start = std::time::Instant::now();
    loop {
        left.retain(|&(pid, started)| identity(pid) == Some(started));
        if left.is_empty() || start.elapsed() >= grace {
            break;
        }
        std::thread::sleep(step);
    }
    for (pid, _) in left {
        signal(pid, Signal::Kill);
    }
}

fn signal(pid: u32, which: Signal) {
    // 0 and negative numbers would signal whole process groups.
    if let Ok(pid @ 1..) = libc::pid_t::try_from(pid) {
        let sig = match which {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
        };
        // SAFETY: a plain signal to one process; a pid that is gone just fails.
        unsafe { libc::kill(pid, sig) };
    }
}

/// The process still runs: it exists and is not a child of the app that
/// has ended and waits to be reaped (a zombie still answers `kill`).
fn alive(pid: u32) -> bool {
    let Ok(id @ 1..) = libc::pid_t::try_from(pid) else {
        return false;
    };
    // SAFETY: signal 0 only checks that the process exists.
    if unsafe { libc::kill(id, 0) } != 0
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    {
        return false;
    }
    // SAFETY: a zeroed siginfo_t is valid; WNOWAIT leaves the zombie to
    // whoever waits for it.
    let ended = unsafe {
        let mut info: libc::siginfo_t = std::mem::zeroed();
        libc::waitid(
            libc::P_PID,
            pid,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        ) == 0
            && info.si_pid() == id
    };
    !ended
}

/// When `pid`, still running, started; `None` once it has ended. A start
/// time that cannot be read counts as 0: such a process is then told only
/// by its pid, as before.
fn identity(pid: u32) -> Option<u64> {
    let id = libc::pid_t::try_from(pid).ok()?;
    alive(pid).then(|| start_time(id).unwrap_or(0))
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/sessions.rs"]
mod tests;
