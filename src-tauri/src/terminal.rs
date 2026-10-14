//! Consoles outside the editor: windows of the app (`terminal.html`, xterm.js)
//! on a pseudo console of `portable-pty` (ConPTY on Windows, a pty on macOS
//! and Linux), with the kit's PowerShell or an agent (DECISIONS.md D-13,
//! D-23). Never a shell of the PC. When the pseudo console cannot start
//! (Windows 10 before 1809), the kit's PowerShell in a classic console.
//!
//! Rules (D-14): windows `terminal-<n>` have the `terminal` capability only:
//! three commands, and each window reaches only the session of its label.
//! The process is in the app's job; closing the window ends it.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, ChildKiller, CommandBuilder, MasterPty, PtySize};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::Threading::CREATE_NEW_CONSOLE;

use crate::editor_cfg::pwsh_args;
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::Kit;

pub const LABEL_PREFIX: &str = "terminal-";

/// Output kept for a window that attaches late or reloads.
const SCROLLBACK: usize = 512 * 1024;

/// "Where is the cursor?": ConPTY asks it once at start and waits for the
/// answer before running the program; the app answers that one. Never
/// kept for a replay, or a reloaded window would answer old questions.
const CURSOR_QUERY: &[u8] = b"\x1b[6n";

static NEXT: AtomicU32 = AtomicU32::new(1);

/// The open sessions, by window label.
#[derive(Default)]
pub struct Terminals(Mutex<HashMap<String, Arc<Session>>>);

pub struct Session {
    title: String,
    writer: Mutex<Box<dyn Write + Send>>,
    /// Dropped when the process ends: that closes the pseudo console, and
    /// the reader gets the end of the output.
    master: Mutex<Option<Box<dyn MasterPty + Send>>>,
    killer: Mutex<Box<dyn ChildKiller + Send + Sync>>,
    out: Mutex<Output>,
}

#[derive(Default)]
struct Output {
    recent: Vec<u8>,
    sink: Option<Channel<InvokeResponseBody>>,
    on_exit: Option<Channel<u32>>,
    code: Option<u32>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Output {
    fn push(&mut self, data: &[u8]) {
        let mut rest = data;
        while let Some(at) = find(rest, CURSOR_QUERY) {
            self.recent.extend_from_slice(&rest[..at]);
            rest = &rest[at + CURSOR_QUERY.len()..];
        }
        self.recent.extend_from_slice(rest);
        if self.recent.len() > SCROLLBACK {
            let cut = self.recent.len() - SCROLLBACK;
            self.recent.drain(..cut);
        }
        if let Some(sink) = &self.sink {
            if sink.send(InvokeResponseBody::Raw(data.to_vec())).is_err() {
                self.sink = None;
            }
        }
    }

    fn exited(&mut self, code: u32) {
        self.code = Some(code);
        if let Some(on_exit) = self.on_exit.take() {
            let _ = on_exit.send(code);
        }
    }
}

impl Terminals {
    fn get(&self, label: &str) -> Result<Arc<Session>, String> {
        lock(&self.0)
            .get(label)
            .cloned()
            .ok_or_else(|| format!("no terminal: {label}"))
    }

    /// Connects the window `label`: what the process wrote so far, then
    /// everything new; `on_exit` gets the exit code. Returns the title.
    pub fn attach(
        &self,
        label: &str,
        output: Channel<InvokeResponseBody>,
        on_exit: Channel<u32>,
    ) -> Result<String, String> {
        let session = self.get(label)?;
        let mut out = lock(&session.out);
        if !out.recent.is_empty() {
            let _ = output.send(InvokeResponseBody::Raw(out.recent.clone()));
        }
        out.sink = Some(output);
        match out.code {
            Some(code) => {
                let _ = on_exit.send(code);
            }
            None => out.on_exit = Some(on_exit),
        }
        Ok(session.title.clone())
    }

    pub fn input(&self, label: &str, data: &[u8]) -> Result<(), String> {
        let session = self.get(label)?;
        let mut writer = lock(&session.writer);
        writer
            .write_all(data)
            .and_then(|()| writer.flush())
            .map_err(|e| e.to_string())
    }

    pub fn resize(&self, label: &str, cols: u16, rows: u16) -> Result<(), String> {
        let session = self.get(label)?;
        let master = lock(&session.master);
        match master.as_ref() {
            Some(m) => m.resize(size(cols, rows)).map_err(|e| e.to_string()),
            None => Ok(()),
        }
    }

    /// The window `label` is closing: its process ends.
    pub fn close(&self, label: &str) {
        if let Some(session) = lock(&self.0).remove(label) {
            let _ = lock(&session.killer).kill();
        }
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        cols: cols.clamp(2, 1000),
        rows: rows.clamp(1, 500),
        pixel_width: 0,
        pixel_height: 0,
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

fn pwsh(kit: &Kit) -> Result<std::path::PathBuf> {
    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
    // Never the shells of the PC: without the kit's PowerShell, no console.
    anyhow::ensure!(pwsh.is_file(), "PowerShell 7 del kit non è installato");
    Ok(pwsh)
}

fn is_batch(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmd") || e.eq_ignore_ascii_case("bat"))
}

/// The command line of `program`, with the kit environment only.
fn builder(kit: &Kit, env: &KitEnv, dir: &Path, program: &Program) -> Result<CommandBuilder> {
    let mut cmd = match program {
        Program::Shell => {
            let mut cmd = CommandBuilder::new(pwsh(kit)?);
            cmd.args(pwsh_args(kit));
            cmd
        }
        Program::Command(path) if is_batch(path) => {
            let mut cmd = CommandBuilder::new(pwsh(kit)?);
            let quoted = path.display().to_string().replace('\'', "''");
            cmd.args([
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
            ]);
            cmd.arg(format!("& '{quoted}'; exit $LASTEXITCODE"));
            cmd
        }
        Program::Command(path) => CommandBuilder::new(path),
    };
    cmd.env_clear();
    for (k, v) in env.iter() {
        cmd.env(k, v);
    }
    cmd.cwd(dir);
    Ok(cmd)
}

/// Starts `program` on a pseudo console and registers it as `label`.
/// `on_exit` runs when the process ends.
fn start(
    terminals: &Terminals,
    label: &str,
    title: &str,
    cmd: CommandBuilder,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    // "Ignore Ctrl+C" passes from a process to its children: the app
    // started in a new process group (from a script, by another program)
    // would hand it to the shell, and Ctrl+C would not stop a command.
    // SAFETY: no handler routine; it only clears the flag of this process.
    let _ = unsafe { SetConsoleCtrlHandler(None, false) };
    // portable-pty panics instead of failing when Windows has no ConPTY
    // (before Windows 10 1809): that must reach the classic console.
    let pair = std::panic::catch_unwind(|| native_pty_system().openpty(size(120, 30)))
        .map_err(|_| anyhow::anyhow!("this Windows has no pseudo console"))?
        .context("pseudo console")?;
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .context("starting the console")?;
    // The slave holds the pseudo console too: only the master may keep it.
    drop(pair.slave);
    if let Err(e) = child.process_id().map_or(Ok(()), jobs::assign_pid) {
        let _ = child.kill();
        return Err(e);
    }
    let mut reader = pair.master.try_clone_reader().context("console output")?;
    let writer = pair.master.take_writer().context("console input")?;
    let session = Arc::new(Session {
        title: title.to_owned(),
        writer: Mutex::new(writer),
        master: Mutex::new(Some(pair.master)),
        killer: Mutex::new(child.clone_killer()),
        out: Mutex::new(Output::default()),
    });
    lock(&terminals.0).insert(label.to_owned(), Arc::clone(&session));

    let output = Arc::clone(&session);
    let reading = std::thread::spawn(move || {
        let mut buf = [0u8; 16 * 1024];
        let mut answered = false;
        loop {
            match reader.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let mut data = &buf[..n];
                    if !answered {
                        if let Some(at) = find(data, CURSOR_QUERY) {
                            answered = true;
                            let _ = lock(&output.writer).write_all(b"\x1b[1;1R");
                            lock(&output.out).push(&data[..at]);
                            data = &data[at + CURSOR_QUERY.len()..];
                        }
                    }
                    lock(&output.out).push(data);
                }
            }
        }
    });
    let label = label.to_owned();
    std::thread::spawn(move || {
        let code = child.wait().map_or(1, |s| s.exit_code());
        crate::log::info(&format!("{label}: process ended ({code})"));
        // Closing the pseudo console flushes the output and ends the reader.
        drop(lock(&session.master).take());
        crate::log::info(&format!("{label}: pseudo console closed"));
        let _ = reading.join();
        lock(&session.out).exited(code);
        crate::log::info(&format!("{label}: exit sent"));
        on_exit();
    });
    Ok(())
}

/// A window of the app on the session `label`.
fn open_window<R: Runtime>(app: &AppHandle<R>, kit: &Kit, label: &str, title: &str) -> Result<()> {
    let window = WebviewWindowBuilder::new(app, label, WebviewUrl::App("terminal.html".into()))
        .title(format!("{title} - Dev-Kit"))
        .inner_size(980.0, 620.0)
        .min_inner_size(420.0, 240.0)
        .data_directory(kit.webview_data())
        .zoom_hotkeys_enabled(false)
        .on_new_window(|_, _| NewWindowResponse::Deny)
        .build()
        .context("terminal window")?;
    // Ctrl+F, F5, Ctrl+P... reach the shell, not the browser.
    crate::editor_window::quiet_webview(&window, "terminal");
    Ok(())
}

/// Runs `program` in a new kit console in `dir`, titled `title`;
/// `on_exit` runs when it ends. Classic console when the pseudo console
/// cannot start.
fn open<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    title: &str,
    program: Program,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    let cmd = builder(kit, env, dir, &program)?;
    let label = format!("{LABEL_PREFIX}{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let terminals = app.state::<Terminals>();
    let (on_exit, fallback) = share(on_exit);
    match start(&terminals, &label, title, cmd, on_exit) {
        Ok(()) => {
            if let Err(e) = open_window(app, kit, &label, title) {
                terminals.close(&label);
                return Err(e);
            }
            Ok(())
        }
        Err(e) => {
            crate::log::warn(&format!("pseudo console: {e:#}; classic console"));
            classic(kit, env, dir, &program, fallback)
        }
    }
}

type OnExit = Box<dyn FnOnce() + Send>;

/// One callback for two paths, of which only one runs.
fn share(on_exit: OnExit) -> (OnExit, OnExit) {
    let slot = Arc::new(Mutex::new(Some(on_exit)));
    let other = Arc::clone(&slot);
    let take = |s: Arc<Mutex<Option<OnExit>>>| -> OnExit {
        Box::new(move || {
            if let Some(f) = lock(&s).take() {
                f();
            }
        })
    };
    (take(slot), take(other))
}

/// The kit's PowerShell or `program` in a classic console window.
fn classic(kit: &Kit, env: &KitEnv, dir: &Path, program: &Program, on_exit: OnExit) -> Result<()> {
    let mut cmd = match program {
        Program::Shell => {
            let mut cmd = jobs::kit_command(pwsh(kit)?, env);
            cmd.args(pwsh_args(kit));
            cmd
        }
        Program::Command(path) if is_batch(path) => jobs::batch_command(path, &[], env),
        Program::Command(path) => jobs::kit_command(path, env),
    };
    cmd.current_dir(dir).creation_flags(CREATE_NEW_CONSOLE.0);
    let mut child = jobs::spawn(&mut cmd)?;
    std::thread::spawn(move || {
        let _ = child.wait();
        on_exit();
    });
    Ok(())
}

/// Runs `command` (an agent, or a `.cmd` wrapper) in a new kit console in
/// `dir`, titled `title`; `on_exit` runs when it ends. Inside the job.
pub fn open_command<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    title: &str,
    command: &Path,
    on_exit: impl FnOnce() + Send + 'static,
) -> Result<()> {
    open(
        app,
        kit,
        env,
        dir,
        title,
        Program::Command(command),
        Box::new(on_exit),
    )
}

/// Opens a kit console (the kit's PowerShell) in `dir`. Inside the job.
pub fn open_console<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
) -> Result<()> {
    let title = dir.file_name().map_or_else(
        || "Terminale".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    open(app, kit, env, dir, &title, Program::Shell, Box::new(|| {}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn batch_runs_through_the_kit_powershell() {
        let tmp = TempDir::new("term");
        let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64);
        kit.prepare().unwrap();
        let script = kit.root().join(r"it's\agy.cmd");
        assert!(
            builder(
                &kit,
                &KitEnv::default(),
                kit.root(),
                &Program::Command(&script)
            )
            .is_err(),
            "no PC shell"
        );

        let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
        std::fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
        std::fs::write(&pwsh, b"").unwrap();
        let cmd = builder(
            &kit,
            &KitEnv::default(),
            kit.root(),
            &Program::Command(&script),
        )
        .unwrap();
        let argv: Vec<String> = cmd
            .get_argv()
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(argv[0], pwsh.display().to_string());
        assert!(argv.last().unwrap().contains("it''s\\agy.cmd'"), "{argv:?}");
        assert!(
            cmd.iter_extra_env_as_str().next().is_none(),
            "no PC variables"
        );
    }

    #[test]
    fn real_console_echoes_and_ends() {
        let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
        let mut cmd = CommandBuilder::new(Path::new(&system_root).join(r"System32\cmd.exe"));
        cmd.args(["/d", "/c", "echo dev-kit-pty& exit 3"]);
        let terminals = Terminals::default();
        let (tx, rx) = std::sync::mpsc::channel();
        start(
            &terminals,
            "terminal-t",
            "t",
            cmd,
            Box::new(move || tx.send(()).unwrap()),
        )
        .unwrap();
        rx.recv_timeout(std::time::Duration::from_secs(20)).unwrap();
        let session = terminals.get("terminal-t").unwrap();
        let out = lock(&session.out);
        assert_eq!(out.code, Some(3));
        assert!(String::from_utf8_lossy(&out.recent).contains("dev-kit-pty"));
        drop(out);
        terminals.close("terminal-t");
        assert!(terminals.get("terminal-t").is_err());
    }

    #[test]
    fn scrollback_is_bounded() {
        let mut out = Output::default();
        out.push(&vec![b'a'; SCROLLBACK]);
        out.push(b"end");
        assert_eq!(out.recent.len(), SCROLLBACK);
        assert!(out.recent.ends_with(b"end"));
        out.push(b"a\x1b[6nb");
        assert!(
            out.recent.ends_with(b"endab"),
            "cursor queries not replayed"
        );
    }
}
