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

use crate::catalog::{Catalog, Group};
use crate::editor::{self, Editor};
use crate::env::KitEnv;
use crate::kit::{validate_project_name, Kit};
use crate::setup_runner::{self, Event, Outcome};
use crate::{editor_window, terminal};

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

/// Forwards installer progress to the app window (`setup-event`).
struct EmitSink<R: Runtime>(AppHandle<R>);

impl<R: Runtime> setup_runner::Sink for EmitSink<R> {
    fn emit(&mut self, event: Event) {
        let _ = self.0.emit_to(MAIN, "setup-event", event);
    }
}

/// Installs the required components that are missing (PowerShell 7, Git,
/// Node.js, the editor...): the editor and the consoles never fall back to
/// the shells of the PC. Blocking: call it off the UI thread.
fn ensure_required<R: Runtime>(app: &AppHandle<R>, kit: &Kit, env: &KitEnv) -> anyhow::Result<()> {
    let missing: Vec<String> = Catalog::get()
        .all()
        .iter()
        .filter(|c| c.group == Group::Required && !c.is_installed(kit))
        .map(|c| c.id.clone())
        .collect();
    if missing.is_empty() {
        return Ok(());
    }
    crate::log::info(&format!(
        "installing missing required components: {missing:?}"
    ));
    let settings = crate::settings::Settings::load(kit).settings;
    let net = crate::net::Net::new(&settings, None)?;
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = crate::install::Ctx {
        kit,
        net: &net,
        env,
        cancel: &cancel,
    };
    let opts = setup_runner::Options {
        only: missing,
        ..Default::default()
    };
    let results = setup_runner::run(&ctx, &opts, &mut EmitSink(app.clone()))?;
    let failed: Vec<String> = results
        .iter()
        .filter(|(_, o)| o.is_failure() || matches!(o, Outcome::Unsupported | Outcome::Cancelled))
        .map(|(id, o)| format!("{id}: {o:?}"))
        .collect();
    anyhow::ensure!(
        failed.is_empty(),
        "componenti obbligatori non installati: {}",
        failed.join("; ")
    );
    Ok(())
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
    let (handle, kit, env) = (app.clone(), state.kit.clone(), state.env.clone());
    let (port, token) = tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&handle, &kit, &env)?;
        editor.ensure_running()
    })
    .await
    .map_err(|e| fail("editor", e))?
    .map_err(|e| fail("editor start", format!("{e:#}")))?;
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
    let (kit, env) = (state.kit.clone(), state.env.clone());
    tauri::async_runtime::spawn_blocking(move || {
        ensure_required(&app, &kit, &env)?;
        terminal::open_console(&kit, &env, &dir)
    })
    .await
    .map_err(|e| fail("terminal", e))?
    .map_err(|e| fail("terminal", format!("{e:#}")))
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
            $crate::commands::setup_cancel
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
