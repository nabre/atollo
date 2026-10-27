//! The app itself: its information, settings, updates, and what it opens
//! outside (the PC's browser, File Explorer).

use super::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: &'static str,
    pub root: String,
    pub data: String,
    pub projects: String,
    pub platform: String,
    pub lang: &'static str,
    pub moved_from: Option<String>,
    pub webview: String,
    pub path_entries: usize,
    /// A folder of the kit path that stops the editor noticing outside changes.
    pub watcher_unsafe_folder: Option<String>,
    /// No choice saved yet: the interface shows the first-start page.
    pub first_run: bool,
    /// Other copies of the kit with programs open (two kits on one PC).
    pub other_kits: Vec<String>,
    /// File system of the kit's disk (`NTFS`, `ReFS`, `FAT32`, `exFAT`,
    /// `other`); `None` when it could not be read.
    pub disk_fs: Option<&'static str>,
    /// `fixed`, `removable`, `remote` or `other`.
    pub disk_kind: Option<&'static str>,
    /// The disk the kit's programs run from is mounted `noexec` (macOS and
    /// Linux, D-105); `None` on Windows.
    pub disk_noexec: Option<bool>,
    /// The app's notice about the kit's disk, translated (noexec,
    /// showexec, FAT32, exFAT); `None` on Windows or when all is well.
    pub disk_note: Option<String>,
    /// The host's temporary folder when the programs' `TEMP` is in it;
    /// `None` when it is `data\tmp` in the kit.
    pub system_temp: Option<String>,
}

/// Off the window's thread: the other kits are a walk of the PC's
/// processes, the disk's facts ask the system.
#[tauri::command]
pub async fn app_info<R: Runtime>(webview: Webview<R>) -> CmdResult<AppInfo> {
    ensure_main(&webview)?;
    off_main(&webview, "app", |_, state| Ok(info(state))).await
}

fn info(state: &AppState) -> AppInfo {
    let kit = &state.kit;
    let env = state.env();
    let system_temp = kit
        .system_temp()
        .filter(|host| {
            env.get("TEMP")
                .is_some_and(|t| std::path::Path::new(t).starts_with(host))
        })
        .map(|host| host.display().to_string());
    AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        root: kit.root().display().to_string(),
        data: kit.data().display().to_string(),
        projects: kit.projects().display().to_string(),
        platform: kit.arch().platform(),
        lang: state.lang(),
        moved_from: state.moved_from.clone(),
        webview: state.webview.clone(),
        path_entries: env.path_entries().len(),
        watcher_unsafe_folder: crate::kit::watcher_unsafe_folder(kit.root()),
        first_run: Choice::read(kit, Catalog::get()).is_none(),
        other_kits: crate::platform::sessions::other_kits(kit)
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        disk_fs: kit.volume().map(|v| v.fs.as_str()),
        disk_kind: kit.volume().map(|v| v.drive.as_str()),
        disk_noexec: (!crate::kit::Os::current().is_windows())
            .then(|| crate::platform::volume::kit_noexec(kit)),
        disk_note: crate::platform::volume::disk_notice(kit, state.lang()).map(|n| n.text),
        system_temp,
    }
}

/// The kit environment from the saved settings and what is installed now,
/// given to the app, the editor and the preparation task. Returns the
/// language. One at a time, reading the file inside: an apply that started
/// before a save cannot put back the old environment after it.
pub(super) fn apply_env(state: &AppState) -> &'static str {
    static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let settings = Settings::load(&state.kit).settings;
    let lang = crate::common::i18n::resolve(
        settings.lang.as_deref(),
        state.host.get("ATOLLO_LANG"),
        crate::platform::host::locale().as_deref(),
    );
    crate::net::ca::refresh(&state.kit, &settings);
    let env = KitEnv::build(&state.kit, &settings, lang, &state.host);
    if let Err(e) = env.create_dirs() {
        crate::common::log::warn(&format!("creating kit folders: {e}"));
    }
    *state
        .lang
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = lang;
    *state
        .env
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = env.clone();
    state.editor.set_env(env.clone());
    state.setup.set_config(env, settings);
    crate::versions::write_project_files(&state.kit);
    crate::versions::side_by_side::refresh(&state.kit);
    lang
}

/// Opens a folder of the kit in File Explorer: `agents` (shared
/// instructions, skills, MCP), `models` (the local models, D-151),
/// `diagnostics` (the zips of «Esporta diagnostica»), `projects` or one
/// project.
#[tauri::command]
pub fn folder_open<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    which: String,
    project: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = match which.as_str() {
        "agents" => state.kit.data().join("agents"),
        "db" => state.kit.data().join("db"),
        "models" => crate::llm::models_dir(&state.kit),
        "diagnostics" => crate::diagnostics::dir(&state.kit),
        // The kit's own folder (a copy imported, N-12).
        "kit" => state.kit.root().to_path_buf(),
        "backup" => {
            let name = project.as_deref().ok_or("no project")?;
            project_dir(&state.kit, Some(name), state.lang())?;
            state.services.backup_dir(name).or_fail_chain("folder")?
        }
        "projects" => project_dir(&state.kit, project.as_deref(), state.lang())?,
        _ => return Err(format!("unknown folder: {which}").into()),
    };
    std::fs::create_dir_all(&dir).or_fail("folder")?;
    crate::platform::jobs::open_with_explorer(&dir).or_fail_chain("explorer")
}

/// «Esporta diagnostica»: the zip for a report of a problem, secrets
/// masked (diagnostics/mod.rs); its path.
#[tauri::command]
pub async fn diagnostics_export<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::diagnostics::export(&kit, &env, lang)
            .map(|p| p.display().to_string())
            .or_fail_chain("diagnostics")
    })
    .await
    .or_fail("diagnostics")?
}

/// «Salute del kit» (diagnostics/health.rs): the drive, the space, the kit's folder,
/// files of another account, the libraries of Linux.
#[tauri::command]
pub async fn kit_health<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::diagnostics::health::Finding>> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::diagnostics::health::check(&kit, lang))
        .await
        .or_fail("health")
}

/// «Segnala un problema»: a new issue of Atollo on GitHub, in the browser,
/// with its version and system filled in (nothing is sent by the app).
#[tauri::command]
pub fn issue_open<R: Runtime>(webview: Webview<R>, state: State<'_, AppState>) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::github::open_web(&crate::diagnostics::issue_path(&state.kit)).or_fail_chain("browser")
}

/// Opens a local address (a project's server) in the PC's browser, outside
/// the job: it stays open after the app.
#[tauri::command]
pub fn url_open<R: Runtime>(webview: Webview<R>, url: String) -> CmdResult<()> {
    ensure_main(&webview)?;
    if !is_local_url(&url) {
        return Err(format!("not a local address: {url}").into());
    }
    open_in_browser(&url)
}

/// The system's window to choose a folder (`files`: a file instead);
/// `None` when it was closed without a choice.
fn pick<R: Runtime>(
    webview: &Webview<R>,
    title: String,
    files: bool,
) -> CmdResult<impl FnOnce() -> anyhow::Result<Option<PathBuf>> + Send + 'static> {
    let owner = crate::platform::desktop::owner(&webview.window()).or_fail("window")?;
    Ok(move || {
        if files {
            let picked = crate::platform::desktop::pick_files(&owner, &title, false)?;
            return Ok(picked.into_iter().next());
        }
        crate::platform::desktop::pick_folder(&owner, &title, None)
    })
}

/// «Esporta il kit» (N-12): a copy of `projects\` and `data\` without what
/// is downloaded again, into the folder chosen, as a folder or a zip. The
/// database servers are stopped for it, the automatic ones started again.
#[tauri::command]
pub async fn kit_export<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    zip: bool,
) -> CmdResult<Option<crate::kit::backup::Exported>> {
    ensure_main(&webview)?;
    let choose = pick(
        &webview,
        tr(&state, "Dove salvare la copia del kit", &[]),
        false,
    )?;
    let (kit, env, services) = (state.kit.clone(), state.env(), Arc::clone(&state.services));
    tauri::async_runtime::spawn_blocking(move || {
        let Some(dest) = choose()? else {
            return Ok(None);
        };
        services.stop_all();
        let result = crate::kit::backup::export(kit.root(), &dest, zip);
        services.autostart(&env);
        result.map(Some)
    })
    .await
    .or_fail("export")?
    .or_fail_chain("export")
}

/// «Importa una copia» (N-12): the copy chosen (a folder or a zip made by
/// «Esporta il kit») checked and put aside in the kit; applied when the app
/// starts again.
#[tauri::command]
pub async fn kit_import<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    zip: bool,
) -> CmdResult<Option<crate::kit::backup::Manifest>> {
    ensure_main(&webview)?;
    let choose = pick(
        &webview,
        tr(&state, "Scegli la copia del kit da importare", &[]),
        zip,
    )?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(source) = choose()? else {
            return Ok(None);
        };
        crate::kit::backup::stage(kit.root(), &source).map(Some)
    })
    .await
    .or_fail("import")?
    .or_fail_chain("import")
}

/// A copy imported waits for the next start.
#[tauri::command]
pub fn kit_import_pending<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<bool> {
    ensure_main(&webview)?;
    Ok(crate::kit::backup::pending(state.kit.root()))
}

/// The copy waiting is given up: the next start is as before. Off the
/// window's thread: the copy is a whole kit's `projects\` and `data\` to
/// delete.
#[tauri::command]
pub async fn kit_import_cancel<R: Runtime>(webview: Webview<R>) -> CmdResult<()> {
    ensure_main(&webview)?;
    off_main(&webview, "import", |_, state| {
        crate::kit::backup::cancel(state.kit.root()).or_fail_chain("import")
    })
    .await
}

/// A project's site from the phone (N-6): its address with this PC's IP on
/// the local network and the QR, when «Rete locale» is on; otherwise none.
#[tauri::command]
pub fn lan_link<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    url: String,
) -> CmdResult<Option<crate::net::lan::LanLink>> {
    ensure_main(&webview)?;
    if !is_local_url(&url) {
        return Ok(None);
    }
    let allowed = state.env().get("ATOLLO_ALLOW_LAN") == Some("1");
    Ok(crate::net::lan::lan_link(&url, allowed))
}

/// Opens Atollo's privacy policy (`docs/PRIVACY.md`, in the app's
/// language) on github.com in the PC's browser.
#[tauri::command]
pub fn privacy_open<R: Runtime>(webview: Webview<R>, state: State<'_, AppState>) -> CmdResult<()> {
    ensure_main(&webview)?;
    let file = match state.lang() {
        "en" => "PRIVACY.md".to_owned(),
        lang => format!("PRIVACY.{lang}.md"),
    };
    crate::github::open_web(&format!("nabre/atollo/blob/main/docs/{file}")).or_fail_chain("browser")
}

/// The PC's browser.
pub(super) fn open_in_browser(url: &str) -> CmdResult<()> {
    crate::platform::jobs::open_url(url).or_fail_chain("browser")
}

/// `http://localhost:<port>/…`, `http://127.0.0.1:<port>/…` or a name under
/// `localhost` (`http://<project>.localhost:<port>/`, the sites of a pool,
/// D-101), which every browser resolves to the PC itself.
pub(super) fn is_local_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let (name, port) = host.rsplit_once(':').unwrap_or((host, "80"));
    let under_localhost = name.strip_suffix(".localhost").is_some_and(|labels| {
        labels
            .split('.')
            .all(|l| !l.is_empty() && l.chars().all(|c| c.is_ascii_alphanumeric() || c == '-'))
    });
    (matches!(name, "localhost" | "127.0.0.1") || under_localhost)
        && port.parse::<u16>().is_ok()
        && !url
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '"')
}

/// The Settings page: what is saved, without the Gemini key itself.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub settings: Settings,
    pub gemini_key_set: bool,
    /// The file could not be read: defaults shown, file not overwritten
    /// until the user saves.
    pub problem: Option<String>,
}

#[tauri::command]
pub fn settings_get<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<SettingsView> {
    ensure_main(&webview)?;
    let loaded = Settings::load(&state.kit);
    let mut settings = loaded.settings;
    let gemini_key_set = settings
        .gemini_api_key
        .as_deref()
        .is_some_and(|k| !k.is_empty());
    settings.gemini_api_key = None;
    Ok(SettingsView {
        settings,
        gemini_key_set,
        problem: loaded.problem,
    })
}

/// Saves the settings and applies them: the kit's environment is rebuilt
/// for consoles, agents, installations and the next start of the editor.
/// The page calls it for each change (D-137). `gemini`: `None` keeps the
/// saved key, `Some("")` removes it. The Tools page's «Piattaforma» and the
/// keys of a newer version stay as they are on disk.
///
/// Off the window's thread: applying them builds the kit's environment
/// again (the certificates, every project's files, the launchers of the
/// versions), seconds on a slow disk. One save at a time, whole, in the
/// order asked ([`super::SETTINGS`]): a save asked while another applies
/// waits for it, so neither puts back the other's values half way, and an
/// older one never applies after a newer one (the page sends the next
/// save only after the one before answered, `ui/src/settings/autosave.ts`;
/// the Gemini key's save comes from another page).
#[tauri::command]
pub async fn settings_set<R: Runtime>(
    webview: Webview<R>,
    updater: State<'_, Arc<Updater>>,
    settings: Settings,
    gemini: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let updater = Arc::clone(&updater);
    super::in_order(
        &super::SETTINGS,
        off_main(&webview, "settings", move |app, state| {
            save_settings(app, state, &updater, settings, gemini)
        }),
    )
    .await
}

/// Saves the Gemini API key alone (`""` removes it), from the agent's page
/// of the AI page (D-165): the settings as they are on disk, read under
/// the same queue as [`settings_set`], so a save of the Settings page
/// asked meanwhile is neither lost nor put back. The same as
/// `settings_set(settings_get().settings, key)`, in one step.
#[tauri::command]
pub async fn gemini_key_set<R: Runtime>(
    webview: Webview<R>,
    updater: State<'_, Arc<Updater>>,
    key: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let updater = Arc::clone(&updater);
    super::in_order(
        &super::SETTINGS,
        off_main(&webview, "settings", move |app, state| {
            let current = Settings::load(&state.kit).settings;
            save_settings(app, state, &updater, current, Some(key))
        }),
    )
    .await
}

/// [`settings_set`]'s work, on its thread.
fn save_settings<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    updater: &Arc<Updater>,
    settings: Settings,
    gemini: Option<String>,
) -> CmdResult<()> {
    // Automatic updates turned on: the check runs now, not at the next start.
    let update_now = write_settings(&state.kit, state.lang(), settings, gemini)?;
    if update_now && state.host.get("ATOLLO_NO_UPDATE").is_none() {
        let (kit, token, updater) = (
            state.kit.clone(),
            github_token(&state.host),
            Arc::clone(updater),
        );
        std::thread::spawn(move || {
            if let Ok(net) = crate::net::Net::new(&Settings::load(&kit).settings, token) {
                updater.check(&net);
            }
        });
    }
    let lang = apply_env(state);
    // Every window, the editor's too, light or dark at once (sent to the
    // window's thread by Tauri).
    app.set_theme(Settings::load(&state.kit).settings.theme.window());
    // The tray menu in the new language (D-145).
    crate::app::tray::refresh(app);
    crate::common::log::info(&format!("settings saved, lang={lang}"));
    Ok(())
}

/// Writes `settings` into settings.json, with the Gemini key as
/// [`settings_set`] says; the Tools page's «Piattaforma» and the keys of a
/// newer version stay as they are on disk. True when automatic updates
/// were just turned on.
pub(super) fn write_settings(
    kit: &Kit,
    lang: &str,
    settings: Settings,
    gemini: Option<String>,
) -> CmdResult<bool> {
    let mut settings = settings;
    for v in [
        &mut settings.lang,
        &mut settings.network.https_proxy,
        &mut settings.network.http_proxy,
        &mut settings.network.no_proxy,
        &mut settings.network.extra_ca_certs,
        &mut settings.rust_toolchain,
    ] {
        *v = v
            .take()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
    }
    // A PEM file written relative to the kit's folder.
    if let Some(ca) = &mut settings.network.extra_ca_certs {
        if std::path::Path::new(ca.as_str()).is_relative() {
            *ca = crate::common::fsx::join_rel(kit.root(), ca)
                .display()
                .to_string();
        }
    }
    let mut update_now = false;
    Settings::update(kit, |saved, _problem| {
        update_now = settings.auto_update && !saved.auto_update;
        // A key that could not be read stays on disk unless removed on purpose.
        settings.gemini_unreadable = gemini.is_none() && saved.gemini_unreadable;
        settings.gemini_api_key = match gemini {
            None => saved.gemini_api_key.take(),
            Some(k) if k.trim().is_empty() => None,
            Some(k) => Some(k.trim().to_owned()),
        };
        settings.tools_target = saved.tools_target.take();
        settings.other = std::mem::take(&mut saved.other);
        settings.network.other = std::mem::take(&mut saved.network.other);
        if let Err((source, arg)) = settings.validate() {
            return Err(CmdError::from(crate::common::i18n::t(
                lang,
                source,
                &[&arg],
            )));
        }
        *saved = settings;
        Ok(())
    })?;
    Ok(update_now)
}

#[tauri::command]
pub fn update_status<R: Runtime>(
    webview: Webview<R>,
    updater: State<'_, Arc<Updater>>,
) -> CmdResult<crate::setup::self_update::Status> {
    ensure_main(&webview)?;
    Ok(updater.status())
}

/// Looks for a new version of the app and downloads it; it replaces the
/// exe when the app closes.
#[tauri::command]
pub async fn update_check<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    updater: State<'_, Arc<Updater>>,
) -> CmdResult<crate::setup::self_update::Status> {
    ensure_main(&webview)?;
    let (kit, host, updater) = (state.kit.clone(), state.host.clone(), Arc::clone(&updater));
    tauri::async_runtime::spawn_blocking(move || {
        let settings = Settings::load(&kit).settings;
        let net = crate::net::Net::new(&settings, github_token(&host))?;
        Ok(updater.check(&net))
    })
    .await
    .or_fail("update")?
}

/// Raises GitHub's API limit when the user has a token.
pub fn github_token(host: &crate::env::Host) -> Option<String> {
    host.get("GITHUB_TOKEN")
        .or(host.get("GH_TOKEN"))
        .map(str::to_owned)
}
