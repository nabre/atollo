//! Saved versions of the languages (D-80) and the relations matrix (D-81).

use super::*;

/// The versioned languages and tools: default folder, saved lines, the
/// kit's preferred line (`versions/mod.rs`). Off the window's thread: it
/// reads the folders of every line and the settings of every project.
#[tauri::command]
pub async fn versions_list<R: Runtime>(
    webview: Webview<R>,
    target: Option<String>,
) -> CmdResult<Vec<crate::versions::Tool>> {
    ensure_main(&webview)?;
    off_main(&webview, "versions", move |_, state| {
        let kit = kit_for(&state.kit, target.as_deref())?;
        Ok(crate::versions::tools(&kit))
    })
    .await
}

/// The lines of `id` that can be downloaded (asks its source); kept for the
/// lists (`versions_list`).
#[tauri::command]
pub async fn versions_available<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    target: Option<String>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    crate::versions::check(&id, None).map_err(|e| e.to_string())?;
    let kit = kit_for(&state.kit, target.as_deref())?;
    let env = state.env();
    let token = github_token(&state.host);
    tauri::async_runtime::spawn_blocking(move || {
        fetch_lines(&kit, &env, token, &id).or_fail_chain("versions")
    })
    .await
    .or_fail("versions")?
}

/// Asks the source of `id` for its lines and keeps them for the lists.
pub(crate) fn fetch_lines(
    kit: &Kit,
    env: &KitEnv,
    token: Option<String>,
    id: &str,
) -> anyhow::Result<Vec<String>> {
    let c = Catalog::get()
        .find(id)
        .ok_or_else(|| anyhow::anyhow!("unknown component: {id}"))?;
    let installer = install::installer(id).ok_or_else(|| anyhow::anyhow!("{id}: no installer"))?;
    // Some lists come from GitHub's API (Python): the kit's account if
    // there is no token of the PC, as the installations do.
    let token = token.or_else(|| crate::github::token(kit, env));
    let net = crate::net::Net::new(&Settings::load(kit).settings, token)?
        .with_github_cache(kit.data().join("cache").join("github"));
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = install::Ctx {
        kit,
        net: &net,
        env,
        cancel: &cancel,
        guest: None,
    };
    let lines = installer.lines(&ctx, c)?;
    if let Err(e) = crate::versions::save_offered(kit, id, &lines) {
        crate::common::log::warn(&format!("versions of {id}: {e:#}"));
    }
    Ok(lines)
}

/// At the start, in the background: the lists of lines missing or older
/// than a day, so the menus to download a version are ready.
pub(crate) fn refresh_stale_lines(kit: &Kit, env: &KitEnv, token: Option<String>) {
    for id in crate::versions::stale_offers(kit) {
        if let Err(e) = fetch_lines(kit, env, token.clone(), &id) {
            crate::common::log::warn(&format!("versions of {id}: {e:#}"));
        }
    }
}

/// Downloads `line` of `id` next to the default folder, with the app's
/// preparation task (progress as `setup`). Off the window's thread: where
/// it goes asks `sbx` ([`tools_kit`]).
#[tauri::command]
pub async fn versions_add<R: Runtime>(
    webview: Webview<R>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::versions::check(&id, Some(&line)).map_err(|e| e.to_string())?;
    off_main(&webview, "versions", move |app, state| {
        download_line(app.clone(), state, id, line, target)
    })
    .await
}

/// Installs `id`, not installed yet, at `line` instead of the latest: it
/// becomes chosen (with what it needs) and its first version is `line`.
/// Off the window's thread, as [`versions_add`].
#[tauri::command]
pub async fn versions_install<R: Runtime>(
    webview: Webview<R>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::versions::check(&id, Some(&line)).map_err(|e| e.to_string())?;
    off_main(&webview, "versions", move |app, state| {
        install_now(app.clone(), state, id, Some(line), target)
    })
    .await
}

/// `id` becomes chosen (with what it needs) and is installed now, at
/// `line` when given (first installation), without waiting for "Apply":
/// the other changes of the Tools page stay pending.
pub(super) fn install_now<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: String,
    line: Option<String>,
    target: Option<String>,
) -> CmdResult<()> {
    let catalog = Catalog::get();
    let Some(component) = catalog.find(&id) else {
        return Err(format!("unknown component: {id}").into());
    };
    let kit = tools_kit(state, kit_for(&state.kit, target.as_deref())?, &id);
    // An agent only for the VM's Linux, while it is on (LEVELS.md §7).
    let vm = crate::levels::agents::known_vm();
    if !crate::levels::installable(component, kit.target(), &vm) {
        return Err(crate::levels::agents::refused_install(component, &vm, state.lang()).into());
    }
    {
        let _one = crate::setup::catalog::choice_lock();
        let mut choice = crate::setup::Choice::read(&kit, catalog)
            .unwrap_or_else(|| crate::setup::Choice::defaults(catalog));
        choice.set(catalog, &id, true);
        choice.write(&kit, catalog).or_fail_chain("components")?;
    }
    let opts = Options {
        target: Some(kit.target()),
        only: vec![id.clone()],
        at: line.map(|l| vec![(id, l)]).unwrap_or_default(),
        vm: Some(vm),
        ..Options::default()
    };
    state.setup.start(opts, false, setup_notify(app))?;
    Ok(())
}

pub(super) fn download_line<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    let kit = tools_kit(state, kit_for(&state.kit, target.as_deref())?, &id);
    let opts = Options {
        target: Some(kit.target()),
        lines: vec![(id, line)],
        ..Options::default()
    };
    state.setup.start(opts, false, setup_notify(app))?;
    Ok(())
}

/// Where component `id` asked for `kit` is installed (D-173): with the VM
/// on, a tool asked for the PC that runs only in the VM
/// (`Component::in_vm` without `on_pc_with_vm`, D-180) goes to the VM's
/// Linux, if it has a build there; a system component, one that runs on
/// the PC also with the VM (a database's server, llama.cpp), or another
/// system asked for, stays as asked. It may ask `sbx` for the VM's status
/// ([`super::setup::vm_tools_target`]): never on the window's thread.
fn tools_kit(state: &AppState, kit: Kit, id: &str) -> Kit {
    let catalog = Catalog::get();
    let Some(component) = catalog.find(id) else {
        return kit;
    };
    if kit.target() != state.kit.target() || catalog.locked_on(kit.target()).contains(id) {
        return kit;
    }
    let Some(guest) = super::setup::vm_tools_target(state) else {
        return kit;
    };
    if component.in_vm() && !component.on_pc_with_vm() && component.is_available_on(guest) {
        state.kit.for_target(guest)
    } else {
        kit
    }
}

/// A line chosen and not on disk is downloaded first (progress as
/// `setup`); until then the default folder is used.
pub(super) fn download_chosen<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: &str,
    line: Option<&str>,
) -> CmdResult<()> {
    let Some(line) = line else { return Ok(()) };
    if crate::versions::to_download(&state.kit, id, line).or_fail_chain("versions")? {
        download_line(app, state, id.to_owned(), line.to_owned(), None)?;
    }
    Ok(())
}

/// Removes a saved line (the default folder goes with the component). Off
/// the window's thread: it deletes the line's folder and rebuilds the
/// kit's environment.
#[tauri::command]
pub async fn versions_remove<R: Runtime>(
    webview: Webview<R>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<Vec<crate::versions::Tool>> {
    ensure_main(&webview)?;
    off_main(&webview, "versions", move |_, state| {
        if state.setup.snapshot().running {
            return Err(tr(state, "C'è già un'installazione in corso.", &[]).into());
        }
        let kit = kit_for(&state.kit, target.as_deref())?;
        crate::versions::remove(&kit, &id, &line).map_err(|e| {
            if e.is::<crate::net::archive::InUse>() {
                tr(
                    state,
                    "La versione {0} è in uso: chiudi i programmi che la usano e riprova.",
                    &[&line],
                )
                .into()
            } else {
                fail("versions", format!("{e:#}"))
            }
        })?;
        if kit.target() == state.kit.target() {
            apply_env(state);
        }
        Ok(crate::versions::tools(&kit))
    })
    .await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectVersions {
    pub tools: Vec<crate::versions::Tool>,
    /// The lines chosen for the project; the others follow the kit.
    pub chosen: crate::versions::Choice,
}

pub(super) fn project_versions_of(kit: &Kit, project: &str) -> ProjectVersions {
    ProjectVersions {
        tools: crate::versions::tools(kit),
        chosen: crate::versions::project_choice(kit, project),
    }
}

/// The project's lines. Off the window's thread, as [`versions_list`].
#[tauri::command]
pub async fn project_versions<R: Runtime>(
    webview: Webview<R>,
    project: String,
) -> CmdResult<ProjectVersions> {
    ensure_main(&webview)?;
    off_main(&webview, "versions", move |_, state| {
        project_dir(&state.kit, Some(&project), state.lang())?;
        Ok(project_versions_of(&state.kit, &project))
    })
    .await
}

/// The line of `id` for the project (`None`: the kit's), downloaded when it
/// is not on disk. Off the window's thread, as [`versions_list`].
#[tauri::command]
pub async fn project_versions_set<R: Runtime>(
    webview: Webview<R>,
    project: String,
    id: String,
    line: Option<String>,
) -> CmdResult<ProjectVersions> {
    ensure_main(&webview)?;
    off_main(&webview, "versions", move |app, state| {
        project_dir(&state.kit, Some(&project), state.lang())?;
        download_chosen(app.clone(), state, &id, line.as_deref())?;
        crate::versions::set_project_choice(&state.kit, &project, &id, line.as_deref())
            .or_fail_chain("versions")?;
        Ok(project_versions_of(&state.kit, &project))
    })
    .await
}

/// The relations matrix (versions/relations/, D-81): the saved one while nothing
/// it was made from changed; `force` builds it again. Off the window's
/// thread: it reads every project's files and asks WebView2 its version.
/// `target`: the system chosen on the Tools page (D-121), this PC's when
/// omitted.
#[tauri::command]
pub async fn relations_matrix<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
    target: Option<String>,
) -> CmdResult<crate::versions::relations::Matrix> {
    ensure_main(&webview)?;
    let (kit, lang) = (kit_for(&state.kit, target.as_deref())?, state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::versions::relations::matrix(&kit, lang, force).or_fail_chain("relations")
    })
    .await
    .or_fail("relations")?
}
