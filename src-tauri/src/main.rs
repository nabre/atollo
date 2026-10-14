//! Dev-Kit: portable development environment for Windows in one exe.
//!
//! Start order (docs/app/PLAN.md §3): kit root → folders and settings →
//! environment of this process inside `data\` → command line or window →
//! one instance per kit → WebView2 → Job Object → main window.
#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod agents;
mod agents_sync;
mod apache;
mod archive;
mod catalog;
mod checkup;
mod cli;
mod commands;
mod db_auth;
mod detect;
mod editor;
mod editor_cfg;
mod editor_ext;
mod editor_theme;
mod editor_window;
mod env;
mod github;
mod i18n;
mod install;
mod instance;
mod jobs;
mod keyring;
mod kit;
mod kit_files;
mod log;
mod needs;
mod net;
mod processes;
mod project_mcp;
mod project_settings;
mod project_vscode;
mod projects;
mod relocate;
mod run;
mod safe_path;
mod self_update;
mod serve;
mod services;
mod sessions;
mod settings;
mod setup_runner;
mod setup_task;
mod signature;
mod stats;
mod terminal;
mod verify;
mod webview;
mod wterm;

use std::path::Path;

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use windows::core::HSTRING;
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

use crate::cli::Cli;
use crate::commands::{AppState, MAIN};
use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::settings::Settings;

fn main() {
    std::process::exit(run());
}

/// Message box for errors before (or instead of) the window; also on
/// stderr, for the command line (a service session has no message box).
fn fatal(lang: &str, message: &str) {
    eprintln!("{message}");
    let title = HSTRING::from(i18n::t(lang, "NEXKORA non può partire", &[]));
    let text = HSTRING::from(message);
    // SAFETY: both strings outlive the modal call.
    unsafe { MessageBoxW(None, &text, &title, MB_OK | MB_ICONERROR) };
}

fn run() -> i32 {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let command = cli::parse(&args);
    let host = Host::capture();
    let windows_lang = i18n::windows_locale();
    // Language before settings exist: for the "cannot write" message.
    let early_lang = i18n::resolve(None, host.get("DEVKIT_LANG"), windows_lang.as_deref());

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(e) => {
            fatal(
                early_lang,
                &i18n::t(early_lang, "Errore imprevisto: {0}", &[&e.to_string()]),
            );
            return 1;
        }
    };
    let kit = match Kit::from_exe(&exe) {
        Ok(kit) => kit,
        Err(e) => {
            fatal(
                early_lang,
                &i18n::t(early_lang, "Errore imprevisto: {0}", &[&e.to_string()]),
            );
            return 1;
        }
    };
    let moved_from = match kit.prepare() {
        Ok(moved) => moved,
        Err(e) => {
            let path = match &e {
                kit::KitError::NotWritable { path, .. } => path.display().to_string(),
                kit::KitError::NoRoot(path) => path.display().to_string(),
            };
            let text = format!(
                "{}\n\n{}",
                i18n::t(early_lang, "Impossibile scrivere nella cartella {0}.", &[&path]),
                i18n::t(early_lang, "Sposta Dev-Kit.exe in una cartella dove puoi scrivere (per esempio una chiavetta o una cartella del tuo profilo) e riavvialo.", &[])
            );
            fatal(early_lang, &text);
            return 1;
        }
    };

    log::init(&kit.logs());
    self_update::remove_old(&exe);
    self_update::record_version(&kit);
    let loaded = Settings::load(&kit);
    let lang = i18n::resolve(
        loaded.settings.lang.as_deref(),
        host.get("DEVKIT_LANG"),
        windows_lang.as_deref(),
    );
    if let Some(problem) = &loaded.problem {
        log::warn(&i18n::t("en", "Le impostazioni in {0} non sono leggibili: uso quelle predefinite e non modifico il file.", &[problem]));
    }
    let kit_env = KitEnv::build(&kit, &loaded.settings, lang, &host);
    if let Err(e) = kit_env.create_dirs() {
        log::warn(&format!("creating kit folders: {e}"));
    }
    env::apply_to_current_process(&kit_env, &kit);
    if let Err(e) = kit_files::write_all(&kit) {
        log::warn(&format!("writing kit files: {e:#}"));
    }

    match command {
        Cli::Gui => {}
        Cli::Env => return cli::print_env(&kit_env, &mut *cli::output()).map_or(1, |()| 0),
        Cli::Prepare(args) => return prepare(&kit, &kit_env, &loaded.settings, &host, lang, &args),
        Cli::Sync { project, verbose } => {
            let report = agents_sync::run(&kit, project.as_deref());
            let mut out = cli::output();
            if verbose {
                for n in &report.notes {
                    let _ = writeln!(out, "  {n}");
                }
            }
            for w in &report.warnings {
                let _ = writeln!(out, "  [!] {w}");
            }
            return 0;
        }
        Cli::Keyring(action) => {
            let result = if action == "restore" {
                keyring::restore(&kit)
            } else {
                keyring::save_after_run(&kit)
            };
            return match result {
                Ok(()) => 0,
                Err(e) => {
                    log::warn(&format!("keyring {action}: {e:#}"));
                    let _ = writeln!(cli::output(), "  [!] Antigravity: {e:#}");
                    1
                }
            };
        }
        Cli::Serve { dir, port } => {
            return match serve::run(&dir, port, lang) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e:#}");
                    1
                }
            };
        }
        Cli::Check => {
            let checks = checkup::check_all(&kit, &kit_env);
            let ok = checks.iter().all(|c| c.ok);
            return cli::print_checks(&checks, &mut *cli::output()).map_or(1, |()| i32::from(!ok));
        }
        Cli::Components => {
            return cli::print_components(&kit, lang, &mut *cli::output()).map_or(1, |()| 0)
        }
        Cli::Version => {
            let _ = writeln!(cli::output(), "NEXKORA {}", env!("CARGO_PKG_VERSION"));
            return 0;
        }
        Cli::Help => {
            let _ = writeln!(cli::output(), "{}", help(lang));
            return 0;
        }
        Cli::Unknown(other) => {
            let mut out = cli::output();
            let _ = writeln!(
                out,
                "{}\n\n{}",
                i18n::t(lang, "Comando sconosciuto: {0}", &[&other]),
                help(lang)
            );
            return 2;
        }
    }

    let guard = match instance::acquire(&kit) {
        Ok(instance::Instance::Primary(guard)) => guard,
        Ok(instance::Instance::Secondary) => {
            if let Err(e) = instance::notify_primary(&kit, "focus") {
                log::warn(&format!("second start, focus request failed: {e:#}"));
            }
            return 0;
        }
        Err(e) => {
            log::error(&format!("single instance: {e:#}"));
            fatal(
                lang,
                &i18n::t(lang, "Errore imprevisto: {0}", &[&format!("{e:#}")]),
            );
            return 1;
        }
    };

    // WebView2 lookup sets a variable for the fixed runtime: still single-threaded here.
    let runtime = webview::locate(&kit);
    let Some(runtime) = runtime else {
        log::error("WebView2 runtime not found");
        let text = format!(
            "{}\n\n{}",
            i18n::t(lang, "Manca Microsoft Edge WebView2, che serve per mostrare le finestre di NEXKORA.", &[]),
            i18n::t(
                lang,
                "Scarica la versione \"Fixed Version\" per {0} da {1} ed estraila in {2}, poi riavvia NEXKORA. Non serve installare nulla.",
                &[kit.arch().as_str(), webview::DOWNLOAD_URL, &webview::fixed_folder(&kit).display().to_string()]
            )
        );
        fatal(lang, &text);
        return 1;
    };
    let webview_desc = match &runtime {
        webview::Runtime::System(v) => format!("system {v}"),
        webview::Runtime::Fixed { folder, version } => {
            format!("fixed {version} ({})", folder.display())
        }
    };

    match jobs::init() {
        Ok(None) => {}
        Ok(Some(note)) => log::warn(&format!("job object: {note}")),
        Err(e) => log::error(&format!("job object: {e:#}")),
    }
    log::info(&format!(
        "start {} root={} platform={} lang={lang} webview={webview_desc} moved_from={:?}",
        env!("CARGO_PKG_VERSION"),
        kit.root().display(),
        kit.arch().platform(),
        moved_from
    ));

    let setup = std::sync::Arc::new(setup_task::SetupTask::new(
        kit.clone(),
        kit_env.clone(),
        loaded.settings.clone(),
        github_token(&host),
    ));
    let state = AppState {
        setup,
        kit: kit.clone(),
        lang: std::sync::RwLock::new(lang),
        host: host.clone(),
        moved_from: moved_from.map(|p| p.display().to_string()),
        webview: webview_desc,
        editor: std::sync::Arc::new(editor::Editor::new(kit.clone(), kit_env.clone())),
        services: std::sync::Arc::new(services::Services::new(kit.clone())),
        env: std::sync::RwLock::new(kit_env),
    };
    // The exe swap of a staged update happens on Exit (`run` may not return).
    let (update_kit, update_exe) = (kit.clone(), exe.clone());
    let result = build_app(tauri::Builder::default(), state, kit.clone())
        .build(tauri::generate_context!())
        .map(|app| {
            app.run(move |_, event| {
                if let tauri::RunEvent::Exit = event {
                    match self_update::apply_staged(&update_kit, &update_exe) {
                        Ok(true) => log::info("update applied"),
                        Ok(false) => {}
                        Err(e) => log::warn(&format!("update: {e:#}")),
                    }
                }
            });
        });
    drop(guard);
    match result {
        Ok(()) => 0,
        Err(e) => {
            log::error(&format!("tauri: {e:#}"));
            fatal(
                lang,
                &i18n::t(lang, "Errore imprevisto: {0}", &[&e.to_string()]),
            );
            1
        }
    }
}

use std::io::Write as _;

/// Raises GitHub's API limit when the user has a token.
fn github_token(host: &Host) -> Option<String> {
    host.get("GITHUB_TOKEN")
        .or(host.get("GH_TOKEN"))
        .map(str::to_owned)
}

/// `Dev-Kit.exe prepare`: 0 all good, 1 some component failed, 3 another
/// preparation is running.
fn prepare(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &cli::PrepareArgs,
) -> i32 {
    let mut out = cli::output();
    if let Err(e) = jobs::init() {
        log::warn(&format!("job object: {e:#}"));
    }
    let net = match net::Net::new(settings, github_token(host)) {
        Ok(net) => net,
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            return 1;
        }
    };
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let ctx = install::Ctx {
        kit,
        net: &net,
        env: kit_env,
        cancel: &cancel,
    };
    log::info(&format!("prepare {args:?}"));
    let mut printer = cli::Printer::new(&mut *out, lang);
    let results = match setup_runner::run(&ctx, &args.options(), &mut printer) {
        Ok(results) => results,
        Err(e) if e.is::<setup_runner::Busy>() => {
            let _ = writeln!(
                out,
                "  x {}",
                i18n::t(
                    lang,
                    "Un'altra preparazione è già in corso in questo kit.",
                    &[]
                )
            );
            return 3;
        }
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            return 1;
        }
    };
    let failed = results.iter().filter(|(_, o)| o.is_failure()).count();
    // The final check of what this run installed (`Test-Installation`).
    let touched: Vec<&str> = results
        .iter()
        .filter(|(_, o)| matches!(o, setup_runner::Outcome::Installed { .. }))
        .map(|(id, _)| id.as_str())
        .collect();
    let checks: Vec<_> = catalog::Catalog::get()
        .all()
        .iter()
        .filter(|c| touched.contains(&c.id.as_str()))
        .filter_map(|c| checkup::check(kit, kit_env, c))
        .collect();
    if !checks.is_empty() {
        let _ = writeln!(out, "\n── {} ──", i18n::t(lang, "Verifica", &[]));
        let _ = cli::print_checks(&checks, &mut *out);
    }
    for (id, outcome) in &results {
        log::info(&format!("prepare {id}: {outcome:?}"));
    }
    i32::from(failed > 0)
}

/// Help text of the command line (Italian source; translated by the catalog).
const HELP: &str = "Uso: Dev-Kit.exe [comando]\n\nComandi:\n  env                     mostra l'ambiente del kit\n  components              elenca linguaggi e strumenti, con quelli installati\n  check                   verifica che i programmi installati rispondano\n  prepare [opzioni]       installa e aggiorna i componenti scelti\n      --only a,b          solo questi componenti\n      --skip a,b          non questi\n      --update-all        aggiorna anche quelli già installati\n      --force             scarica di nuovo\n  sync [opzioni]          allinea istruzioni, skill e server MCP degli agenti\n      --project <cartella> anche le istruzioni di questo progetto\n      --verbose           mostra ogni modifica\n  keyring restore|save    accesso di Antigravity (lo usa agy.cmd del kit)\n  serve <cartella> <porta> server di file per un sito statico\n  version                 mostra la versione\n  help                    mostra questo aiuto\n\nSenza comando apre la finestra di NEXKORA.";

fn help(lang: &str) -> String {
    i18n::t(lang, HELP, &[])
}

/// The app: state, commands, main window, focus requests from a second start.
fn build_app<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    state: AppState,
    kit: Kit,
) -> tauri::Builder<R> {
    builder
        .manage(state)
        .manage(terminal::Consoles::default())
        .manage(commands::Quitting::default())
        .invoke_handler(command_handler!())
        .on_window_event(|window, event| {
            if window.label().starts_with(editor_window::LABEL_PREFIX)
                && matches!(
                    event,
                    tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
                )
            {
                log::info(&format!("{}: {event:?}", window.label()));
            }
            if window.label().starts_with(editor_window::LABEL_PREFIX)
                && matches!(event, tauri::WindowEvent::Destroyed)
            {
                editor_window::closed(window.label());
            }
            // The editor's title bar shows maximize or restore (D-69).
            if window.label().starts_with(editor_window::LABEL_PREFIX)
                && matches!(event, tauri::WindowEvent::Resized(_))
            {
                if let Some(w) = window.get_webview_window(window.label()) {
                    editor_window::sync_maximized(&w);
                }
            }
            // With something open, the interface asks before closing.
            if window.label() == MAIN {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    commands::on_main_close(window, api);
                }
            }
            // Closing the app window ends the session: editor windows,
            // server and consoles go with it (they are in the job).
            if window.label() == MAIN && matches!(event, tauri::WindowEvent::Destroyed) {
                // The databases shut down cleanly first (D-73), off the UI thread.
                let services =
                    std::sync::Arc::clone(&window.app_handle().state::<AppState>().services);
                let handle = window.app_handle().clone();
                std::thread::spawn(move || {
                    services.stop_all();
                    handle.exit(0);
                });
            }
        })
        .setup(move |app| {
            open_main_window(app.handle(), &kit.webview_data())?;
            // Extensions follow the projects open in the editor; the editor
            // windows offer to restart them (D-71).
            // Databases marked "start with Dev-Kit" (D-73).
            let (services, env) = {
                let state = app.state::<AppState>();
                (
                    std::sync::Arc::clone(&state.services),
                    state.env.read().map(|e| e.clone()).unwrap_or_default(),
                )
            };
            std::thread::spawn(move || services.autostart(&env));
            let editor = std::sync::Arc::clone(&app.state::<AppState>().editor);
            let notify = app.handle().clone();
            editor.set_on_installed(std::sync::Arc::new(move |ids: &[String]| {
                editor_window::notify_installed(&notify, ids);
            }));
            let (refresh, refresh_kit) = (app.handle().clone(), kit.clone());
            editor_window::watch_projects(
                kit.clone(),
                editor,
                std::sync::Arc::new(move || editor_window::refresh_hidden(&refresh, &refresh_kit)),
            );
            // Projects' actions: state and output go to the app window.
            let events = app.handle().clone();
            app.manage(std::sync::Arc::new(run::Runner::new(
                kit.clone(),
                std::sync::Arc::new(move |e: run::Event| {
                    let _ = events.emit_to(MAIN, "run", e);
                }),
            )));
            app.manage(std::sync::Arc::new(github::Login::default()));
            let updater = std::sync::Arc::new(self_update::Updater::new(kit.clone()));
            app.manage(std::sync::Arc::clone(&updater));
            let settings = Settings::load(&kit).settings;
            // DEVKIT_NO_UPDATE: no automatic check (as for the kit's scripts).
            if settings.auto_update
                && app
                    .state::<AppState>()
                    .host
                    .get("DEVKIT_NO_UPDATE")
                    .is_none()
            {
                let token = github_token(&app.state::<AppState>().host);
                std::thread::spawn(move || {
                    // After the start, not competing with it.
                    std::thread::sleep(std::time::Duration::from_secs(20));
                    if let Ok(net) = net::Net::new(&settings, token) {
                        updater.check(&net);
                    }
                });
            }
            start_missing_required(app.handle(), &kit);
            // Antigravity's login left in Windows by a run that did not end
            // cleanly, the kit moved, the agents aligned; then the watcher.
            let background = kit.clone();
            let env = app.state::<AppState>().env();
            std::thread::spawn(move || {
                if let Err(e) = keyring::save(&background) {
                    log::warn(&format!("antigravity login: {e:#}"));
                }
                // Git follows the GitHub account, also after a sign-in or a
                // sign-out made with gh in a console.
                let signed_in = github::token(&background, &env).is_some();
                if let Err(e) = github::sync_git(&background, &env, signed_in) {
                    log::warn(&format!("github: git helper: {e:#}"));
                }
                let _ = agents_sync::run(&background, None);
                agents::watch(background);
            });
            let handle = app.handle().clone();
            instance::listen(&kit, move |message| {
                if message.trim() == "focus" {
                    if let Some(window) = handle.get_webview_window(MAIN) {
                        let _ = window.unminimize();
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            });
            Ok(())
        })
}

/// First start, a required component deleted, or the editor of another
/// release: install what is missing without asking; the "Linguaggi e
/// strumenti" page shows the progress.
fn start_missing_required<R: tauri::Runtime>(app: &tauri::AppHandle<R>, kit: &Kit) {
    let mut missing = setup_task::missing_required(kit);
    missing.extend(setup_task::outdated_app(kit));
    if missing.is_empty() {
        return;
    }
    let opts = setup_runner::Options {
        only: missing,
        ..setup_runner::Options::default()
    };
    let state = app.state::<AppState>();
    if let Err(e) = state
        .setup
        .start(opts, true, commands::setup_notify(app.clone()))
    {
        log::warn(&format!("automatic setup: {e}"));
    }
}

fn open_main_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    data_dir: &Path,
) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("NEXKORA")
        .inner_size(1180.0, 760.0)
        .min_inner_size(820.0, 560.0)
        // The interface draws its own title bar (ui/src/titlebar.tsx).
        .decorations(false)
        .shadow(true)
        .data_directory(data_dir.to_path_buf())
        .build()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::Arch;
    use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
    use tauri::webview::InvokeRequest;

    fn state() -> AppState {
        let kit = Kit::at(std::env::temp_dir().join("devkit-ipc-test"), Arch::X64);
        let editor = std::sync::Arc::new(editor::Editor::new(kit.clone(), KitEnv::default()));
        let setup = std::sync::Arc::new(setup_task::SetupTask::new(
            kit.clone(),
            KitEnv::default(),
            Settings::default(),
            None,
        ));
        AppState {
            setup,
            editor,
            services: std::sync::Arc::new(services::Services::new(kit.clone())),
            kit,
            lang: std::sync::RwLock::new("it"),
            moved_from: None,
            webview: "test".into(),
            env: std::sync::RwLock::new(KitEnv::default()),
            host: env::Host::default(),
        }
    }

    fn invoke(
        webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
        url: &str,
    ) -> Result<serde_json::Value, serde_json::Value> {
        get_ipc_response(
            webview,
            InvokeRequest {
                cmd: "app_info".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: url.parse().unwrap(),
                body: tauri::ipc::InvokeBody::default(),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<serde_json::Value>().unwrap())
    }

    #[test]
    fn help_is_translated() {
        assert_ne!(i18n::t("en", HELP, &[]), HELP);
    }

    /// DECISIONS.md D-14: an editor window (local server URL) cannot call
    /// the app's commands; the main window can.
    #[test]
    fn editor_windows_cannot_invoke_commands() {
        let app = mock_builder()
            .manage(state())
            .invoke_handler(command_handler!())
            .build(tauri::generate_context!())
            .unwrap();
        let main = WebviewWindowBuilder::new(&app, MAIN, WebviewUrl::App("index.html".into()))
            .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
            .build()
            .unwrap();
        let r = invoke(&main, "http://tauri.localhost");
        assert!(r.is_ok(), "main window must work: {r:?}");

        let editor_url = "http://127.0.0.1:7000/";
        let editor = WebviewWindowBuilder::new(
            &app,
            "editor-1",
            WebviewUrl::External(editor_url.parse().unwrap()),
        )
        .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
        .build()
        .unwrap();
        assert!(
            invoke(&editor, editor_url).is_err(),
            "editor window must be refused"
        );

        // Even a window labelled like an editor but loading app pages is refused.
        let other =
            WebviewWindowBuilder::new(&app, "editor-2", WebviewUrl::App("index.html".into()))
                .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
                .build()
                .unwrap();
        assert!(invoke(&other, "http://tauri.localhost").is_err());
    }
}
