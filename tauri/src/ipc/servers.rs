//! Shared Web Server pools (`servers/mod.rs`): list, create/edit manual
//! profiles, start/stop/reload VirtualHost pools, and assign projects.

use super::*;
use crate::servers::{WebServerEngine, WebServerProfileInput, WebServersView};

/// With the VM on (D-175) a pool never starts: it would run the projects'
/// code on the PC, out of their VMs. Each site runs in its project's VM,
/// started from the project; the pools are the box without the VM.
fn refused_with_vm(kit: &Kit, host: &crate::env::Host, env: &KitEnv, lang: &str) -> CmdResult<()> {
    if crate::isolation::status(kit, host, env, false).level == crate::isolation::Level::Vm {
        return Err(crate::common::i18n::t(
            lang,
            "Con la VM attiva ogni sito gira nella VM del suo progetto: avvialo dalla pagina del progetto. I server condivisi servono quando si lavora senza VM.",
            &[],
        )
        .into());
    }
    Ok(())
}

#[tauri::command]
pub async fn servers_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    fresh: Option<bool>,
) -> CmdResult<WebServersView> {
    ensure_main(&webview)?;
    // `fresh` (the refresh button) reads the projects again; the page's
    // poll leaves it out and gets the cached scan (BUG-07).
    let (servers, lang, fresh) = (
        Arc::clone(&state.servers),
        state.lang(),
        fresh.unwrap_or(false),
    );
    tauri::async_runtime::spawn_blocking(move || servers.list(lang, fresh))
        .await
        .or_fail("servers")
}

// save and remove wait for `store_lock` (and remove for `op_lock`, held up
// to seconds by a start or reload): off the IPC thread, as server_stop.
#[tauri::command]
pub async fn server_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    profile: WebServerProfileInput,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (servers, lang) = (Arc::clone(&state.servers), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        servers.save_profile(profile, lang).map_err(CmdError::from)
    })
    .await
    .or_fail("servers")?
}

#[tauri::command]
pub async fn server_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let servers = Arc::clone(&state.servers);
    tauri::async_runtime::spawn_blocking(move || servers.remove_profile(&id).or_fail("servers"))
        .await
        .or_fail("servers")?
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
    let (host, base) = (state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        refused_with_vm(&kit, &host, &base, lang)?;
        let view = servers.list(lang, true);
        let Some(pool) = view.pools.iter().find(|p| p.id == id) else {
            return Err(fail("servers", format!("unknown server pool {id}")));
        };
        let mut wanted = vec![pool.engine.as_str()];
        if pool.php_line.is_some() {
            wanted.push("php");
        }
        let env = ensure(&app, &task, &kit, lang, &wanted)?;
        servers
            .start(&id, &env, lang)
            .map_err(|e| CmdError::from(e.to_string()))
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
    let (kit, host) = (state.kit.clone(), state.host.clone());
    tauri::async_runtime::spawn_blocking(move || {
        refused_with_vm(&kit, &host, &env, lang)?;
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
