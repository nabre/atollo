//! Shared Web Server pools (`servers/mod.rs`): list, create/edit manual
//! profiles, start/stop/reload VirtualHost pools, and assign projects.

use super::*;
use crate::servers::{WebServerEngine, WebServerProfileInput, WebServersView};

/// The request of the Web Server pool `id` (E6–E8, `Origin::Pool`): its
/// folder `data/servers/<id>`, no project's.
pub(crate) fn pool_request(kit: &Kit, id: &str) -> crate::levels::Request {
    crate::levels::Request {
        origin: crate::levels::Origin::Pool {
            pool: id.to_owned(),
        },
        place: crate::levels::place::place_of(kit, &kit.data().join("servers").join(id)),
    }
}

/// The environment the pool `id` starts with, from the kit's `base`, once
/// `decide` places it (D-179): on the PC without the VM, sealed for no
/// project (R6); refused with the VM on, each site running in its
/// project's VM (R2).
pub(crate) fn pool_env(
    kit: &Kit,
    host: &crate::env::Host,
    base: &KitEnv,
    id: &str,
    lang: &str,
) -> CmdResult<KitEnv> {
    let req = pool_request(kit, id);
    match super::levels::place_now(kit, base, &req, None, lang, host)? {
        super::levels::Placed::Pc { env, .. } => Ok(env),
        super::levels::Placed::Vm { .. } => Err("levels: a pool is never in the VM".into()),
    }
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
        // Refused before anything is installed for it.
        pool_env(&kit, &host, &base, &id, lang)?;
        let view = servers.list(lang, true);
        let Some(pool) = view.pools.iter().find(|p| p.id == id) else {
            return Err(fail("servers", format!("unknown server pool {id}")));
        };
        let mut wanted = vec![pool.engine.as_str()];
        if pool.php_line.is_some() {
            wanted.push("php");
        }
        let env = pool_env(
            &kit,
            &host,
            &ensure(&app, &task, &kit, lang, &wanted)?,
            &id,
            lang,
        )?;
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
        let env = pool_env(&kit, &host, &env, &id, lang)?;
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
