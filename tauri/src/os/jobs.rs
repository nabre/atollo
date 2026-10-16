//! One Windows Job Object for the whole app, so that no program started by
//! the kit outlives it, even when the app is killed (DECISIONS.md D-07).
//!
//! The app puts itself in the job at start: every child is then created
//! inside it, with no window between `CreateProcess` and assignment. The job
//! has `KILL_ON_JOB_CLOSE` (the handle is closed by Windows when the app ends,
//! however it ends) and `BREAKAWAY_OK`, used only for programs that must stay
//! open after the app (the system browser, File Explorer).
//!
//! macOS and Linux have no Job Object: each program of a `Job` leads a
//! process group of its own, killed (`killpg`) when the job is dropped, and
//! what the app started is stopped at exit by `sessions::stop_children`.

use std::ffi::OsStr;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::path::PathBuf;
use std::process::{Child, Command};
#[cfg(windows)]
use std::sync::OnceLock;

use anyhow::{Context, Result};
#[cfg(windows)]
use windows::core::{PCWSTR, PWSTR};
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE};
#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
#[cfg(windows)]
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
#[cfg(windows)]
use windows::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, OpenThread, ResumeThread, TerminateProcess,
    WaitForSingleObject, CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW,
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, INFINITE, PROCESS_INFORMATION, STARTUPINFOW,
    THREAD_SUSPEND_RESUME,
};

use crate::env::KitEnv;

/// The process groups of the programs started in the job (Unix).
#[cfg(unix)]
pub struct Job {
    groups: std::sync::Mutex<Vec<i32>>,
}

#[cfg(unix)]
impl Job {
    pub fn new() -> Result<Job> {
        Ok(Job {
            groups: std::sync::Mutex::new(Vec::new()),
        })
    }

    /// Adds the group led by `child` (started with `process_group(0)`).
    pub fn assign(&self, child: &Child) -> Result<()> {
        let pid = i32::try_from(child.id()).context("pid")?;
        self.groups
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(pid);
        Ok(())
    }
}

#[cfg(unix)]
impl Drop for Job {
    fn drop(&mut self) {
        let groups = self.groups.get_mut().unwrap_or_else(|e| e.into_inner());
        for &pgid in groups.iter() {
            // SAFETY: a plain signal to a process group this job created.
            unsafe { libc::killpg(pgid, libc::SIGKILL) };
        }
    }
}

/// No Job Object on this system: nothing to join. Children are stopped at
/// exit by `sessions::stop_children`; the returned text says so, for the log.
#[cfg(unix)]
pub fn init() -> Result<Option<String>> {
    Ok(Some(
        "no job object on this system; children are stopped at exit".into(),
    ))
}

/// A command for a kit program: the kit environment only, no inherited
/// variables, in a process group of its own (no terminal signals).
#[cfg(unix)]
pub fn kit_command(program: impl AsRef<std::ffi::OsStr>, env: &KitEnv) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    cmd.envs(env.iter());
    cmd.process_group(0);
    cmd
}

/// A shell script run as `/bin/sh ./<name> <args>` in its own folder (the
/// Unix side of the `.bat` of D-17). `args` must be fixed words of the app.
#[cfg(unix)]
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
#[cfg(unix)]
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    cmd.spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))
}

/// A process started by [`spawn_console`].
#[cfg(unix)]
pub struct Console {
    child: std::sync::Mutex<Child>,
    pid: u32,
}

#[cfg(unix)]
impl Console {
    pub fn id(&self) -> u32 {
        self.pid
    }

    /// Waits until the process ends.
    pub fn wait(&self) {
        let _ = self.child.lock().unwrap_or_else(|e| e.into_inner()).wait();
    }
}

/// Starts `program` with `args`, the kit environment only, in `dir`, in a
/// process group of its own; the program opens its own window.
#[cfg(unix)]
pub fn spawn_console(
    program: &std::path::Path,
    args: &[String],
    env: &KitEnv,
    dir: &std::path::Path,
) -> Result<Console> {
    let child = kit_command(program, env)
        .args(args)
        .current_dir(dir)
        .stdin(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("starting {}", program.display()))?;
    Ok(Console {
        pid: child.id(),
        child: std::sync::Mutex::new(child),
    })
}

/// Starts a program in `job`, as the leader of a new process group: dropping
/// the job kills the group, its children included.
#[cfg(unix)]
pub fn spawn_in(cmd: &mut Command, job: &Job) -> Result<Child> {
    cmd.process_group(0);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    if let Err(e) = job.assign(&child) {
        let _ = child.kill();
        return Err(e);
    }
    Ok(child)
}

/// Starts a program that must stay open after the app (browser, file
/// manager): its own process group, never stopped by the app. `flags` are
/// Windows creation flags, ignored here.
#[cfg(unix)]
pub fn spawn_detached(cmd: &mut Command, _flags: u32) -> Result<Child> {
    cmd.process_group(0);
    cmd.spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))
}

/// Opens a web address or a folder with the system's handler (`open` on
/// macOS, `xdg-open` elsewhere), outside the app's processes.
#[cfg(unix)]
pub fn open_with_explorer(target: impl AsRef<OsStr>) -> Result<()> {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else {
        "xdg-open"
    };
    spawn_detached(Command::new(opener).arg(target), 0).map(|_| ())
}

#[cfg(windows)]
pub struct Job {
    handle: HANDLE,
}

// SAFETY: a job handle is a kernel handle usable from any thread.
#[cfg(windows)]
unsafe impl Send for Job {}
#[cfg(windows)]
unsafe impl Sync for Job {}

#[cfg(windows)]
impl Job {
    /// A new job that kills its processes when its last handle closes.
    pub fn new() -> Result<Job> {
        // SAFETY: no security attributes (handle not inheritable), no name.
        let handle =
            unsafe { CreateJobObjectW(None, PCWSTR::null()) }.context("CreateJobObjectW")?;
        let job = Job { handle };
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags =
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;
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
}

#[cfg(windows)]
impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by CreateJobObjectW and is closed once.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

/// The app's job, and whether the app itself is inside it.
#[cfg(windows)]
struct AppJob {
    job: Job,
    self_assigned: bool,
}

#[cfg(windows)]
static APP_JOB: OnceLock<AppJob> = OnceLock::new();

/// Creates the app's job and moves the app into it. If Windows refuses (the
/// app already runs in a job that forbids nesting), children are assigned one
/// by one instead; the returned text explains why, for the log.
#[cfg(windows)]
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
#[cfg(windows)]
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
#[cfg(windows)]
pub fn batch_command(script: &std::path::Path, args: &[&str], env: &KitEnv) -> Command {
    let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
    let mut cmd = kit_command(
        std::path::Path::new(system_root).join(r"System32\cmd.exe"),
        env,
    );
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

/// Starts a program that belongs to the session: it ends with the app.
#[cfg(windows)]
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    let child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    if let Some(app) = APP_JOB.get().filter(|j| !j.self_assigned) {
        app.job.assign(&child)?;
    }
    Ok(child)
}

/// A process started by [`spawn_console`]; the handle closes on drop.
#[cfg(windows)]
pub struct Console {
    process: HANDLE,
    pid: u32,
}

// SAFETY: a process handle is a kernel handle usable from any thread.
#[cfg(windows)]
unsafe impl Send for Console {}

#[cfg(windows)]
impl Console {
    pub fn id(&self) -> u32 {
        self.pid
    }

    /// Waits until the process ends.
    pub fn wait(&self) {
        // SAFETY: the handle is valid until drop.
        let _ = unsafe { WaitForSingleObject(self.process, INFINITE) };
    }
}

#[cfg(windows)]
impl Drop for Console {
    fn drop(&mut self) {
        // SAFETY: returned by CreateProcessW, closed once.
        let _ = unsafe { CloseHandle(self.process) };
    }
}

/// One argument of a Windows command line. No argument here contains a
/// double quote (paths cannot): only spaces need quotes.
#[cfg(windows)]
fn quote(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t']) {
        return arg.to_owned();
    }
    // Backslashes before the closing quote are doubled.
    let tail = arg.len() - arg.trim_end_matches('\\').len();
    format!("\"{arg}{}\"", "\\".repeat(tail))
}

/// Starts `program` with `args`, the kit environment only, in `dir`, inside
/// the app's job, without standard handles. A console host (`conhost.exe`)
/// given standard handles writes the console to them as VT text instead of
/// showing its window, and `std::process::Command` always passes them.
#[cfg(windows)]
pub fn spawn_console(
    program: &std::path::Path,
    args: &[String],
    env: &KitEnv,
    dir: &std::path::Path,
) -> Result<Console> {
    anyhow::ensure!(
        args.iter().all(|a| !a.contains('"')),
        "double quote in an argument"
    );
    let wide = |s: &std::ffi::OsStr| -> Vec<u16> { s.encode_wide().chain([0]).collect() };
    let mut line: Vec<u16> = std::iter::once(quote(&program.display().to_string()))
        .chain(args.iter().map(|a| quote(a)))
        .collect::<Vec<_>>()
        .join(" ")
        .encode_utf16()
        .chain([0])
        .collect();
    let mut vars: Vec<(&str, &str)> = env.iter().collect();
    vars.sort_by_key(|(k, _)| k.to_uppercase());
    let mut block: Vec<u16> = vars
        .iter()
        .flat_map(|(k, v)| {
            format!("{k}={v}")
                .encode_utf16()
                .chain([0])
                .collect::<Vec<_>>()
        })
        .collect();
    if block.is_empty() {
        block.push(0);
    }
    block.push(0);
    let (program, dir) = (wide(program.as_os_str()), wide(dir.as_os_str()));
    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: every buffer is NUL-terminated and outlives the call; the
    // command line is writable, as CreateProcessW requires.
    unsafe {
        CreateProcessW(
            PCWSTR(program.as_ptr()),
            Some(PWSTR(line.as_mut_ptr())),
            None,
            None,
            false,
            CREATE_UNICODE_ENVIRONMENT | CREATE_SUSPENDED,
            Some(block.as_ptr().cast()),
            PCWSTR(dir.as_ptr()),
            &startup,
            &mut info,
        )
    }
    .context("CreateProcessW")?;
    let console = Console {
        process: info.hProcess,
        pid: info.dwProcessId,
    };
    let assigned = match APP_JOB.get().filter(|j| !j.self_assigned) {
        // SAFETY: the process handle is valid (owned by `console`).
        Some(app) => unsafe { AssignProcessToJobObject(app.job.handle, info.hProcess) }
            .context("AssignProcessToJobObject"),
        None => Ok(()),
    };
    // SAFETY: the main thread handle of the new process, closed once here.
    unsafe {
        if assigned.is_ok() {
            ResumeThread(info.hThread);
        } else {
            let _ = TerminateProcess(info.hProcess, 1);
        }
        let _ = CloseHandle(info.hThread);
    }
    assigned.map(|()| console)
}

/// Starts a program in `job` (a job of its own, nested in the app's): it is
/// created suspended, assigned, then resumed, so not even its first children
/// escape the job. Dropping the job ends the whole tree.
#[cfg(windows)]
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
        return Err(e);
    }
    Ok(child)
}

/// Resumes the threads of a process created suspended.
#[cfg(windows)]
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
#[cfg(windows)]
pub fn spawn_detached(cmd: &mut Command, flags: u32) -> Result<Child> {
    cmd.creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB.0);
    cmd.spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))
}

/// The PC's Windows folder, from this process (`C:\Windows` when unset).
#[cfg(windows)]
pub fn system_root() -> PathBuf {
    PathBuf::from(std::env::var_os("SystemRoot").unwrap_or_else(|| r"C:\Windows".into()))
}

/// Opens a web address in the PC's browser, or a folder in File Explorer,
/// through Explorer and outside the job: both stay open after the app.
#[cfg(windows)]
pub fn open_with_explorer(target: impl AsRef<OsStr>) -> Result<()> {
    spawn_detached(
        Command::new(system_root().join("explorer.exe")).arg(target),
        0,
    )
    .map(|_| ())
}

#[cfg(all(test, windows))]
#[path = "../../../test/rust/os/jobs.rs"]
mod tests;
