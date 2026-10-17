//! The kit's terminals (terminals.rs): the default one and the user's, with
//! their tools and versions, opened in any folder of the PC; on macOS and
//! Linux also the sessions of the app's terminal (`pty_*`).

use super::*;
use crate::editor::terminals::{self, Terminal, TerminalsView};
#[cfg(unix)]
use tauri::Manager;

#[tauri::command]
pub async fn terminals_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<TerminalsView> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || terminals::list(&kit, lang))
        .await
        .or_fail("terminals")
}

/// Saves a new terminal (empty id) or the changes of one; returns its id.
#[tauri::command]
pub async fn terminal_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    terminal: Terminal,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        terminals::save(&kit, terminal, lang).map_err(CmdError::from)
    })
    .await
    .or_fail("terminals")?
}

#[tauri::command]
pub async fn terminal_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        terminals::remove(&kit, &id).or_fail_chain("terminals")
    })
    .await
    .or_fail("terminals")?
}

/// Opens the terminal `id` in `dir` (`None`: its own folder), installing
/// first what every console needs (the kit's PowerShell).
#[tauri::command]
pub async fn terminal_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    dir: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let Some(terminal) = terminals::find(&kit, &id, lang) else {
            return Err(crate::i18n::t(lang, "Terminale non trovato: {0}", &[&id]).into());
        };
        let dir = terminals::start_dir(&kit, &terminal, dir.as_deref(), lang)?;
        let env = terminals::env(&kit, &terminal, ensure(&app, &task, &kit, lang, &[])?);
        terminal::open_console(&app, &kit, &env, &dir).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminals")?
}

/// The system's "Select folder" window, starting in `start`; `None` when it
/// was closed without a choice.
#[tauri::command]
pub async fn terminal_dir_pick<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    start: Option<String>,
) -> CmdResult<Option<String>> {
    ensure_main(&webview)?;
    #[cfg(windows)]
    let owner = webview.window().hwnd().or_fail("window")?.0 as isize;
    #[cfg(unix)]
    let window = webview.window();
    let title = crate::i18n::t(state.lang(), "Scegli la cartella del terminale", &[]);
    let projects = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || {
        let start = start.map(PathBuf::from).unwrap_or(projects);
        #[cfg(windows)]
        let picked = terminals::pick_dir(owner, &title, Some(&start));
        #[cfg(unix)]
        let picked = crate::editor::picker::folder(&window, &title, Some(&start));
        picked.map(|d| d.map(|d| d.display().to_string()))
    })
    .await
    .or_fail("terminals")?
    .or_fail_chain("terminals")
}

// The app's terminal on macOS and Linux (editor/pty.rs): its sessions are
// opened by `terminal_open`, `terminal_launch` and `agent_launch`, with the
// checks and the environment of each; the panel of the main window shows,
// types into, resizes and closes them.

/// The terminal sessions, running or ended but still shown.
#[cfg(unix)]
#[tauri::command]
pub fn pty_sessions<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
) -> CmdResult<Vec<crate::editor::pty::PtyInfo>> {
    ensure_main(&webview)?;
    Ok(app.state::<terminal::Consoles>().ptys.list())
}

/// The output kept by session `id`, up to the chunk number it carries.
#[cfg(unix)]
#[tauri::command]
pub fn pty_backlog<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<crate::editor::pty::Chunk> {
    ensure_main(&webview)?;
    app.state::<terminal::Consoles>()
        .ptys
        .backlog(id)
        .or_fail_chain("terminal")
}

/// What was typed in session `id`; queued, never blocking the window.
/// `binary`: bytes that are not text, one character per byte (xterm.js's
/// `onBinary`).
#[cfg(unix)]
#[tauri::command]
pub fn pty_write<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    data: String,
    binary: Option<bool>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let ptys = &app.state::<terminal::Consoles>().ptys;
    if binary == Some(true) {
        let Some(bytes) = crate::editor::pty::latin1(&data) else {
            return Err("terminal input: not bytes".to_owned().into());
        };
        return ptys.write_bytes(id, &bytes).or_fail_chain("terminal");
    }
    ptys.write(id, &data).or_fail_chain("terminal")
}

#[cfg(unix)]
#[tauri::command]
pub fn pty_resize<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    cols: u16,
    rows: u16,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    app.state::<terminal::Consoles>()
        .ptys
        .resize(id, cols, rows)
        .or_fail_chain("terminal")
}

/// Ends the program of session `id` (hang-up, then its tree after a
/// moment) and forgets the session.
#[cfg(unix)]
#[tauri::command]
pub async fn pty_close<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let ptys = app.state::<terminal::Consoles>().ptys.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ptys.close(id, crate::editor::pty::GRACE)
            .or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}
