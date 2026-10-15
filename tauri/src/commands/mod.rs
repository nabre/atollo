//! Tauri commands, the only way the interface reaches Rust.
//!
//! Three locks keep editor windows out (DECISIONS.md D-14): the commands are
//! listed in `build.rs` (`AppManifest::commands`), only the `main` capability
//! allows them, and each command checks the calling window's label.
//! Errors reach the interface as short text ([`CmdError`]); details go to
//! the log.
//!
//! One file per page or domain of the interface (the same split as
//! `ui/src/api`); this file keeps the state shared by all of them, the checks
//! every command makes first (`ensure_main`, `project_dir`) and the installs
//! a command may need before it runs (`ensure`). A new command goes in the
//! file of its domain, in `command_handler!` below and in `build.rs`.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State, Webview};

use crate::editor::{self, terminal, Editor};
use crate::env::KitEnv;
use crate::install;
use crate::kit::{validate_project_name, Kit};
use crate::projects::run::{Runner, State as RunState};
use crate::settings::Settings;
use crate::setup::catalog::{Catalog, Choice, Group};
use crate::setup::runner::Options;
use crate::setup::self_update::Updater;
use crate::setup::task::{Notify, SetupTask, Snapshot};

pub(crate) mod agents;
pub(crate) mod app;
pub(crate) mod databases;
pub(crate) mod editing;
mod error;
pub(crate) mod frameworks;
pub(crate) mod github;
pub(crate) mod projects;
pub(crate) mod runs;
pub(crate) mod servers;
pub(crate) mod services;
pub(crate) mod setup;
pub(crate) mod versions;
pub(crate) mod window;

use error::{fail, CmdError, CmdResult, OrFail};

// Helpers shared by more than one group of commands.
use app::{apply_env, github_token, open_in_browser};
use services::start_error;
use setup::emulates_x64;

pub use setup::setup_notify;
pub use window::{on_main_close, Quitting};

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
    pub services: Arc<crate::db::services::Services>,
    /// Shared Apache and Nginx VirtualHost server pools.
    pub servers: Arc<crate::servers::WebServers>,
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

fn ensure_main<R: Runtime>(webview: &Webview<R>) -> CmdResult<()> {
    if webview.label() == MAIN {
        Ok(())
    } else {
        Err("forbidden".into())
    }
}

/// `None` = the projects folder itself; a name must be a valid project
/// name of an existing folder (no paths, no `..`).
fn project_dir(kit: &Kit, project: Option<&str>, lang: &str) -> CmdResult<PathBuf> {
    let Some(name) = project else {
        return Ok(kit.projects());
    };
    validate_project_name(name).map_err(|e| e.to_string())?;
    let dir = kit.projects().join(name);
    // A link or junction is never a project (D-77).
    if crate::os::safe_path::is_dir(&kit.projects(), &dir) {
        Ok(dir)
    } else {
        Err(crate::i18n::t(lang, "Progetto non trovato: {0}", &[name]).into())
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
) -> CmdResult<KitEnv> {
    use tauri::Manager;
    let env = || app.state::<AppState>().env();
    let catalog = Catalog::get();
    let arm64 = kit.arch() == crate::kit::Arch::Arm64;
    let needed = with_needs(catalog, wanted);
    if let Some(c) = needed.iter().find(|c| arm64 && c.no_arm64) {
        return Err(crate::i18n::t(lang, "{0} non è disponibile sui PC ARM64.", &[&c.name]).into());
    }
    if let Some(c) = needed
        .iter()
        .find(|c| arm64 && c.x64_only && !emulates_x64())
    {
        return Err(crate::i18n::t(
            lang,
            "{0} gira solo con Windows 11 sui PC ARM64.",
            &[&c.name],
        )
        .into());
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
        let mut ids = crate::setup::task::missing_required(kit);
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
    )
    .into())
}

/// A project just copied (clone) gets what its code needs, waiting for the
/// downloads: the languages it uses are installed, each version it declares
/// gets the newest line that fits (downloaded if none is on disk), the
/// other languages the newest on disk. A failure only goes to the log: the
/// project is there, its page tells what is missing.
fn settle_project<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    kit: &Kit,
    lang: &str,
    project: &str,
    dir: &std::path::Path,
) {
    let requires = crate::detect::requires(dir);
    let ids: Vec<&str> = requires.iter().map(String::as_str).collect();
    if let Err(e) = ensure(app, task, kit, lang, &ids) {
        crate::log::warn(&format!("{project}: {e}"));
    }
    let lines = crate::versions::relations::settle(kit, lang, project);
    if !lines.is_empty() {
        let opts = Options {
            lines,
            ..Options::default()
        };
        while task.snapshot().running {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        if task.start(opts, true, setup_notify(app.clone())).is_ok() {
            while task.snapshot().running {
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
        }
    }
    crate::versions::pin_newest(kit, project);
}

/// `ids` and, transitively, what they need, in catalog order.
fn with_needs<'a>(catalog: &'a Catalog, ids: &[&str]) -> Vec<&'a crate::setup::catalog::Component> {
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

fn lang_of(state: &AppState) -> &'static str {
    state.lang()
}

fn tr(state: &AppState, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang_of(state), source, args)
}

/// The handler with every command; keep in sync with `build.rs` COMMANDS.
#[macro_export]
macro_rules! command_handler {
    () => {
        tauri::generate_handler![
            $crate::commands::app::app_info,
            $crate::commands::editing::editor_status,
            $crate::commands::editing::editor_open,
            $crate::commands::editing::editor_restart,
            $crate::commands::projects::projects_list,
            $crate::commands::editing::terminal_open,
            $crate::commands::setup::components_list,
            $crate::commands::setup::components_sync,
            $crate::commands::setup::setup_state,
            $crate::commands::setup::setup_start,
            $crate::commands::setup::setup_cancel,
            $crate::commands::setup::downloads_add,
            $crate::commands::setup::downloads_pick,
            $crate::commands::setup::setup_by_hand_open,
            $crate::commands::setup::components_apply,
            $crate::commands::agents::agents_status,
            $crate::commands::agents::agents_stats,
            $crate::commands::agents::agent_launch,
            $crate::commands::agents::agent_sign_out,
            $crate::commands::app::folder_open,
            $crate::commands::projects::project_create,
            $crate::commands::projects::project_delete,
            $crate::commands::frameworks::frameworks_get,
            $crate::commands::frameworks::frameworks_set,
            $crate::commands::frameworks::frameworks_plan,
            $crate::commands::frameworks::frameworks_create,
            $crate::commands::frameworks::frameworks_cancel,
            $crate::commands::frameworks::frameworks_targets,
            $crate::commands::projects::project_clone,
            $crate::commands::projects::project_site,
            $crate::commands::projects::project_install_missing,
            $crate::commands::editing::project_extensions,
            $crate::commands::versions::versions_list,
            $crate::commands::versions::versions_available,
            $crate::commands::versions::versions_add,
            $crate::commands::versions::versions_remove,
            $crate::commands::versions::project_versions,
            $crate::commands::versions::project_versions_set,
            $crate::commands::versions::relations_matrix,
            $crate::commands::editing::extensions_matrix,
            $crate::commands::github::github_status,
            $crate::commands::github::github_sign_in,
            $crate::commands::github::github_sign_in_cancel,
            $crate::commands::github::github_sign_out,
            $crate::commands::github::github_repos,
            $crate::commands::github::github_clone,
            $crate::commands::github::github_rename,
            $crate::commands::github::github_delete,
            $crate::commands::github::github_open,
            $crate::commands::databases::project_backups,
            $crate::commands::databases::project_db_export,
            $crate::commands::databases::project_db_restore,
            $crate::commands::projects::project_mcp,
            $crate::commands::projects::project_mcp_set,
            $crate::commands::editing::project_editor_settings,
            $crate::commands::editing::project_editor_settings_set,
            $crate::commands::projects::project_action_save,
            $crate::commands::projects::project_action_remove,
            $crate::commands::projects::project_action_default,
            $crate::commands::projects::project_status,
            $crate::commands::databases::project_databases,
            $crate::commands::databases::project_db_create,
            $crate::commands::databases::project_env_preview,
            $crate::commands::databases::project_env_apply,
            $crate::commands::services::services_list,
            $crate::commands::services::service_start,
            $crate::commands::services::service_stop,
            $crate::commands::services::service_set,
            $crate::commands::services::service_line,
            $crate::commands::servers::servers_list,
            $crate::commands::servers::server_save,
            $crate::commands::servers::server_remove,
            $crate::commands::servers::server_start,
            $crate::commands::servers::server_stop,
            $crate::commands::servers::server_reload,
            $crate::commands::servers::server_assign_project,
            $crate::commands::databases::db_browse_tree,
            $crate::commands::databases::db_browse_rows,
            $crate::commands::databases::db_admin_open,
            $crate::commands::runs::run_list,
            $crate::commands::runs::run_start,
            $crate::commands::runs::run_stop,
            $crate::commands::runs::run_log,
            $crate::commands::runs::run_clear,
            $crate::commands::runs::run_ports,
            $crate::commands::runs::run_set_port,
            $crate::commands::app::url_open,
            $crate::commands::app::settings_get,
            $crate::commands::app::settings_set,
            $crate::commands::app::update_status,
            $crate::commands::app::update_check,
            $crate::commands::window::eject_list,
            $crate::commands::window::eject_run,
            $crate::commands::window::app_quit
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
}
