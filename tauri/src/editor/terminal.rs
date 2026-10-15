//! Consoles outside the editor: the kit's PowerShell or an agent, in the
//! kit's portable Windows Terminal (`editor/wterm.rs`, D-68) or, when the settings
//! ask for it or that terminal cannot run, in a classic Windows console
//! (DECISIONS.md D-13, D-23, D-57). Never a shell of the PC, and never the
//! PC's terminal program: the classic console is hosted by Windows' own
//! `conhost.exe`, started directly, so Windows does not hand it to the PC's
//! default terminal.
//!
//! The console is in the app's job: it ends with the app, and "Rimuovi
//! l'unità" ends it.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::Result;
use tauri::{AppHandle, Manager, Runtime};
use windows::Win32::System::Console::SetConsoleCtrlHandler;

use crate::editor::cfg::pwsh_args;
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::Kit;

/// The consoles open now: process ids of their `conhost.exe`, or of the
/// kit's PowerShell in a Windows Terminal tab.
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
    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
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

/// Starts the classic console `line` in `dir`, in the job, and counts it
/// until it closes; then `on_exit` runs.
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

/// Opens `program` (`None`: the kit's shell) in the kit's Windows Terminal,
/// or in a classic console when the settings ask for it, that terminal
/// cannot run or its console did not start.
fn open<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    program: Option<&Path>,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    let consoles = app.state::<Consoles>();
    let classic = crate::settings::Settings::load(kit)
        .settings
        .classic_console();
    if !classic && crate::editor::wterm::usable(kit) {
        match crate::editor::wterm::open(kit, env, dir, program) {
            Ok(console) => {
                track(&consoles, console, on_exit);
                return Ok(());
            }
            Err(e) => crate::log::warn(&format!("windows terminal: {e:#}; classic console")),
        }
    }
    let program = program.map_or(Program::Shell, Program::Command);
    let line = line(kit, env, &program)?;
    start(&consoles, env, dir, &line, on_exit)
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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn batch_runs_through_the_kit_powershell_in_conhost() {
        let tmp = TempDir::new("term");
        let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64);
        kit.prepare().unwrap();
        let script = kit.root().join(r"it's\agy.cmd");
        let env = KitEnv::default();
        let batch = Program::Command(&script);
        assert!(line(&kit, &env, &batch).is_err(), "no PC shell");
        assert!(line(&kit, &env, &Program::Shell).is_err());

        let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
        std::fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
        std::fs::write(&pwsh, b"").unwrap();
        let l = line(&kit, &env, &batch).unwrap();
        assert!(l
            .conhost
            .to_string_lossy()
            .to_lowercase()
            .ends_with(r"\system32\conhost.exe"));
        assert_eq!(l.args[0], pwsh.display().to_string());
        assert!(
            l.args.last().unwrap().contains(r"it''s\agy.cmd'"),
            "{:?}",
            l.args
        );

        let shell = line(&kit, &env, &Program::Shell).unwrap();
        assert_eq!(shell.args[0], pwsh.display().to_string());
        assert!(shell.args.iter().any(|a| a.ends_with("profile.ps1")));
    }

    /// A real conhost runs the program (in a folder with spaces) with the
    /// kit environment, and the console is counted until it closes.
    #[test]
    fn real_console_runs_and_ends() {
        let tmp = TempDir::new("term-real");
        let dir = tmp.0.join("a b");
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("probe.cmd");
        std::fs::write(&script, "@echo %ATOLLO_PROBE%> ran.txt\r\n").unwrap();
        let mut env = KitEnv::default();
        env.set("ATOLLO_PROBE", "kit-env");
        // Through cmd here: the test kit has no PowerShell.
        let l = Line {
            conhost: conhost(&env),
            args: vec![
                r"C:\Windows\System32\cmd.exe".into(),
                "/d".into(),
                "/c".into(),
                script.display().to_string(),
            ],
        };
        let consoles = Consoles::default();
        let (tx, rx) = std::sync::mpsc::channel();
        start(
            &consoles,
            &env,
            &dir,
            &l,
            Box::new(move || tx.send(()).unwrap()),
        )
        .unwrap();
        assert_eq!(consoles.count(), 1);
        rx.recv_timeout(std::time::Duration::from_secs(20)).unwrap();
        assert_eq!(consoles.count(), 0);
        let ran = std::fs::read_to_string(dir.join("ran.txt")).unwrap();
        assert_eq!(ran.trim(), "kit-env", "the kit environment only");
    }
}
