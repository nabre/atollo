//! How shut in the projects' programs are (D-167): the status bar's notice.

use super::*;

/// Async: `sbx` may take a few seconds to answer. `force`: the notice's
/// "Ricontrolla", which asks again within the minute.
#[tauri::command]
pub async fn isolation_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
) -> CmdResult<crate::isolation::Status> {
    ensure_main(&webview)?;
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::isolation::status(&kit, &host, &env, force))
        .await
        .or_fail("isolation")
}
