//! Saved versions of the languages (D-80) and the relations matrix (D-81).

use super::*;

/// The versioned languages and tools: default folder, saved lines, the
/// kit's preferred line (`versions/mod.rs`).
#[tauri::command]
pub fn versions_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::versions::Tool>, String> {
    ensure_main(&webview)?;
    Ok(crate::versions::tools(&state.kit))
}

/// The lines of `id` that can be downloaded (asks its source); kept for the
/// lists (`versions_list`).
#[tauri::command]
pub async fn versions_available<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<String>, String> {
    ensure_main(&webview)?;
    crate::versions::check(&id, None).map_err(|e| e.to_string())?;
    let (kit, env) = (state.kit.clone(), state.env());
    let token = github_token(&state.host);
    tauri::async_runtime::spawn_blocking(move || {
        let c = Catalog::get().find(&id).ok_or("unknown component")?;
        let installer = install::installer(&id).ok_or("no installer")?;
        // Some lists come from GitHub's API (Python): the kit's account if
        // there is no token of the PC, as the installations do.
        let token = token.or_else(|| crate::github::token(&kit, &env));
        let net = crate::net::Net::new(&Settings::load(&kit).settings, token)
            .map_err(|e| fail("versions", format!("{e:#}")))?
            .with_github_cache(kit.data().join("cache").join("github"));
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let ctx = install::Ctx {
            kit: &kit,
            net: &net,
            env: &env,
            cancel: &cancel,
        };
        let lines = installer
            .lines(&ctx, c)
            .map_err(|e| fail("versions", format!("{e:#}")))?;
        if let Err(e) = crate::versions::save_offered(&kit, &id, &lines) {
            crate::log::warn(&format!("versions of {id}: {e:#}"));
        }
        Ok(lines)
    })
    .await
    .map_err(|e| fail("versions", e))?
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::versions::check(&id, Some(&line)).map_err(|e| e.to_string())?;
    download_line(app, &state, id, line)
}

pub(super) fn download_line<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: String,
    line: String,
) -> Result<(), String> {
    let opts = Options {
        lines: vec![(id, line)],
        ..Options::default()
    };
    state.setup.start(opts, false, setup_notify(app))
}

/// A line chosen and not on disk is downloaded first (progress as
/// `setup`); until then the default folder is used.
pub(super) fn download_chosen<R: Runtime>(
    app: AppHandle<R>,
    state: &AppState,
    id: &str,
    line: Option<&str>,
) -> Result<(), String> {
    let Some(line) = line else { return Ok(()) };
    if crate::versions::to_download(&state.kit, id, line)
        .map_err(|e| fail("versions", format!("{e:#}")))?
    {
        download_line(app, state, id.to_owned(), line.to_owned())?;
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
) -> Result<Vec<crate::versions::Tool>, String> {
    ensure_main(&webview)?;
    if state.setup.snapshot().running {
        return Err(tr(&state, "C'è già un'installazione in corso.", &[]));
    }
    crate::versions::remove(&state.kit, &id, &line).map_err(|e| {
        if e.is::<crate::net::archive::InUse>() {
            tr(
                &state,
                "La versione {0} è in uso: chiudi i programmi che la usano e riprova.",
                &[&line],
            )
        } else {
            fail("versions", format!("{e:#}"))
        }
    })?;
    apply_env(&state, Settings::load(&state.kit).settings);
    Ok(crate::versions::tools(&state.kit))
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
) -> Result<ProjectVersions, String> {
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
) -> Result<ProjectVersions, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    download_chosen(app, &state, &id, line.as_deref())?;
    crate::versions::set_project_choice(&state.kit, &project, &id, line.as_deref())
        .map_err(|e| fail("versions", format!("{e:#}")))?;
    Ok(project_versions_of(&state.kit, &project))
}

/// The relations matrix (versions/relations/, D-81): the saved one while nothing
/// it was made from changed; `force` builds it again. Off the window's
/// thread: it reads every project's files and asks WebView2 its version.
#[tauri::command]
pub async fn relations_matrix<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
) -> Result<crate::versions::relations::Matrix, String> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::versions::relations::matrix(&kit, lang, force)
            .map_err(|e| fail("relations", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("relations", e))?
}
