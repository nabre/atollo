//! The kit's processes on Windows (`platform::jobs`): one Job Object for
//! the whole app, so that no program started by the kit outlives it, even
//! when the app is killed (DECISIONS.md D-07).
//!
//! The app puts itself in the job at start: every child is then created
//! inside it, with no window between `CreateProcess` and assignment. The job
//! has `KILL_ON_JOB_CLOSE` (the handle is closed by Windows when the app ends,
//! however it ends) and `BREAKAWAY_OK`, used only for programs that must stay
//! open after the app (the system browser, File Explorer).

use std::ffi::OsStr;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectBasicProcessIdList,
    JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
    JOBOBJECT_BASIC_PROCESS_ID_LIST, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_BREAKAWAY_OK, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenThread, ResumeThread, CREATE_BREAKAWAY_FROM_JOB,
    CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, CREATE_SUSPENDED, THREAD_SUSPEND_RESUME,
};

use crate::env::KitEnv;

pub struct Job {
    handle: HANDLE,
}

// SAFETY: a job handle is a kernel handle usable from any thread.
unsafe impl Send for Job {}
unsafe impl Sync for Job {}

impl Job {
    /// A new job that kills its processes when its last handle closes.
    pub fn new() -> Result<Job> {
        Job::with_breakaway(true)
    }

    /// A job none of whose processes can leave it (a terminal session): a
    /// program asking for `CREATE_BREAKAWAY_FROM_JOB` (Codex's background
    /// server) stays in it and ends with the session instead of outliving
    /// the app.
    pub fn contained() -> Result<Job> {
        Job::with_breakaway(false)
    }

    fn with_breakaway(breakaway: bool) -> Result<Job> {
        // SAFETY: no security attributes (handle not inheritable), no name.
        let handle =
            unsafe { CreateJobObjectW(None, PCWSTR::null()) }.context("CreateJobObjectW")?;
        let job = Job { handle };
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = if breakaway {
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK
        } else {
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        };
        // SAFETY: `info` is the structure that JobObjectExtendedLimitInformation expects.
        unsafe {
            SetInformationJobObject(
                job.handle,
                JobObjectExtendedLimitInformation,
                std::ptr::from_ref(&info).cast(),
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        }
        .context("SetInformationJobObject")?;
        Ok(job)
    }

    /// Puts the calling process in the job.
    pub fn assign_current(&self) -> Result<()> {
        // SAFETY: both handles are valid; GetCurrentProcess is a pseudo handle.
        unsafe { AssignProcessToJobObject(self.handle, GetCurrentProcess()) }
            .context("AssignProcessToJobObject(current)")
    }

    /// Puts an already started child in the job.
    pub fn assign(&self, child: &Child) -> Result<()> {
        let process = HANDLE(child.as_raw_handle());
        // SAFETY: the child handle stays valid while `child` lives.
        unsafe { AssignProcessToJobObject(self.handle, process) }
            .context("AssignProcessToJobObject")
    }

    /// Puts a process started by other code in the job (the program of a
    /// terminal session, started by portable-pty): `process` must stay
    /// valid for the call.
    pub fn assign_raw(&self, process: std::os::windows::io::RawHandle) -> Result<()> {
        // SAFETY: the caller keeps the process handle open for the call.
        unsafe { AssignProcessToJobObject(self.handle, HANDLE(process)) }
            .context("AssignProcessToJobObject")
    }

    /// The processes in the job now.
    pub fn pids(&self) -> Vec<u32> {
        // The header (two counts) and room for 1024 ids, as usize words.
        let head = std::mem::offset_of!(JOBOBJECT_BASIC_PROCESS_ID_LIST, ProcessIdList)
            / size_of::<usize>();
        let mut buf = vec![0usize; head + 1024];
        let bytes = u32::try_from(buf.len() * size_of::<usize>()).unwrap_or(u32::MAX);
        // SAFETY: `buf` is a writable buffer of `bytes` bytes, aligned for
        // JOBOBJECT_BASIC_PROCESS_ID_LIST.
        let ok = unsafe {
            QueryInformationJobObject(
                Some(self.handle),
                JobObjectBasicProcessIdList,
                buf.as_mut_ptr().cast(),
                bytes,
                None,
            )
        };
        if ok.is_err() {
            return Vec::new();
        }
        // SAFETY: the buffer now starts with the structure Windows filled.
        let list = unsafe { &*buf.as_ptr().cast::<JOBOBJECT_BASIC_PROCESS_ID_LIST>() };
        let n = (list.NumberOfProcessIdsInList as usize).min(buf.len() - head);
        buf[head..head + n]
            .iter()
            .filter_map(|&pid| u32::try_from(pid).ok())
            .collect()
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by CreateJobObjectW and is closed once.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

/// The app's job, and whether the app itself is inside it.
struct AppJob {
    job: Job,
    self_assigned: bool,
}

static APP_JOB: OnceLock<AppJob> = OnceLock::new();

/// Creates the app's job and moves the app into it. If Windows refuses (the
/// app already runs in a job that forbids nesting), children are assigned one
/// by one instead; the returned text explains why, for the log.
pub fn init() -> Result<Option<String>> {
    let job = Job::new()?;
    let (self_assigned, note) = match job.assign_current() {
        Ok(()) => (true, None),
        Err(e) => (
            false,
            Some(format!("{e:#}; children are assigned one by one")),
        ),
    };
    let _ = APP_JOB.set(AppJob { job, self_assigned });
    Ok(note)
}

/// A command for a kit program: the kit environment only, no inherited
/// variables, no console window.
pub fn kit_command(program: impl AsRef<std::ffi::OsStr>, env: &KitEnv) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    cmd.envs(env.iter());
    cmd.creation_flags((CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP).0);
    cmd
}

/// A `.bat`/`.cmd` run as `cmd.exe /d /s /c "".\<name>" <args>"` in its own
/// folder (DECISIONS.md D-17): the folder never goes through cmd, which
/// would expand `%VAR%` in it even inside quotes; the quoting keeps spaces,
/// `&` and parentheses of the name intact. The current folder is the
/// script's: do not change it. `args` must be fixed words of the app, never
/// text from the user: cmd would interpret `&`, `|` and `%` in them.
pub fn batch_command(script: &std::path::Path, args: &[&str], env: &KitEnv) -> Command {
    let mut cmd = kit_command(cmd_exe(env), env);
    if let Some(dir) = script.parent().filter(|d| !d.as_os_str().is_empty()) {
        cmd.current_dir(dir);
    }
    let name = script.file_name().unwrap_or(script.as_os_str());
    let mut line = format!("\".\\{}\"", std::path::Path::new(name).display());
    for a in args {
        line.push(' ');
        line.push_str(a);
    }
    cmd.raw_arg(format!("/d /s /c \"{line}\""));
    cmd
}

/// The PC's `cmd.exe`, from the kit environment's `SystemRoot`.
fn cmd_exe(env: &KitEnv) -> PathBuf {
    let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
    std::path::Path::new(system_root).join(r"System32\cmd.exe")
}

/// A command line of the user (a project's action) through `cmd.exe /d /s
/// /c "<line>"`, passed as is: cmd reads its own quoting.
pub fn shell_command(line: &str, env: &KitEnv) -> Command {
    let mut cmd = kit_command(cmd_exe(env), env);
    cmd.raw_arg(format!("/d /s /c \"{line}\""));
    cmd
}

/// Starts a program that belongs to the session: it ends with the app.
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    let child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    if let Some(app) = APP_JOB.get().filter(|j| !j.self_assigned) {
        app.job.assign(&child)?;
    }
    Ok(child)
}

/// Puts a process started by other code (suspended) in the app's job when
/// the app could not enter it itself; otherwise the process is already in
/// it, as every child.
pub fn assign_to_app_raw(process: std::os::windows::io::RawHandle) -> Result<()> {
    match APP_JOB.get().filter(|j| !j.self_assigned) {
        Some(app) => app.job.assign_raw(process),
        None => Ok(()),
    }
}

/// Starts a program in `job` (a job of its own, nested in the app's): it is
/// created suspended, assigned, then resumed, so not even its first children
/// escape the job. Dropping the job ends the whole tree.
pub fn spawn_in(cmd: &mut Command, job: &Job) -> Result<Child> {
    cmd.creation_flags((CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP | CREATE_SUSPENDED).0);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    let resume = || -> Result<()> {
        job.assign(&child)?;
        resume_threads(child.id())
    };
    if let Err(e) = resume() {
        let _ = child.kill();
        let _ = child.wait();
        return Err(e);
    }
    Ok(child)
}

/// Resumes the threads of a process created suspended.
fn resume_threads(pid: u32) -> Result<()> {
    // SAFETY: snapshot of all threads; closed below.
    let snap =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) }.context("thread snapshot")?;
    let mut entry = THREADENTRY32 {
        dwSize: size_of::<THREADENTRY32>() as u32,
        ..Default::default()
    };
    let mut resumed = 0;
    // SAFETY: `entry` has its size set; the snapshot handle is valid.
    let mut ok = unsafe { Thread32First(snap, &mut entry) }.is_ok();
    while ok {
        if entry.th32OwnerProcessID == pid {
            // SAFETY: thread id from the snapshot; handle closed right after.
            if let Ok(thread) =
                unsafe { OpenThread(THREAD_SUSPEND_RESUME, false, entry.th32ThreadID) }
            {
                // SAFETY: valid thread handle.
                unsafe { ResumeThread(thread) };
                let _ = unsafe { CloseHandle(thread) };
                resumed += 1;
            }
        }
        // SAFETY: as above.
        ok = unsafe { Thread32Next(snap, &mut entry) }.is_ok();
    }
    // SAFETY: snapshot handle from CreateToolhelp32Snapshot.
    let _ = unsafe { CloseHandle(snap) };
    anyhow::ensure!(resumed > 0, "no thread to resume in process {pid}");
    Ok(())
}

/// Starts a program that must stay open after the app (browser, Explorer).
pub fn spawn_detached(cmd: &mut Command, flags: u32) -> Result<Child> {
    cmd.creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB.0);
    cmd.spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))
}

/// The PC's Windows folder, from this process (`C:\Windows` when unset).
pub fn system_root() -> PathBuf {
    PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()))
}

/// Opens a folder in File Explorer, outside the job: it stays open after
/// the app. Web addresses go through `open_url`.
pub fn open_with_explorer(target: impl AsRef<OsStr>) -> Result<()> {
    spawn_detached(
        Command::new(system_root().join("explorer.exe")).arg(target),
        0,
    )
    .map(|_| ())
}

/// Opens a web address in the PC's browser, outside the job. Not through
/// `explorer.exe`: Windows 11 drops an address with a query (`?atollo=…`)
/// without opening the browser.
pub fn open_url(url: &str) -> Result<()> {
    spawn_detached(
        Command::new(system_root().join(r"System32\rundll32.exe"))
            .args(["url.dll,FileProtocolHandler", url]),
        0,
    )
    .map(|_| ())
}

/// No guard on Windows: the job does its work.
pub fn guard_pid() -> Option<u32> {
    None
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/jobs.rs"]
mod tests;
