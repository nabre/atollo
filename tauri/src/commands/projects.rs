//! Projects: list, creation, clone, deletion, actions, site, status and MCP
//! servers.

use super::*;

/// The projects with what each one is and how it starts (detection reads
/// files: off the UI thread).
#[tauri::command]
pub async fn projects_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::projects::ProjectInfo>> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::projects::list(&kit, lang))
        .await
        .or_fail("projects")
}

/// A new empty project in `projects\`.
#[tauri::command]
pub fn project_create<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let project = crate::projects::create(&state.kit, &name, state.lang()).map(|d| {
        d.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    })?;
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
) -> CmdResult<()> {
    ensure_main(&webview)?;
    Ok(crate::projects::delete_empty(
        &state.kit,
        &project,
        state.lang(),
    )?)
}

/// Clones `owner/repo` or an `https://` Git URL into `projects\<name>`.
#[tauri::command]
pub async fn project_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    source: String,
    name: Option<String>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        let dir =
            crate::projects::clone(&kit, &env, &source, name.as_deref(), lang, token.as_deref())
                .or_fail_chain("clone")?;
        let project = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        settle_project(&app, &task, &kit, lang, &project, &dir);
        Ok(project)
    })
    .await
    .or_fail("clone")?
}

/// Adds or changes an action of the project in its panel.json (D-78); it
/// applies from the next start of that action.
#[tauri::command]
pub fn project_action_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    action: crate::detect::ActionEdit,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::detect::save_action(&dir, &action, state.lang())?)
}

/// The project's own MCP servers (.mcp.json, D-78).
#[tauri::command]
pub fn project_mcp<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::mcp::McpView> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::projects::mcp::get(&dir))
}

#[tauri::command]
pub fn project_mcp_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    servers: Vec<crate::projects::mcp::McpServer>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::mcp::set(&dir, &servers).or_fail_chain("project mcp")
}

/// Removes an action written in panel.json (a detected one comes back).
#[tauri::command]
pub fn project_action_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::detect::remove_action(&dir, &id, state.lang())?)
}

/// The action that "Avvia" starts.
#[tauri::command]
pub fn project_action_default<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::detect::set_default_action(&dir, &id, state.lang())?)
}

/// "Servi come sito web": a folder of the project served by the file server
/// (`static`), PHP (`php`), Apache (`apache`) or Nginx (`nginx`), or the site removed (`none`).
#[tauri::command]
pub fn project_site<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    folder: String,
    kind: crate::detect::SiteKind,
    options: Option<crate::detect::SiteOptions>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let key = crate::projects::run::key_of(&project, crate::detect::SITE_ACTION);
    if runner
        .list()
        .iter()
        .any(|r| r.key == key && matches!(r.state, RunState::Starting | RunState::Running))
    {
        return Err(tr(
            &state,
            "Ferma prima il sito web: la modifica vale dal prossimo avvio.",
            &[],
        )
        .into());
    }
    Ok(crate::detect::set_site_with_options(
        &dir,
        &folder,
        kind,
        &options.unwrap_or_default(),
        state.lang(),
    )?)
}

/// Installs the kit components a project needs and are missing (its
/// `missing`), saved as chosen like those of an action. Waits for the end.
#[tauri::command]
pub async fn project_install_missing<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let missing = crate::projects::info(&kit, &dir, lang).missing;
        let wanted: Vec<&str> = missing.iter().map(String::as_str).collect();
        ensure(&app, &task, &kit, lang, &wanted).map(|_| ())
    })
    .await
    .or_fail("install")?
}

/// Git state and agent instructions of a project, for its page.
#[tauri::command]
pub async fn project_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::ProjectStatus> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let root = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || crate::projects::status(&dir, &root))
        .await
        .or_fail("project")
}
