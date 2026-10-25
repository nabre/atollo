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
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || crate::agents::status(&kit))
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
    // Started again after "Rimuovi l'unità": the user goes on, so do the
    // agents' background alignments.
    state.closing.reopen();
    crate::projects::settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    let (host, kit_env) = (state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        // In a sandbox of its own when every project runs in a microVM
        // (D-167): `sbx`'s agent, its keys added by `sbx`'s proxy; an agent
        // `sbx` has none for is not started on the PC instead.
        let base = crate::versions::project_env(&kit, &project, kit_env.clone());
        if let Some(vm) = crate::isolation::vm_for(&kit, &host, &base, &project, &dir) {
            let Some(vm) = vm.for_agent(&project, &agent) else {
                return Err(crate::common::i18n::t(
                    lang,
                    "{0} non ha una sandbox di Docker Sandboxes: nella VM si usano Claude Code e Codex.",
                    &[&agent],
                )
                .into());
            };
            let (vm, args) = agent_vm_ready(
                &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm,
            )?;
            let launch = vm.launch(&kit, args, &agent, lang).or_fail_chain("vm")?;
            return crate::editor::terminal::open_launch(&app, &vm.session_env(&base), &dir, launch)
                .or_fail_chain("agent");
        }
        let wanted: Vec<&str> = crate::setup::needs::of_agent(&kit, &agent, Some(&dir))
            .into_iter()
            .collect();
        let mut env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &wanted)?);
        // The project's databases of the ready MCP servers (N-7): in the
        // agent's environment, never in a file. Never the GitHub token: the
        // kit's github server reads it itself (SEC-V7-01).
        for (var, value) in
            crate::agents::presets::launch_vars(&kit, &env, &dir, || services.project_dbs(&project))
        {
            env.set(var, value);
        }
        crate::agents::launch(&app, &kit, &env, &agent, &dir, lang).or_fail_chain("agent")
    })
    .await
    .or_fail("agent")?
}

/// The sandbox of an agent of `project` made ready like the project's VM
/// (D-175): the kit's tools for Linux, the project's databases and the
/// ports to them, started; with the arguments of its session. If an editor
/// has a session for the project, its profile of the agent gets them too.
#[allow(clippy::too_many_arguments)]
pub(crate) fn agent_vm_ready<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    services: &crate::db::services::Services,
    kit: &Kit,
    host: &crate::env::Host,
    lang: &str,
    project: &str,
    dir: &std::path::Path,
    kit_env: &KitEnv,
    vm: crate::isolation::Vm,
) -> CmdResult<(crate::isolation::Vm, Vec<String>)> {
    let env = crate::versions::project_env(kit, project, kit_env.clone());
    let vm = super::terminals::project_vm_ready(
        app, task, services, kit, host, lang, project, dir, kit_env, vm,
    )?;
    vm.start(&env, lang).or_fail_chain("vm")?;
    let args = vm.agent_session_args(&env).or_fail_chain("vm")?;
    Ok((vm, args))
}

/// The agents' sandboxes the editor's profiles ask for (`vm.ps1`,
/// `isolation::session::take_requests`), made ready one by one while the
/// app runs.
pub(crate) fn watch_agent_requests<R: Runtime>(app: AppHandle<R>) {
    use crate::isolation::session;
    use tauri::Manager;
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_millis(500));
        let state = app.state::<AppState>();
        for (project, agent) in session::take_requests(&state.kit) {
            let (kit, host, kit_env, lang) = (
                state.kit.clone(),
                state.host.clone(),
                state.env(),
                state.lang(),
            );
            let (task, services) = (Arc::clone(&state.setup), Arc::clone(&state.services));
            let app = app.clone();
            std::thread::spawn(move || {
                let ready = project_dir(&kit, Some(&project), lang)
                    .map_err(|e| e.to_string())
                    .and_then(|dir| {
                        let base = crate::versions::project_env(&kit, &project, kit_env.clone());
                        let vm = crate::isolation::vm_for(&kit, &host, &base, &project, &dir)
                            .and_then(|vm| vm.for_agent(&project, &agent))
                            .ok_or_else(|| "no VM".to_owned())?;
                        agent_vm_ready(
                            &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm,
                        )
                        .map_err(|e| e.to_string())
                    });
                let noted = match ready {
                    Ok((_, args)) => session::write_agent(&kit, &project, &agent, &args),
                    Err(why) => session::agent_failed(&kit, &project, &agent, &why),
                };
                if let Err(e) = noted {
                    crate::common::log::warn(&format!("editor session {project} {agent}: {e:#}"));
                }
            });
        }
    });
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
        // Nothing to rebuild once saved: the token is read when Claude
        // Code is started (`agents::launch_env`), not kept in the kit's
        // environment (SEC-V6-02).
        crate::agents::claude_token::link(&app, &kit, &env, lang, || {})
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
    Ok(())
}

/// "Esci". Async, so off the window's thread: `codex logout` and the save
/// of Antigravity's login take a moment (BUG-V6-29).
#[tauri::command]
pub async fn agent_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::agents::sign_out(&kit, &env, &agent, lang).or_fail_chain("sign out")
    })
    .await
    .or_fail("sign out")?
}
