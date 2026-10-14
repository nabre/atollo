//! Tauri commands, the only way the interface reaches Rust.
//!
//! Three locks keep editor windows out (DECISIONS.md D-14): the commands are
//! listed in `build.rs` (`AppManifest::commands`), only the `main` capability
//! allows them, and each command checks the calling window's label.
//! Errors reach the interface as short text; details go to the log.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::{AppHandle, Emitter, Runtime, State, Webview};

use crate::catalog::{Catalog, Choice, Group};
use crate::editor::{self, Editor};
use crate::env::KitEnv;
use crate::kit::{validate_project_name, Kit};
use crate::run::{Runner, State as RunState};
use crate::self_update::Updater;
use crate::settings::Settings;
use crate::setup_runner::Options;
use crate::setup_task::{self, Notify, SetupTask, Snapshot};
use crate::{editor_window, install, terminal};

/// Label of the app window; the only one allowed to call commands.
pub const MAIN: &str = "main";

pub struct AppState {
    pub kit: Kit,
    pub lang: std::sync::RwLock<&'static str>,
    /// The real environment at start, to rebuild the kit's one.
    pub host: crate::env::Host,
    pub moved_from: Option<String>,
    pub webview: String,
    pub env: std::sync::RwLock<KitEnv>,
    pub editor: Arc<Editor>,
    pub setup: Arc<SetupTask>,
}

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
}

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
    pub version: Option<String>,
    /// This version of the app can install it.
    pub available: bool,
    pub x64_only: bool,
    /// Components it needs (Apache needs PHP, Kotlin needs Java).
    pub needs: Vec<String>,
}

impl AppState {
    pub fn env(&self) -> KitEnv {
        self.env
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn lang(&self) -> &'static str {
        *self
            .lang
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

fn ensure_main<R: Runtime>(webview: &Webview<R>) -> Result<(), String> {
    if webview.label() == MAIN {
        Ok(())
    } else {
        Err("forbidden".into())
    }
}

fn fail(context: &str, e: impl std::fmt::Display) -> String {
    crate::log::error(&format!("{context}: {e}"));
    e.to_string()
}

/// `None` = the projects folder itself; a name must be a valid project
/// name of an existing folder (no paths, no `..`).
fn project_dir(kit: &Kit, project: Option<&str>) -> Result<PathBuf, String> {
    let Some(name) = project else {
        return Ok(kit.projects());
    };
    validate_project_name(name).map_err(|e| e.to_string())?;
    let dir = kit.projects().join(name);
    if dir.is_dir() {
        Ok(dir)
    } else {
        Err(format!("project not found: {name}"))
    }
}

/// Installs the required components that are missing (PowerShell 7, Git,
/// Node.js, the editor...) with the app's preparation task and waits for it:
/// the editor and the consoles never fall back to the shells of the PC.
/// Progress reaches the window as `setup` events. Blocking: call it off the
/// UI thread.
fn ensure_required<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    kit: &Kit,
) -> Result<(), String> {
    let missing = setup_task::missing_required(kit);
    if missing.is_empty() {
        return Ok(());
    }
    if !task.snapshot().running {
        crate::log::info(&format!(
            "installing missing required components: {missing:?}"
        ));
        let opts = Options {
            only: missing,
            ..Options::default()
        };
        // Refused only if a run started meanwhile: then that one is awaited.
        let _ = task.start(opts, true, setup_notify(app.clone()));
    }
    while task.snapshot().running {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    let still = setup_task::missing_required(kit);
    if still.is_empty() {
        return Ok(());
    }
    let why = task
        .snapshot()
        .error
        .map(|e| format!(" ({e})"))
        .unwrap_or_default();
    Err(format!(
        "Componenti del kit non installati: {}{why}. Riprova da Linguaggi e strumenti.",
        still.join(", ")
    ))
}

#[tauri::command]
pub fn app_info<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<AppInfo, String> {
    ensure_main(&webview)?;
    let kit = &state.kit;
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION"),
        root: kit.root().display().to_string(),
        data: kit.data().display().to_string(),
        projects: kit.projects().display().to_string(),
        platform: kit.arch().platform(),
        lang: state.lang(),
        moved_from: state.moved_from.clone(),
        webview: state.webview.clone(),
        path_entries: state.env().path_entries().len(),
        watcher_unsafe_folder: crate::kit::watcher_unsafe_folder(kit.root()),
        first_run: Choice::read(kit, Catalog::get()).is_none(),
        other_kits: crate::sessions::other_kits(kit)
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
    })
}

#[tauri::command]
pub fn editor_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<editor::Status, String> {
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
    project: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, project.as_deref())?;
    let title = project.clone().unwrap_or_else(|| "projects".into());
    let editor = Arc::clone(&state.editor);
    let (handle, task, kit) = (app.clone(), Arc::clone(&state.setup), state.kit.clone());
    let (port, token) = tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&handle, &task, &kit)?;
        editor
            .ensure_running()
            .map_err(|e| fail("editor start", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("editor", e))??;
    let _ = app.emit_to(MAIN, "editor-status", state.editor.status());
    let url = editor::url(port, &token, Some(&dir));
    editor_window::open(&app, &url, port, &title, &state.kit.webview_data())
        .map_err(|e| fail("editor window", format!("{e:#}")))
}

#[tauri::command]
pub async fn editor_restart<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let editor = Arc::clone(&state.editor);
    tauri::async_runtime::spawn_blocking(move || {
        editor.stop();
        editor.ensure_running().map(|_| ())
    })
    .await
    .map_err(|e| fail("editor", e))?
    .map_err(|e| fail("editor restart", format!("{e:#}")))?;
    let _ = app.emit_to(MAIN, "editor-status", state.editor.status());
    Ok(())
}

/// The projects with what each one is and how it starts (detection reads
/// files: off the UI thread).
#[tauri::command]
pub async fn projects_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::projects::ProjectInfo>, String> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::projects::list(&kit, lang))
        .await
        .map_err(|e| fail("projects", e))
}

#[tauri::command]
pub async fn terminal_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, project.as_deref())?;
    let (task, kit, env) = (Arc::clone(&state.setup), state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        terminal::open_console(&app, &kit, &env, &dir)
            .map_err(|e| fail("terminal", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("terminal", e))?
}

/// The session of a console window: its own label only (D-14).
fn terminal_label<R: Runtime>(webview: &Webview<R>) -> Result<&str, String> {
    let label = webview.label();
    if label.starts_with(terminal::LABEL_PREFIX) {
        Ok(label)
    } else {
        Err("forbidden".into())
    }
}

/// A console window connects to its process: past and new output on
/// `output` (bytes), the exit code on `exit`. Returns the title.
#[tauri::command]
pub fn terminal_attach<R: Runtime>(
    webview: Webview<R>,
    terminals: State<'_, terminal::Terminals>,
    output: Channel<InvokeResponseBody>,
    exit: Channel<u32>,
) -> Result<String, String> {
    terminals.attach(terminal_label(&webview)?, output, exit)
}

/// Keys and pasted text of a console window, to its process.
#[tauri::command]
pub fn terminal_input<R: Runtime>(
    webview: Webview<R>,
    terminals: State<'_, terminal::Terminals>,
    data: String,
) -> Result<(), String> {
    terminals.input(terminal_label(&webview)?, data.as_bytes())
}

#[tauri::command]
pub fn terminal_resize<R: Runtime>(
    webview: Webview<R>,
    terminals: State<'_, terminal::Terminals>,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    terminals.resize(terminal_label(&webview)?, cols, rows)
}

#[tauri::command]
pub fn components_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<ComponentInfo>, String> {
    ensure_main(&webview)?;
    let kit = &state.kit;
    let catalog = Catalog::get();
    let choice = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    Ok(catalog
        .all()
        .iter()
        .map(|c| ComponentInfo {
            id: c.id.clone(),
            name: c.name.clone(),
            group: c.group,
            desc: c.desc.clone(),
            mb: c.mb,
            chosen: choice.is_chosen(&c.id),
            installed: c.is_installed(kit),
            version: c.installed_version(kit),
            available: install::installer(&c.id).is_some(),
            x64_only: c.x64_only,
            needs: c.needs.clone(),
        })
        .collect())
}

/// Sends every change of the preparation to the app window.
pub fn setup_notify<R: Runtime>(app: AppHandle<R>) -> Notify {
    Box::new(move |snapshot: &Snapshot| {
        let _ = app.emit_to(MAIN, "setup", snapshot);
    })
}

#[tauri::command]
pub fn setup_state<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Snapshot, String> {
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    let only = only.unwrap_or_default();
    if let Some(bad) = only.iter().find(|id| Catalog::get().find(id).is_none()) {
        return Err(format!("unknown component: {bad}"));
    }
    let opts = Options {
        only,
        update_all,
        ..Options::default()
    };
    state.setup.start(opts, false, setup_notify(app))
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    let catalog = Catalog::get();
    if let Some(bad) = chosen.iter().find(|id| catalog.find(id).is_none()) {
        return Err(format!("unknown component: {bad}"));
    }
    if state.setup.snapshot().running {
        return Err("an installation is already running".into());
    }
    let text: String = catalog
        .all()
        .iter()
        .map(|c| {
            format!(
                "{}={}
",
                c.id,
                u8::from(chosen.contains(&c.id))
            )
        })
        .collect();
    let choice = Choice::parse(&text, catalog);
    choice
        .write(&state.kit, catalog)
        .map_err(|e| fail("components", format!("{e:#}")))?;
    let opts = Options {
        update_all,
        ..Options::default()
    };
    state.setup.start(opts, false, setup_notify(app))
}

#[tauri::command]
pub fn setup_cancel<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    state.setup.cancel(&setup_notify(app));
    Ok(())
}

#[tauri::command]
pub fn agents_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::agents::AgentInfo>, String> {
    ensure_main(&webview)?;
    Ok(crate::agents::status(&state.kit, &state.env()))
}

/// Opens an agent in a kit console in the project (or `projects\`).
#[tauri::command]
pub async fn agent_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    project: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    if !crate::agents::IDS.contains(&agent.as_str()) {
        return Err(format!("unknown agent: {agent}"));
    }
    let dir = project_dir(&state.kit, project.as_deref())?;
    let (task, kit, env) = (Arc::clone(&state.setup), state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        crate::agents::launch(&app, &kit, &env, &agent, &dir)
            .map_err(|e| fail("agent", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("agent", e))?
}

#[tauri::command]
pub fn agent_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::agents::sign_out(&state.kit, &state.env(), &agent)
        .map_err(|e| fail("sign out", format!("{e:#}")))
}

/// Opens a folder of the kit in File Explorer: `agents` (shared
/// instructions, skills, MCP), `projects` or one project.
#[tauri::command]
pub fn folder_open<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    which: String,
    project: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = match which.as_str() {
        "agents" => state.kit.data().join("agents"),
        "projects" => project_dir(&state.kit, project.as_deref())?,
        _ => return Err(format!("unknown folder: {which}")),
    };
    std::fs::create_dir_all(&dir).map_err(|e| fail("folder", e))?;
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    crate::jobs::spawn_detached(
        std::process::Command::new(std::path::Path::new(&system_root).join("explorer.exe"))
            .arg(&dir),
        0,
    )
    .map(|_| ())
    .map_err(|e| fail("explorer", format!("{e:#}")))
}

fn lang_of(state: &AppState) -> &'static str {
    state.lang()
}

fn tr(state: &AppState, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang_of(state), source, args)
}

/// A new empty project in `projects\`.
#[tauri::command]
pub fn project_create<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    name: String,
) -> Result<String, String> {
    ensure_main(&webview)?;
    crate::projects::create(&state.kit, &name, state.lang())
        .map(|d| {
            d.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
        })
        .map_err(|e| format!("{e:#}"))
}

/// Deletes a project whose folder is empty.
#[tauri::command]
pub fn project_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::projects::delete_empty(&state.kit, &project, state.lang()).map_err(|e| format!("{e:#}"))
}

/// Clones `owner/repo` or an `https://` Git URL into `projects\<name>`.
#[tauri::command]
pub async fn project_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    source: String,
    name: Option<String>,
) -> Result<String, String> {
    ensure_main(&webview)?;
    let (task, kit, env, lang) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.env(),
        state.lang(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        crate::projects::clone(&kit, &env, &source, name.as_deref(), lang)
            .map(|d| {
                d.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .map_err(|e| fail("clone", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("clone", e))?
}

/// "Servi come sito web": a folder of the project served by the file server
/// (`static`) or by PHP (`php`), or the site removed (`none`).
#[tauri::command]
pub fn project_site<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    folder: String,
    kind: crate::detect::SiteKind,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project))?;
    let key = crate::run::key_of(&project, crate::detect::SITE_ACTION);
    if runner
        .list()
        .iter()
        .any(|r| r.key == key && matches!(r.state, RunState::Starting | RunState::Running))
    {
        return Err(tr(
            &state,
            "Ferma prima il sito web: la modifica vale dal prossimo avvio.",
            &[],
        ));
    }
    crate::detect::set_site(&dir, &folder, kind, state.lang()).map_err(|e| format!("{e:#}"))
}

#[tauri::command]
pub fn run_list<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> Result<Vec<crate::run::RunInfo>, String> {
    ensure_main(&webview)?;
    Ok(runner.list())
}

/// Starts an action of a project (detected again: panel.json may have changed).
#[tauri::command]
pub async fn run_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    action: String,
) -> Result<crate::run::RunInfo, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project))?;
    let (task, kit, env, lang) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.env(),
        state.lang(),
    );
    let runner = Arc::clone(&runner);
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        let detected = crate::detect::detect(&dir, lang);
        let Some(a) = detected.actions.iter().find(|a| a.id == action) else {
            return Err(crate::i18n::t(
                lang,
                "Azione non trovata: premi Aggiorna",
                &[],
            ));
        };
        runner
            .start(&project, &dir, a, &env, lang)
            .map_err(|e| fail("run", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("run", e))?
}

#[tauri::command]
pub fn run_stop<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    runner.stop(&key);
    Ok(())
}

#[tauri::command]
pub fn run_log<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<Vec<String>, String> {
    ensure_main(&webview)?;
    Ok(runner.log(&key))
}

#[tauri::command]
pub fn run_clear<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    runner.clear(&key);
    Ok(())
}

/// The ports remembered for the web actions, and the locked ones.
#[tauri::command]
pub fn run_ports<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> Result<crate::run::Ports, String> {
    ensure_main(&webview)?;
    Ok(runner.ports())
}

/// The user's port for an action (`None`: automatic again).
#[tauri::command]
pub fn run_set_port<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    key: String,
    port: Option<u16>,
) -> Result<crate::run::Ports, String> {
    ensure_main(&webview)?;
    runner
        .set_port(&key, port, state.lang())
        .map_err(|e| format!("{e:#}"))
}

/// Opens a local address (a project's server) in the PC's browser, outside
/// the job: it stays open after the app.
#[tauri::command]
pub fn url_open<R: Runtime>(webview: Webview<R>, url: String) -> Result<(), String> {
    ensure_main(&webview)?;
    if !is_local_url(&url) {
        return Err(format!("not a local address: {url}"));
    }
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    crate::jobs::spawn_detached(
        std::process::Command::new(std::path::Path::new(&system_root).join("explorer.exe"))
            .arg(&url),
        0,
    )
    .map(|_| ())
    .map_err(|e| fail("browser", format!("{e:#}")))
}

/// `http://localhost:<port>/…` or `http://127.0.0.1:<port>/…` only.
fn is_local_url(url: &str) -> bool {
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
) -> Result<SettingsView, String> {
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
) -> Result<(), String> {
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
        return Err(tr(&state, source, &[&arg]));
    }
    settings
        .save(kit)
        .map_err(|e| fail("settings", format!("{e:#}")))?;
    let lang = crate::i18n::resolve(
        settings.lang.as_deref(),
        state.host.get("DEVKIT_LANG"),
        crate::i18n::windows_locale().as_deref(),
    );
    let env = KitEnv::build(kit, &settings, lang, &state.host);
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
    crate::log::info(&format!("settings saved, lang={lang}"));
    Ok(())
}

#[tauri::command]
pub fn update_status<R: Runtime>(
    webview: Webview<R>,
    updater: State<'_, Arc<Updater>>,
) -> Result<crate::self_update::Status, String> {
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
) -> Result<crate::self_update::Status, String> {
    ensure_main(&webview)?;
    let (kit, host, updater) = (state.kit.clone(), state.host.clone(), Arc::clone(&updater));
    tauri::async_runtime::spawn_blocking(move || {
        let settings = Settings::load(&kit).settings;
        let net =
            crate::net::Net::new(&settings, github_token(&host)).map_err(|e| format!("{e:#}"))?;
        Ok(updater.check(&net))
    })
    .await
    .map_err(|e| fail("update", e))?
}

/// Raises GitHub's API limit when the user has a token.
pub fn github_token(host: &crate::env::Host) -> Option<String> {
    host.get("GITHUB_TOKEN")
        .or(host.get("GH_TOKEN"))
        .map(str::to_owned)
}

/// What is still open in this kit: "Rimuovi l'unità" and the close question.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenNow {
    pub processes: Vec<crate::sessions::KitProcess>,
    pub editor: bool,
    pub terminals: usize,
    pub actions: usize,
    pub setup: bool,
}

impl OpenNow {
    pub fn any(&self) -> bool {
        !self.processes.is_empty()
            || self.editor
            || self.terminals > 0
            || self.actions > 0
            || self.setup
    }
}

pub fn open_now<R: Runtime>(app: &AppHandle<R>) -> OpenNow {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let actions = app.try_state::<Arc<Runner>>().map_or(0, |r| r.running());
    OpenNow {
        processes: crate::sessions::foreign_processes(&state.kit),
        editor: state.editor.status().running,
        terminals: app
            .webview_windows()
            .keys()
            .filter(|l| l.starts_with(terminal::LABEL_PREFIX))
            .count(),
        actions,
        setup: state.setup.snapshot().running,
    }
}

#[tauri::command]
pub fn eject_list<R: Runtime>(app: AppHandle<R>, webview: Webview<R>) -> Result<OpenNow, String> {
    ensure_main(&webview)?;
    Ok(open_now(&app))
}

/// "Rimuovi l'unità": stops the installation, the projects' actions, the
/// editor, the consoles and every program of the kit, and puts Antigravity's
/// login back into the kit. Returns what is still open (normally nothing);
/// then the interface asks to close the app.
#[tauri::command]
pub async fn eject_run<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<OpenNow, String> {
    ensure_main(&webview)?;
    use tauri::Manager;
    state.setup.cancel(&setup_notify(app.clone()));
    if let Some(r) = app.try_state::<Arc<Runner>>() {
        r.stop_all();
    }
    for (label, window) in app.webview_windows() {
        if label.starts_with(terminal::LABEL_PREFIX)
            || label.starts_with(editor_window::LABEL_PREFIX)
        {
            let _ = window.destroy();
        }
    }
    let (kit, editor, setup) = (
        state.kit.clone(),
        Arc::clone(&state.editor),
        Arc::clone(&state.setup),
    );
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        editor.stop();
        let pids: Vec<u32> = crate::sessions::kit_processes(&kit)
            .iter()
            .map(|p| p.pid)
            .collect();
        crate::sessions::stop(&pids);
        // The installation ends at the end of its step: waited a little.
        for _ in 0..100 {
            if !setup.snapshot().running {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        if let Err(e) = crate::keyring::save_after_run(&kit) {
            crate::log::warn(&format!("eject, antigravity login: {e:#}"));
        }
        crate::log::info("eject: programs of the kit stopped");
        let _ = handle.emit_to(MAIN, "editor-status", editor.status());
        Ok(open_now(&handle))
    })
    .await
    .map_err(|e| fail("eject", e))?
}

/// Closes the app after the user confirmed (close question, eject).
#[tauri::command]
pub fn app_quit<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    quit: State<'_, Quitting>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    quit.0.store(true, std::sync::atomic::Ordering::SeqCst);
    use tauri::Manager;
    if let Some(r) = app.try_state::<Arc<Runner>>() {
        r.stop_all();
    }
    app.exit(0);
    Ok(())
}

/// Set once the user confirmed closing: the window closes without asking.
#[derive(Default)]
pub struct Quitting(pub std::sync::atomic::AtomicBool);

/// The app window is closing: with the editor, consoles, projects'
/// servers or an installation open, the interface asks first
/// (`close-requested` event); otherwise it closes.
pub fn on_main_close<R: Runtime>(window: &tauri::Window<R>, api: &tauri::CloseRequestApi) {
    use tauri::Manager;
    let app = window.app_handle();
    if app
        .try_state::<Quitting>()
        .is_some_and(|q| q.0.load(std::sync::atomic::Ordering::SeqCst))
    {
        return;
    }
    let open = open_now(app);
    if open.any() {
        api.prevent_close();
        let _ = app.emit_to(MAIN, "close-requested", open);
    }
}

/// The handler with every command; keep in sync with `build.rs` COMMANDS.
#[macro_export]
macro_rules! command_handler {
    () => {
        tauri::generate_handler![
            $crate::commands::app_info,
            $crate::commands::editor_status,
            $crate::commands::editor_open,
            $crate::commands::editor_restart,
            $crate::commands::projects_list,
            $crate::commands::terminal_open,
            $crate::commands::terminal_attach,
            $crate::commands::terminal_input,
            $crate::commands::terminal_resize,
            $crate::commands::components_list,
            $crate::commands::setup_state,
            $crate::commands::setup_start,
            $crate::commands::setup_cancel,
            $crate::commands::components_apply,
            $crate::commands::agents_status,
            $crate::commands::agent_launch,
            $crate::commands::agent_sign_out,
            $crate::commands::folder_open,
            $crate::commands::project_create,
            $crate::commands::project_delete,
            $crate::commands::project_clone,
            $crate::commands::project_site,
            $crate::commands::run_list,
            $crate::commands::run_start,
            $crate::commands::run_stop,
            $crate::commands::run_log,
            $crate::commands::run_clear,
            $crate::commands::run_ports,
            $crate::commands::run_set_port,
            $crate::commands::url_open,
            $crate::commands::settings_get,
            $crate::commands::settings_set,
            $crate::commands::update_status,
            $crate::commands::update_check,
            $crate::commands::eject_list,
            $crate::commands::eject_run,
            $crate::commands::app_quit
        ]
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use std::fs;

    #[test]
    fn project_names_cannot_escape() {
        let tmp = TempDir::new("cmd-proj");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        fs::create_dir_all(kit.projects().join("app")).unwrap();
        assert_eq!(project_dir(&kit, None).unwrap(), kit.projects());
        assert_eq!(
            project_dir(&kit, Some("app")).unwrap(),
            kit.projects().join("app")
        );
        for bad in ["..", r"..\data", "a/b", "C:\\x", "missing", "con"] {
            assert!(project_dir(&kit, Some(bad)).is_err(), "{bad}");
        }
    }
}
