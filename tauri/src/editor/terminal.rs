//! Consoles outside the editor: the kit's PowerShell or an agent, as a
//! session of the app's own terminal (`pty.rs`; DECISIONS.md D-109, D-148)
//! on every system. The main window shows it in its terminal panel, or a
//! window of its own once detached (`terminal-<id>`, D-149). Never a shell
//! of the PC, and never the PC's terminal program. It ends with the app,
//! and "Rimuovi l'unità" ends it.

use std::path::Path;

use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

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

/// Opens `launch` in a session of the app's terminal, and tells the main
/// window to show it.
fn open<R: Runtime>(
    app: &AppHandle<R>,
    env: &KitEnv,
    dir: &Path,
    launch: pty::Launch,
    on_exit: Box<dyn FnOnce() + Send>,
) -> Result<()> {
    let main = crate::ipc::MAIN;
    let (out, end) = (app.clone(), app.clone());
    let lang = app
        .try_state::<crate::ipc::AppState>()
        .map_or("it", |s| s.lang());
    app.state::<Consoles>().ptys.open(
        &launch,
        env,
        dir,
        lang,
        // Told before the session's first output or its exit.
        |info| {
            crate::common::log::info(&format!("terminal {}: {}", info.id, info.title));
            let _ = app.emit_to(main, crate::ipc::events::PTY_OPENED, info);
        },
        move |id, chunk| emit_output(&out, id, chunk),
        move |info| {
            emit_exit(&end, &info);
            on_exit();
        },
    )?;
    Ok(())
}

/// A session's output, to the panel and to the session's own window: only
/// the one that listens (the view that shows it) receives it. Every session
/// goes through here, since any of them can be detached (D-149).
pub fn emit_output<R: Runtime>(app: &AppHandle<R>, id: u64, chunk: &pty::Chunk) {
    let event = crate::ipc::events::pty_output(id);
    let _ = app.emit_to(crate::ipc::MAIN, &event, chunk);
    let _ = app.emit_to(label(id).as_str(), &event, chunk);
}

/// A session's end, to the panel and to the session's own window.
pub fn emit_exit<R: Runtime>(app: &AppHandle<R>, info: &pty::PtyInfo) {
    let _ = app.emit_to(crate::ipc::MAIN, crate::ipc::events::PTY_EXIT, info);
    let _ = app.emit_to(label(info.id).as_str(), crate::ipc::events::PTY_EXIT, info);
}

/// Opens the kit's shell (its PowerShell; on macOS and Linux bash when it
/// is not installed) in a new session in `dir`. Inside the job.
pub fn open_console<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
) -> Result<()> {
    open(app, env, dir, pty::shell(kit, dir), Box::new(|| {}))
}

/// Runs `launch` (a project's microVM, D-167) in a new session of the
/// app's terminal in `dir`. Inside the job.
pub fn open_launch<R: Runtime>(
    app: &AppHandle<R>,
    env: &KitEnv,
    dir: &Path,
    launch: pty::Launch,
) -> Result<()> {
    open(app, env, dir, launch, Box::new(|| {}))
}

/// The label of the window of a detached session (D-149).
pub const LABEL_PREFIX: &str = "terminal-";

pub fn label(id: u64) -> String {
    format!("{LABEL_PREFIX}{id}")
}

/// The session a window shows: `terminal-<id>`.
pub fn session_of(label: &str) -> Option<u64> {
    let id = label.strip_prefix(LABEL_PREFIX)?;
    // Digits only: `terminal-+1` or `terminal-01` name no session.
    if id.is_empty()
        || !id.bytes().all(|b| b.is_ascii_digit())
        || (id.len() > 1 && id.starts_with('0'))
    {
        return None;
    }
    id.parse().ok()
}

/// Moves session `id` into a window of its own (`terminal.html`, its
/// view only), as VSCodium's «Move Terminal into New Window»; a window
/// already open comes to the front. The panel drops its tab.
pub fn detach<R: Runtime>(app: &AppHandle<R>, id: u64) -> Result<()> {
    let label = label(id);
    if let Some(window) = app.get_webview_window(&label) {
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }
    let ptys = &app.state::<Consoles>().ptys;
    let info = ptys.info(id)?;
    let state = app.state::<crate::ipc::AppState>();
    let lang = serde_json::to_string(state.lang())?;
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("terminal.html".into()))
        .title(format!("{} - Atollo", info.title))
        .inner_size(960.0, 600.0)
        .min_inner_size(420.0, 260.0)
        // The window's own title bar: the page is only the terminal.
        .initialization_script(format!("window.__ATOLLO_LANG__ = {lang};"))
        .data_directory(state.kit.webview_data())
        .theme(
            crate::common::settings::Settings::load(&state.kit)
                .settings
                .theme
                .window(),
        )
        .zoom_hotkeys_enabled(false)
        .build()?;
    // F5, Ctrl+F, Ctrl+P... go to the program, as in the panel (D-148).
    super::window::quiet_webview(&window, "terminal");
    let info = ptys.set_detached(id, true)?;
    let _ = app.emit_to(crate::ipc::MAIN, crate::ipc::events::PTY_MOVED, &info);
    Ok(())
}

/// Brings session `id` back to the panel of the main window, which shows
/// it, and closes its window.
pub fn dock<R: Runtime>(app: &AppHandle<R>, id: u64) -> Result<()> {
    let info = app.state::<Consoles>().ptys.set_detached(id, false)?;
    let _ = app.emit_to(crate::ipc::MAIN, crate::ipc::events::PTY_MOVED, &info);
    if let Some(window) = app.get_webview_window(&label(id)) {
        let _ = window.close();
    }
    Ok(())
}

/// The window of a detached session was closed: a program that still
/// runs goes back to the panel (closing a window never stops it), an
/// ended one is forgotten.
pub fn closed<R: Runtime>(app: &AppHandle<R>, window: &str) {
    let Some(id) = session_of(window) else {
        return;
    };
    let ptys = app.state::<Consoles>().ptys.clone();
    let Ok(info) = ptys.info(id) else {
        return;
    };
    if !info.detached {
        return;
    }
    if info.running {
        let _ = dock(app, id);
        return;
    }
    let handle = app.clone();
    std::thread::spawn(move || {
        if ptys.close(id, pty::GRACE).is_ok() {
            let _ = handle.emit_to(crate::ipc::MAIN, crate::ipc::events::PTY_CLOSED, id);
        }
    });
}

#[cfg(test)]
#[path = "../../../test/rust/editor/terminal.rs"]
mod tests;
