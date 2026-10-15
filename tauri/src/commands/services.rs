//! The kit's database servers (db/services.rs, D-73).

use super::*;

/// The database servers (D-73). Async, so off the window's thread: the
/// pages ask every few seconds and a server that does not answer yet takes
/// up to 0.8 s (a sync command would hold the window's clicks meanwhile).
#[tauri::command]
pub async fn services_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || services.list())
        .await
        .map_err(|e| fail("service", e))
}

/// Starts a database server, installing it first if needed; waits until it
/// answers.
#[tauri::command]
pub async fn service_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    if !crate::db::services::SERVERS.contains(&id.as_str()) {
        return Err(fail("service", format!("unknown server {id}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[id.as_str()])?;
        services
            .start(&id, &env)
            .map_err(|e| start_error(lang, &e))?;
        Ok(services.list())
    })
    .await
    .map_err(|e| fail("service", e))?
}

pub(super) fn start_error(lang: &str, e: &anyhow::Error) -> String {
    match e.downcast_ref::<crate::db::services::PortInUse>() {
        Some(p) => crate::i18n::t(
            lang,
            "La porta {0} è già usata da un altro programma: cambiala qui sotto.",
            &[&p.0.to_string()],
        ),
        None => fail("service", format!("{e:#}")),
    }
}

#[tauri::command]
pub async fn service_stop<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        services.stop(&id);
        services.list()
    })
    .await
    .map_err(|e| fail("service", e))
}

/// Port (null: the default) and "start with Atollo" of a server; the port
/// applies at the next start.
/// The version a database server runs (None: the newest installed), while
/// it is stopped; each version keeps its own data (D-81).
#[tauri::command]
pub async fn service_line<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    line: Option<String>,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        services
            .set_line(&id, line.as_deref())
            .map_err(|e| fail("service", format!("{e:#}")))?;
        Ok(services.list())
    })
    .await
    .map_err(|e| fail("service", e))?
}

#[tauri::command]
pub async fn service_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    port: Option<u16>,
    autostart: bool,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        services
            .set(&id, port, autostart)
            .map_err(|e| fail("service", format!("{e:#}")))?;
        Ok(services.list())
    })
    .await
    .map_err(|e| fail("service", e))?
}
