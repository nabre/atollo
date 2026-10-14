//! Tauri commands, the only way the interface reaches Rust.
//!
//! Three locks keep editor windows out (DECISIONS.md D-14): the commands are
//! listed in `build.rs` (`AppManifest::commands`), only the `main` capability
//! allows them, and each command checks the calling window's label.
//! Errors reach the interface as short text; details go to the log.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
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
    /// Database servers (D-73).
    pub services: Arc<crate::services::Services>,
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
fn project_dir(kit: &Kit, project: Option<&str>, lang: &str) -> Result<PathBuf, String> {
    let Some(name) = project else {
        return Ok(kit.projects());
    };
    validate_project_name(name).map_err(|e| e.to_string())?;
    let dir = kit.projects().join(name);
    // A link or junction is never a project (D-77).
    if crate::safe_path::is_dir(&kit.projects(), &dir) {
        Ok(dir)
    } else {
        Err(crate::i18n::t(lang, "Progetto non trovato: {0}", &[name]))
    }
}

/// Installs what an action of the window needs and is missing, with the
/// app's preparation task, and waits for it: the required components and
/// the parts of the app (PowerShell 7, GitHub CLI, the editor), plus
/// `wanted` (an agent, Git for a clone, Node.js for `npm`...) with what they
/// need (DECISIONS.md D-59). `wanted` is saved as chosen in
/// `components.txt`, so "Linguaggi e strumenti" keeps it. The editor and the
/// consoles never fall back to the programs of the PC. Progress reaches the
/// window as `setup` events. Blocking: call it off the UI thread. Returns
/// the kit environment, built again if something was installed (see
/// [`setup_notify`]).
fn ensure<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    kit: &Kit,
    lang: &str,
    wanted: &[&str],
) -> Result<KitEnv, String> {
    use tauri::Manager;
    let env = || app.state::<AppState>().env();
    let catalog = Catalog::get();
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let needed = with_needs(catalog, wanted);
    if let Some(c) = needed.iter().find(|c| arm64 && c.no_arm64) {
        return Err(crate::i18n::t(
            lang,
            "{0} non è disponibile sui PC ARM64.",
            &[&c.name],
        ));
    }
    if !wanted.is_empty() {
        let mut choice = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
        let before = choice.clone();
        for id in wanted {
            choice.set(catalog, id, true);
        }
        if choice != before {
            if let Err(e) = choice.write(kit, catalog) {
                crate::log::warn(&format!("components.txt: {e:#}"));
            }
        }
    }
    let missing = |kit: &Kit| -> Vec<String> {
        let mut ids = setup_task::missing_required(kit);
        for c in &needed {
            if install::installer(&c.id).is_some() && !c.is_installed(kit) && !ids.contains(&c.id) {
                ids.push(c.id.clone());
            }
        }
        ids
    };
    let todo = missing(kit);
    if todo.is_empty() {
        return Ok(env());
    }
    if !task.snapshot().running {
        crate::log::info(&format!("installing what is needed: {todo:?}"));
        let opts = Options {
            only: todo,
            ..Options::default()
        };
        // Refused only if a run started meanwhile: then that one is awaited.
        let _ = task.start(opts, true, setup_notify(app.clone()));
    }
    while task.snapshot().running {
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
    let still = missing(kit);
    if still.is_empty() {
        return Ok(env());
    }
    let why = task
        .snapshot()
        .error
        .map(|e| format!(" ({e})"))
        .unwrap_or_default();
    Err(crate::i18n::t(
        lang,
        "Componenti del kit non installati: {0}{1}. Riprova da Linguaggi e strumenti.",
        &[&still.join(", "), &why],
    ))
}

/// `ids` and, transitively, what they need, in catalog order.
fn with_needs<'a>(catalog: &'a Catalog, ids: &[&str]) -> Vec<&'a crate::catalog::Component> {
    let mut found: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    let mut todo: Vec<&str> = ids.to_vec();
    while let Some(id) = todo.pop() {
        if let Some(c) = catalog.find(id) {
            if found.insert(c.id.as_str()) {
                todo.extend(c.needs.iter().map(String::as_str));
            }
        }
    }
    catalog
        .all()
        .iter()
        .filter(|c| found.contains(c.id.as_str()))
        .collect()
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
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_settings::opened(&state.kit, &project);
    let title = project.clone();
    let editor = Arc::clone(&state.editor);
    let (handle, task, kit) = (app.clone(), Arc::clone(&state.setup), state.kit.clone());
    let lang = state.lang();
    let (port, token) = tauri::async_runtime::spawn_blocking(move || {
        ensure(&handle, &task, &kit, lang, &[])?;
        editor
            .ensure_running()
            .map_err(|e| fail("editor start", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("editor", e))??;
    let _ = app.emit_to(MAIN, "editor-status", state.editor.status());
    let url = editor::url(port, &token, Some(&dir));
    editor_window::open(
        &app,
        &url,
        port,
        &title,
        &dir,
        &state.kit.webview_data(),
        lang,
    )
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
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_settings::opened(&state.kit, &project);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[])?;
        terminal::open_console(&app, &kit, &env, &dir)
            .map_err(|e| fail("terminal", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("terminal", e))?
}

#[tauri::command]
pub fn components_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<ComponentInfo>, String> {
    ensure_main(&webview)?;
    Ok(component_infos(&state.kit, state.lang()))
}

/// The catalog as the "Linguaggi e strumenti" page shows it. A `noArm64`
/// component is not available on an ARM64 kit.
fn component_infos(kit: &Kit, lang: &str) -> Vec<ComponentInfo> {
    let catalog = Catalog::get();
    let choice = Choice::read(kit, catalog).unwrap_or_else(|| Choice::defaults(catalog));
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let locked = catalog.locked();
    catalog
        .all()
        .iter()
        .map(|c| {
            let no_arm64 = c.no_arm64 && arm64;
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
                available: install::installer(&c.id).is_some() && !no_arm64,
                x64_only: c.x64_only,
                no_arm64,
                needs: c.needs.clone(),
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

/// The kit environment from `settings` and what is installed now, given to
/// the app, the editor and the preparation task. Returns the language.
fn apply_env(state: &AppState, settings: Settings) -> &'static str {
    let lang = crate::i18n::resolve(
        settings.lang.as_deref(),
        state.host.get("ATOLLO_LANG"),
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
    lang
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
        return Err(tr(&state, "C'è già un'installazione in corso.", &[]));
    }
    let text = choice_text(&state.kit, &chosen);
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

/// `components.txt` for the ids in `chosen`. What this kit cannot install
/// (`noArm64` on ARM64) keeps its saved choice: the same kit, back on an x64
/// PC, must not lose it.
fn choice_text(kit: &Kit, chosen: &[String]) -> String {
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

/// Plan limits and usage of the installed agents (`stats.rs`). `force`:
/// the "Refresh" button, which asks the servers sooner.
#[tauri::command]
pub async fn agents_stats<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    force: bool,
) -> Result<crate::stats::Report, String> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || crate::stats::collect(&kit, &env, lang, force))
        .await
        .map_err(|e| fail("stats", e))
}

/// Opens an agent in a kit console in the project (never elsewhere).
#[tauri::command]
pub async fn agent_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    agent: String,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    if !crate::agents::IDS.contains(&agent.as_str()) {
        return Err(format!("unknown agent: {agent}"));
    }
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_settings::update(&state.kit, &project, |s| {
        s.agent = Some(agent.clone());
    });
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let wanted: Vec<&str> = crate::needs::of_agent(&kit, &agent).into_iter().collect();
        let env = ensure(&app, &task, &kit, lang, &wanted)?;
        crate::agents::launch(&app, &kit, &env, &agent, &dir, lang)
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
        "db" => state.kit.data().join("db"),
        "backup" => {
            let name = project.as_deref().ok_or("no project")?;
            project_dir(&state.kit, Some(name), state.lang())?;
            state.services.backup_dir(name)
        }
        "projects" => project_dir(&state.kit, project.as_deref(), state.lang())?,
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
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        crate::projects::clone(&kit, &env, &source, name.as_deref(), lang, token.as_deref())
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

// --- GitHub page: the kit's GitHub account (github.rs, DECISIONS.md D-70) ---

type GithubLogin<'a> = State<'a, Arc<crate::github::Login>>;

/// Sends the account's state to the window (`github` event).
fn github_notify<R: Runtime>(app: AppHandle<R>) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        use tauri::Manager;
        let state = app.state::<AppState>();
        let login = app.state::<Arc<crate::github::Login>>();
        let _ = app.emit_to(MAIN, "github", login.status(&state.kit, &state.env()));
    })
}

/// A refusal of GitHub's API as a short text.
fn github_error(lang: &str, e: &crate::net::ApiError) -> String {
    crate::log::warn(&format!("github: {:?} {}", e.status, e.detail));
    match e.status {
        Some(401) => crate::i18n::t(lang, "GitHub ha rifiutato l'accesso: accedi di nuovo.", &[]),
        None => crate::i18n::t(lang, "GitHub non risponde: {0}", &[&e.detail]),
        Some(_) => crate::i18n::t(lang, "GitHub: {0}", &[&e.detail]),
    }
}

/// The account's token, or the text asking to sign in.
fn github_token_of(kit: &Kit, env: &KitEnv, lang: &str) -> Result<String, String> {
    crate::github::token(kit, env)
        .ok_or_else(|| crate::i18n::t(lang, "Accedi prima a GitHub.", &[]))
}

fn github_net(kit: &Kit) -> Result<crate::net::Net, String> {
    crate::net::Net::new(&Settings::load(kit).settings, None)
        .map_err(|e| fail("github", format!("{e:#}")))
}

fn check_full_name(full_name: &str) -> Result<(), String> {
    if crate::github::valid_full_name(full_name) {
        Ok(())
    } else {
        Err(format!("not a GitHub repository: {full_name}"))
    }
}

#[tauri::command]
pub async fn github_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    login: GithubLogin<'_>,
) -> Result<crate::github::Status, String> {
    ensure_main(&webview)?;
    let (kit, env, login) = (state.kit.clone(), state.env(), Arc::clone(&login));
    tauri::async_runtime::spawn_blocking(move || login.status(&kit, &env))
        .await
        .map_err(|e| fail("github", e))
}

/// Starts the sign-in (`delete`: with the permission to delete
/// repositories) and waits for the one-time code, opening the GitHub page
/// where it goes. The end arrives as a `github` event.
#[tauri::command]
pub async fn github_sign_in<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    login: GithubLogin<'_>,
    delete: bool,
) -> Result<crate::github::Status, String> {
    ensure_main(&webview)?;
    let (task, kit, lang, login) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.lang(),
        Arc::clone(&login),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["gh"])?;
        login
            .start(&kit, &env, delete, github_notify(app.clone()))
            .map_err(|e| fail("github sign-in", format!("{e:#}")))?;
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let status = login.status(&kit, &env);
            if status.code.is_some() || !status.signing_in || std::time::Instant::now() > until {
                return Ok(status);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    })
    .await
    .map_err(|e| fail("github", e))?
}

#[tauri::command]
pub fn github_sign_in_cancel<R: Runtime>(
    webview: Webview<R>,
    login: GithubLogin<'_>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    login.cancel();
    Ok(())
}

#[tauri::command]
pub async fn github_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::github::sign_out(&kit, &env))
        .await
        .map_err(|e| fail("github", e))?
        .map_err(|e| fail("github sign-out", format!("{e:#}")))
}

#[tauri::command]
pub async fn github_repos<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<crate::github::Repos, String> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        crate::github::repos(&kit, &github_net(&kit)?, &token).map_err(|e| github_error(lang, &e))
    })
    .await
    .map_err(|e| fail("github", e))?
}

/// Clones a repository of the account into `projects\`; returns the project.
#[tauri::command]
pub async fn github_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
) -> Result<String, String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        crate::github::clone_repo(&kit, &env, &full_name, token.as_deref(), lang)
            .map_err(|e| fail("clone", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("clone", e))?
}

#[tauri::command]
pub async fn github_rename<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
    name: String,
) -> Result<crate::github::Renamed, String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let name = name.trim().to_owned();
    if !crate::github::valid_repo_name(&name) {
        return Err(tr(
            &state,
            "Nome non valido: solo lettere, cifre, «.», «-» e «_».",
            &[],
        ));
    }
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        crate::github::rename(&kit, &env, &github_net(&kit)?, &token, &full_name, &name)
            .map_err(|e| github_error(lang, &e))
    })
    .await
    .map_err(|e| fail("github", e))?
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Deleted {
    Deleted,
    /// GitHub refused: the account lacks the permission to delete
    /// repositories (or is not an administrator of this one).
    NeedsPermission,
}

#[tauri::command]
pub async fn github_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
) -> Result<Deleted, String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        match crate::github::delete(&github_net(&kit)?, &token, &full_name) {
            Ok(()) => {
                crate::log::info(&format!("github: {full_name} deleted"));
                Ok(Deleted::Deleted)
            }
            Err(e) if matches!(e.status, Some(403 | 404)) => Ok(Deleted::NeedsPermission),
            Err(e) => Err(github_error(lang, &e)),
        }
    })
    .await
    .map_err(|e| fail("github", e))?
}

/// Opens a repository on github.com (`None`: the page where the sign-in
/// code goes) in the PC's browser.
#[tauri::command]
pub fn github_open<R: Runtime>(
    webview: Webview<R>,
    full_name: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let path = match full_name {
        Some(f) => {
            check_full_name(&f)?;
            f
        }
        None => crate::github::DEVICE_PATH.to_owned(),
    };
    crate::github::open_web(&path).map_err(|e| fail("browser", format!("{e:#}")))
}

/// Adds or changes an action of the project in its panel.json (D-78); it
/// applies from the next start of that action.
#[tauri::command]
pub fn project_action_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    action: crate::detect::ActionEdit,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::save_action(&dir, &action, state.lang()).map_err(|e| format!("{e:#}"))
}

/// The project's editor settings in its .vscode\settings.json (D-78).
#[tauri::command]
pub fn project_editor_settings<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::project_vscode::EditorSettingsView, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::project_vscode::get(&dir))
}

#[tauri::command]
pub fn project_editor_settings_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    settings: crate::project_vscode::EditorSettings,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_vscode::set(&dir, &settings)
        .map_err(|e| fail("editor settings", format!("{e:#}")))
}

/// The project's own MCP servers (.mcp.json, D-78).
#[tauri::command]
pub fn project_mcp<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::project_mcp::McpView, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(crate::project_mcp::get(&dir))
}

#[tauri::command]
pub fn project_mcp_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    servers: Vec<crate::project_mcp::McpServer>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::project_mcp::set(&dir, &servers).map_err(|e| fail("project mcp", format!("{e:#}")))
}

/// Removes an action written in panel.json (a detected one comes back).
#[tauri::command]
pub fn project_action_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::remove_action(&dir, &id, state.lang()).map_err(|e| format!("{e:#}"))
}

/// The action that "Avvia" starts.
#[tauri::command]
pub fn project_action_default<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    id: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    crate::detect::set_default_action(&dir, &id, state.lang()).map_err(|e| format!("{e:#}"))
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
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
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

/// Installs the kit components a project needs and are missing (its
/// `missing`), saved as chosen like those of an action. Waits for the end.
#[tauri::command]
pub async fn project_install_missing<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let missing = crate::projects::info(&kit, &dir, lang).missing;
        let wanted: Vec<&str> = missing.iter().map(String::as_str).collect();
        ensure(&app, &task, &kit, lang, &wanted).map(|_| ())
    })
    .await
    .map_err(|e| fail("install", e))?
}

/// The kit's matrix of editor extensions by language ("Linguaggi e strumenti").
#[tauri::command]
pub fn extensions_matrix<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<crate::editor_ext::MatrixInfo, String> {
    ensure_main(&webview)?;
    Ok(crate::editor_ext::matrix_info(&state.kit))
}

/// The editor extensions added to or removed from those of the project's
/// languages; the new ones are installed now if the editor runs.
#[tauri::command]
pub async fn project_extensions<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    added: Vec<String>,
    removed: Vec<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let choice = crate::editor_ext::clean_choice(&added, &removed);
    crate::project_settings::update(&state.kit, &project, |s| s.extensions = choice);
    let editor = Arc::clone(&state.editor);
    // Busy installing: the watch of the open projects asks again (main.rs).
    tauri::async_runtime::spawn_blocking(move || {
        editor.sync_extensions();
    })
    .await
    .map_err(|e| fail("extensions", e))
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
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let runner = Arc::clone(&runner);
    tauri::async_runtime::spawn_blocking(move || {
        let detected = crate::detect::detect(&dir, lang);
        let Some(a) = detected.actions.iter().find(|a| a.id == action) else {
            return Err(crate::i18n::t(
                lang,
                "Azione non trovata: premi Aggiorna",
                &[],
            ));
        };
        let wanted: Vec<&str> = crate::needs::of_action(a).into_iter().collect();
        let env = ensure(&app, &task, &kit, lang, &wanted)?;
        runner
            .start(&project, &dir, a, &env, lang)
            .map_err(|e| fail("run", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("run", e))?
}

/// The databases of a project (D-74): those created, those it seems to use,
/// and whether its `.env` is ignored by Git.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDatabases {
    pub dbs: Vec<crate::db_auth::ProjectDb>,
    /// `mariadb`, `mongodb`, `sqlite` found by `detect::requires`.
    pub detected: Vec<String>,
    pub env_exists: bool,
    pub env_ignored: bool,
}

/// Git state and agent instructions of a project, for its page.
#[tauri::command]
pub async fn project_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<crate::projects::ProjectStatus, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let root = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || crate::projects::status(&dir, &root))
        .await
        .map_err(|e| fail("project", e))
}

#[tauri::command]
pub fn project_databases<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<ProjectDatabases, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(ProjectDatabases {
        dbs: state.services.project_dbs(&project),
        detected: crate::detect::requires(&dir)
            .into_iter()
            .filter(|id| matches!(id.as_str(), "mariadb" | "mongodb" | "sqlite"))
            .collect(),
        env_exists: dir.join(".env").is_file(),
        env_ignored: crate::db_auth::env_ignored(&dir),
    })
}

/// Creates the project's database and user: installs and starts the server
/// first if needed.
#[tauri::command]
pub async fn project_db_create<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> Result<crate::db_auth::ProjectDb, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    if !crate::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("database", format!("unknown server {engine}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.as_str()])?;
        services
            .start(&engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        services
            .project_db_create(&project, &engine)
            .map_err(|e| fail("database", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("database", e))?
}

fn project_db_of(
    state: &AppState,
    project: &str,
    engine: &str,
) -> Result<crate::db_auth::ProjectDb, String> {
    state
        .services
        .project_dbs(project)
        .into_iter()
        .find(|d| d.engine == engine)
        .ok_or_else(|| tr(state, "Crea prima il database del progetto.", &[]))
}

/// The lines of the project's `.env` that writing the connection changes.
#[tauri::command]
pub fn project_env_preview<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> Result<Vec<crate::db_auth::EnvChange>, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    Ok(crate::db_auth::env_changes(
        &crate::db_auth::env_source(&dir),
        &db,
    ))
}

/// Writes the connection into the project's `.env` (from `.env.example`
/// when it has none); only the lines shown by the preview change.
#[tauri::command]
pub fn project_env_apply<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    crate::db_auth::env_apply(&dir, &db).map_err(|e| fail(".env", format!("{e:#}")))
}

/// The copies of the project's databases (D-78), newest first.
#[tauri::command]
pub fn project_backups<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<Vec<crate::services::Backup>, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(state.services.backups(&project))
}

/// A copy of the project's database; starts the server if needed.
#[tauri::command]
pub async fn project_db_export<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> Result<String, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    if !crate::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("database", format!("unknown server {engine}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.as_str()])?;
        services
            .start(&engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        let file = services
            .export(&project, &engine)
            .map_err(|e| fail("export", format!("{e:#}")))?;
        Ok(file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default())
    })
    .await
    .map_err(|e| fail("export", e))?
}

/// Puts a copy back into the project's database; starts the server if needed.
#[tauri::command]
pub async fn project_db_restore<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    file: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let engine = if file.ends_with(".archive.gz") {
        "mongodb"
    } else {
        "mariadb"
    };
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine])?;
        services
            .start(engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        services
            .restore(&project, &file)
            .map_err(|e| fail("restore", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("restore", e))?
}

/// The database servers (D-73).
#[tauri::command]
pub fn services_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    Ok(state.services.list())
}

/// Starts a database server, installing it first if needed; waits until it
/// answers.
#[tauri::command]
pub async fn service_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<crate::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    if !crate::services::SERVERS.contains(&id.as_str()) {
        return Err(fail("service", format!("unknown server {id}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[id.as_str()])?;
        match services.start(&id, &env) {
            Ok(_) => Ok(services.list()),
            Err(e) => match e.downcast_ref::<crate::services::PortInUse>() {
                Some(p) => Err(crate::i18n::t(
                    lang,
                    "La porta {0} è già usata da un altro programma: cambiala qui sotto.",
                    &[&p.0.to_string()],
                )),
                None => Err(fail("service", format!("{e:#}"))),
            },
        }
    })
    .await
    .map_err(|e| fail("service", e))?
}

#[tauri::command]
pub async fn service_stop<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> Result<Vec<crate::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        services.stop(&id);
        services.list()
    })
    .await
    .map_err(|e| fail("service", e))
}

/// Port (null: the default) and "start with Atollo" of a server; the port
/// applies at the next start.
#[tauri::command]
pub fn service_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    port: Option<u16>,
    autostart: bool,
) -> Result<Vec<crate::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    state
        .services
        .set(&id, port, autostart)
        .map_err(|e| fail("service", format!("{e:#}")))?;
    Ok(state.services.list())
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
    let lang = apply_env(&state, settings);
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
    /// Database servers started by the app.
    pub databases: usize,
}

impl OpenNow {
    pub fn any(&self) -> bool {
        !self.processes.is_empty()
            || self.editor
            || self.terminals > 0
            || self.actions > 0
            || self.setup
            || self.databases > 0
    }
}

pub fn open_now<R: Runtime>(app: &AppHandle<R>) -> OpenNow {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let actions = app.try_state::<Arc<Runner>>().map_or(0, |r| r.running());
    OpenNow {
        processes: crate::sessions::foreign_processes(&state.kit),
        editor: state.editor.status().running,
        terminals: app.state::<terminal::Consoles>().count(),
        actions,
        setup: state.setup.snapshot().running,
        databases: state.services.running(),
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
        if label.starts_with(editor_window::LABEL_PREFIX) {
            let _ = window.destroy();
        }
    }
    let consoles = app.state::<terminal::Consoles>().pids();
    let (kit, editor, setup) = (
        state.kit.clone(),
        Arc::clone(&state.editor),
        Arc::clone(&state.setup),
    );
    let handle = app.clone();
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        // Asked to shut down before the kit's processes are ended.
        services.stop_all();
        editor.stop();
        crate::sessions::stop(&consoles);
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
    // The databases shut down cleanly first (D-73), off this thread.
    let services = Arc::clone(&app.state::<AppState>().services);
    let handle = app.clone();
    std::thread::spawn(move || {
        services.stop_all();
        handle.exit(0);
    });
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
            $crate::commands::components_list,
            $crate::commands::setup_state,
            $crate::commands::setup_start,
            $crate::commands::setup_cancel,
            $crate::commands::components_apply,
            $crate::commands::agents_status,
            $crate::commands::agents_stats,
            $crate::commands::agent_launch,
            $crate::commands::agent_sign_out,
            $crate::commands::folder_open,
            $crate::commands::project_create,
            $crate::commands::project_delete,
            $crate::commands::project_clone,
            $crate::commands::project_site,
            $crate::commands::project_install_missing,
            $crate::commands::project_extensions,
            $crate::commands::extensions_matrix,
            $crate::commands::github_status,
            $crate::commands::github_sign_in,
            $crate::commands::github_sign_in_cancel,
            $crate::commands::github_sign_out,
            $crate::commands::github_repos,
            $crate::commands::github_clone,
            $crate::commands::github_rename,
            $crate::commands::github_delete,
            $crate::commands::github_open,
            $crate::commands::project_backups,
            $crate::commands::project_db_export,
            $crate::commands::project_db_restore,
            $crate::commands::project_mcp,
            $crate::commands::project_mcp_set,
            $crate::commands::project_editor_settings,
            $crate::commands::project_editor_settings_set,
            $crate::commands::project_action_save,
            $crate::commands::project_action_remove,
            $crate::commands::project_action_default,
            $crate::commands::project_status,
            $crate::commands::project_databases,
            $crate::commands::project_db_create,
            $crate::commands::project_env_preview,
            $crate::commands::project_env_apply,
            $crate::commands::services_list,
            $crate::commands::service_start,
            $crate::commands::service_stop,
            $crate::commands::service_set,
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
        assert_eq!(project_dir(&kit, None, "it").unwrap(), kit.projects());
        assert_eq!(
            project_dir(&kit, Some("app"), "it").unwrap(),
            kit.projects().join("app")
        );
        for bad in ["..", r"..\data", "a/b", "C:\\x", "missing", "con"] {
            assert!(project_dir(&kit, Some(bad), "it").is_err(), "{bad}");
        }
    }

    #[test]
    fn r_is_not_available_on_arm64() {
        let tmp = TempDir::new("cmd-comps");
        for (arch, arm64) in [(Arch::X64, false), (Arch::Arm64, true)] {
            let kit = Kit::at(tmp.0.join(arch.as_str()), arch);
            kit.prepare().unwrap();
            let infos = component_infos(&kit, "it");
            let r = infos.iter().find(|c| c.id == "r").unwrap();
            assert_eq!(
                (r.available, r.no_arm64, r.x64_only),
                (!arm64, arm64, false)
            );
            let php = infos.iter().find(|c| c.id == "php").unwrap();
            assert!(php.available && !php.no_arm64 && php.x64_only, "emulated");
        }
    }

    #[test]
    fn unavailable_components_keep_their_choice() {
        let tmp = TempDir::new("cmd-choice");
        let catalog = Catalog::get();
        for (arch, kept) in [(Arch::X64, false), (Arch::Arm64, true)] {
            let kit = Kit::at(tmp.0.join(arch.as_str()), arch);
            kit.prepare().unwrap();
            let mut saved = Choice::defaults(catalog);
            saved.set(catalog, "r", true);
            saved.write(&kit, catalog).unwrap();
            let text = choice_text(&kit, &["git".into(), "php".into()]);
            let choice = Choice::parse(&text, catalog);
            assert_eq!(choice.is_chosen("r"), kept, "{arch:?}");
            assert!(choice.is_chosen("php") && !choice.is_chosen("go"));
        }
    }
}
