//! The kit's terminals (terminals.rs): the default one and the user's, with
//! their tools and versions, opened in any folder of the PC; the sessions
//! of the app's terminal (`pty_*`).

use super::*;
use crate::editor::terminals::{self, Terminal, TerminalsView};
use tauri::{Emitter, Manager};

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
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        terminals::remove(&kit, &id).or_fail_chain("terminals")?;
        // The sandboxes an earlier Atollo made for it, one per folder, and
        // their disks go with it (D-169; none made since D-178).
        let names = crate::isolation::terminal_sandboxes(&kit, &id);
        crate::isolation::remove_owned(&kit, &host, &env, &names);
        Ok(())
    })
    .await
    .or_fail("terminals")?
}

/// Opens the terminal `id` in `dir` (`None`: its own folder), installing
/// first what every console needs (the kit's PowerShell). Level 1 (D-179):
/// `decide` places it on the PC, with or without the VM (R3/R4).
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
    let host = state.host.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(terminal) = terminals::find(&kit, &id, lang) else {
            return Err(crate::common::i18n::t(lang, "Terminale non trovato: {0}", &[&id]).into());
        };
        let dir = terminals::start_dir(&kit, &terminal, dir.as_deref(), lang)?;
        let base = ensure(&app, &task, &kit, lang, &[])?;
        let req = kit_terminal_request(&kit, &terminal, &dir);
        // The terminal's tools, at the project's lines in a project's
        // folder (Q1), sealed the level-1 way, never an agent's command.
        let placed = super::levels::place_now(&kit, &base, &req, Some(&terminal), lang, &host)?;
        let ready = crate::levels::prepare::Ready::on_the_pc(placed.placement, placed.env);
        crate::levels::start::open_terminal(&app, &kit, &ready, lang).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminals")?
}

/// The request of the kit terminal `terminal` opened in `dir` (E1,
/// `Origin::KitTerminal`): the place from the folder (`place_of`).
pub(crate) fn kit_terminal_request(
    kit: &Kit,
    terminal: &Terminal,
    dir: &std::path::Path,
) -> crate::levels::Request {
    crate::levels::Request {
        origin: crate::levels::Origin::KitTerminal {
            terminal: terminal.id.clone(),
        },
        place: crate::levels::place::place_of(kit, dir),
    }
}

/// The kit's tools `wanted` (id → the line chosen; `None`: the newest) for
/// the VM's Linux (`guest`, the kit scoped to its target), installed in
/// `data/apps/linux-…` when missing, waiting for the downloads (their
/// progress in the status bar): those with a build for it that this PC can
/// install, not those that finish on their own system
/// (`router::bootstrap`). A failure only goes to the log: the session
/// opens with what is there. The app's `levels::prepare::Effects` (D-179),
/// and «Installa i mancanti» of a project with the VM on. `tick` gets the
/// setup task's snapshot while one of these runs (a VM's preparation shows
/// its download).
pub(crate) fn ensure_guest_tools<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    guest: &Kit,
    wanted: &std::collections::BTreeMap<String, Option<String>>,
    tick: &dyn Fn(&crate::setup::task::Snapshot),
) {
    // In the setup task's queue, its own job; `tick` follows it.
    for opts in crate::levels::prepare::guest_plans(guest, wanted) {
        if let Err(e) = task.start_and_wait(opts, true, setup_notify(app.clone()), tick) {
            crate::common::log::warn(&format!("tools for the VM: {e}"));
        }
    }
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
    let owner = crate::platform::desktop::owner(&webview.window()).or_fail("window")?;
    let title = crate::common::i18n::t(state.lang(), "Scegli la cartella del terminale", &[]);
    let projects = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || {
        let start = start.map(PathBuf::from).unwrap_or(projects);
        crate::platform::desktop::pick_folder(&owner, &title, Some(&start))
            .map(|d| d.map(|d| d.display().to_string()))
    })
    .await
    .or_fail("terminals")?
    .or_fail_chain("terminals")
}

// The app's terminal on every system (editor/pty.rs): its sessions are
// opened by `terminal_open`, `terminal_launch` and `agent_launch`, with the
// checks and the environment of each; the panel of the main window shows,
// types into, resizes and closes them, and so does the window of a
// detached session (`terminal-<id>`, D-149), for that session only.

/// The main window, or the window of session `id`.
/// A web link of session `id`'s output, clicked: the PC's browser, with
/// the PC's environment (`sbx login`'s page, D-169); `http` and `https`
/// only.
#[tauri::command]
pub fn pty_link_open<R: Runtime>(webview: Webview<R>, id: u64, url: String) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let url: tauri::Url = url.parse().or_fail("link")?;
    crate::editor::window::open_in_browser(&url).or_fail_chain("browser")
}

fn ensure_pty<R: Runtime>(webview: &Webview<R>, id: u64) -> CmdResult<()> {
    if webview.label() == MAIN || terminal::session_of(webview.label()) == Some(id) {
        Ok(())
    } else {
        Err("forbidden".into())
    }
}

/// The terminal sessions, running or ended but still shown; to the window
/// of a detached session, only its own.
#[tauri::command]
pub fn pty_sessions<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
) -> CmdResult<Vec<crate::editor::pty::PtyInfo>> {
    let ptys = &app.state::<terminal::Consoles>().ptys;
    if let Some(id) = terminal::session_of(webview.label()) {
        return Ok(ptys.info(id).into_iter().collect());
    }
    ensure_main(&webview)?;
    Ok(ptys.list())
}

/// The output kept by session `id`, up to the chunk number it carries.
/// Off the window's thread: up to 256 KB copied, then sent as JSON, where
/// a terminal's escape bytes grow sixfold.
#[tauri::command]
pub async fn pty_backlog<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<crate::editor::pty::Chunk> {
    ensure_pty(&webview, id)?;
    tauri::async_runtime::spawn_blocking(move || {
        app.state::<terminal::Consoles>()
            .ptys
            .backlog(id)
            .or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}

/// What was typed in session `id`; queued, never blocking the window.
/// `binary`: bytes that are not text, one character per byte (xterm.js's
/// `onBinary`).
#[tauri::command]
pub fn pty_write<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    data: String,
    binary: Option<bool>,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let ptys = &app.state::<terminal::Consoles>().ptys;
    if binary == Some(true) {
        let Some(bytes) = crate::editor::pty::latin1(&data) else {
            return Err("terminal input: not bytes".to_owned().into());
        };
        return ptys.write_bytes(id, &bytes).or_fail_chain("terminal");
    }
    ptys.write(id, &data).or_fail_chain("terminal")
}

#[tauri::command]
pub fn pty_resize<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    cols: u16,
    rows: u16,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    app.state::<terminal::Consoles>()
        .ptys
        .resize(id, cols, rows)
        .or_fail_chain("terminal")
}

/// Ends the program of session `id` (hang-up, then its tree after a
/// moment) and forgets the session; the panel drops its tab and its own
/// window, if any, closes.
#[tauri::command]
pub async fn pty_close<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let ptys = app.state::<terminal::Consoles>().ptys.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ptys.close(id, crate::editor::pty::GRACE)
            .or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")??;
    let _ = app.emit_to(MAIN, super::events::PTY_CLOSED, id);
    if let Some(window) = app.get_webview_window(&terminal::label(id)) {
        let _ = window.close();
    }
    Ok(())
}

/// Moves session `id` from the panel into a window of its own (D-149).
/// Async: a window is never built on the thread that runs the windows.
#[tauri::command]
pub async fn pty_detach<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    terminal::detach(&app, id).or_fail_chain("terminal")
}

/// Brings session `id` back to the panel and closes its window.
#[tauri::command]
pub async fn pty_dock<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    terminal::dock(&app, id).or_fail_chain("terminal")
}
