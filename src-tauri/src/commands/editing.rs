//! The editor, the kit's console and a project's editor settings and
//! extensions.

use super::*;

#[tauri::command]
pub fn editor_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<editor::Status, String> {
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_settings::opened(&state.kit, &project);
    let title = project.clone();
    let editor = Arc::clone(&state.editor);
    let (handle, task, kit) = (app.clone(), Arc::clone(&state.setup), state.kit.clone());
    let lang = state.lang();
    let (port, token) = tauri::async_runtime::spawn_blocking(move || {
        ensure(&handle, &task, &kit, lang, &[])?;
        editor
            .ensure_running()
            .map_err(|e| fail("editor start", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("editor", e))??;
    let _ = app.emit_to(MAIN, "editor-status", state.editor.status());
    let url = editor::url(port, &token, Some(&dir));
    editor_window::open(
        &app,
        &url,
        port,
        &title,
        &dir,
        &state.kit.webview_data(),
        lang,
    )
    .map_err(|e| fail("editor window", format!("{e:#}")))
}

#[tauri::command]
pub async fn editor_restart<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let editor = Arc::clone(&state.editor);
    tauri::async_runtime::spawn_blocking(move || {
        editor.stop();
        editor.ensure_running().map(|_| ())
    })
    .await
    .map_err(|e| fail("editor", e))?
    .map_err(|e| fail("editor restart", format!("{e:#}")))?;
    let _ = app.emit_to(MAIN, "editor-status", state.editor.status());
    Ok(())
}

#[tauri::command]
pub async fn terminal_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_settings::opened(&state.kit, &project);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &[])?);
        terminal::open_console(&app, &kit, &env, &dir)
            .map_err(|e| fail("terminal", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("terminal", e))?
}

/// The project's editor settings in its .vscode\settings.json (D-78).
#[tauri::command]
pub fn project_editor_settings<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::project_vscode::EditorSettingsView, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::project_vscode::get(&dir))
}

#[tauri::command]
pub fn project_editor_settings_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    settings: crate::project_vscode::EditorSettings,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_vscode::set(&dir, &settings)
        .map_err(|e| fail("editor settings", format!("{e:#}")))
}

/// The kit's matrix of editor extensions by language ("Linguaggi e strumenti").
#[tauri::command]
pub fn extensions_matrix<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<crate::editor_ext::MatrixInfo, String> {
    ensure_main(&webview)?;
    Ok(crate::editor_ext::matrix_info(&state.kit))
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let choice = crate::editor_ext::clean_choice(&added, &removed);
    crate::project_settings::update(&state.kit, &project, |s| s.extensions = choice);
    crate::editor_window::refresh_hidden(&app, &state.kit);
    let editor = Arc::clone(&state.editor);
    // Busy installing: the watch of the open projects asks again (main.rs).
    tauri::async_runtime::spawn_blocking(move || {
        editor.sync_extensions();
    })
    .await
    .map_err(|e| fail("extensions", e))
}
