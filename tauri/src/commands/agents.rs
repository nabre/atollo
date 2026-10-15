//! Claude Code, Codex and Antigravity: state, usage, launch, sign-out.

use super::*;

/// Async, so off the window's thread: the page asks every few seconds and
/// the processes and sign-ins take a moment (a sync command would hold the
/// window's clicks meanwhile).
#[tauri::command]
pub async fn agents_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::agents::AgentInfo>, String> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::agents::status(&kit, &env))
        .await
        .map_err(|e| fail("agents", e))
}

/// Plan limits and usage of the installed agents (`stats.rs`). `force`:
/// the "Refresh" button, which asks the servers sooner.
#[tauri::command]
pub async fn agents_stats<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
) -> Result<crate::stats::Report, String> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::stats::collect(&kit, &env, lang, force))
        .await
        .map_err(|e| fail("stats", e))
}

/// Opens an agent in a kit console in the project (never elsewhere).
#[tauri::command]
pub async fn agent_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    if !crate::agents::IDS.contains(&agent.as_str()) {
        return Err(format!("unknown agent: {agent}"));
    }
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let wanted: Vec<&str> = crate::needs::of_agent(&kit, &agent).into_iter().collect();
        let env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &wanted)?);
        crate::agents::launch(&app, &kit, &env, &agent, &dir, lang)
            .map_err(|e| fail("agent", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("agent", e))?
}

#[tauri::command]
pub fn agent_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::agents::sign_out(&state.kit, &state.env(), &agent)
        .map_err(|e| fail("sign out", format!("{e:#}")))
}
