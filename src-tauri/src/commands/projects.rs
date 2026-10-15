//! Projects: list, creation, clone, deletion, actions, site, status and MCP
//! servers.

use super::*;

/// The projects with what each one is and how it starts (detection reads
/// files: off the UI thread).
#[tauri::command]
pub async fn projects_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::projects::ProjectInfo>, String> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::projects::list(&kit, lang))
        .await
        .map_err(|e| fail("projects", e))
}

/// A new empty project in `projects\`.
#[tauri::command]
pub fn project_create<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    ensure_main(&webview)?;
    let project = crate::projects::create(&state.kit, &name, state.lang())
        .map(|d| {
            d.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .map_err(|e| format!("{e:#}"))?;
    // It keeps the newest version of each language: its code is written
    // for those, newer ones installed later do not change it.
    crate::versions::pin_newest(&state.kit, &project);
    Ok(project)
}

/// Deletes a project whose folder is empty.
#[tauri::command]
pub fn project_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::projects::delete_empty(&state.kit, &project, state.lang()).map_err(|e| format!("{e:#}"))
}

/// Clones `owner/repo` or an `https://` Git URL into `projects\<name>`.
#[tauri::command]
pub async fn project_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    source: String,
    name: Option<String>,
) -> Result<String, String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        let dir =
            crate::projects::clone(&kit, &env, &source, name.as_deref(), lang, token.as_deref())
                .map_err(|e| fail("clone", format!("{e:#}")))?;
        let project = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        settle_project(&app, &task, &kit, lang, &project, &dir);
        Ok(project)
    })
    .await
    .map_err(|e| fail("clone", e))?
}

/// Adds or changes an action of the project in its panel.json (D-78); it
/// applies from the next start of that action.
#[tauri::command]
pub fn project_action_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    action: crate::detect::ActionEdit,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::save_action(&dir, &action, state.lang()).map_err(|e| format!("{e:#}"))
}

/// The project's own MCP servers (.mcp.json, D-78).
#[tauri::command]
pub fn project_mcp<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::project_mcp::McpView, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::project_mcp::get(&dir))
}

#[tauri::command]
pub fn project_mcp_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    servers: Vec<crate::project_mcp::McpServer>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_mcp::set(&dir, &servers).map_err(|e| fail("project mcp", format!("{e:#}")))
}

/// Removes an action written in panel.json (a detected one comes back).
#[tauri::command]
pub fn project_action_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::remove_action(&dir, &id, state.lang()).map_err(|e| format!("{e:#}"))
}

/// The action that "Avvia" starts.
#[tauri::command]
pub fn project_action_default<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::set_default_action(&dir, &id, state.lang()).map_err(|e| format!("{e:#}"))
}

/// "Servi come sito web": a folder of the project served by the file server
/// (`static`) or by PHP (`php`), or the site removed (`none`).
#[tauri::command]
pub fn project_site<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    folder: String,
    kind: crate::detect::SiteKind,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let key = crate::run::key_of(&project, crate::detect::SITE_ACTION);
    if runner
        .list()
        .iter()
        .any(|r| r.key == key && matches!(r.state, RunState::Starting | RunState::Running))
    {
        return Err(tr(
            &state,
            "Ferma prima il sito web: la modifica vale dal prossimo avvio.",
            &[],
        ));
    }
    crate::detect::set_site(&dir, &folder, kind, state.lang()).map_err(|e| format!("{e:#}"))
}

/// Installs the kit components a project needs and are missing (its
/// `missing`), saved as chosen like those of an action. Waits for the end.
#[tauri::command]
pub async fn project_install_missing<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let missing = crate::projects::info(&kit, &dir, lang).missing;
        let wanted: Vec<&str> = missing.iter().map(String::as_str).collect();
        ensure(&app, &task, &kit, lang, &wanted).map(|_| ())
    })
    .await
    .map_err(|e| fail("install", e))?
}

/// Git state and agent instructions of a project, for its page.
#[tauri::command]
pub async fn project_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::projects::ProjectStatus, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let root = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || crate::projects::status(&dir, &root))
        .await
        .map_err(|e| fail("project", e))
}
