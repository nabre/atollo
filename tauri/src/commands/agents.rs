//! Claude Code, Codex and Antigravity: state, usage, launch, sign-out.

use super::*;

/// Async, so off the window's thread: the page asks every few seconds and
/// the processes and sign-ins take a moment (a sync command would hold the
/// window's clicks meanwhile).
#[tauri::command]
pub async fn agents_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::agents::AgentInfo>> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::agents::status(&kit, &env))
        .await
        .or_fail("agents")
}

/// Plan limits and usage of the installed agents (`agents/stats/`). `force`:
/// the "Refresh" button, which asks the servers sooner.
#[tauri::command]
pub async fn agents_stats<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
) -> CmdResult<crate::agents::stats::Report> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::agents::stats::collect(&kit, &env, lang, force)
    })
    .await
    .or_fail("stats")
}

/// Opens an agent in a kit console in the project (never elsewhere).
#[tauri::command]
pub async fn agent_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    if !crate::agents::IDS.contains(&agent.as_str()) {
        return Err(format!("unknown agent: {agent}").into());
    }
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let wanted: Vec<&str> = crate::setup::needs::of_agent(&kit, &agent)
            .into_iter()
            .collect();
        let env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &wanted)?);
        crate::agents::launch(&app, &kit, &env, &agent, &dir, lang).or_fail_chain("agent")
    })
    .await
    .or_fail("agent")?
}

/// "Collega con un token" (Claude Code on macOS, D-115): `claude
/// setup-token` in a session of the app's terminal, its token saved in the
/// kit. Async: the session runs until the user finishes in the browser.
/// The page polls `agents_status`, so it sees the saved token by itself.
#[tauri::command]
pub async fn agent_token_link<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let saved = app.clone();
        crate::agents::claude_token::link(&app, &kit, &env, lang, move || {
            use tauri::Manager;
            token_changed(&saved.state::<AppState>());
        })
        .or_fail_chain("claude setup-token")
    })
    .await
    .or_fail("claude setup-token")?
}

/// "Salva il token": a token pasted by the user. Neither the token nor an
/// error about it is logged or echoed (save's errors never contain it).
#[tauri::command]
pub fn agent_token_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    token: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::agents::claude_token::save(&state.kit, &token).or_fail_chain("claude token")?;
    token_changed(&state);
    Ok(())
}

/// Claude Code's token saved or removed: the kit's environment is built
/// again, so that terminals and editor opened from now on get it, or no
/// longer get it (D-115).
fn token_changed(state: &AppState) {
    apply_env(state, Settings::load(&state.kit).settings);
}

#[tauri::command]
pub fn agent_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::agents::sign_out(&state.kit, &state.env(), &agent).or_fail_chain("sign out")?;
    if agent == "claude" {
        token_changed(&state);
    }
    Ok(())
}
