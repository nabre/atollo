//! Components: the choice, the installation in progress, files downloaded
//! by hand (D-82).

use super::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInfo {
    pub id: String,
    pub name: String,
    pub group: Group,
    pub desc: String,
    pub mb: u32,
    pub chosen: bool,
    pub installed: bool,
    /// Something of it is on disk, even half installed: it can be removed.
    pub present: bool,
    /// A system component, or needed by one: always installed.
    pub locked: bool,
    pub version: Option<String>,
    /// This version of the app can install it on this kit.
    pub available: bool,
    pub x64_only: bool,
    /// No ARM64 build and this kit is ARM64: why it is not available.
    pub no_arm64: bool,
    /// Components it needs (Apache needs PHP, Kotlin needs Java).
    pub needs: Vec<String>,
    /// x64 only, on an ARM64 PC with Windows 10: only Windows 11 runs x64
    /// programs there (D-81). Not available.
    pub needs_windows11: bool,
    /// One folder per minor line, side by side (versions/mod.rs); the others
    /// are always at their latest version.
    pub versioned: bool,
    /// Its official site.
    pub home: String,
}

/// When the Tools page opens: what was deleted by hand from the kit's
/// folder is no longer chosen (versions::sync_with_disk); the names of
/// those. Not while an installation runs (its folders come and go).
#[tauri::command]
pub fn components_sync<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    if state.setup.snapshot().running {
        return Ok(Vec::new());
    }
    let off = crate::versions::sync_with_disk(&state.kit).or_fail_chain("components")?;
    if !off.is_empty() {
        apply_env(&state, Settings::load(&state.kit).settings);
    }
    Ok(off)
}

#[tauri::command]
pub fn components_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<ComponentInfo>> {
    ensure_main(&webview)?;
    Ok(component_infos(&state.kit, state.lang()))
}

/// The PC runs x64 programs: x64 itself, or Windows 11 on ARM64 (Windows 10
/// on ARM emulates x86 only). An unknown build counts as yes.
pub(super) fn emulates_x64() -> bool {
    crate::editor::wterm::windows_build().is_none_or(|b| b >= 22000)
}

/// The catalog as the "Linguaggi e strumenti" page shows it. A `noArm64`
/// component is not available on an ARM64 kit.
pub(super) fn component_infos(kit: &Kit, lang: &str) -> Vec<ComponentInfo> {
    let catalog = Catalog::get();
    let choice = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let locked = catalog.locked();
    // The order of the page, which an installation follows too.
    catalog
        .in_page_order()
        .into_iter()
        .map(|c| {
            let no_arm64 = c.no_arm64 && arm64;
            let needs_windows11 = arm64 && c.x64_only && !emulates_x64();
            ComponentInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                group: c.group,
                desc: crate::i18n::t(lang, &c.desc, &[]),
                mb: c.mb,
                chosen: choice.is_chosen(&c.id),
                installed: c.is_installed(kit),
                present: c.is_present(kit),
                locked: locked.contains(c.id.as_str()),
                version: c.installed_version(kit),
                available: install::installer(&c.id).is_some() && !no_arm64 && !needs_windows11,
                x64_only: c.x64_only,
                no_arm64,
                needs: c.needs.clone(),
                needs_windows11,
                versioned: c.versioned,
                home: c.home.clone(),
            }
        })
        .collect()
}

/// Sends every change of the preparation to the app window.
/// At the end of a run the kit environment is built again: its PATH lists
/// only the programs present (Git, Node.js...), so one just installed is
/// seen by the editor, the consoles and the projects without a restart.
pub fn setup_notify<R: Runtime>(app: AppHandle<R>) -> Notify {
    Box::new(move |snapshot: &Snapshot| {
        if !snapshot.running && !snapshot.steps.is_empty() {
            use tauri::Manager;
            if let Some(state) = app.try_state::<AppState>() {
                apply_env(&state, Settings::load(&state.kit).settings);
            }
        }
        let _ = app.emit_to(MAIN, "setup", snapshot);
    })
}

#[tauri::command]
pub fn setup_state<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Snapshot> {
    ensure_main(&webview)?;
    Ok(state.setup.snapshot())
}

/// Installs the components chosen (or only `only`); `update_all` also
/// updates what is installed. Returns at once; progress comes as `setup`.
#[tauri::command]
pub fn setup_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    only: Option<Vec<String>>,
    update_all: bool,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let only = only.unwrap_or_default();
    if let Some(bad) = only.iter().find(|id| Catalog::get().find(id).is_none()) {
        return Err(format!("unknown component: {bad}").into());
    }
    let opts = Options {
        only,
        update_all,
        ..Options::default()
    };
    Ok(state.setup.start(opts, false, setup_notify(app))?)
}

/// Installs `id` now, at its latest version, from the table of the Tools
/// page: it becomes chosen, the other changes of the page stay pending.
#[tauri::command]
pub fn components_install<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    super::versions::install_now(app, &state, id, None)
}

/// The choice of the "Linguaggi e strumenti" page: `chosen` are the ids to
/// keep installed (required ones always are, dependencies follow). Saved in
/// `components.txt`, then installed and removed in the background.
#[tauri::command]
pub fn components_apply<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    chosen: Vec<String>,
    update_all: bool,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let catalog = Catalog::get();
    if let Some(bad) = chosen.iter().find(|id| catalog.find(id).is_none()) {
        return Err(format!("unknown component: {bad}").into());
    }
    if state.setup.snapshot().running {
        return Err(tr(&state, "C'è già un'installazione in corso.", &[]).into());
    }
    let text = choice_text(&state.kit, &chosen);
    let choice = Choice::parse(&text, catalog);
    choice
        .write(&state.kit, catalog)
        .or_fail_chain("components")?;
    let opts = Options {
        update_all,
        ..Options::default()
    };
    Ok(state.setup.start(opts, false, setup_notify(app))?)
}

/// `components.txt` for the ids in `chosen`. What this kit cannot install
/// (`noArm64` on ARM64) keeps its saved choice: the same kit, back on an x64
/// PC, must not lose it.
pub(super) fn choice_text(kit: &Kit, chosen: &[String]) -> String {
    let catalog = Catalog::get();
    let saved = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    catalog
        .all()
        .iter()
        .map(|c| {
            let on = if c.no_arm64 && arm64 {
                saved.is_chosen(&c.id)
            } else {
                chosen.contains(&c.id)
            };
            format!(
                "{}={}
",
                c.id,
                u8::from(on)
            )
        })
        .collect()
}

#[tauri::command]
pub fn setup_cancel<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    state.setup.cancel(&setup_notify(app));
    Ok(())
}

/// Files downloaded by hand (setup/downloads.rs, D-82), dropped on the window:
/// kept in `data\downloads` for the next installation. Their names.
#[tauri::command]
pub async fn downloads_add<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    paths: Vec<String>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let files: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
        crate::setup::downloads::add(&kit, &files)
    })
    .await
    .or_fail("downloads")?
    .or_fail_chain("downloads")
}

/// The same, picked in Windows' "Open" window; empty when closed.
#[tauri::command]
pub async fn downloads_pick<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    let owner = webview.window().hwnd().or_fail("window")?.0 as isize;
    let title = crate::i18n::t(state.lang(), "Scegli il file scaricato", &[]);
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let files = crate::setup::downloads::pick(owner, &title)?;
        if files.is_empty() {
            return Ok(Vec::new());
        }
        crate::setup::downloads::add(&kit, &files)
    })
    .await
    .or_fail("downloads")?
    .or_fail_chain("downloads")
}

/// Opens in the PC's browser the address of the download that failed in
/// the step `name` of `id`: only one the preparation reported.
#[tauri::command]
pub fn setup_by_hand_open<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    name: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let url = state
        .setup
        .snapshot()
        .steps
        .into_iter()
        .rfind(|s| s.id == id && s.name == name)
        .and_then(|s| match s.outcome {
            Some(crate::setup::runner::Outcome::Failed {
                by_hand: Some(b), ..
            }) => Some(b.url),
            _ => None,
        })
        .filter(|u| u.starts_with("https://") || u.starts_with("http://"))
        .ok_or_else(|| format!("{id}: no download to do by hand"))?;
    crate::os::jobs::open_with_explorer(&url).or_fail_chain("browser")
}

#[cfg(test)]
#[path = "../../../test/rust/commands/setup.rs"]
mod tests;
