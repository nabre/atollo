//! What a program started by the app gets on Linux (`os/unix/jobs.rs`).

use std::os::unix::process::CommandExt;
use std::process::Command;

/// The kernel sends `SIGTERM` to the program when the app dies
/// (`PR_SET_PDEATHSIG`), and a program whose app died before it started
/// does not start. Only from the app's main thread: the signal comes when
/// the *thread* that started the program ends, and the other threads
/// (tasks, the async pool) end long before the app. The guard covers the
/// rest. `SIGTERM`, not `SIGKILL`: databases write to disk on it, and the
/// guard sends `SIGKILL` 5 s later.
pub fn end_with_app(cmd: &mut Command) {
    // SAFETY: plain queries of this process and thread.
    let (app, thread) = unsafe { (libc::getpid(), libc::syscall(libc::SYS_gettid)) };
    if libc::c_long::from(app) != thread {
        return;
    }
    // SAFETY: the closure runs between fork and exec and only makes
    // system calls; its error needs no allocation.
    unsafe {
        cmd.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() != app {
                return Err(std::io::Error::from_raw_os_error(libc::ESRCH));
            }
            Ok(())
        });
    }
}
