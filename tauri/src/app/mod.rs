//! The app's window side: Tauri's context, the app built with its state,
//! commands and window events, the main window, and what starts in the
//! background once it is open; the icon in the tray (`tray`) and the
//! command line of the same exe (`cli`).

pub mod cli;
pub mod tray;

use std::path::Path;

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::common::log;
use crate::common::settings::Settings;
use crate::ipc::{AppState, MAIN};
use crate::kit::Kit;
use crate::{agents, editor, github, ipc, llm, net, platform, projects, setup};

/// The app's context, expanded once for the app and its tests: on macOS
/// `generate_context!` embeds the Info.plist under a global symbol, so a
/// second expansion in the same binary does not link.
pub(crate) fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// The app: state, commands, main window, focus requests from a second start.
pub(crate) fn build_app<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    state: AppState,
    kit: Kit,
) -> tauri::Builder<R> {
    builder
        .manage(state)
        .manage(editor::terminal::Consoles::default())
        .manage(ipc::Quitting::default())
        .invoke_handler(crate::command_handler!())
        .on_window_event(|window, event| {
            if window.label().starts_with(editor::window::LABEL_PREFIX)
                && matches!(
                    event,
                    tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
                )
            {
                log::info(&format!("{}: {event:?}", window.label()));
            }
            if window.label().starts_with(editor::window::LABEL_PREFIX)
                && matches!(event, tauri::WindowEvent::Destroyed)
            {
                editor::window::closed(window.label());
            }
            // A detached terminal's window closed (D-149).
            if window.label().starts_with(editor::terminal::LABEL_PREFIX)
                && matches!(event, tauri::WindowEvent::Destroyed)
            {
                editor::terminal::closed(window.app_handle(), window.label());
            }
            // The editor's title bar shows maximize or restore (D-69).
            if window.label().starts_with(editor::window::LABEL_PREFIX)
                && matches!(event, tauri::WindowEvent::Resized(_))
            {
                if let Some(w) = window.get_webview_window(window.label()) {
                    editor::window::sync_maximized(&w);
                }
            }
            // With something open, the interface asks before closing.
            if window.label() == MAIN {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    ipc::on_main_close(window, api);
                }
            }
            // Closing the app window ends the session: editor windows,
            // server and consoles go with it (they are in the job).
            if window.label() == MAIN && matches!(event, tauri::WindowEvent::Destroyed) {
                // The databases shut down cleanly first (D-73), off the UI thread.
                let services =
                    std::sync::Arc::clone(&window.app_handle().state::<AppState>().services);
                let servers =
                    std::sync::Arc::clone(&window.app_handle().state::<AppState>().servers);
                let llm = std::sync::Arc::clone(&window.app_handle().state::<AppState>().llm);
                let (kit, host, env) = {
                    let state = window.app_handle().state::<AppState>();
                    (state.kit.clone(), state.host.clone(), state.env())
                };
                let handle = window.app_handle().clone();
                std::thread::spawn(move || {
                    servers.stop_all();
                    llm.stop_all();
                    llm::pull::cancel_all(std::time::Duration::from_secs(10));
                    services.stop_all();
                    // The projects' microVMs shut down too (D-167).
                    crate::isolation::stop_daemon(&kit, &host, &env);
                    handle.exit(0);
                });
            }
        })
        .setup(move |app| {
            open_main_window(
                app.handle(),
                &kit.webview_data(),
                Settings::load(&kit).settings.theme.window(),
            )?;
            // The icon in the notification area / menu bar (D-145).
            tray::create(app.handle());
            // Extensions follow the projects open in the editor; the editor
            // windows offer to restart them (D-71).
            // Databases marked "start with Atollo" (D-73).
            // Then the server profiles marked "Avvia con Atollo", and the
            // server of the local models when it is marked too (D-151).
            let (services, servers, llm, env, lang, host) = {
                let state = app.state::<AppState>();
                (
                    std::sync::Arc::clone(&state.services),
                    std::sync::Arc::clone(&state.servers),
                    std::sync::Arc::clone(&state.llm),
                    state.env.read().map(|e| e.clone()).unwrap_or_default(),
                    state.lang(),
                    state.host.clone(),
                )
            };
            let pools_kit = kit.clone();
            std::thread::spawn(move || {
                services.autostart(&env);
                // The pools are the box without the VM (D-175): with the VM
                // on, the sites run in their projects' VMs.
                let level = crate::isolation::status(&pools_kit, &host, &env, false).level;
                if level == crate::isolation::Level::Vm {
                    log::info("server pools: not started, every site runs in its VM");
                } else {
                    servers.autostart(&env, lang);
                }
                llm.autostart(&env);
            });
            let editor = std::sync::Arc::clone(&app.state::<AppState>().editor);
            let notify = app.handle().clone();
            editor.set_on_installed(std::sync::Arc::new(move |ids: &[String]| {
                editor::window::notify_installed(&notify, ids);
            }));
            let (refresh, refresh_kit) = (app.handle().clone(), kit.clone());
            editor::window::watch_projects(
                kit.clone(),
                editor,
                std::sync::Arc::new(move || editor::window::refresh_hidden(&refresh, &refresh_kit)),
            );
            // Projects' actions: state and output go to the app window.
            let events = app.handle().clone();
            app.manage(std::sync::Arc::new(projects::run::Runner::new(
                kit.clone(),
                std::sync::Arc::new(move |e: projects::run::Event| {
                    let _ = events.emit_to(MAIN, ipc::events::RUN, e);
                }),
            )));
            app.manage(std::sync::Arc::new(github::Login::default()));
            let updater = std::sync::Arc::new(setup::self_update::Updater::new(kit.clone()));
            app.manage(std::sync::Arc::clone(&updater));
            // ATOLLO_NO_UPDATE: no automatic check (as for the kit's scripts).
            if app
                .state::<AppState>()
                .host
                .get("ATOLLO_NO_UPDATE")
                .is_none()
            {
                let (token, update_kit) = (
                    crate::github_token(&app.state::<AppState>().host),
                    kit.clone(),
                );
                std::thread::spawn(move || {
                    // After the start, not competing with it; the setting
                    // as it is then (turned off meanwhile: no check).
                    std::thread::sleep(std::time::Duration::from_secs(20));
                    let settings = Settings::load(&update_kit).settings;
                    if !settings.auto_update {
                        return;
                    }
                    if let Ok(net) = net::Net::new(&settings, token) {
                        updater.check(&net);
                    }
                });
            }
            // The versions each source offers, for the menus that download
            // one; ATOLLO_NO_UPDATE: nothing asked automatically.
            if app
                .state::<AppState>()
                .host
                .get("ATOLLO_NO_UPDATE")
                .is_none()
            {
                let (lines_kit, env) = (kit.clone(), app.state::<AppState>().env());
                let token = crate::github_token(&app.state::<AppState>().host);
                let events = app.handle().clone();
                std::thread::spawn(move || {
                    // After the start, not competing with it.
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    ipc::versions::refresh_stale_lines(&lines_kit, &env, token.clone());
                    // The new versions of what is installed (N-9): at most
                    // once a day, only with «Aggiornamento automatico» on.
                    let auto = Settings::load(&lines_kit).settings.auto_update;
                    if auto && setup::updates::stale(&lines_kit) {
                        match setup::updates::check(&lines_kit, &env, token) {
                            Ok(_) => {
                                let _ = events.emit_to(MAIN, ipc::events::UPDATES, ());
                            }
                            Err(e) => log::warn(&format!("new versions: {e:#}")),
                        }
                    }
                });
            }
            start_missing_required(app.handle(), &kit);
            // The editor's VM sessions of the last run hold nothing now: each
            // editor that opens notes its own (D-175).
            if let Err(e) = crate::isolation::session::clear_all(&kit) {
                log::warn(&format!("editor sessions: {e:#}"));
            }
            // Docker Sandboxes failed to install (the network, a checksum):
            // tried again every hour while the kit has none (D-175).
            let (retry_app, retry_kit) = (app.handle().clone(), kit.clone());
            std::thread::spawn(move || loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
                if crate::isolation::kit_sbx(&retry_kit).is_none()
                    && crate::isolation::install_failure(&retry_kit).is_some()
                {
                    start_missing_required(&retry_app, &retry_kit);
                }
            });
            // Antigravity's login left in Windows by a run that did not end
            // cleanly, the kit moved, the agents aligned; then the watcher.
            let background = kit.clone();
            let env = app.state::<AppState>().env();
            let closing = app.state::<AppState>().closing.clone();
            std::thread::spawn(move || {
                if let Some(Err(e)) = closing.run(|| agents::keyring::save(&background)) {
                    log::warn(&format!("antigravity login: {e:#}"));
                }
                // Git follows the GitHub account, also after a sign-in or a
                // sign-out made with gh in a console.
                let signed_in = github::token(&background, &env).is_some();
                if let Err(e) = github::sync_git(&background, &env, signed_in) {
                    log::warn(&format!("github: git helper: {e:#}"));
                }
                let _ = closing.run(|| agents::sync::run(&background, None));
                agents::watch(background, closing);
            });
            let handle = app.handle().clone();
            platform::instance::listen(&kit, move |message| {
                if message.trim() == "focus" {
                    tray::show_main(&handle);
                }
            });
            Ok(())
        })
}

/// A required component deleted, or the editor of another release: install
/// what is missing without asking; the "Linguaggi e strumenti" page shows
/// the progress. Not on the first start (no choice saved yet): the
/// first-start page asks language and network first (a proxy, the
/// certificates of an inspecting firewall), and its «Installa» brings the
/// required components too (D-137).
pub(crate) fn start_missing_required<R: tauri::Runtime>(app: &tauri::AppHandle<R>, kit: &Kit) {
    let catalog = setup::catalog::Catalog::get();
    if setup::catalog::Choice::read(kit, catalog).is_none() {
        return;
    }
    let mut missing = setup::task::missing_required(kit);
    missing.extend(setup::task::outdated_app(kit));
    if missing.is_empty() {
        return;
    }
    let opts = setup::runner::Options {
        only: missing,
        ..setup::runner::Options::default()
    };
    let state = app.state::<AppState>();
    if let Err(e) = state
        .setup
        .start(opts, true, ipc::setup_notify(app.clone()))
    {
        log::warn(&format!("automatic setup: {e}"));
    }
}

pub(crate) fn open_main_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    data_dir: &Path,
    theme: Option<tauri::Theme>,
) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("Atollo")
        .inner_size(1180.0, 760.0)
        .min_inner_size(820.0, 560.0)
        // The interface draws its own title bar (ui/src/shell/titlebar.tsx).
        .decorations(false)
        .shadow(true)
        .data_directory(data_dir.to_path_buf())
        .theme(theme)
        .build()?;
    Ok(())
}
