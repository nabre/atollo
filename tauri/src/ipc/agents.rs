//! Claude Code, Codex, Antigravity and Pi: state, usage, launch (only in
//! the VM, D-178, D-179), sign-out of what the kit's data still holds.

use super::*;

/// The agents with the VM as the status says now (LEVELS.md §7): each
/// available only while it is on, else with the reason. Async, so off the
/// window's thread: the page asks every few seconds and the status may
/// ask `sbx` (a sync command would hold the window's clicks meanwhile).
#[tauri::command]
pub async fn agents_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::agents::AgentInfo>> {
    ensure_main(&webview)?;
    let (kit, host, env, lang) = (
        state.kit.clone(),
        state.host.clone(),
        state.env(),
        state.lang(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let vm =
            crate::levels::agents::vm_state(&crate::isolation::status(&kit, &host, &env, false));
        crate::agents::status(&kit, &vm, lang)
    })
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
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::agents::stats::collect(&kit, lang, force))
        .await
        .or_fail("stats")
}

/// Opens an agent on the project, always in the VM (D-178, D-179): in the
/// project's sandbox with its build for Linux (D-184). Without the VM no agent
/// starts: the error says why the VM is off and what to do. Never on the
/// PC.
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
    let lang = state.lang();
    let dir = project_dir(&state.kit, Some(&project), lang)?;
    // Started again after "Rimuovi l'unità": the user goes on, so do the
    // agents' background alignments.
    state.closing.reopen();
    crate::projects::settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (host, kit, kit_env) = (state.host.clone(), state.kit.clone(), state.env());
    let fx = super::levels::AppEffects::of(&app, &state, kit_env.clone());
    tauri::async_runtime::spawn_blocking(move || {
        // Level 3 (D-179): only in a VM (R12, R13), refused without it (R1)
        // with why and what to do; made ready by `levels::prepare` (Pi's or
        // Antigravity's build for the VM's Linux, the kit's tools, the
        // databases, Pi's local models, the Gemini key through `sbx`'s
        // proxy); Pi's session keeps the models' port open while it runs
        // (`levels::start::open_terminal`).
        let req = agent_request(&kit, &agent, &dir)?;
        let ready = super::levels::ready_now(&fx, &kit, &kit_env, &req, &[], lang, &host)?;
        crate::levels::start::open_terminal(&app, &kit, &ready, lang).or_fail_chain("agent")
    })
    .await
    .or_fail("agent")?
}

/// The request of `agent` (its catalog id) asked from the project's page
/// for the project in `dir` (E10, `Origin::AgentButton`).
pub(crate) fn agent_request(
    kit: &Kit,
    agent: &str,
    dir: &std::path::Path,
) -> CmdResult<crate::levels::Request> {
    let agent = crate::levels::Agent::from_id(agent)
        .ok_or_else(|| CmdError::from(format!("unknown agent: {agent}")))?;
    Ok(crate::levels::Request {
        origin: crate::levels::Origin::AgentButton { agent },
        place: crate::levels::place::place_of(kit, dir),
    })
}

/// The ready MCP servers (N-7, D-156), the kit's and the projects'.
#[tauri::command]
pub fn mcp_presets<R: Runtime>(
    webview: Webview<R>,
) -> CmdResult<Vec<crate::agents::presets::Preset>> {
    ensure_main(&webview)?;
    Ok(crate::agents::presets::all())
}

/// The kit's MCP servers (`data\agents\mcp.json`), every agent's.
#[tauri::command]
pub fn kit_mcp<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<crate::projects::mcp::McpView> {
    ensure_main(&webview)?;
    Ok(crate::projects::mcp::kit_get(&state.kit))
}

/// Writes the kit's MCP servers, then aligns the agents so that each has
/// them at once (not only at its next start).
#[tauri::command]
pub async fn kit_mcp_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    servers: Vec<crate::projects::mcp::McpServer>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::projects::mcp::kit_set(&kit, &servers)?;
        let report = crate::agents::sync::run(&kit, None);
        for w in &report.warnings {
            crate::common::log::warn(w);
        }
        anyhow::Ok(())
    })
    .await
    .or_fail("kit mcp")?
    .or_fail_chain("kit mcp")
}

/// «Esci»: the sign-in of the PC an earlier Atollo saved in the kit's
/// data, removed (files only: no agent's program runs on the PC, LEVELS.md
/// E17).
#[tauri::command]
pub fn agent_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::agents::sign_out(&state.kit, &agent).or_fail_chain("sign out")
}

/// `id` as an agent Atollo knows.
fn agent_of(id: &str) -> CmdResult<crate::levels::Agent> {
    crate::levels::Agent::from_id(id).ok_or_else(|| format!("unknown agent: {id}").into())
}

/// `name` checked as an account's name, with the reason in the user's
/// language.
fn account_name(state: &AppState, name: &str) -> CmdResult<String> {
    let name = name.trim();
    if !crate::agents::accounts::valid_name(name) {
        return Err(tr(
            state,
            "Nome dell'account non valido: usa lettere, cifre, trattini e spazi, senza un punto all'inizio o alla fine.",
            &[],
        )
        .into());
    }
    Ok(name.to_owned())
}

/// The agents' accounts (D-185), each with the projects that use it. Off
/// the window's thread: it reads every project's settings.
#[tauri::command]
pub async fn agent_accounts<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::agents::accounts::AgentAccounts>> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || crate::agents::accounts::list(&kit))
        .await
        .or_fail("accounts")
}

/// A new account for `agent`.
#[tauri::command]
pub fn agent_account_create<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let name = account_name(&state, &name)?;
    crate::agents::accounts::create(&state.kit, agent_of(&agent)?, &name)
        .map(|_| ())
        .or_fail_chain("account")
}

/// `agent`'s account `from` renamed `to`; the projects that used it follow.
#[tauri::command]
pub fn agent_account_rename<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    from: String,
    to: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let to = account_name(&state, &to)?;
    crate::agents::accounts::rename(&state.kit, agent_of(&agent)?, &from, &to)
        .or_fail_chain("account")
}

/// `agent`'s account `name` deleted with its sign-in; refused for the
/// default one and while a project uses it.
#[tauri::command]
pub fn agent_account_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let agent = agent_of(&agent)?;
    match crate::agents::accounts::delete_refused(&state.kit, agent, &name) {
        Some(users) if users.is_empty() => {
            return Err(tr(&state, "L'account predefinito non si elimina.", &[]).into())
        }
        Some(users) => {
            return Err(tr(
                &state,
                "L'account {0} è usato da: {1}. Scegli un altro account in quei progetti, poi eliminalo.",
                &[&name, &users.join(", ")],
            )
            .into())
        }
        None => {}
    }
    crate::agents::accounts::delete(&state.kit, agent, &name).or_fail_chain("account")
}

/// «Esci» for `agent`'s account `name`: its credentials removed.
#[tauri::command]
pub fn agent_account_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::agents::accounts::sign_out(&state.kit, agent_of(&agent)?, &name)
        .or_fail_chain("sign out")
}

/// The account `project` uses for each agent (its id → the account).
#[tauri::command]
pub fn project_agent_accounts<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<std::collections::BTreeMap<String, String>> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::levels::Agent::ALL
        .into_iter()
        .filter(|a| crate::agents::accounts::env_var(*a).is_some())
        .map(|a| {
            (
                a.id().to_owned(),
                crate::agents::accounts::chosen(&state.kit, &project, a),
            )
        })
        .collect())
}

/// `project` uses `agent`'s account `name` (made if new) from the agent's
/// next session in its sandbox.
#[tauri::command]
pub fn project_agent_account_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    agent: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let (agent, name) = (agent_of(&agent)?, account_name(&state, &name)?);
    crate::agents::accounts::create(&state.kit, agent, &name).or_fail_chain("account")?;
    crate::agents::accounts::set(&state.kit, &project, agent, &name);
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/agents.rs"]
mod tests;
