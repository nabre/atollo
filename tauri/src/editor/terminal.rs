//! Consoles outside the editor: the kit's PowerShell or an agent, in a
//! classic Windows console (DECISIONS.md D-57, D-95). Never a shell of the
//! PC, and never the PC's terminal program: the console is hosted by
//! Windows' own `conhost.exe`, started directly, so Windows does not hand
//! it to the PC's default terminal.
//!
//! The console is in the app's job: it ends with the app, and "Rimuovi
//! l'unità" ends it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::Result;
use tauri::{AppHandle, Manager, Runtime};
use windows::Win32::System::Console::SetConsoleCtrlHandler;

use super::cfg::pwsh_args;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs;

/// The consoles open now: process ids of their `conhost.exe`.
#[derive(Default, Clone)]
pub struct Consoles(Arc<Mutex<BTreeSet<u32>>>);

impl Consoles {
    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeSet<u32>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn count(&self) -> usize {
        self.lock().len()
    }

    pub fn pids(&self) -> Vec<u32> {
        self.lock().iter().copied().collect()
    }
}

/// What a console runs.
enum Program<'a> {
    /// The kit's PowerShell with the kit's profile.
    Shell,
    /// An agent, or a `.cmd` wrapper (run by the kit's PowerShell: cmd
    /// would read `&` and `%` in the path).
    Command(&'a Path),
}

fn pwsh(kit: &Kit) -> Result<PathBuf> {
    let pwsh = crate::setup::catalog::kit_path(kit, r"pwsh\pwsh.exe");
    // Never the shells of the PC: without the kit's PowerShell, no console.
    anyhow::ensure!(pwsh.is_file(), "PowerShell 7 del kit non è installato");
    Ok(pwsh)
}

fn is_batch(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
}

/// The console host and its arguments: `program` and its own.
struct Line {
    conhost: PathBuf,
    args: Vec<String>,
}

/// `conhost.exe <program>`: conhost hands its other arguments to the
/// program, quoting the ones with spaces.
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
fn conhost(env: &KitEnv) -> PathBuf {
    let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
    Path::new(system_root).join(r"System32\conhost.exe")
}

/// Counts `console` until it closes; then `on_exit` runs.
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

/// Opens a kit console (the kit's PowerShell) in `dir`. Inside the job.
pub fn open_console<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
) -> Result<()> {
    open(app, kit, env, dir, None, Box::new(|| {}))
}

#[cfg(test)]
#[path = "../../../test/rust/editor/terminal.rs"]
mod tests;
