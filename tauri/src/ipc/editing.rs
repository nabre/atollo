//! The editor, the kit's console and a project's editor settings and
//! extensions.

use super::*;

#[tauri::command]
pub fn editor_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<editor::Status> {
    ensure_main(&webview)?;
    Ok(state.editor.status())
}

/// Starts the server if needed (off the UI thread) and opens a window on
/// the project, or on `projects\` when `project` is `None`.
#[tauri::command]
pub async fn editor_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::opened(&state.kit, &project);
    let title = project.clone();
    let editor = Arc::clone(&state.editor);
    let (handle, task, kit) = (app.clone(), Arc::clone(&state.setup), state.kit.clone());
    let lang = state.lang();
    // No VM is prepared for the editor (D-179): its terminals are level 1
    // on the PC, each task and agent asks the levels when it starts
    // (`levels::requests`).
    let (port, token) = tauri::async_runtime::spawn_blocking(move || {
        ensure(&handle, &task, &kit, lang, &[])?;
        let (port, token) = editor.ensure_running().or_fail_chain("editor start")?;
        Ok::<_, CmdError>((port, token))
    })
    .await
    .or_fail("editor")??;
    let _ = app.emit_to(MAIN, super::events::EDITOR_STATUS, state.editor.status());
    let url = editor::url(port, &token, Some(&dir));
    editor::window::open(
        &app,
        &url,
        port,
        &title,
        &dir,
        &state.kit.webview_data(),
        lang,
    )
    .or_fail_chain("editor window")
}

#[tauri::command]
pub async fn editor_restart<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let editor = Arc::clone(&state.editor);
    tauri::async_runtime::spawn_blocking(move || {
        editor.stop();
        editor.ensure_running().map(|_| ())
    })
    .await
    .or_fail("editor")?
    .or_fail_chain("editor restart")?;
    editor::window::notify_agents_removed(&app, &state.editor, state.lang());
    let _ = app.emit_to(MAIN, super::events::EDITOR_STATUS, state.editor.status());
    Ok(())
}

#[tauri::command]
pub async fn terminal_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::opened(&state.kit, &project);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let host = state.host.clone();
    let fx = super::levels::AppEffects::of(&app, &state, state.env());
    tauri::async_runtime::spawn_blocking(move || {
        let kit_env = ensure(&app, &task, &kit, lang, &[])?;
        // Level 2 (D-179): in the project's microVM when it is on (R14),
        // made ready by `levels::prepare` (the kit's tools, the databases
        // on the PC, D-170, D-171), else on the PC, sealed (R5); a network
        // share or a pending restart of the kit's daemon refused (R9, R10).
        // A VM without the kit's `sbx` is an error, never the PC (I7).
        let req = project_terminal_request(&kit, &dir);
        let fx = super::levels::AppEffects {
            kit_env: kit_env.clone(),
            ..fx
        };
        let ready = super::levels::ready_now(&fx, &kit, &kit_env, &req, &[], lang, &host)?;
        crate::levels::start::open_terminal(&app, &kit, &ready, lang).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}

/// The request of the project terminal of the project in `dir` (E3,
/// `Origin::ProjectTerminal`).
pub(crate) fn project_terminal_request(kit: &Kit, dir: &std::path::Path) -> crate::levels::Request {
    crate::levels::Request {
        origin: crate::levels::Origin::ProjectTerminal,
        place: crate::levels::place::place_of(kit, dir),
    }
}

/// The project's editor settings in its .vscode\settings.json (D-78).
#[tauri::command]
pub fn project_editor_settings<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::vscode::EditorSettingsView> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::projects::vscode::get(&dir))
}

#[tauri::command]
pub fn project_editor_settings_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    settings: crate::projects::vscode::EditorSettings,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::vscode::set(&dir, &settings).or_fail_chain("editor settings")
}

/// The kit's matrix of editor extensions by language ("Linguaggi e
/// strumenti"), for `target`: the system chosen on the Tools page (D-121),
/// this PC's when omitted. Off the window's thread: it reads the editor's
/// installed extensions.
#[tauri::command]
pub async fn extensions_matrix<R: Runtime>(
    webview: Webview<R>,
    target: Option<String>,
) -> CmdResult<crate::editor::ext::MatrixInfo> {
    ensure_main(&webview)?;
    off_main(&webview, "extensions", move |_, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        Ok(crate::editor::ext::matrix_info(&kit))
    })
    .await
}

/// The editor extensions added to or removed from those of the project's
/// languages; the new ones are installed now if the editor runs.
#[tauri::command]
pub async fn project_extensions<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    added: Vec<String>,
    removed: Vec<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let choice = crate::editor::ext::clean_choice(&added, &removed);
    crate::projects::settings::update(&state.kit, &project, |s| s.extensions = choice);
    crate::editor::window::refresh_hidden(&app, &state.kit);
    let editor = Arc::clone(&state.editor);
    // Busy installing: the watch of the open projects asks again (app/mod.rs).
    tauri::async_runtime::spawn_blocking(move || {
        editor.sync_extensions();
    })
    .await
    .or_fail("extensions")
}
