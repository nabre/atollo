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
    let dir_vm = dir.clone();
    let (port, token, req) = tauri::async_runtime::spawn_blocking(move || {
        ensure(&handle, &task, &kit, lang, &[])?;
        let (port, token) = editor.ensure_running().or_fail_chain("editor start")?;
        // Before the window: a terminal never opens before the box is known.
        let req = editor_vm_begin(&kit, &host, &kit_env, &dir_vm, lang);
        Ok::<_, CmdError>((port, token, req))
    })
    .await
    .or_fail("editor")??;
    let _ = app.emit_to(MAIN, super::events::EDITOR_STATUS, state.editor.status());
    if let Some(req) = req {
        editor_vm_session(&app, &state, &project, &dir, req);
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

/// The box of the editor's terminals, tasks and agents for the project in
/// `dir` (D-175, `isolation::session`; replaced by `levels::requests` in
/// step 5 of LEVELS.md §12.2), noted before its window opens. Decided as
/// the project's terminal (R14): with the VM on, the level and the project
/// pending, the request to make ready; without the VM the level says so
/// with why (D-178): nothing of the editor starts on the PC, its window
/// still opens. A refusal with the VM on (R9, R10): the session says why.
fn editor_vm_begin(
    kit: &Kit,
    host: &crate::env::Host,
    kit_env: &KitEnv,
    dir: &std::path::Path,
    lang: &str,
) -> Option<crate::levels::Request> {
    use crate::isolation::session;
    use crate::levels::{Placement, VmState};
    let req = project_terminal_request(kit, dir);
    let crate::levels::Place::Project { name: project, .. } = &req.place else {
        return None;
    };
    let warn = |e: anyhow::Error| {
        crate::common::log::warn(&format!("editor session {project}: {e:#}"));
    };
    let facts = super::levels::facts_now(kit, host, kit_env, &req);
    if let VmState::Off { blockers } = &facts.vm {
        let status = crate::isolation::Status::of(blockers.clone(), None);
        let why = session::no_vm_text(&status, lang);
        if let Err(e) =
            session::set_level(kit, false, &why).and_then(|()| session::clear(kit, project))
        {
            warn(e);
        }
        return None;
    }
    if let Err(e) = session::set_level(kit, true, "").and_then(|()| session::pending(kit, project))
    {
        warn(e);
    }
    match crate::levels::decide(&req, &facts) {
        Placement::Refused { reason, .. } => {
            let why = crate::levels::labels::refusal_text(&reason, lang);
            if let Err(e) = session::failed(kit, project, &why) {
                warn(e);
            }
            None
        }
        _ => Some(req),
    }
}

/// The project's VM for the editor's terminals, tasks and agents (D-175),
/// made ready off the window's opening by `levels::prepare` (its tools,
/// databases and ports): a terminal opened meanwhile waits for it.
fn editor_vm_session<R: Runtime>(
    app: &AppHandle<R>,
    state: &State<'_, AppState>,
    project: &str,
    dir: &std::path::Path,
    req: crate::levels::Request,
) {
    use crate::isolation::session;
    let (kit, lang, host, kit_env) = (
        state.kit.clone(),
        state.lang(),
        state.host.clone(),
        state.env(),
    );
    let fx = super::levels::AppEffects::of(app, state, kit_env.clone());
    let (project, dir) = (project.to_owned(), dir.to_path_buf());
    std::thread::spawn(move || {
        let ready = super::levels::ready_now(&fx, &kit, &kit_env, &req, &[], lang, &host)
            .map_err(|e| e.to_string())
            .and_then(|ready| {
                let (Some(vm), crate::levels::env::Environment::Vm { sbx_env, .. }) =
                    (&ready.vm, &ready.env)
                else {
                    return Err("levels: the editor's session is in the VM".to_owned());
                };
                session::write(&kit, &project, &dir, vm, sbx_env).map_err(|e| format!("{e:#}"))
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
    let host = state.host.clone();
    let fx = super::levels::AppEffects::of(&app, &state, state.env());
    tauri::async_runtime::spawn_blocking(move || {
        let kit_env = ensure(&app, &task, &kit, lang, &[])?;
        // Level 2 (D-179): in the project's microVM when it is on (R14),
        // made ready by `levels::prepare` (the kit's tools, the databases
        // on the PC, D-170, D-171), else on the PC, sealed (R5); a network
        // share or a pending restart of the kit's daemon refused (R9, R10).
        // A VM without the kit's `sbx` is an error, never the PC (I7).
        let req = project_terminal_request(&kit, &dir);
        let fx = super::levels::AppEffects {
            kit_env: kit_env.clone(),
            ..fx
        };
        let ready = super::levels::ready_now(&fx, &kit, &kit_env, &req, &[], lang, &host)?;
        crate::levels::start::open_terminal(&app, &kit, &ready, lang).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")?
}

/// The request of the project terminal of the project in `dir` (E3,
/// `Origin::ProjectTerminal`).
pub(crate) fn project_terminal_request(kit: &Kit, dir: &std::path::Path) -> crate::levels::Request {
    crate::levels::Request {
        origin: crate::levels::Origin::ProjectTerminal,
        place: crate::levels::place::place_of(kit, dir),
    }
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
