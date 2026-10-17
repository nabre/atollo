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
    /// Linux, WP-1.7); `None` on Windows.
    pub disk_noexec: Option<bool>,
    /// The app's notice about the kit's disk, translated (noexec,
    /// showexec, FAT32, exFAT); `None` on Windows or when all is well.
    pub disk_note: Option<String>,
    /// The host's temporary folder when the programs' `TEMP` is in it;
    /// `None` when it is `data\tmp` in the kit.
    pub system_temp: Option<String>,
}

#[tauri::command]
pub fn app_info<R: Runtime>(webview: Webview<R>, state: State<'_, AppState>) -> CmdResult<AppInfo> {
    ensure_main(&webview)?;
    let kit = &state.kit;
    let env = state.env();
    let system_temp = kit
        .system_temp()
        .filter(|host| {
            env.get("TEMP")
                .is_some_and(|t| std::path::Path::new(t).starts_with(host))
        })
        .map(|host| host.display().to_string());
    Ok(AppInfo {
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
        other_kits: crate::os::sessions::other_kits(kit)
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        disk_fs: kit.volume().map(|v| v.fs.as_str()),
        disk_kind: kit.volume().map(|v| v.drive.as_str()),
        disk_noexec: (!crate::kit::Os::current().is_windows())
            .then(|| crate::os::volume::kit_noexec(kit)),
        disk_note: crate::os::volume::disk_notice(kit, state.lang()).map(|n| n.text),
        system_temp,
    })
}

/// The kit environment from `settings` and what is installed now, given to
/// the app, the editor and the preparation task. Returns the language.
pub(super) fn apply_env(state: &AppState, settings: Settings) -> &'static str {
    let lang = crate::i18n::resolve(
        settings.lang.as_deref(),
        state.host.get("DEVKIT_LANG"),
        crate::i18n::windows_locale().as_deref(),
    );
    let env = KitEnv::build(&state.kit, &settings, lang, &state.host);
    if let Err(e) = env.create_dirs() {
        crate::log::warn(&format!("creating kit folders: {e}"));
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
/// instructions, skills, MCP), `projects` or one project.
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
        "backup" => {
            let name = project.as_deref().ok_or("no project")?;
            project_dir(&state.kit, Some(name), state.lang())?;
            state.services.backup_dir(name).or_fail_chain("folder")?
        }
        "projects" => project_dir(&state.kit, project.as_deref(), state.lang())?,
        _ => return Err(format!("unknown folder: {which}").into()),
    };
    std::fs::create_dir_all(&dir).or_fail("folder")?;
    crate::os::jobs::open_with_explorer(&dir).or_fail_chain("explorer")
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

/// The PC's browser.
pub(super) fn open_in_browser(url: &str) -> CmdResult<()> {
    crate::os::jobs::open_url(url).or_fail_chain("browser")
}

/// `http://localhost:<port>/…` or `http://127.0.0.1:<port>/…` only.
pub(super) fn is_local_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("http://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    let (name, port) = host.rsplit_once(':').unwrap_or((host, "80"));
    matches!(name, "localhost" | "127.0.0.1")
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
/// `gemini`: `None` keeps the saved key, `Some("")` removes it.
#[tauri::command]
pub fn settings_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    mut settings: Settings,
    gemini: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let kit = &state.kit;
    settings.gemini_api_key = match gemini {
        None => Settings::load(kit).settings.gemini_api_key,
        Some(k) if k.trim().is_empty() => None,
        Some(k) => Some(k.trim().to_owned()),
    };
    for v in [
        &mut settings.lang,
        &mut settings.network.https_proxy,
        &mut settings.network.http_proxy,
        &mut settings.network.no_proxy,
        &mut settings.network.extra_ca_certs,
        &mut settings.python_version,
        &mut settings.php_version,
        &mut settings.rust_toolchain,
    ] {
        *v = v
            .take()
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty());
    }
    if let Err((source, arg)) = settings.validate() {
        return Err(tr(&state, source, &[&arg]).into());
    }
    settings.save(kit).or_fail_chain("settings")?;
    let lang = apply_env(&state, settings);
    crate::log::info(&format!("settings saved, lang={lang}"));
    Ok(())
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
