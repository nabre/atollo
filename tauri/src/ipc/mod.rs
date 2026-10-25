//! The IPC between the interface and Rust: the Tauri commands, the only
//! way the interface reaches Rust, and the names of the events Rust sends
//! back ([`events`]). The other half is `ui/src/ipc`.
//!
//! Three locks keep editor windows out (DECISIONS.md D-14): the commands are
//! listed in `build.rs` (`AppManifest::commands`), only the `main` capability
//! allows them, and each command checks the calling window's label.
//! Errors reach the interface as short text ([`CmdError`]); details go to
//! the log.
//!
//! One file per page or domain of the interface (the same split as
//! `ui/src/ipc`); this file keeps the state shared by all of them, the checks
//! every command makes first (`ensure_main`, `project_dir`) and the installs
//! a command may need before it runs (`ensure`). A new command goes in the
//! file of its domain, in `command_handler!` below and in `build.rs`.

use std::path::PathBuf;
use std::sync::Arc;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime, State, Webview};

use crate::common::settings::Settings;
use crate::editor::{self, terminal, Editor};
use crate::env::KitEnv;
use crate::install;
use crate::kit::{validate_project_name, Kit};
use crate::projects::run::{Runner, State as RunState};
use crate::setup::runner::Options;
use crate::setup::self_update::Updater;
use crate::setup::task::{Notify, SetupTask, Snapshot};
use crate::setup::{Catalog, Choice, Group};

pub(crate) mod agents;
pub(crate) mod app;
pub(crate) mod databases;
pub(crate) mod editing;
mod error;
pub mod events;
pub(crate) mod frameworks;
pub(crate) mod github;
pub(crate) mod llm;
pub(crate) mod orphans;
pub(crate) mod projects;
pub(crate) mod runs;
pub(crate) mod servers;
pub(crate) mod services;
pub(crate) mod setup;
pub(crate) mod terminals;
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
    /// Shared Caddy and Apache VirtualHost server pools.
    pub servers: Arc<crate::servers::WebServers>,
    /// The server of the local models (D-151).
    pub llm: Arc<crate::llm::service::Llm>,
    /// "Rimuovi l'unità" under way: the agents' background alignments and
    /// saves stop (BUG-V6-15).
    pub closing: crate::agents::Closing,
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

/// Returns `kit` scoped to `target` (`win32-x64`, `linux-x64`, `darwin-arm64`…)
/// when given, or `kit.clone()` when omitted.
pub(super) fn kit_for(kit: &Kit, target: Option<&str>) -> CmdResult<Kit> {
    match target.map(str::trim).filter(|s| !s.is_empty()) {
        None => Ok(kit.clone()),
        Some(raw) => {
            let t =
                crate::kit::Target::parse(raw).ok_or_else(|| format!("unknown target: {raw}"))?;
            Ok(kit.for_target(t))
        }
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
        Err(crate::common::i18n::t(lang, "Progetto non trovato: {0}", &[name]).into())
    }
}

#[cfg(test)]
pub(crate) use crate::router::availability::unavailable_msg;

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
    let target = kit.target();
    let win_arm64 = target == crate::kit::Target::WIN32_ARM64;
    let needed = with_needs(catalog, wanted, target);
    // With the catalog's reason (Android, SQLite on Linux ARM64) or Rosetta 2
    // missing; the same sentences as before otherwise.
    if let Some(msg) = needed
        .iter()
        .find_map(|c| crate::router::availability::unavailable_message_on_host(c, target, lang))
    {
        return Err(msg.into());
    }
    if let Some(c) = needed
        .iter()
        .find(|c| win_arm64 && c.x64_only && !emulates_x64())
    {
        return Err(crate::common::i18n::t(
            lang,
            "{0} gira solo con Windows 11 sui PC ARM64.",
            &[&c.name],
        )
        .into());
    }
    if !wanted.is_empty() {
        // Two actions at once (an agent and a clone) each add theirs:
        // neither write drops the other's (BUG-V6-33).
        let _one = crate::setup::catalog::choice_lock();
        let mut choice = Choice::read_or_defaults(kit, catalog);
        let before = choice.clone();
        for id in wanted {
            choice.set(catalog, id, true);
        }
        if choice != before {
            if let Err(e) = choice.write(kit, catalog) {
                crate::common::log::warn(&format!("components.txt: {e:#}"));
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
        crate::common::log::info(&format!("installing what is needed: {todo:?}"));
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
    Err(crate::common::i18n::t(
        lang,
        "Componenti del kit non installati: {0}{1}. Riprova da Strumenti.",
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
        crate::common::log::warn(&format!("{project}: {e}"));
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

/// `ids` and, transitively, what they need on `target`, in catalog order.
fn with_needs<'a>(
    catalog: &'a Catalog,
    ids: &[&str],
    target: crate::kit::Target,
) -> Vec<&'a crate::setup::Component> {
    let found = catalog.with_needs_on(ids, target);
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
    crate::common::i18n::t(lang_of(state), source, args)
}

/// The handler with every command; keep in sync with `build.rs` COMMANDS.
#[macro_export]
macro_rules! command_handler {
    () => {
        tauri::generate_handler![
            $crate::ipc::app::app_info,
            $crate::ipc::editing::editor_status,
            $crate::ipc::editing::editor_open,
            $crate::ipc::editing::editor_restart,
            $crate::ipc::projects::projects_list,
            $crate::ipc::editing::terminal_open,
            $crate::ipc::terminals::terminals_list,
            $crate::ipc::terminals::terminal_save,
            $crate::ipc::terminals::terminal_remove,
            $crate::ipc::terminals::terminal_launch,
            $crate::ipc::terminals::terminal_dir_pick,
            // The app's terminal on every system; permission
            // `allow-pty-sessions` (permissions/pty.toml).
            $crate::ipc::terminals::pty_sessions,
            $crate::ipc::terminals::pty_backlog,
            $crate::ipc::terminals::pty_write,
            $crate::ipc::terminals::pty_resize,
            $crate::ipc::terminals::pty_close,
            $crate::ipc::terminals::pty_detach,
            $crate::ipc::terminals::pty_dock,
            // The local models; permission `allow-local-models`
            // (permissions/llm.toml).
            $crate::ipc::llm::llm_status,
            $crate::ipc::llm::llm_models,
            $crate::ipc::llm::llm_start,
            $crate::ipc::llm::llm_stop,
            $crate::ipc::llm::llm_set,
            $crate::ipc::llm::llm_hf_inspect,
            $crate::ipc::llm::llm_pull,
            $crate::ipc::llm::llm_pull_cancel,
            $crate::ipc::llm::llm_remove,
            $crate::ipc::llm::llm_hf_token_set,
            $crate::ipc::llm::llm_key,
            $crate::ipc::setup::components_list,
            $crate::ipc::setup::components_sync,
            $crate::ipc::setup::components_check,
            $crate::ipc::setup::components_integrity,
            $crate::ipc::setup::updates_list,
            $crate::ipc::setup::updates_check,
            $crate::ipc::setup::component_link_open,
            $crate::ipc::setup::components_repair,
            $crate::ipc::setup::target_profiles,
            $crate::ipc::setup::tools_target_set,
            $crate::ipc::setup::setup_state,
            $crate::ipc::setup::setup_start,
            $crate::ipc::setup::setup_cancel,
            $crate::ipc::setup::downloads_add,
            $crate::ipc::setup::downloads_pick,
            $crate::ipc::setup::setup_by_hand_open,
            $crate::ipc::setup::components_apply,
            $crate::ipc::setup::components_install,
            $crate::ipc::setup::conda_kept,
            $crate::ipc::setup::conda_kept_free,
            $crate::ipc::agents::agents_status,
            $crate::ipc::agents::agents_stats,
            $crate::ipc::agents::agent_launch,
            $crate::ipc::agents::mcp_presets,
            $crate::ipc::agents::kit_mcp,
            $crate::ipc::agents::kit_mcp_set,
            $crate::ipc::agents::agent_sign_out,
            $crate::ipc::agents::agent_token_link,
            $crate::ipc::agents::agent_token_save,
            $crate::ipc::app::folder_open,
            $crate::ipc::projects::project_create,
            $crate::ipc::projects::project_delete,
            $crate::ipc::projects::project_delete_plan,
            $crate::ipc::frameworks::frameworks_get,
            $crate::ipc::frameworks::frameworks_set,
            $crate::ipc::frameworks::frameworks_plan,
            $crate::ipc::frameworks::frameworks_create,
            $crate::ipc::frameworks::frameworks_cancel,
            $crate::ipc::frameworks::frameworks_targets,
            $crate::ipc::projects::project_clone,
            $crate::ipc::projects::project_site,
            $crate::ipc::projects::project_install_missing,
            $crate::ipc::editing::project_extensions,
            $crate::ipc::versions::versions_list,
            $crate::ipc::versions::versions_available,
            $crate::ipc::versions::versions_add,
            $crate::ipc::versions::versions_install,
            $crate::ipc::versions::versions_remove,
            $crate::ipc::versions::project_versions,
            $crate::ipc::versions::project_versions_set,
            $crate::ipc::versions::relations_matrix,
            $crate::ipc::editing::extensions_matrix,
            $crate::ipc::github::github_status,
            $crate::ipc::github::github_sign_in,
            $crate::ipc::github::github_sign_in_cancel,
            $crate::ipc::github::github_sign_out,
            $crate::ipc::github::github_repos,
            $crate::ipc::github::github_clone,
            $crate::ipc::github::github_rename,
            $crate::ipc::github::github_delete,
            $crate::ipc::github::github_open,
            $crate::ipc::github::git_identity_get,
            $crate::ipc::github::git_identity_set,
            $crate::ipc::databases::project_backups,
            $crate::ipc::databases::project_db_export,
            $crate::ipc::databases::project_db_restore,
            $crate::ipc::projects::project_base_preview,
            $crate::ipc::projects::project_save_base,
            $crate::ipc::projects::project_mcp,
            $crate::ipc::projects::project_mcp_set,
            $crate::ipc::editing::project_editor_settings,
            $crate::ipc::editing::project_editor_settings_set,
            $crate::ipc::projects::project_action_save,
            $crate::ipc::projects::project_action_remove,
            $crate::ipc::projects::project_action_default,
            $crate::ipc::projects::project_pin,
            $crate::ipc::projects::project_status,
            $crate::ipc::databases::project_databases,
            $crate::ipc::databases::project_db_create,
            $crate::ipc::databases::project_db_line,
            $crate::ipc::databases::project_env_preview,
            $crate::ipc::databases::project_env_apply,
            $crate::ipc::databases::project_env_block,
            $crate::ipc::services::services_list,
            $crate::ipc::services::service_start,
            $crate::ipc::services::service_stop,
            $crate::ipc::services::service_set,
            $crate::ipc::services::service_line,
            $crate::ipc::services::service_line_port,
            $crate::ipc::servers::servers_list,
            $crate::ipc::servers::server_save,
            $crate::ipc::servers::server_remove,
            $crate::ipc::servers::server_start,
            $crate::ipc::servers::server_stop,
            $crate::ipc::servers::server_reload,
            $crate::ipc::servers::server_assign_project,
            $crate::ipc::orphans::projects_orphans,
            $crate::ipc::orphans::project_orphan_adopt,
            $crate::ipc::databases::db_admin_open,
            $crate::ipc::runs::run_list,
            $crate::ipc::runs::run_start,
            $crate::ipc::runs::run_stop,
            $crate::ipc::runs::run_log,
            $crate::ipc::runs::run_clear,
            $crate::ipc::runs::run_ports,
            $crate::ipc::runs::run_set_port,
            $crate::ipc::app::url_open,
            $crate::ipc::app::lan_link,
            $crate::ipc::app::kit_export,
            $crate::ipc::app::kit_import,
            $crate::ipc::app::kit_import_pending,
            $crate::ipc::app::kit_import_cancel,
            $crate::ipc::app::diagnostics_export,
            $crate::ipc::app::issue_open,
            $crate::ipc::app::kit_health,
            $crate::ipc::app::privacy_open,
            $crate::ipc::setup::rust_toolchain_set,
            $crate::ipc::app::settings_get,
            $crate::ipc::app::settings_set,
            $crate::ipc::app::update_status,
            $crate::ipc::app::update_check,
            $crate::ipc::window::eject_list,
            $crate::ipc::window::eject_run,
            $crate::ipc::window::app_quit
        ]
    };
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/mod.rs"]
mod tests;
