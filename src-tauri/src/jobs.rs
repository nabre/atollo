//! One Windows Job Object for the whole app, so that no program started by
//! the kit outlives it, even when the app is killed (DECISIONS.md D-07).
//!
//! The app puts itself in the job at start: every child is then created
//! inside it, with no window between `CreateProcess` and assignment. The job
//! has `KILL_ON_JOB_CLOSE` (the handle is closed by Windows when the app ends,
//! however it ends) and `BREAKAWAY_OK`, used only for programs that must stay
//! open after the app (the system browser, File Explorer).

use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::AsRawHandle;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use std::sync::OnceLock;

use anyhow::{Context, Result};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows::Win32::System::Threading::{
    CreateProcessW, GetCurrentProcess, OpenProcess, OpenThread, ResumeThread, TerminateProcess,
    WaitForSingleObject, CREATE_BREAKAWAY_FROM_JOB, CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW,
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, INFINITE, PROCESS_INFORMATION,
    PROCESS_SYNCHRONIZE, STARTUPINFOW, THREAD_SUSPEND_RESUME,
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

impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: the handle was returned by CreateJobObjectW and is closed once.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

/// The app's job, and whether the app itself is inside it.
#[allow(dead_code, reason = "read by spawn(), used from phase 2")]
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
#[allow(dead_code, reason = "first caller in phase 2 (installer)")]
pub fn kit_command(program: impl AsRef<std::ffi::OsStr>, env: &KitEnv) -> Command {
    let mut cmd = Command::new(program);
    cmd.env_clear();
    cmd.envs(env.iter());
    cmd.creation_flags((CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP).0);
    cmd
}

/// A `.bat`/`.cmd` run as `cmd.exe /d /s /c ""<path>" <args>"`: the quoting
/// that keeps paths with spaces, `&` and parentheses intact (DECISIONS.md
/// D-17). `args` must be fixed words of the app, never text from the user:
/// cmd would interpret `&`, `|` and `%` in them.
pub fn batch_command(script: &std::path::Path, args: &[&str], env: &KitEnv) -> Command {
    let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
    let mut cmd = kit_command(
        std::path::Path::new(system_root).join(r"System32\cmd.exe"),
        env,
    );
    let mut line = format!("\"{}\"", script.display());
    for a in args {
        line.push(' ');
        line.push_str(a);
    }
    cmd.raw_arg(format!("/d /s /c \"{line}\""));
    cmd
}

/// Starts a program that belongs to the session: it ends with the app.
#[allow(dead_code, reason = "first caller in phase 2 (installer)")]
pub fn spawn(cmd: &mut Command) -> Result<Child> {
    let child = cmd
        .spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))?;
    if let Some(app) = APP_JOB.get().filter(|j| !j.self_assigned) {
        app.job.assign(&child)?;
    }
    Ok(child)
}

/// A process started by [`spawn_console`] or found by [`Console::open`]; the
/// handle closes on drop.
pub struct Console {
    process: HANDLE,
    pid: u32,
}

// SAFETY: a process handle is a kernel handle usable from any thread.
unsafe impl Send for Console {}

impl Console {
    /// A process started by another one (a Windows Terminal tab), to wait for.
    pub fn open(pid: u32) -> Result<Console> {
        // SAFETY: wait-only handle, owned by the result and closed on drop.
        let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }
            .with_context(|| format!("OpenProcess {pid}"))?;
        Ok(Console { process, pid })
    }

    pub fn id(&self) -> u32 {
        self.pid
    }

    /// Waits until the process ends.
    pub fn wait(&self) {
        // SAFETY: the handle is valid until drop.
        let _ = unsafe { WaitForSingleObject(self.process, INFINITE) };
    }
}

impl Drop for Console {
    fn drop(&mut self) {
        // SAFETY: returned by CreateProcessW or OpenProcess, closed once.
        let _ = unsafe { CloseHandle(self.process) };
    }
}

/// One argument of a Windows command line. No argument here contains a
/// double quote (paths cannot): only spaces need quotes.
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
#[allow(dead_code, reason = "first caller in phase 6 (browser, Explorer)")]
pub fn spawn_detached(cmd: &mut Command, flags: u32) -> Result<Child> {
    cmd.creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB.0);
    cmd.spawn()
        .with_context(|| format!("starting {:?}", cmd.get_program()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn system32(exe: &str) -> String {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        format!(r"{root}\System32\{exe}")
    }

    /// The test runner itself must not join a kill-on-close job, so the test
    /// assigns a child to a private job and closes it.
    #[test]
    fn spawn_in_runs_and_drop_kills() {
        let job = Job::new().unwrap();
        let mut child = spawn_in(
            Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
            &job,
        )
        .unwrap();
        std::thread::sleep(Duration::from_millis(300));
        assert!(child.try_wait().unwrap().is_none(), "running after resume");
        drop(job);
        let start = Instant::now();
        while child.try_wait().unwrap().is_none() {
            assert!(start.elapsed() < Duration::from_secs(5), "still alive");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn batch_files_in_odd_folders() {
        let dir = std::env::temp_dir().join(format!("devkit-bat-{} (x) & y", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let bat = dir.join("hello.cmd");
        std::fs::write(&bat, "@echo hi %1\r\n").unwrap();
        let mut env_vars = vec![(
            "SystemRoot".to_owned(),
            std::env::var("SystemRoot").unwrap(),
        )];
        env_vars.push(("PATH".to_owned(), String::new()));
        let out = batch_command(&bat, &["there"], &KitEnv::default())
            .envs(env_vars)
            .output()
            .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "hi there");
    }

    #[test]
    fn closing_the_job_kills_its_processes() {
        let job = Job::new().unwrap();
        let mut child = Command::new(system32("ping.exe"))
            .args(["-n", "60", "127.0.0.1"])
            .creation_flags(CREATE_NO_WINDOW.0)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        job.assign(&child).unwrap();
        assert!(child.try_wait().unwrap().is_none(), "still running");
        drop(job);
        let start = Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "child survived the job"
            );
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn kit_command_does_not_inherit_the_environment() {
        let env = KitEnv::default();
        let cmd = kit_command(system32("cmd.exe"), &env);
        let inherited: Vec<_> = cmd.get_envs().collect();
        assert!(inherited.is_empty());
        let out = kit_command(system32("cmd.exe"), &env)
            .args(["/d", "/c", "set"])
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout).to_ascii_uppercase();
        assert!(!text.contains("USERPROFILE="), "{text}");
    }
}
