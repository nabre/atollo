//! The kit's terminals (terminals.rs): the default one and the user's, with
//! their tools and versions, opened in any folder of the PC.

use super::*;
use crate::editor::terminals::{self, Terminal, TerminalsView};

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

/// Windows' "Select folder" window, starting in `start`; `None` when it
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
    #[cfg(not(windows))]
    let owner = 0;
    let title = crate::i18n::t(state.lang(), "Scegli la cartella del terminale", &[]);
    let projects = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || {
        let start = start.map(PathBuf::from).unwrap_or(projects);
        terminals::pick_dir(owner, &title, Some(&start)).map(|d| d.map(|d| d.display().to_string()))
    })
    .await
    .or_fail("terminals")?
    .or_fail_chain("terminals")
}
