//! The pseudo console of the app's terminal on Windows (DECISIONS.md D-148).
//!
//! The kit's ConPTY, the component `conpty` (`data\apps\<target>\conpty`:
//! `conpty.dll` and `OpenConsole.exe` of Microsoft's NuGet package, the
//! host of Windows Terminal and VS Code), is loaded by its full path, and
//! `conpty.dll` starts `OpenConsole.exe` from its own folder. Windows' own
//! ConPTY (kernel32) only while the component is missing, and the session
//! says so: with it, Claude Code draws but gets no keys (D-47). Never a
//! `conpty.dll` found by name in the PC's folders: no `LoadLibrary` of a
//! bare name happens here.
//!
//! - the program is created suspended, put in the app's job and in a job of
//!   its session, then resumed: none of its children escapes the session,
//!   not even one asking to break away (Codex's background server);
//! - the pseudo console is closed when the program ends (the reader then
//!   ends) or when the session is closed: its processes get
//!   `CTRL_CLOSE_EVENT`, as when a console window closes; what is left is
//!   killed with the session's job;
//! - the keys go to the pseudo console's input in order, written by a
//!   thread of the session, so a program that does not read never blocks
//!   the window.

use std::ffi::c_void;
use std::fs::File;
use std::io::Write;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Mutex, MutexGuard, PoisonError};

use anyhow::{Context, Result};
use windows::core::{s, w, HRESULT, PCWSTR, PWSTR};
use windows::Win32::Foundation::{HANDLE, HMODULE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Console::{SetConsoleCtrlHandler, COORD, HPCON};
use windows::Win32::System::LibraryLoader::{
    GetModuleHandleW, GetProcAddress, LoadLibraryExW, LOAD_WITH_ALTERED_SEARCH_PATH,
};
use windows::Win32::System::Pipes::CreatePipe;
use windows::Win32::System::Threading::{
    DeleteProcThreadAttributeList, GetExitCodeProcess, InitializeProcThreadAttributeList,
    ResumeThread, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
    CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE,
    LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
    STARTF_USESTDHANDLES, STARTUPINFOEXW,
};

use crate::env::KitEnv;
use crate::os::windows::jobs::{self, Job};
use crate::platform::pty::Io;

/// The kit's ConPTY: both files in the component's folder.
pub const FILES: [&str; 2] = ["conpty.dll", "OpenConsole.exe"];

/// `PSEUDOCONSOLE_RESIZE_QUIRK`: the terminal (xterm.js) reflows its own
/// lines on a resize, the console host does not repaint them.
const RESIZE_QUIRK: u32 = 0x2;
/// `PSEUDOCONSOLE_GLYPH_WIDTH_WCSWIDTH` (`conpty.h` of the package): the
/// kit's ConPTY counts the cells of emoji and wide characters as xterm.js
/// with its Unicode 11 widths does. Windows' own ConPTY does not know it.
const GLYPH_WIDTH_WCSWIDTH: u32 = 0x10;

type CreateFn = unsafe extern "system" fn(COORD, HANDLE, HANDLE, u32, *mut HPCON) -> HRESULT;
type ResizeFn = unsafe extern "system" fn(HPCON, COORD) -> HRESULT;
type CloseFn = unsafe extern "system" fn(HPCON);

/// The three functions of a ConPTY.
#[derive(Clone, Copy)]
struct Api {
    create: CreateFn,
    resize: ResizeFn,
    close: CloseFn,
    /// The kit's (`conpty.dll`), not Windows' own.
    kit: bool,
}

/// The kit's ConPTY once loaded: never unloaded, every later session uses
/// it. While it is not loaded, every session looks for it again (the
/// component may be installed meanwhile).
static KIT: Mutex<Option<Api>> = Mutex::new(None);

/// The functions exported by `module`.
fn functions(module: HMODULE, kit: bool) -> Option<Api> {
    // SAFETY: the names are NUL-terminated; the module stays loaded for the
    // whole process (kernel32, or the kit's DLL, never freed); the
    // signatures are those of the Windows SDK.
    unsafe {
        let create = GetProcAddress(module, s!("CreatePseudoConsole"))?;
        let resize = GetProcAddress(module, s!("ResizePseudoConsole"))?;
        let close = GetProcAddress(module, s!("ClosePseudoConsole"))?;
        Some(Api {
            create: std::mem::transmute::<unsafe extern "system" fn() -> isize, CreateFn>(create),
            resize: std::mem::transmute::<unsafe extern "system" fn() -> isize, ResizeFn>(resize),
            close: std::mem::transmute::<unsafe extern "system" fn() -> isize, CloseFn>(close),
            kit,
        })
    }
}

/// Both files of the kit's ConPTY are in `dir`.
pub fn installed_in(dir: &Path) -> bool {
    FILES.iter().all(|f| dir.join(f).is_file())
}

fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    s.encode_wide().chain([0]).collect()
}

/// The kit's ConPTY in `dir` when both of its files are there, else
/// Windows' own.
fn api(dir: Option<&Path>) -> Result<Api> {
    let mut kit = KIT.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(api) = *kit {
        return Ok(api);
    }
    if let Some(dir) = dir.filter(|d| installed_in(d)) {
        let dll = dir.join(FILES[0]);
        let path = wide(dll.as_os_str());
        // By its full path: the DLLs it needs are looked for from its
        // folder first, then in System32.
        // SAFETY: a NUL-terminated path; the module is never freed.
        match unsafe { LoadLibraryExW(PCWSTR(path.as_ptr()), None, LOAD_WITH_ALTERED_SEARCH_PATH) }
        {
            Ok(module) => match functions(module, true) {
                Some(api) => {
                    crate::common::log::info(&format!(
                        "terminal: ConPTY of the kit, {}",
                        dll.display()
                    ));
                    *kit = Some(api);
                    return Ok(api);
                }
                None => crate::common::log::warn(&format!(
                    "{}: no pseudo console functions",
                    dll.display()
                )),
            },
            Err(e) => crate::common::log::warn(&format!("{}: {e}", dll.display())),
        }
    }
    // SAFETY: kernel32 is loaded in every process.
    let kernel = unsafe { GetModuleHandleW(w!("kernel32.dll")) }.context("kernel32")?;
    functions(kernel, false)
        .context("this Windows has no pseudo console (Windows 10 1809 or later is needed)")
}

/// One argument of a Windows command line, quoted as the C runtime and
/// `CommandLineToArgvW` read it back.
pub fn quote_arg(arg: &str) -> String {
    if !arg.is_empty() && !arg.contains([' ', '\t', '\n', '\u{b}', '"']) {
        return arg.to_owned();
    }
    let mut out = String::from('"');
    let mut slashes = 0usize;
    for c in arg.chars() {
        match c {
            '\\' => slashes += 1,
            '"' => {
                out.extend(std::iter::repeat_n('\\', slashes * 2 + 1));
                out.push('"');
                slashes = 0;
            }
            _ => {
                out.extend(std::iter::repeat_n('\\', slashes));
                out.push(c);
                slashes = 0;
            }
        }
    }
    out.extend(std::iter::repeat_n('\\', slashes * 2));
    out.push('"');
    out
}

/// The command line of `program` with `args`.
pub fn command_line(program: &Path, args: &[String]) -> String {
    std::iter::once(quote_arg(&program.display().to_string()))
        .chain(args.iter().map(|a| quote_arg(a)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// `env` as the environment block of `CreateProcessW`: sorted without
/// regard to case, as Windows expects.
pub fn env_block(env: &KitEnv) -> Vec<u16> {
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
    block
}

/// An open pseudo console; closed on drop.
struct Pcon {
    hpc: HPCON,
    resize: ResizeFn,
    close: CloseFn,
}

// SAFETY: a pseudo console handle can be used and closed from any thread.
unsafe impl Send for Pcon {}

impl Drop for Pcon {
    fn drop(&mut self) {
        // SAFETY: created by `create` of the same ConPTY, closed once.
        unsafe { (self.close)(self.hpc) };
    }
}

/// The program of a session; its handle closes on drop.
pub struct Process {
    handle: OwnedHandle,
}

impl Process {
    /// Waits for the program to end; its exit code.
    pub fn wait(&self) -> Option<u32> {
        let h = HANDLE(self.handle.as_raw_handle());
        // SAFETY: the handle is valid while `self` lives.
        unsafe { WaitForSingleObject(h, INFINITE) };
        let mut code = 0u32;
        // SAFETY: as above; `code` is writable.
        unsafe { GetExitCodeProcess(h, &mut code) }.ok()?;
        Some(code)
    }
}

/// A session's pseudo console and program, as the session uses them.
pub struct ConIo {
    pid: u32,
    pcon: Mutex<Option<Pcon>>,
    input: Mutex<Option<Sender<Vec<u8>>>>,
    job: Mutex<Option<Job>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl ConIo {
    /// Closes the pseudo console: its processes get `CTRL_CLOSE_EVENT`,
    /// the console host ends and the reader with it.
    fn close_console(&self) {
        let pcon = lock(&self.pcon).take();
        drop(pcon);
    }
}

impl Io for ConIo {
    fn write(&self, data: &[u8]) -> Result<()> {
        match lock(&self.input).as_ref() {
            Some(tx) => tx.send(data.to_vec()).context("terminal closed"),
            None => anyhow::bail!("terminal closed"),
        }
    }

    fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        let pcon = lock(&self.pcon);
        let Some(p) = pcon.as_ref() else {
            anyhow::bail!("terminal closed");
        };
        let size = COORD {
            X: i16::try_from(cols).unwrap_or(i16::MAX),
            Y: i16::try_from(rows).unwrap_or(i16::MAX),
        };
        // SAFETY: an open pseudo console of the same ConPTY.
        unsafe { (p.resize)(p.hpc, size) }
            .ok()
            .context("ResizePseudoConsole")
    }

    fn pid(&self) -> Option<u32> {
        Some(self.pid)
    }

    fn hang_up(&self) {
        self.close_console();
    }

    fn left(&self, _leader_running: bool) -> Vec<u32> {
        lock(&self.job).as_ref().map(Job::pids).unwrap_or_default()
    }

    fn kill(&self, pids: &[u32]) {
        crate::platform::sessions::stop(pids);
    }

    fn release(&self) {
        lock(&self.input).take();
        self.close_console();
        // Closing the job kills what is left of the session.
        lock(&self.job).take();
    }

    fn program_ended(&self) {
        self.close_console();
    }
}

pub struct Spawned {
    pub io: ConIo,
    pub reader: File,
    pub process: Process,
    /// Windows' own ConPTY: the kit's is not installed.
    pub system: bool,
}

/// The size before the view measures itself (`pty_resize`).
const SIZE: COORD = COORD { X: 80, Y: 24 };

/// A new pipe: (read end, write end).
fn pipe() -> Result<(OwnedHandle, OwnedHandle)> {
    let (mut read, mut write) = (HANDLE::default(), HANDLE::default());
    // SAFETY: two writable handles; not inheritable.
    unsafe { CreatePipe(&mut read, &mut write, None, 0) }.context("CreatePipe")?;
    // SAFETY: both handles were just created and are owned from now on.
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read.0),
            OwnedHandle::from_raw_handle(write.0),
        )
    })
}

/// A process attribute list holding the pseudo console; deleted on drop.
struct Attributes {
    buf: Vec<u64>,
}

impl Attributes {
    fn with_console(hpc: HPCON) -> Result<Attributes> {
        let mut size = 0usize;
        // SAFETY: asks only for the size (fails with "insufficient buffer").
        let _ = unsafe { InitializeProcThreadAttributeList(None, 1, None, &mut size) };
        anyhow::ensure!(size > 0, "InitializeProcThreadAttributeList: no size");
        let mut a = Attributes {
            buf: vec![0u64; size.div_ceil(8)],
        };
        // SAFETY: a buffer of at least `size` bytes, 8-aligned.
        unsafe { InitializeProcThreadAttributeList(Some(a.list()), 1, None, &mut size) }
            .context("InitializeProcThreadAttributeList")?;
        // SAFETY: the list is initialized; the attribute's value is the
        // pseudo console handle itself, as the documentation says.
        let set = unsafe {
            UpdateProcThreadAttribute(
                a.list(),
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                Some(hpc.0 as *const c_void),
                size_of::<HPCON>(),
                None,
                None,
            )
        };
        if let Err(e) = set {
            // SAFETY: initialized above, deleted once (the drop is skipped).
            unsafe { DeleteProcThreadAttributeList(a.list()) };
            std::mem::forget(a);
            return Err(e).context("UpdateProcThreadAttribute");
        }
        Ok(a)
    }

    fn list(&mut self) -> LPPROC_THREAD_ATTRIBUTE_LIST {
        LPPROC_THREAD_ATTRIBUTE_LIST(self.buf.as_mut_ptr().cast())
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: initialized in `with_console`, deleted once.
        unsafe { DeleteProcThreadAttributeList(self.list()) };
    }
}

/// Starts `program` with `args`, the environment `env` only, in `dir`, on a
/// new pseudo console: the kit's ConPTY of `conpty` when installed there.
pub fn spawn(
    program: &Path,
    args: &[String],
    env: &KitEnv,
    dir: &Path,
    conpty: Option<&Path>,
) -> Result<Spawned> {
    let api = api(conpty)?;
    let (in_read, in_write) = pipe()?;
    let (out_read, out_write) = pipe()?;
    let mut hpc = HPCON::default();
    let flags = if api.kit {
        RESIZE_QUIRK | GLYPH_WIDTH_WCSWIDTH
    } else {
        RESIZE_QUIRK
    };
    // SAFETY: two valid pipe ends (the console host duplicates them) and a
    // writable handle.
    unsafe {
        (api.create)(
            SIZE,
            HANDLE(in_read.as_raw_handle()),
            HANDLE(out_write.as_raw_handle()),
            flags,
            &mut hpc,
        )
    }
    .ok()
    .context("CreatePseudoConsole")?;
    let pcon = Pcon {
        hpc,
        resize: api.resize,
        close: api.close,
    };
    // The console host has its own copies: the reader ends when it does.
    drop((in_read, out_write));

    let mut attributes = Attributes::with_console(pcon.hpc)?;
    let mut startup = STARTUPINFOEXW::default();
    startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
    // No standard handle of the app's (a log file, the console of a
    // developer) reaches the program: only the pseudo console.
    startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
    startup.StartupInfo.hStdInput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdOutput = INVALID_HANDLE_VALUE;
    startup.StartupInfo.hStdError = INVALID_HANDLE_VALUE;
    startup.lpAttributeList = attributes.list();

    let mut line: Vec<u16> = command_line(program, args)
        .encode_utf16()
        .chain([0])
        .collect();
    let block = env_block(env);
    let (app, cwd) = (wide(program.as_os_str()), wide(dir.as_os_str()));
    // "Ignore Ctrl+C" passes from a process to its children: the app
    // started in a new process group (from a script, by another program)
    // would hand it to the shell, and Ctrl+C would not stop a command.
    // SAFETY: no handler routine; it only clears the flag of this process.
    let _ = unsafe { SetConsoleCtrlHandler(None, false) };
    let mut info = PROCESS_INFORMATION::default();
    // SAFETY: every buffer is NUL-terminated and outlives the call; the
    // command line is writable; the attribute list holds the console.
    unsafe {
        windows::Win32::System::Threading::CreateProcessW(
            PCWSTR(app.as_ptr()),
            Some(PWSTR(line.as_mut_ptr())),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_UNICODE_ENVIRONMENT | CREATE_SUSPENDED,
            Some(block.as_ptr().cast()),
            PCWSTR(cwd.as_ptr()),
            &startup.StartupInfo,
            &mut info,
        )
    }
    .with_context(|| format!("starting {}", program.display()))?;
    drop(attributes);
    // SAFETY: both handles were just returned and are owned from now on.
    let (process, thread) = unsafe {
        (
            OwnedHandle::from_raw_handle(info.hProcess.0),
            OwnedHandle::from_raw_handle(info.hThread.0),
        )
    };
    // In the app's job (when the app could not enter it itself) and in the
    // session's, before its first instruction: never a session outside them.
    let joined = (|| -> Result<Job> {
        jobs::assign_to_app_raw(process.as_raw_handle())?;
        let job = Job::contained()?;
        job.assign_raw(process.as_raw_handle())?;
        Ok(job)
    })();
    let job = match joined {
        Ok(job) => job,
        Err(e) => {
            // SAFETY: the process handle is valid; it never ran.
            let _ = unsafe { TerminateProcess(HANDLE(process.as_raw_handle()), 1) };
            return Err(e);
        }
    };
    // SAFETY: the main thread of the suspended process; closed on drop.
    unsafe { ResumeThread(HANDLE(thread.as_raw_handle())) };
    drop(thread);

    let mut writer = File::from(in_write);
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
    Ok(Spawned {
        io: ConIo {
            pid: info.dwProcessId,
            pcon: Mutex::new(Some(pcon)),
            input: Mutex::new(Some(tx)),
            job: Mutex::new(Some(job)),
        },
        reader: File::from(out_read),
        process: Process { handle: process },
        system: !api.kit,
    })
}

#[cfg(test)]
#[path = "../../../../test/rust/os/windows/conpty.rs"]
mod tests;
