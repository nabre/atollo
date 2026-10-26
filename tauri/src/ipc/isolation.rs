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

/// Atollo's sandboxes and the room their state takes (D-169).
#[tauri::command]
pub async fn vms_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<crate::isolation::VmList> {
    ensure_main(&webview)?;
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        crate::isolation::vm_list(&kit, &host, &env).or_fail_chain("vms")
    })
    .await
    .or_fail("vms")?
}

/// Removes one of Atollo's sandboxes with its disk and kit (D-169).
#[tauri::command]
pub async fn vm_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        crate::isolation::remove_vm(&kit, &host, &env, &name).or_fail_chain("vms")
    })
    .await
    .or_fail("vms")?
}

/// The notice's «Lancia»: `fix` with the `sbx` found, in a new session of
/// the app's terminal in the projects' folder.
#[tauri::command]
pub async fn isolation_run<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    fix: crate::isolation::Fix,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, host, env, lang) = (
        state.kit.clone(),
        state.host.clone(),
        state.env(),
        state.lang(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let dir = kit.projects();
        let Some((launch, env)) = crate::isolation::fix_launch(&kit, &host, &env, fix, &dir) else {
            return Err(
                crate::common::i18n::t(lang, "Docker Sandboxes (sbx) non è nel kit.", &[]).into(),
            );
        };
        // A kit's program (`sbx`) the user runs, on the PC.
        let at = crate::levels::labels::SessionLabel::pc(crate::levels::Level::Manual);
        crate::editor::terminal::open_launch(&app, &env, &dir, launch, at).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}
