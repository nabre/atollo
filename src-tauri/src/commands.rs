//! Tauri commands, the only way the interface reaches Rust.
//!
//! Three locks keep editor windows out (DECISIONS.md D-14): the commands are
//! listed in `build.rs` (`AppManifest::commands`), only the `main` capability
//! allows them, and each command checks the calling window's label.
//! Errors reach the interface as short text; details go to the log.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::UNIX_EPOCH;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State, Webview};

use crate::catalog::{Catalog, Choice, Group};
use crate::editor::{self, Editor};
use crate::env::KitEnv;
use crate::kit::{validate_project_name, Kit};
use crate::setup_runner::Options;
use crate::setup_task::{self, Notify, SetupTask, Snapshot};
use crate::{editor_window, install, terminal};

/// Label of the app window; the only one allowed to call commands.
pub const MAIN: &str = "main";

pub struct AppState {
    pub kit: Kit,
    pub lang: &'static str,
    pub moved_from: Option<String>,
    pub webview: String,
    pub env: KitEnv,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectEntry {
    pub name: String,
    /// Seconds since 1970, for sorting and "modified" labels.
    pub modified: u64,
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
        lang: state.lang,
        moved_from: state.moved_from.clone(),
        webview: state.webview.clone(),
        path_entries: state.env.path_entries().len(),
        watcher_unsafe_folder: crate::kit::watcher_unsafe_folder(kit.root()),
        first_run: Choice::read(kit, Catalog::get()).is_none(),
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

#[tauri::command]
pub fn projects_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectEntry>, String> {
    ensure_main(&webview)?;
    let mut out: Vec<ProjectEntry> = fs::read_dir(state.kit.projects())
        .map_err(|e| fail("projects", e))?
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            if name.starts_with('.') {
                return None;
            }
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            Some(ProjectEntry { name, modified })
        })
        .collect();
    out.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.name.cmp(&b.name))
    });
    Ok(out)
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
    let (task, kit, env) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.env.clone(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        terminal::open_console(&kit, &env, &dir).map_err(|e| fail("terminal", format!("{e:#}")))
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
    Ok(crate::agents::status(&state.kit, &state.env))
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
    let (task, kit, env) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.env.clone(),
    );
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &task, &kit)?;
        crate::agents::launch(&kit, &env, &agent, &dir).map_err(|e| fail("agent", format!("{e:#}")))
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
    crate::agents::sign_out(&state.kit, &state.env, &agent)
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
            $crate::commands::agent_launch,
            $crate::commands::agent_sign_out,
            $crate::commands::folder_open
        ]
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

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
