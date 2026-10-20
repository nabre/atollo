//! Saved versions of the languages (D-80) and the relations matrix (D-81).

use super::*;

/// The versioned languages and tools: default folder, saved lines, the
/// kit's preferred line (`versions/mod.rs`).
#[tauri::command]
pub fn versions_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    target: Option<String>,
) -> CmdResult<Vec<crate::versions::Tool>> {
    ensure_main(&webview)?;
    let kit = kit_for(&state.kit, target.as_deref())?;
    Ok(crate::versions::tools(&kit))
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
    };
    let lines = installer.lines(&ctx, c)?;
    if let Err(e) = crate::versions::save_offered(kit, id, &lines) {
        crate::log::warn(&format!("versions of {id}: {e:#}"));
    }
    Ok(lines)
}

/// At the start, in the background: the lists of lines missing or older
/// than a day, so the menus to download a version are ready.
pub(crate) fn refresh_stale_lines(kit: &Kit, env: &KitEnv, token: Option<String>) {
    for id in crate::versions::stale_offers(kit) {
        if let Err(e) = fetch_lines(kit, env, token.clone(), &id) {
            crate::log::warn(&format!("versions of {id}: {e:#}"));
        }
    }
}

/// Downloads `line` of `id` next to the default folder, with the app's
/// preparation task (progress as `setup`).
#[tauri::command]
pub fn versions_add<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::versions::check(&id, Some(&line)).map_err(|e| e.to_string())?;
    download_line(app, &state, id, line, target)
}

/// Installs `id`, not installed yet, at `line` instead of the latest: it
/// becomes chosen (with what it needs) and its first version is `line`.
#[tauri::command]
pub fn versions_install<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::versions::check(&id, Some(&line)).map_err(|e| e.to_string())?;
    install_now(app, &state, id, Some(line), target)
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
    if state.setup.snapshot().running {
        return Err(tr(state, "C'è già un'installazione in corso.", &[]).into());
    }
    let kit = kit_for(&state.kit, target.as_deref())?;
    let catalog = Catalog::get();
    if catalog.find(&id).is_none() {
        return Err(format!("unknown component: {id}").into());
    }
    let mut choice = crate::setup::catalog::Choice::read(&kit, catalog)
        .unwrap_or_else(|| crate::setup::catalog::Choice::defaults(catalog));
    choice.set(catalog, &id, true);
    choice.write(&kit, catalog).or_fail_chain("components")?;
    let opts = Options {
        target: Some(kit.target()),
        only: vec![id.clone()],
        at: line.map(|l| vec![(id, l)]).unwrap_or_default(),
        ..Options::default()
    };
    Ok(state.setup.start(opts, false, setup_notify(app))?)
}

pub(super) fn download_line<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<()> {
    let kit = kit_for(&state.kit, target.as_deref())?;
    let opts = Options {
        target: Some(kit.target()),
        lines: vec![(id, line)],
        ..Options::default()
    };
    Ok(state.setup.start(opts, false, setup_notify(app))?)
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

/// Removes a saved line (the default folder goes with the component).
#[tauri::command]
pub fn versions_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    line: String,
    target: Option<String>,
) -> CmdResult<Vec<crate::versions::Tool>> {
    ensure_main(&webview)?;
    if state.setup.snapshot().running {
        return Err(tr(&state, "C'è già un'installazione in corso.", &[]).into());
    }
    let kit = kit_for(&state.kit, target.as_deref())?;
    crate::versions::remove(&kit, &id, &line).map_err(|e| {
        if e.is::<crate::net::archive::InUse>() {
            tr(
                &state,
                "La versione {0} è in uso: chiudi i programmi che la usano e riprova.",
                &[&line],
            )
            .into()
        } else {
            fail("versions", format!("{e:#}"))
        }
    })?;
    if kit.target() == state.kit.target() {
        apply_env(&state);
    }
    Ok(crate::versions::tools(&kit))
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

#[tauri::command]
pub fn project_versions<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<ProjectVersions> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(project_versions_of(&state.kit, &project))
}

/// The line of `id` for the project (`None`: the kit's), downloaded when it
/// is not on disk.
#[tauri::command]
pub fn project_versions_set<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
    line: Option<String>,
) -> CmdResult<ProjectVersions> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    download_chosen(app, &state, &id, line.as_deref())?;
    crate::versions::set_project_choice(&state.kit, &project, &id, line.as_deref())
        .or_fail_chain("versions")?;
    Ok(project_versions_of(&state.kit, &project))
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
