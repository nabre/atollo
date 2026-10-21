//! Consoles outside the editor: the kit's PowerShell or an agent, as a
//! session of the app's own terminal (`pty.rs`; DECISIONS.md D-109, D-148)
//! on every system. The main window shows it in its terminal panel. Never a
//! shell of the PC, and never the PC's terminal program. It ends with the
//! app, and "Rimuovi l'unità" ends it.

use std::path::Path;

use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use super::pty;
use crate::env::KitEnv;
use crate::kit::Kit;

/// The app's terminal sessions.
#[derive(Default, Clone)]
pub struct Consoles {
    pub ptys: pty::Sessions,
}

impl Consoles {
    /// Sessions whose program still runs: the close question lists them.
    pub fn count(&self) -> usize {
        self.ptys.running()
    }

    /// The programs of the sessions that still run and the jobs left in
    /// their sessions: "Rimuovi l'unità" ends them with their trees.
    pub fn pids(&self) -> Vec<u32> {
        self.ptys.pids()
    }

    /// At the app's exit: every terminal session ends.
    pub fn close_all(&self) {
        self.ptys.close_all(pty::GRACE);
    }
}

/// `s` for the inside of a PowerShell `'...'` string: every quote PowerShell
/// closes the string on doubled, the typographic ones U+2018..U+201B too
/// (a kit in `D:\O’Brien\Atollo`: BUG-V6-31).
pub(crate) fn ps_single_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') {
            out.push(c);
        }
        out.push(c);
    }
    out
}

/// Opens `program` (`None`: the kit's shell) in a session of the app's
/// terminal, and tells the main window to show it.
fn open<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    program: Option<&Path>,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    let main = crate::commands::MAIN;
    let launch = match program {
        None => pty::shell(kit, dir),
        Some(command) => pty::command(kit, command, dir),
    };
    let (out, end) = (app.clone(), app.clone());
    let lang = app
        .try_state::<crate::commands::AppState>()
        .map_or("it", |s| s.lang());
    app.state::<Consoles>().ptys.open(
        &launch,
        env,
        dir,
        lang,
        // Told before the session's first output or its exit.
        |info| {
            crate::log::info(&format!("terminal {}: {}", info.id, info.title));
            let _ = app.emit_to(main, pty::OPENED_EVENT, info);
        },
        move |id, chunk| {
            let _ = out.emit_to(main, &pty::output_event(id), chunk);
        },
        move |info| {
            let _ = end.emit_to(main, pty::EXIT_EVENT, &info);
            on_exit();
        },
    )?;
    Ok(())
}

/// Runs `command` (an agent, or a `.cmd` wrapper) in a new session of the
/// app's terminal in `dir`; `on_exit` runs when it ends. Inside the job.
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

/// Opens the kit's shell (its PowerShell; on macOS and Linux bash when it
/// is not installed) in a new session in `dir`. Inside the job.
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
