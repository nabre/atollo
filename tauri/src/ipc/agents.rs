//! Claude Code, Codex, Antigravity and Pi: state, usage, launch (always in
//! the VM, D-178), sign-out.

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

/// Opens an agent on the project, always in the VM (D-178): Claude Code and
/// Codex in their `sbx` sandboxes, Pi in the project's VM; Antigravity, a
/// desktop editor, never on a project. Without the VM no agent starts: the
/// error says why the VM is off and what to do. Never on the PC.
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
    if let Some(why) = refused_on_project(&agent, lang) {
        return Err(why.into());
    }
    let dir = project_dir(&state.kit, Some(&project), lang)?;
    // Started again after "Rimuovi l'unità": the user goes on, so do the
    // agents' background alignments.
    state.closing.reopen();
    crate::projects::settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (task, kit) = (Arc::clone(&state.setup), state.kit.clone());
    let services = Arc::clone(&state.services);
    let (host, kit_env) = (state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        let base = crate::versions::project_env(&kit, &project, kit_env.clone());
        let Some(vm) = crate::isolation::vm_for(&kit, &host, &base, &project, &dir) else {
            let status = crate::isolation::status(&kit, &host, &base, false);
            return Err(no_vm_for_agents(&status, lang).into());
        };
        let (vm, args) = agent_vm_ready(
            &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm, &agent,
        )?;
        // Pi's session, on the PC: while it runs, the port of the local
        // models stays open (D-178); it ends with the session, or now when
        // the session does not open.
        let pi = (agent == "pi").then(|| crate::isolation::pi_session(&vm.name));
        let launch = vm.launch(&kit, args, &agent, lang).or_fail_chain("vm")?;
        crate::editor::terminal::open_launch_then(
            &app,
            &vm.session_env(&base),
            &dir,
            launch,
            crate::levels::labels::SessionLabel::vm(crate::levels::Level::Agent, &vm.name),
            Box::new(move || drop(pi)),
        )
        .or_fail_chain("agent")
    })
    .await
    .or_fail("agent")?
}

/// Why `agent` never starts on a project, if it does not (D-178):
/// Antigravity is a desktop editor, it cannot run in a microVM; an agent
/// with no way into the VM is refused too, never started on the PC.
pub(crate) fn refused_on_project(agent: &str, lang: &str) -> Option<String> {
    if agent == "antigravity" {
        return Some(crate::common::i18n::t(
            lang,
            "Antigravity è un editor per il desktop e non può girare in una microVM: su un progetto non si apre. Nella VM usa Claude Code, Codex o Pi.",
            &[],
        ));
    }
    (!crate::isolation::vm::starts_in_vm(agent)).then(|| {
        crate::common::i18n::t(
            lang,
            "{0} non parte nella VM: nella VM si usano Claude Code, Codex e Pi.",
            &[agent],
        )
    })
}

/// The refusal of an agent without the VM (D-178): the agents start only
/// in the project's VM, then why it is off and what to do.
pub(crate) fn no_vm_for_agents(status: &crate::isolation::Status, lang: &str) -> String {
    format!(
        "{}\n{}",
        crate::common::i18n::t(
            lang,
            "Gli agenti partono solo nella VM del progetto, e la VM non è attiva.",
            &[],
        ),
        crate::isolation::why_no_vm(status, lang)
    )
}

/// The session of `agent` for `project` in the VM (D-175, D-178), with
/// `vm` the project's: Claude Code and Codex in a sandbox of their own
/// made ready like the project's VM (the kit's tools for Linux, the
/// project's databases and the ports to them), started; Pi in the
/// project's VM itself, its build for Linux installed for the VM when
/// missing (D-173), started like a command of the project's terminal.
/// With the arguments of the session. If an editor has a session for the
/// project, its profile of the agent gets them too.
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
    agent: &str,
) -> CmdResult<(crate::isolation::Vm, Vec<String>)> {
    if let Some(why) = refused_on_project(agent, lang) {
        return Err(why.into());
    }
    let env = crate::versions::project_env(kit, project, kit_env.clone());
    if agent == "pi" {
        let guest_dir = crate::isolation::vm::guest_path(kit.target().is_windows(), dir)
            .ok_or_else(|| {
                crate::common::i18n::t(
                    lang,
                    "La cartella {0} non si apre nella VM: scegli una cartella di un disco del PC.",
                    &[&dir.display().to_string()],
                )
            })?;
        let vm = super::terminals::project_vm_ready(
            app, task, services, kit, host, lang, project, dir, kit_env, vm,
        )?;
        let pi = crate::isolation::vm::guest_target(kit.target())
            .map(|t| kit.for_target(t))
            .and_then(|guest| {
                let wanted = std::iter::once(("pi".to_owned(), None)).collect();
                super::terminals::ensure_guest_tools(app, task, &guest, &wanted);
                crate::isolation::pi_in_vm(kit, &guest)
            })
            .ok_or_else(|| {
                crate::common::i18n::t(
                    lang,
                    "Pi per il Linux della VM non è installato e non si è potuto installare: guarda la barra di stato o la pagina Strumenti.",
                    &[],
                )
            })?;
        let db_ports = vm.host_ports().to_vec();
        let (vm, port) = pi_local_models(app, kit, lang, vm)?;
        vm.start(&env, lang).or_fail_chain("vm")?;
        if let Some(port) = port {
            crate::isolation::watch_pi_port(
                vm.clone(),
                env.clone(),
                project.to_owned(),
                port,
                db_ports,
            );
        }
        let args = crate::isolation::vm::pi_session_args(&vm.name, &guest_dir, &pi, vm.tools());
        return Ok((vm, args));
    }
    let Some(vm) = vm.for_agent(project, agent) else {
        return Err(format!("unknown agent: {agent}").into());
    };
    let vm = super::terminals::project_vm_ready(
        app, task, services, kit, host, lang, project, dir, kit_env, vm,
    )?;
    vm.start(&env, lang).or_fail_chain("vm")?;
    let args = vm.agent_session_args(&env).or_fail_chain("vm")?;
    Ok((vm, args))
}

/// Pi on the kit's local models (D-152), in the project's VM (D-178): the
/// server started on the PC, its port opened for this sandbox only (like a
/// database's, D-171) and its address and key in Pi's variables, through
/// the VM's way to the PC, in a file readable by its owner only
/// (`pi.env` beside the sandbox's kit, with the database's variables). A
/// provider of the cloud needs nothing: `vm` as it is. With the port to
/// close once Pi ends (`isolation::watch_pi_port`), when it is not also a
/// database's.
fn pi_local_models<R: Runtime>(
    app: &AppHandle<R>,
    kit: &Kit,
    lang: &str,
    vm: crate::isolation::Vm,
) -> CmdResult<(crate::isolation::Vm, Option<u16>)> {
    use tauri::Manager;
    if crate::agents::sync::pi::local_model(kit).is_none() {
        return Ok((vm, None));
    }
    let (Some(port), Some(key)) = (
        crate::llm::settings::load(kit).port,
        crate::llm::settings::key(kit),
    ) else {
        return Ok((vm, None));
    };
    // Not a database's: a watcher of the port waits again, it never closes
    // it while this Pi is being started, and keeps this port from now.
    let watched = (!vm.host_ports().contains(&port)).then_some(port);
    if let Some(port) = watched {
        crate::isolation::pi_port_touch(&vm.name, port);
    }
    if let Some(state) = app.try_state::<AppState>() {
        if let Err(e) = state.llm.start(&state.env(), lang) {
            crate::common::log::warn(&format!("pi: local models: {e:#}"));
        }
    }
    let mut tools = vm.tools().clone();
    let mut text = tools
        .env_file
        .as_ref()
        .and_then(|f| std::fs::read_to_string(f).ok())
        .unwrap_or_default();
    for (k, v) in crate::isolation::pi_local_models_env(port, &key) {
        text.push_str(&format!("{k}={v}\n"));
    }
    let file = crate::isolation::kit::dir(kit, &vm.name).join("pi.env");
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).or_fail("pi")?;
    }
    crate::platform::files::replace_private(&file, &text).or_fail_chain("pi")?;
    tools.env_file = Some(file);
    Ok((vm.with_tools(tools).with_host_port(port), watched))
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
                            .ok_or_else(|| {
                                let status = crate::isolation::status(&kit, &host, &base, false);
                                no_vm_for_agents(&status, lang)
                            })?;
                        agent_vm_ready(
                            &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env,
                            vm, &agent,
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

#[cfg(test)]
#[path = "../../../test/rust/ipc/agents.rs"]
mod tests;
