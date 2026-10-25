//! No program of the kit outlives the app on macOS and Linux, even when the
//! app is killed (D-105, the Unix side of `KILL_ON_JOB_CLOSE`, D-07).
//!
//! At start the app forks a guard: a copy of itself that does nothing but
//! read a pipe. The app writes there the process group of every program it
//! starts (`os/unix/jobs.rs`), and nobody else holds the pipe's write end (it is
//! close-on-exec). When the app ends, however it ends, the pipe closes: the
//! guard sends `SIGTERM` to the groups that are still the kit's, waits up to
//! 5 s (databases write to disk at `SIGTERM`), sends `SIGKILL` to what is
//! left and exits.
//!
//! Why a fork and not a new program: no command line, no `main` to change,
//! nothing to find on disk (an AppImage's program goes away with its mount).
//! The child of a fork in a program with threads may only make system
//! calls: the guard allocates nothing, takes no lock and keeps its table on
//! the stack. Only groups are signalled, and a group only while it is still
//! the one the app started: its leader is gone (the members keep the number
//! from being reused) or is the same process (same start time).
//!
//! A group whose last process ended is forgotten within a second (the guard
//! checks its table every `TICK_MS`), long before its number can come back
//! as the group of a program that is not the kit's: many programs are
//! started without being unwatched (one per database query, the tools).
//!
//! The guard ignores `SIGTERM`, `SIGQUIT`, `SIGHUP` and `SIGINT`: it has the
//! app's name, so `pkill`/`killall` by name reach both, and the guard must
//! outlive the app to do its work. It still ends on its own once the pipe
//! closes, and `SIGKILL` ends it.
//!
//! Limits: a program that leaves its group (`setsid`, a shell's job
//! control) is not followed; with `SLOTS` live groups at once a further one
//! is not followed either (the guard cannot log: nothing is open); the guard
//! holds the app's file open until it exits, at most 5 s after the app.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

use anyhow::{Context, Result};

use crate::os::current::process_info::{close_from, start_time};

/// The groups the guard watches at most at once.
const SLOTS: usize = 1024;

/// How often the guard forgets the groups that emptied, in milliseconds.
const TICK_MS: libc::c_int = 1000;

/// One message: the group (negative: no longer watched) and the start time
/// of its leader, little-endian, padded to 16 bytes (one atomic write).
const RECORD: usize = 16;

/// Waiting between `SIGTERM` and `SIGKILL`: 50 steps of 100 ms.
const GRACE_STEPS: u32 = 50;

/// One message for the guard: `pgid` watched (`add`) or not any more.
pub(crate) fn encode(pgid: i32, start: u64, add: bool) -> [u8; RECORD] {
    let mut out = [0u8; RECORD];
    let id = if add { pgid } else { pgid.wrapping_neg() };
    out[..4].copy_from_slice(&id.to_le_bytes());
    out[8..].copy_from_slice(&start.to_le_bytes());
    out
}

/// The group and start time of a message, `add` when positive.
pub(crate) fn decode(record: &[u8; RECORD]) -> (i32, u64, bool) {
    let id = i32::from_le_bytes([record[0], record[1], record[2], record[3]]);
    let mut start = [0u8; 8];
    start.copy_from_slice(&record[8..]);
    (id.wrapping_abs(), u64::from_le_bytes(start), id > 0)
}

/// The pipe to the guard, and the guard's pid (0: no guard).
static PIPE: Mutex<Option<std::io::PipeWriter>> = Mutex::new(None);
static GUARD: AtomicU32 = AtomicU32::new(0);

/// The guard's pid, so that the lists of the kit's programs leave it out.
pub fn pid() -> Option<u32> {
    Some(GUARD.load(Ordering::Relaxed)).filter(|&p| p != 0)
}

/// Starts the guard (once; later calls do nothing).
pub fn start() -> Result<()> {
    use std::os::fd::AsRawFd;
    let mut pipe = PIPE.lock().unwrap_or_else(|e| e.into_inner());
    if pipe.is_some() {
        return Ok(());
    }
    // Both ends close-on-exec: no program the app starts holds them.
    let (reader, writer) = std::io::pipe().context("guard pipe")?;
    // SAFETY: fcntl on a descriptor owned by `writer`; a full pipe makes
    // a message fail instead of blocking the app.
    unsafe {
        let fd = writer.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
    let child = fork_guard(reader)?;
    GUARD.store(child.unsigned_abs(), Ordering::Relaxed);
    *pipe = Some(writer);
    Ok(())
}

/// Forks a guard that reads `reader`; returns its pid.
fn fork_guard(reader: std::io::PipeReader) -> Result<libc::pid_t> {
    use std::os::fd::AsRawFd;
    let read_fd = reader.as_raw_fd();
    // SAFETY: the child runs only `guard_main`, which makes system calls
    // and never returns.
    let child = unsafe { libc::fork() };
    if child < 0 {
        return Err(std::io::Error::last_os_error()).context("fork of the guard");
    }
    if child == 0 {
        // SAFETY: in the new process, with `read_fd` open.
        unsafe { guard_main(read_fd) }
    }
    Ok(child)
}

/// Sends one message to the guard; without a guard nothing happens.
fn send(record: &[u8; RECORD]) {
    use std::io::Write;
    let mut pipe = PIPE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(w) = pipe.as_mut() {
        // At most PIPE_BUF bytes: written whole or not at all.
        if let Err(e) = w.write_all(record) {
            crate::common::log::warn(&format!("guard: {e}"));
        }
    }
}

/// From now on the guard ends the group `pgid` if the app dies.
pub fn watch(pgid: i32) {
    if pgid > 1 {
        send(&encode(pgid, start_time(pgid).unwrap_or(0), true));
    }
}

/// The group `pgid` was ended by the app: the guard forgets it.
pub fn unwatch(pgid: i32) {
    if pgid > 1 {
        send(&encode(pgid, 0, false));
    }
}

/// The last error number.
fn errno() -> i32 {
    std::io::Error::last_os_error().raw_os_error().unwrap_or(0)
}

/// Whether signalling the group `pgid` reaches only the group the app
/// started: its leader is gone (no process can then take the number while
/// members are left), or it is the same process as at `start`.
fn still_ours(pgid: i32, start: u64) -> bool {
    // SAFETY: signal 0 only checks that the process exists.
    if unsafe { libc::kill(pgid, 0) } == -1 && errno() == libc::ESRCH {
        return true;
    }
    start != 0 && start_time(pgid) == Some(start)
}

/// Some process is left in the group `pgid`.
fn group_alive(pgid: i32) -> bool {
    // SAFETY: signal 0 to a group only checks that it has members.
    unsafe { libc::killpg(pgid, 0) == 0 || errno() == libc::EPERM }
}

/// The guard's whole life, in the child of `fork`: system calls only.
unsafe fn guard_main(read_fd: libc::c_int) -> ! {
    // Its own group, no terminal signals, no hold on the kit's folder or on
    // the app's files: only the pipe stays open, as descriptor 3.
    libc::setpgid(0, 0);
    for signal in [libc::SIGHUP, libc::SIGINT, libc::SIGTERM, libc::SIGQUIT] {
        libc::signal(signal, libc::SIG_IGN);
    }
    libc::chdir(c"/".as_ptr());
    if read_fd != 3 {
        libc::dup2(read_fd, 3);
    }
    close_from(4);
    let null = libc::open(c"/dev/null".as_ptr(), libc::O_RDWR);
    for fd in 0..3 {
        libc::dup2(null, fd);
    }
    if null > 3 {
        libc::close(null);
    }

    let mut groups = [(0i32, 0u64); SLOTS];
    let mut record = [0u8; RECORD];
    let mut have = 0usize;
    loop {
        let mut wait = libc::pollfd {
            fd: 3,
            events: libc::POLLIN,
            revents: 0,
        };
        let ready = libc::poll(&mut wait, 1, TICK_MS);
        if ready == 0 || (ready < 0 && errno() == libc::EINTR) {
            prune(&mut groups, group_alive);
            continue;
        }
        let rest = record.get_mut(have..).unwrap_or_default();
        let n = libc::read(3, rest.as_mut_ptr().cast(), rest.len());
        if n < 0 && errno() == libc::EINTR {
            continue;
        }
        if n <= 0 {
            break; // the app is gone
        }
        have += n.unsigned_abs();
        if have < RECORD {
            continue;
        }
        have = 0;
        let (pgid, start, add) = decode(&record);
        if add {
            prune(&mut groups, group_alive);
            insert(&mut groups, pgid, start);
        } else {
            forget(&mut groups, pgid);
        }
    }

    for slot in groups.iter_mut().filter(|s| s.0 != 0) {
        if still_ours(slot.0, slot.1) {
            libc::killpg(slot.0, libc::SIGTERM);
        } else {
            *slot = (0, 0);
        }
    }
    let pause = libc::timespec {
        tv_sec: 0,
        tv_nsec: 100_000_000,
    };
    for _ in 0..GRACE_STEPS {
        if !groups.iter().any(|s| s.0 != 0 && group_alive(s.0)) {
            libc::_exit(0);
        }
        libc::nanosleep(&pause, std::ptr::null_mut());
    }
    for slot in groups.iter().filter(|s| s.0 != 0) {
        if still_ours(slot.0, slot.1) {
            libc::killpg(slot.0, libc::SIGKILL);
        }
    }
    libc::_exit(0)
}

/// Adds a group to the table (or renews its start time); `false` when the
/// table is full and the group is not followed. No allocation.
pub(crate) fn insert(groups: &mut [(i32, u64)], pgid: i32, start: u64) -> bool {
    if let Some(slot) = groups.iter_mut().find(|s| s.0 == pgid) {
        *slot = (pgid, start);
        return true;
    }
    match groups.iter_mut().find(|s| s.0 == 0) {
        Some(slot) => {
            *slot = (pgid, start);
            true
        }
        None => false,
    }
}

/// Forgets the group `pgid`.
pub(crate) fn forget(groups: &mut [(i32, u64)], pgid: i32) {
    for slot in groups.iter_mut().filter(|s| s.0 == pgid) {
        *slot = (0, 0);
    }
}

/// Forgets every group with no process left, before its number can be
/// reused by a group that is not the kit's. No allocation.
pub(crate) fn prune(groups: &mut [(i32, u64)], alive: impl Fn(i32) -> bool) {
    for slot in groups.iter_mut().filter(|s| s.0 != 0) {
        if !alive(slot.0) {
            *slot = (0, 0);
        }
    }
}

/// Closes the descriptors from `first` up to the limit (kernels without
/// `close_range`), at most 65536 of them.
pub(crate) unsafe fn close_loop(first: libc::c_uint) {
    let max = libc::sysconf(libc::_SC_OPEN_MAX).clamp(0, 65_536);
    let mut fd = libc::c_int::try_from(first).unwrap_or(libc::c_int::MAX);
    while libc::c_long::from(fd) < max {
        libc::close(fd);
        fd += 1;
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/guard.rs"]
mod tests;
