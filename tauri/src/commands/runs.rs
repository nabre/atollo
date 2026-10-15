//! Projects' actions started by the kit (projects/run.rs): start, stop, log, ports.

use super::*;

#[tauri::command]
pub fn run_list<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> Result<Vec<crate::projects::run::RunInfo>, String> {
    ensure_main(&webview)?;
    Ok(runner.list())
}

/// Starts an action of a project (detected again: panel.json may have changed).
#[tauri::command]
pub async fn run_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    action: String,
) -> Result<crate::projects::run::RunInfo, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let runner = Arc::clone(&runner);
    tauri::async_runtime::spawn_blocking(move || {
        let detected = crate::detect::detect(&dir, lang);
        let Some(a) = detected.actions.iter().find(|a| a.id == action) else {
            return Err(crate::i18n::t(
                lang,
                "Azione non trovata: premi Aggiorna",
                &[],
            ));
        };
        let wanted: Vec<&str> = crate::setup::needs::of_action(a).into_iter().collect();
        let env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &wanted)?);
        runner
            .start(&project, &dir, a, &env, lang)
            .map_err(|e| fail("run", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("run", e))?
}

#[tauri::command]
pub fn run_stop<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    runner.stop(&key);
    Ok(())
}

#[tauri::command]
pub fn run_log<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<Vec<String>, String> {
    ensure_main(&webview)?;
    Ok(runner.log(&key))
}

#[tauri::command]
pub fn run_clear<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    runner.clear(&key);
    Ok(())
}

/// The ports remembered for the web actions, and the locked ones.
#[tauri::command]
pub fn run_ports<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> Result<crate::projects::run::Ports, String> {
    ensure_main(&webview)?;
    Ok(runner.ports())
}

/// The user's port for an action (`None`: automatic again).
#[tauri::command]
pub fn run_set_port<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    key: String,
    port: Option<u16>,
) -> Result<crate::projects::run::Ports, String> {
    ensure_main(&webview)?;
    runner
        .set_port(&key, port, state.lang())
        .map_err(|e| format!("{e:#}"))
}
