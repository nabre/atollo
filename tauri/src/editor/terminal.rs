//! Consoles outside the editor: the kit's PowerShell or an agent.
//!
//! On Windows, a classic Windows console (DECISIONS.md D-57, D-95). Never a
//! shell of the PC, and never the PC's terminal program: the console is
//! hosted by Windows' own `conhost.exe`, started directly, so Windows does
//! not hand it to the PC's default terminal. The console is in the app's
//! job: it ends with the app, and "Rimuovi l'unità" ends it.
//!
//! On macOS and Linux, a session of the app's own terminal (`pty.rs`,
//! D-109): the main window shows it in its terminal panel. It ends with the
//! app, and "Rimuovi l'unità" ends it.

#[cfg(windows)]
use std::collections::BTreeSet;
use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::Result;
use tauri::{AppHandle, Manager, Runtime};
#[cfg(windows)]
use windows::Win32::System::Console::SetConsoleCtrlHandler;

#[cfg(windows)]
use super::cfg::pwsh_args;
use super::pty;
use crate::env::KitEnv;
use crate::kit::Kit;
#[cfg(windows)]
use crate::os::jobs;

/// The consoles open now: process ids of their `conhost.exe` (Windows),
/// and the app's terminal sessions (macOS and Linux; none on Windows).
#[derive(Default, Clone)]
pub struct Consoles {
    #[cfg(windows)]
    conhosts: Arc<Mutex<BTreeSet<u32>>>,
    pub ptys: pty::Sessions,
}

impl Consoles {
    #[cfg(windows)]
    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeSet<u32>> {
        self.conhosts.lock().unwrap_or_else(PoisonError::into_inner)
    }

    #[cfg(windows)]
    pub fn count(&self) -> usize {
        self.lock().len()
    }

    #[cfg(not(windows))]
    pub fn count(&self) -> usize {
        self.ptys.running()
    }

    #[cfg(windows)]
    pub fn pids(&self) -> Vec<u32> {
        self.lock().iter().copied().collect()
    }

    /// The programs of the sessions that still run: "Rimuovi l'unità" ends
    /// them with their trees.
    #[cfg(not(windows))]
    pub fn pids(&self) -> Vec<u32> {
        self.ptys.pids()
    }

    /// At the app's exit: every terminal session ends (Windows' consoles
    /// end with the job).
    pub fn close_all(&self) {
        self.ptys.close_all(pty::GRACE);
    }
}

/// What a console runs.
#[cfg(windows)]
enum Program<'a> {
    /// The kit's PowerShell with the kit's profile.
    Shell,
    /// An agent, or a `.cmd` wrapper (run by the kit's PowerShell: cmd
    /// would read `&` and `%` in the path).
    Command(&'a Path),
}

#[cfg(windows)]
fn pwsh(kit: &Kit) -> Result<PathBuf> {
    let pwsh = crate::setup::catalog::kit_path(kit, r"pwsh\pwsh.exe");
    // Never the shells of the PC: without the kit's PowerShell, no console.
    anyhow::ensure!(pwsh.is_file(), "PowerShell 7 del kit non è installato");
    Ok(pwsh)
}

#[cfg(windows)]
fn is_batch(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
}

/// The console host and its arguments: `program` and its own.
#[cfg(windows)]
struct Line {
    conhost: PathBuf,
    args: Vec<String>,
}

/// `conhost.exe <program>`: conhost hands its other arguments to the
/// program, quoting the ones with spaces.
#[cfg(windows)]
fn line(kit: &Kit, env: &KitEnv, program: &Program) -> Result<Line> {
    let mut args = Vec::new();
    match program {
        Program::Shell => {
            args.push(pwsh(kit)?.display().to_string());
            args.extend(pwsh_args(kit));
        }
        Program::Command(path) if is_batch(path) => {
            let quoted = path.display().to_string().replace('\'', "''");
            args.push(pwsh(kit)?.display().to_string());
            args.extend(
                [
                    "-NoLogo",
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                ]
                .map(String::from),
            );
            args.push(format!("& '{quoted}'; exit $LASTEXITCODE"));
        }
        Program::Command(path) => args.push(path.display().to_string()),
    }
    Ok(Line {
        conhost: conhost(env),
        args,
    })
}

/// Windows' console host.
#[cfg(windows)]
fn conhost(env: &KitEnv) -> PathBuf {
    let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
    Path::new(system_root).join(r"System32\conhost.exe")
}

/// Counts `console` until it closes; then `on_exit` runs.
#[cfg(windows)]
fn track(consoles: &Consoles, console: jobs::Console, on_exit: Box<dyn FnOnce() + Send>) {
    let pid = console.id();
    consoles.lock().insert(pid);
    let consoles = consoles.clone();
    std::thread::spawn(move || {
        console.wait();
        consoles.lock().remove(&pid);
        on_exit();
    });
}

/// Starts the console `line` in `dir`, in the job, and counts it until it
/// closes; then `on_exit` runs.
#[cfg(windows)]
fn start(
    consoles: &Consoles,
    env: &KitEnv,
    dir: &Path,
    line: &Line,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    // "Ignore Ctrl+C" passes from a process to its children: the app
    // started in a new process group (from a script, by another program)
    // would hand it to the shell, and Ctrl+C would not stop a command.
    // SAFETY: no handler routine; it only clears the flag of this process.
    let _ = unsafe { SetConsoleCtrlHandler(None, false) };
    let console = jobs::spawn_console(&line.conhost, &line.args, env, dir)?;
    track(consoles, console, on_exit);
    Ok(())
}

/// Opens `program` (`None`: the kit's shell) in a console.
#[cfg(windows)]
fn open<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    program: Option<&Path>,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    let program = program.map_or(Program::Shell, Program::Command);
    let line = line(kit, env, &program)?;
    start(&app.state::<Consoles>(), env, dir, &line, on_exit)
}

/// Opens `program` (`None`: the kit's shell) in a session of the app's
/// terminal, and tells the main window to show it.
#[cfg(unix)]
fn open<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    program: Option<&Path>,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    use tauri::Emitter;
    let main = crate::commands::MAIN;
    let launch = match program {
        None => pty::shell(kit, dir),
        Some(command) => pty::command(command, dir),
    };
    let (out, end) = (app.clone(), app.clone());
    let info = app.state::<Consoles>().ptys.open(
        &launch,
        env,
        dir,
        move |id, chunk| {
            let _ = out.emit_to(main, &pty::output_event(id), chunk);
        },
        move |info| {
            let _ = end.emit_to(main, pty::EXIT_EVENT, &info);
            on_exit();
        },
    )?;
    crate::log::info(&format!("terminal {}: {}", info.id, info.title));
    let _ = app.emit_to(main, pty::OPENED_EVENT, &info);
    Ok(())
}

/// Runs `command` (an agent, or a `.cmd` wrapper) in a new kit console in
/// `dir`; `on_exit` runs when it ends. Inside the job.
pub fn open_command<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    command: &Path,
    on_exit: impl FnOnce() + Send + 'static,
) -> Result<()> {
    open(app, kit, env, dir, Some(command), Box::new(on_exit))
}

/// Opens a kit console (the kit's PowerShell; on macOS and Linux bash when
/// it is not installed) in `dir`. Inside the job.
pub fn open_console<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
) -> Result<()> {
    open(app, kit, env, dir, None, Box::new(|| {}))
}

#[cfg(all(test, windows))]
#[path = "../../../test/rust/editor/terminal.rs"]
mod tests;
