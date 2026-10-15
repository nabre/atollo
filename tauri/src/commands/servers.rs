//! Shared Web Server pools (`servers/mod.rs`): list, create/edit manual
//! profiles, start/stop/reload VirtualHost pools, and assign projects.

use super::*;
use crate::servers::{WebServerEngine, WebServerProfileInput, WebServersView};

#[tauri::command]
pub async fn servers_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<WebServersView> {
    ensure_main(&webview)?;
    let (servers, lang) = (Arc::clone(&state.servers), state.lang());
    tauri::async_runtime::spawn_blocking(move || servers.list(lang))
        .await
        .or_fail("servers")
}

#[tauri::command]
pub fn server_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    profile: WebServerProfileInput,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    state.servers.save_profile(profile, state.lang())?;
    Ok(())
}

#[tauri::command]
pub fn server_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    state.servers.remove_profile(&id).or_fail("servers")
}

#[tauri::command]
pub async fn server_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (task, kit, lang, servers) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.lang(),
        Arc::clone(&state.servers),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let view = servers.list(lang);
        let Some(pool) = view.pools.iter().find(|p| p.id == id) else {
            return Err(fail("servers", format!("unknown server pool {id}")));
        };
        let mut wanted = vec![pool.engine.as_str()];
        if pool.php_line.is_some() {
            wanted.push("php");
        }
        let env = ensure(&app, &task, &kit, lang, &wanted)?;
        servers.start(&id, &env, lang).map_err(|e| CmdError::from(e.to_string()))
    })
    .await
    .or_fail("servers")?
}

#[tauri::command]
pub async fn server_stop<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let servers = Arc::clone(&state.servers);
    tauri::async_runtime::spawn_blocking(move || {
        servers.stop(&id);
    })
    .await
    .or_fail("servers")
}

#[tauri::command]
pub async fn server_reload<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (servers, env, lang) = (Arc::clone(&state.servers), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        servers
            .reload(&id, &env, lang)
            .map_err(|e| CmdError::from(e.to_string()))
    })
    .await
    .or_fail("servers")?
}

#[tauri::command]
pub async fn server_assign_project<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    folder: String,
    engine: WebServerEngine,
    server_id: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (servers, env, lang) = (Arc::clone(&state.servers), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        servers
            .assign_project(&project, &folder, engine, server_id, &env, lang)
            .map_err(|e| CmdError::from(e.to_string()))
    })
    .await
    .or_fail("servers")?
}
