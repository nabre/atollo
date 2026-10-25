//! The kit's processes on macOS and Linux (`platform::jobs`): there is no
//! Job Object, so each program of a `Job` leads a process group of its own,
//! killed (`killpg`) when the job is dropped, and what the app started is
//! stopped at exit by `platform::sessions::stop_children`. If the app is
//! killed, the guard (`os/unix/guard.rs`) ends the groups of the programs
//! it started; on Linux a program started from the main thread also gets
//! `SIGTERM` from the kernel when the app dies (`os/linux/child.rs`,
//! D-105).

use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use anyhow::{Context, Result};

use crate::env::KitEnv;

/// The process groups of the programs started in the job (Unix).
pub struct Job {
    id: u64,
    groups: std::sync::Mutex<Vec<i32>>,
}

/// Which job last started a process with a given pid (0: none, a program
/// started by [`spawn`] or [`spawn_detached`]). A job's leader is often
/// reaped long before the job is dropped, and its pid can then lead a new
/// group: the job signals a group only while the pid is still its own here.
static OWNERS: std::sync::Mutex<std::collections::BTreeMap<i32, u64>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

fn claim(pid: u32, job: u64) {
    if let Ok(pid) = i32::try_from(pid) {
        OWNERS
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(pid, job);
    }
}

/// Whether `killpg(pgid)` can only reach the job's own group. When no
/// process has the pid `pgid`, the group is the job's (its members keep the
/// number from being reused) or empty. When one does, it must be the job's
/// leader, not yet reaped: a child of the app that `waitid` still sees.
fn own_group(pgid: i32) -> bool {
    // SAFETY: signal 0 only checks that the process exists.
    if unsafe { libc::kill(pgid, 0) } == -1
        && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
    {
        return true;
    }
    let Ok(id) = libc::id_t::try_from(pgid) else {
        return false;
    };
    // SAFETY: a zeroed siginfo_t is valid; WNOWAIT leaves a zombie in place.
    unsafe {
        let mut info: libc::siginfo_t = std::mem::zeroed();
        libc::waitid(
            libc::P_PID,
            id,
            &mut info,
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        ) == 0
    }
}

impl Job {
    pub fn new() -> Result<Job> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        Ok(Job {
            id: NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            groups: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Adds the group led by `child` (started with `process_group(0)`).
    pub fn assign(&self, child: &Child) -> Result<()> {
        let pid = i32::try_from(child.id()).context("pid")?;
        claim(child.id(), self.id);
        self.groups
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(pid);
        crate::os::unix::guard::watch(pid);
        Ok(())
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        let groups = self.groups.get_mut().unwrap_or_else(|e| e.into_inner());
        let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
        for &pgid in groups.iter() {
            if owners.get(&pgid) != Some(&self.id) {
                continue; // the pid went to a later program
            }
            owners.remove(&pgid);
            if own_group(pgid) {
                // SAFETY: a plain signal to a process group this job created.
                unsafe { libc::killpg(pgid, libc::SIGKILL) };
            }
            crate::os::unix::guard::unwatch(pgid);
        }
    }
}

/// No Job Object on this system: the guard takes its place (D-105). Must
/// run before the app starts its first program. A guard that cannot start
/// leaves the programs to `sessions::stop_children` at a normal exit; the
/// returned text says so, for the log.
pub fn init() -> Result<Option<String>> {
    match crate::os::unix::guard::start() {
        Ok(()) => Ok(None),
        Err(e) => Ok(Some(format!(
            "{e:#}; children are stopped at a normal exit only"
        ))),
    }
}

/// The guard watches the group `child` leads, if it leads one: a program
/// started without `process_group(0)` is in the app's own group, which is
/// never signalled.
fn watch_group(child: &Child) {
    let Ok(pid) = libc::pid_t::try_from(child.id()) else {
        return;
    };
    // SAFETY: a plain query; a pid that is gone just fails.
    if unsafe { libc::getpgid(pid) } == pid {
        crate::os::unix::guard::watch(pid);
    }
}

/// A command for a kit program: the kit environment only, no inherited
/// variables, in a process group of its own (no terminal signals).
pub fn kit_command(program: impl AsRef<std::ffi::OsStr>, env: &KitEnv) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    cmd.envs(env.iter());
    cmd.process_group(0);
    cmd
}

/// A shell script run as `/bin/sh ./<name> <args>` in its own folder (the
/// Unix side of the `.bat` of D-17). `args` must be fixed words of the app.
pub fn batch_command(script: &std::path::Path, args: &[&str], env: &KitEnv) -> Command {
    let mut cmd = kit_command("/bin/sh", env);
    if let Some(dir) = script.parent().filter(|d| !d.as_os_str().is_empty()) {
        cmd.current_dir(dir);
    }
    let name = script.file_name().unwrap_or(script.as_os_str());
    cmd.arg(std::path::Path::new(".").join(name));
    cmd.args(args);
    cmd
}

/// Starts a program that belongs to the session: stopped at exit.
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    crate::os::current::child::end_with_app(cmd);
    let child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    claim(child.id(), 0);
    watch_group(&child);
    Ok(child)
}

/// Starts a program in `job`, as the leader of a new process group: dropping
/// the job kills the group, its children included.
pub fn spawn_in(cmd: &mut Command, job: &Job) -> Result<Child> {
    cmd.process_group(0);
    crate::os::current::child::end_with_app(cmd);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    if let Err(e) = job.assign(&child) {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e);
    }
    Ok(child)
}

/// Starts a program that must stay open after the app (browser, file
/// manager): its own process group, never stopped by the app. Returns its
/// pid: the program is still the app's child, and a thread waits for it,
/// or every `open` and `xdg-open` (which end at once) would stay a zombie
/// until the app exits (BUG-V7-20).
fn spawn_detached(cmd: &mut Command) -> Result<u32> {
    cmd.process_group(0);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    let pid = child.id();
    claim(pid, 0);
    std::thread::spawn(move || {
        let _ = child.wait();
        // Its pid can now go to another program: the mark it left (no job)
        // goes too, unless a later program has claimed the pid since.
        if let Ok(pid) = i32::try_from(pid) {
            let mut owners = OWNERS.lock().unwrap_or_else(|e| e.into_inner());
            if owners.get(&pid) == Some(&0) {
                owners.remove(&pid);
            }
        }
    });
    Ok(pid)
}

/// Opens a web address or a folder with the system's handler (`open` on
/// macOS, `xdg-open` elsewhere), outside the app's processes.
pub fn open_with_explorer(target: impl AsRef<OsStr>) -> Result<()> {
    spawn_detached(Command::new(crate::os::current::OPENER).arg(target)).map(|_| ())
}

pub fn open_url(url: &str) -> Result<()> {
    open_with_explorer(url)
}

/// A command line of the user (a project's action) through `/bin/sh -c
/// <line>`, with the kit environment.
pub fn shell_command(line: &str, env: &KitEnv) -> Command {
    let mut cmd = kit_command("/bin/sh", env);
    cmd.arg("-c").arg(line);
    cmd
}

/// Quotes a path for `/bin/sh` when it holds anything but plain
/// characters: in single quotes.
pub fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "._-/+@".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', r"'\''"))
    }
}

/// `"$VAR/a/b"`.
pub fn shell_var_path(var: &str, rel: &[&str]) -> String {
    format!("\"${var}/{}\"", rel.join("/"))
}

/// The guard's pid, so that the lists of the kit's programs leave it out.
pub fn guard_pid() -> Option<u32> {
    crate::os::unix::guard::pid()
}
