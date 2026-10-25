//! The PTY of the app's terminal on macOS and Linux (`platform::pty`):
//! portable-pty; the program leads a session and a process group of its
//! own (`setsid` of the PTY), put in a job (`platform::jobs`).

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, PoisonError};

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};

use crate::env::KitEnv;
use crate::platform::jobs::Job;
use crate::platform::pty::{Io, Program, Spawned};

struct Started {
    io: PtyIo,
    reader: Box<dyn Read + Send>,
    child: Box<dyn Child + Send + Sync>,
}

struct PtyIo {
    pid: Option<u32>,
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    input: Mutex<Option<Sender<Vec<u8>>>>,
    job: Mutex<Option<Job>>,
}

/// The size before the view measures itself (`pty_resize`).
const SIZE: PtySize = PtySize {
    rows: 24,
    cols: 80,
    pixel_width: 0,
    pixel_height: 0,
};

fn start(launch: &Program, env: &KitEnv, dir: &Path) -> Result<Started> {
    let pair = native_pty_system().openpty(SIZE).context("openpty")?;
    let mut cmd = CommandBuilder::new(launch.program);
    cmd.args(launch.args);
    cmd.cwd(dir);
    cmd.env_clear();
    for (k, v) in env.iter() {
        cmd.env(k, v);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("SHELL", launch.shell);
    let locale = utf8_locale(
        crate::kit::Os::current(),
        env.get("LC_ALL"),
        env.get("LC_CTYPE"),
        env.get("LANG"),
    );
    if let Some((name, value)) = locale {
        cmd.env(name, value);
    }
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .with_context(|| format!("starting {}", launch.program.display()))?;
    // The reader ends only once every copy of the PTY's other side is
    // closed: the app keeps none.
    drop(pair.slave);
    let pid = child.process_id();
    // The program leads its own group (setsid): the job kills that
    // group when the session lets it go, as for the app's other jobs.
    let prepared = (|| -> Result<_> {
        let job = Job::new()?;
        let leader: &dyn Child = &*child;
        let std_child = leader
            .downcast_ref::<std::process::Child>()
            .context("pty child is not a std::process::Child")?;
        job.assign(std_child)?;
        let reader = pair.master.try_clone_reader().context("pty reader")?;
        let writer = pair.master.take_writer().context("pty writer")?;
        Ok((job, reader, writer))
    })();
    // Never a session outside the job on any error once it started:
    // its group would outlive it (BUG-V6-25).
    let (job, reader, mut writer) = match prepared {
        Ok(p) => p,
        Err(e) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(e);
        }
    };
    let (tx, rx) = channel::<Vec<u8>>();
    std::thread::spawn(move || {
        for data in rx {
            if writer
                .write_all(&data)
                .and_then(|()| writer.flush())
                .is_err()
            {
                break;
            }
        }
    });
    Ok(Started {
        io: PtyIo {
            pid,
            master: Mutex::new(Some(pair.master)),
            input: Mutex::new(Some(tx)),
            job: Mutex::new(Some(job)),
        },
        reader,
        child,
    })
}

impl Io for PtyIo {
    fn write(&self, data: &[u8]) -> Result<()> {
        let input = self.input.lock().unwrap_or_else(PoisonError::into_inner);
        match input.as_ref() {
            Some(tx) => tx.send(data.to_vec()).context("terminal closed"),
            None => anyhow::bail!("terminal closed"),
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let master = self.master.lock().unwrap_or_else(PoisonError::into_inner);
        match master.as_ref() {
            Some(m) => m.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            }),
            None => anyhow::bail!("terminal closed"),
        }
    }

    fn pid(&self) -> Option<u32> {
        self.pid
    }

    fn hang_up(&self) {
        // Only while the program has not been reaped (the session
        // checks `running`): its pid cannot belong to another process.
        if let Some(pid @ 1..) = self.pid.and_then(|p| libc::pid_t::try_from(p).ok()) {
            // SAFETY: a plain signal to one process.
            unsafe { libc::kill(pid, libc::SIGHUP) };
        }
    }

    fn left(&self, leader_running: bool) -> Vec<u32> {
        let Some(leader) = self.pid else {
            return Vec::new();
        };
        // Zombies are not listed (no program path): a job that ended
        // but was not reaped yet does not hold the session up.
        let procs: Vec<(u32, u32)> = crate::platform::processes::all()
            .iter()
            .filter_map(|p| {
                let pid = libc::pid_t::try_from(p.pid).ok()?;
                // SAFETY: only reads the session id; -1 when it is gone.
                let sid = unsafe { libc::getsid(pid) };
                Some((p.pid, u32::try_from(sid).ok()?))
            })
            .collect();
        in_session(&procs, leader, leader_running)
    }

    fn kill(&self, pids: &[u32]) {
        crate::platform::sessions::stop(pids);
    }

    fn release(&self) {
        self.input
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        self.master
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        // Dropping the job kills what is left of the group.
        self.job
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    }
}

/// Starts `launch` on a new PTY with the kit environment only.
pub fn spawn(launch: &Program, env: &KitEnv, dir: &Path, _lang: &str) -> Result<Spawned> {
    let started = start(launch, env, dir)?;
    let mut child = started.child;
    Ok(Spawned {
        io: Box::new(started.io),
        reader: started.reader,
        wait: Box::new(move || match child.wait() {
            Ok(status) if status.signal().is_none() => Some(status.exit_code()),
            _ => None,
        }),
        note: None,
    })
}

/// A file this user may run: `access(X_OK)` also refuses a file on a
/// `noexec` mount.
pub fn runnable(program: &Path) -> bool {
    use std::os::unix::ffi::OsStrExt;
    let Ok(path) = std::ffi::CString::new(program.as_os_str().as_bytes()) else {
        return false;
    };
    // SAFETY: a NUL-terminated path, only read.
    program.is_file() && unsafe { libc::access(path.as_ptr(), libc::X_OK) } == 0
}

/// The variable and value to add so that a session on `os` reads and
/// writes UTF-8 (typed accents, file names such as `projects/già`), from
/// the kit environment's `LC_ALL`, `LC_CTYPE` and `LANG`; `None` when the
/// first of them that is set (the one the C library reads) already names a
/// UTF-8 locale. An app started from Finder or by launchd has none of them,
/// and bash 3.2 or pwsh would then run in the C locale. `LANG` when none is
/// set, as VS Code does (`en_US.UTF-8` on macOS, which has no `C.UTF-8`);
/// `LC_CTYPE` over a `LANG` of another charset, which keeps its messages;
/// never over an `LC_ALL` chosen by the user. Not on Windows, where the
/// console is UTF-16 and the kit's profile sets the code pages.
pub(crate) fn utf8_locale(
    os: crate::kit::Os,
    lc_all: Option<&str>,
    lc_ctype: Option<&str>,
    lang: Option<&str>,
) -> Option<(&'static str, &'static str)> {
    fn set(v: Option<&str>) -> Option<&str> {
        v.filter(|v| !v.is_empty())
    }
    let utf8 = |v: &str| {
        let v = v.to_ascii_lowercase();
        v.contains("utf-8") || v.contains("utf8")
    };
    let value = if os.is_macos() {
        "en_US.UTF-8"
    } else {
        "C.UTF-8"
    };
    match (set(lc_all), set(lc_ctype), set(lang)) {
        (Some(_), _, _) => None,
        (None, Some(v), _) | (None, None, Some(v)) if utf8(v) => None,
        (None, None, None) => Some(("LANG", value)),
        _ => Some(("LC_CTYPE", value)),
    }
}

/// The processes of `procs` (pid, session id) left in the session that
/// `leader` led. Every job of the session keeps its id, the leader's pid,
/// unless it calls `setsid` itself, and the system gives that pid to no new
/// process while one of them lives. So a process with that pid, once the
/// leader was reaped, means the session is gone and the pid now another
/// program's: then none. Never the app itself. Unix only: on Windows the
/// session's Job Object holds them.
pub(crate) fn in_session(procs: &[(u32, u32)], leader: u32, leader_running: bool) -> Vec<u32> {
    if leader == 0 || (!leader_running && procs.iter().any(|&(pid, _)| pid == leader)) {
        return Vec::new();
    }
    let me = std::process::id();
    procs
        .iter()
        .filter(|&&(pid, sid)| sid == leader && pid != me)
        .map(|&(pid, _)| pid)
        .collect()
}

#[cfg(test)]
#[path = "../../../../test/rust/os/unix/pty.rs"]
mod tests;
