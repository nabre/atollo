//! Projects: list, creation, clone, deletion, actions, site, status and MCP
//! servers.

use super::*;

/// The projects with what each one is and how it starts (detection reads
/// files: off the UI thread).
#[tauri::command]
pub async fn projects_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::projects::ProjectInfo>> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    let app = tauri::Manager::app_handle(&webview).clone();
    tauri::async_runtime::spawn_blocking(move || {
        // What a project misses is counted on the PC, where its code runs
        // (D-183).
        let list = crate::projects::list_in(&kit, &kit, lang);
        // The interface reads the list after every change of a project:
        // the tray menu follows (D-145).
        crate::app::tray::refresh(&app);
        list
    })
    .await
    .or_fail("projects")
}

/// A new empty project in `projects\`. Off the window's thread: it writes
/// the project's files and reads the versions on disk of every language.
#[tauri::command]
pub async fn project_create<R: Runtime>(webview: Webview<R>, name: String) -> CmdResult<String> {
    ensure_main(&webview)?;
    off_main(&webview, "project", move |_, state| {
        let project = crate::projects::create(&state.kit, &name, state.lang()).map(|d| {
            d.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })?;
        // It keeps the newest version of each language: its code is written
        // for those, newer ones installed later do not change it.
        crate::versions::pin_newest(&state.kit, &project);
        Ok(project)
    })
    .await
}

/// What deleting a project takes away: its folder, its databases on the
/// kit's servers, the copies of them (D-89). Off the window's thread: it
/// reads the project's folder, its databases and its copies.
#[tauri::command]
pub async fn project_delete_plan<R: Runtime>(
    webview: Webview<R>,
    project: String,
) -> CmdResult<crate::projects::DeletePlan> {
    ensure_main(&webview)?;
    off_main(&webview, "project", move |_, state| {
        Ok(crate::projects::delete_plan(
            &state.kit,
            &project,
            state.lang(),
        )?)
    })
    .await
}

/// Deletes a project whose folder is empty, with its databases and users on
/// the kit's servers (started if stopped; off the UI thread). A failed drop
/// deletes nothing; `force_without_db` deletes without touching the servers.
#[tauri::command]
pub async fn project_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    force_without_db: Option<bool>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, lang, env) = (state.kit.clone(), state.lang(), state.env());
    let (services, servers) = (Arc::clone(&state.services), Arc::clone(&state.servers));
    let host = state.host.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let dbs = crate::db::services::ProjectDrops {
            services: &services,
            env: env.clone(),
            project: project.clone(),
        };
        crate::projects::delete(&kit, &project, lang, &dbs, force_without_db == Some(true))
            .or_fail_chain("project delete")?;
        // Its sandboxes and their disks go with it (D-169).
        let names = crate::isolation::project_sandboxes(&project);
        crate::isolation::remove_owned(&kit, &host, &env, &names);
        // Its site leaves the pools' list now, not after `LIST_TTL`.
        servers.invalidate();
        Ok(())
    })
    .await
    .or_fail("project delete")?
}

/// Clones `owner/repo` or an `https://` Git URL into `projects\<name>`.
#[tauri::command]
pub async fn project_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    source: String,
    name: Option<String>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let servers = Arc::clone(&state.servers);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        let dir =
            crate::projects::clone(&kit, &env, &source, name.as_deref(), lang, token.as_deref())
                .or_fail_chain("clone")?;
        let project = dir
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        settle_project(&app, &task, &kit, lang, &project, &dir);
        // A new project: the pools are scanned again for its site.
        servers.invalidate();
        Ok(project)
    })
    .await
    .or_fail("clone")?
}

/// Adds or changes an action of the project in its panel.json (D-78); it
/// applies from the next start of that action.
#[tauri::command]
pub fn project_action_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    action: crate::detect::ActionEdit,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::save_action(&dir, &action, state.lang())?;
    // Its `server` may have moved the site to another pool.
    state.servers.invalidate();
    Ok(())
}

/// «Salva come base» (N-10): what would be copied, by top folder and file.
#[tauri::command]
pub async fn project_base_preview<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<Vec<crate::projects::frameworks::mine::Entry>> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        crate::projects::frameworks::mine::preview(&kit, &env, &dir)
    })
    .await
    .or_fail("base")
}

/// Saves the project's files (but the top entries in `skip`) as a base of
/// «Le mie basi»; returns its id.
#[tauri::command]
pub async fn project_save_base<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    name: String,
    desc: String,
    skip: Vec<String>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        crate::projects::frameworks::mine::save(&kit, &env, &dir, &name, &desc, &skip)
    })
    .await
    .or_fail("base")?
    .or_fail_chain("base")
}

/// The project's own MCP servers (.mcp.json, D-78).
#[tauri::command]
pub fn project_mcp<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::mcp::McpView> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::projects::mcp::get(&dir))
}

#[tauri::command]
pub fn project_mcp_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    servers: Vec<crate::projects::mcp::McpServer>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::mcp::set(&dir, &servers).or_fail_chain("project mcp")
}

/// Removes an action written in panel.json (a detected one comes back).
#[tauri::command]
pub fn project_action_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::remove_action(&dir, &id, state.lang())?;
    state.servers.invalidate();
    Ok(())
}

/// The action that "Avvia" starts.
#[tauri::command]
pub fn project_action_default<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::detect::set_default_action(&dir, &id, state.lang())?)
}

/// The project among the favorites of the Projects home, or not.
#[tauri::command]
pub fn project_pin<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    pinned: bool,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::update(&state.kit, &project, |s| s.pinned = pinned);
    Ok(())
}

/// Whether the project's VM gets the kit's GitHub access (D-172).
#[tauri::command]
pub fn project_github_vm<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<bool> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::projects::settings::load(&state.kit, &project).github_in_vm)
}

/// Gives the project's VM the kit's GitHub access, or takes it back (D-172):
/// saved, then applied at its sandbox's next start; taken back now if the
/// sandbox has it.
#[tauri::command]
pub async fn project_github_vm_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    on: bool,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        crate::projects::settings::update(&kit, &project, |s| s.github_in_vm = on);
        if !on {
            if let Some(sbx) = crate::isolation::Sbx::find(&kit, &host) {
                let sandbox = crate::isolation::vm::name(&project);
                crate::isolation::github_for_sandbox(&kit, &sbx, &host, &env, &sandbox, false);
            }
        }
    })
    .await
    .or_fail("project")
}

/// "Servi come sito web": a folder of the project served by the file server
/// (`static`), PHP (`php`), Apache (`apache`) or Caddy (`caddy`), or the site removed (`none`).
#[tauri::command]
pub fn project_site<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    folder: String,
    kind: crate::detect::SiteKind,
    options: Option<crate::detect::SiteOptions>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let key = crate::projects::run::key_of(&project, crate::detect::SITE_ACTION);
    if runner
        .list()
        .iter()
        .any(|r| r.key == key && matches!(r.state, RunState::Starting | RunState::Running))
    {
        return Err(tr(
            &state,
            "Ferma prima il sito web: la modifica vale dal prossimo avvio.",
            &[],
        )
        .into());
    }
    crate::detect::set_site_with_options(
        &dir,
        &folder,
        kind,
        &options.unwrap_or_default(),
        state.lang(),
    )?;
    // The site (and its `server`) changed: the pools are scanned again.
    state.servers.invalidate();
    Ok(())
}

/// Installs the kit components a project needs and are missing (its
/// `missing`), saved as chosen like those of an action. Waits for the end.
#[tauri::command]
pub async fn project_install_missing<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        // On the PC, where the project's code runs (D-183); with the VM on
        // also for its Linux, where the agents use them. Asked here, since
        // it may wait for `sbx`.
        let missing = crate::projects::info(&kit, &dir, lang).missing;
        let wanted: Vec<&str> = missing.iter().map(String::as_str).collect();
        let on_pc = ensure(&app, &task, &kit, lang, &wanted).map(|_| ());
        let guest = super::setup::vm_tools_target(&tauri::Manager::state::<AppState>(&app))
            .map(|t| kit.for_target(t));
        if let Some(guest) = guest {
            let missing = crate::projects::info_in(&kit, &guest, &dir, lang).missing;
            let wanted = missing.into_iter().map(|id| (id, None)).collect();
            super::terminals::ensure_guest_tools(&app, &task, &guest, &wanted, &|_| ());
        }
        on_pc
    })
    .await
    .or_fail("install")?
}

/// Git state and agent instructions of a project, for its page.
#[tauri::command]
pub async fn project_status<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::ProjectStatus> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    // Its page is open: what its files need, settled in the background
    // when they changed (D-186).
    super::settle_when_changed(&app, &state.setup, &state.kit, state.lang(), &project, &dir);
    let root = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || crate::projects::status(&dir, &root))
        .await
        .or_fail("project")
}
