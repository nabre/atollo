//! The editor, the kit's console and a project's editor settings and
//! extensions.

use super::*;

#[tauri::command]
pub fn editor_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<editor::Status> {
    ensure_main(&webview)?;
    Ok(state.editor.status())
}

/// Starts the server if needed (off the UI thread) and opens a window on
/// the project, or on `projects\` when `project` is `None`.
#[tauri::command]
pub async fn editor_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::opened(&state.kit, &project);
    let title = project.clone();
    let editor = Arc::clone(&state.editor);
    let (handle, task, kit) = (app.clone(), Arc::clone(&state.setup), state.kit.clone());
    let lang = state.lang();
    let (host, kit_env) = (state.host.clone(), state.env());
    let (for_vm, dir_vm) = (project.clone(), dir.clone());
    let (port, token, vm) = tauri::async_runtime::spawn_blocking(move || {
        ensure(&handle, &task, &kit, lang, &[])?;
        let (port, token) = editor.ensure_running().or_fail_chain("editor start")?;
        // Before the window: a terminal never opens before the box is known.
        let vm = editor_vm_begin(&kit, &host, &kit_env, &for_vm, &dir_vm);
        Ok::<_, CmdError>((port, token, vm))
    })
    .await
    .or_fail("editor")??;
    let _ = app.emit_to(MAIN, super::events::EDITOR_STATUS, state.editor.status());
    if let Some(vm) = vm {
        editor_vm_session(&app, &state, &project, &dir, vm);
    }
    let url = editor::url(port, &token, Some(&dir));
    editor::window::open(
        &app,
        &url,
        port,
        &title,
        &dir,
        &state.kit.webview_data(),
        lang,
    )
    .or_fail_chain("editor window")
}

/// The box of the editor's terminals, tasks and agents for `project`
/// (D-175, `isolation::session`), noted before its window opens: the VM's
/// level and the project pending, its `Vm` to make ready; without the VM
/// the level says so and the terminals are the kit's on the PC. The level
/// on, but no `sbx` to run it: the session says why, nothing runs on the
/// PC instead.
fn editor_vm_begin(
    kit: &Kit,
    host: &crate::env::Host,
    kit_env: &KitEnv,
    project: &str,
    dir: &std::path::Path,
) -> Option<crate::isolation::Vm> {
    use crate::isolation::{session, Level};
    let env = crate::versions::project_env(kit, project, kit_env.clone());
    let warn = |e: anyhow::Error| {
        crate::common::log::warn(&format!("editor session {project}: {e:#}"));
    };
    if crate::isolation::status(kit, host, &env, false).level != Level::Vm {
        if let Err(e) = session::set_level(kit, false).and_then(|()| session::clear(kit, project)) {
            warn(e);
        }
        return None;
    }
    if let Err(e) = session::set_level(kit, true).and_then(|()| session::pending(kit, project)) {
        warn(e);
    }
    let vm = crate::isolation::vm_for(kit, host, &env, project, dir);
    if vm.is_none() {
        if let Err(e) = session::failed(kit, project, "Docker Sandboxes of the kit not found") {
            warn(e);
        }
    }
    vm
}

/// The project's VM for the editor's terminals, tasks and agents (D-175),
/// made ready off the window's opening (its tools, databases and ports): a
/// terminal opened meanwhile waits for it.
fn editor_vm_session<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
    project: &str,
    dir: &std::path::Path,
    vm: crate::isolation::Vm,
) {
    use crate::isolation::session;
    let (app, task, kit, lang) = (
        app.clone(),
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.lang(),
    );
    let (host, services, kit_env) = (state.host.clone(), Arc::clone(&state.services), state.env());
    let (project, dir) = (project.to_owned(), dir.to_path_buf());
    std::thread::spawn(move || {
        let env = crate::versions::project_env(&kit, &project, kit_env.clone());
        let ready = super::terminals::project_vm_ready(
            &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm,
        )
        .map_err(|e| e.to_string())
        .and_then(|vm| {
            vm.start(&env, lang).map_err(|e| format!("{e:#}"))?;
            session::write(&kit, &project, &dir, &vm, &env).map_err(|e| format!("{e:#}"))
        });
        if let Err(why) = ready {
            crate::common::log::warn(&format!("editor session {project}: {why}"));
            if let Err(e) = session::failed(&kit, &project, &why) {
                crate::common::log::warn(&format!("editor session {project}: {e:#}"));
            }
        }
    });
}

#[tauri::command]
pub async fn editor_restart<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let editor = Arc::clone(&state.editor);
    tauri::async_runtime::spawn_blocking(move || {
        editor.stop();
        editor.ensure_running().map(|_| ())
    })
    .await
    .or_fail("editor")?
    .or_fail_chain("editor restart")?;
    let _ = app.emit_to(MAIN, super::events::EDITOR_STATUS, state.editor.status());
    Ok(())
}

#[tauri::command]
pub async fn terminal_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::settings::opened(&state.kit, &project);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let (host, services) = (state.host.clone(), Arc::clone(&state.services));
    tauri::async_runtime::spawn_blocking(move || {
        let kit_env = ensure(&app, &task, &kit, lang, &[])?;
        let env = crate::versions::project_env(&kit, &project, kit_env.clone());
        // In the project's microVM when every project runs in one (D-167),
        // with the kit's tools and its databases on the PC (D-170, D-171).
        if let Some(vm) = crate::isolation::vm_for(&kit, &host, &env, &project, &dir) {
            // Checked before the sandbox is made: a folder with no path in
            // the VM (a network share) never gets one.
            let guest_dir = crate::isolation::vm::guest_path(kit.target().is_windows(), &dir)
                .ok_or_else(|| {
                    crate::common::i18n::t(
                    lang,
                    "La cartella {0} non si apre nella VM: scegli una cartella di un disco del PC.",
                    &[&dir.display().to_string()],
                )
                })?;
            let vm = super::terminals::project_vm_ready(
                &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm,
            )?;
            vm.start(&env, lang).or_fail_chain("vm")?;
            let args = crate::isolation::vm::kit_terminal_args(&vm.name, &guest_dir, vm.tools());
            let launch = vm.launch(&kit, args, "VM", lang).or_fail_chain("vm")?;
            return terminal::open_launch(&app, &vm.session_env(&env), &dir, launch)
                .or_fail_chain("terminal");
        }
        terminal::open_console(&app, &kit, &env, &dir).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}

/// The project's editor settings in its .vscode\settings.json (D-78).
#[tauri::command]
pub fn project_editor_settings<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<crate::projects::vscode::EditorSettingsView> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::projects::vscode::get(&dir))
}

#[tauri::command]
pub fn project_editor_settings_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    settings: crate::projects::vscode::EditorSettings,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::projects::vscode::set(&dir, &settings).or_fail_chain("editor settings")
}

/// The kit's matrix of editor extensions by language ("Linguaggi e
/// strumenti"), for `target`: the system chosen on the Tools page (D-121),
/// this PC's when omitted.
#[tauri::command]
pub fn extensions_matrix<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    target: Option<String>,
) -> CmdResult<crate::editor::ext::MatrixInfo> {
    ensure_main(&webview)?;
    let kit = kit_for(&state.kit, target.as_deref())?;
    Ok(crate::editor::ext::matrix_info(&kit))
}

/// The editor extensions added to or removed from those of the project's
/// languages; the new ones are installed now if the editor runs.
#[tauri::command]
pub async fn project_extensions<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    added: Vec<String>,
    removed: Vec<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let choice = crate::editor::ext::clean_choice(&added, &removed);
    crate::projects::settings::update(&state.kit, &project, |s| s.extensions = choice);
    crate::editor::window::refresh_hidden(&app, &state.kit);
    let editor = Arc::clone(&state.editor);
    // Busy installing: the watch of the open projects asks again (app/mod.rs).
    tauri::async_runtime::spawn_blocking(move || {
        editor.sync_extensions();
    })
    .await
    .or_fail("extensions")
}
