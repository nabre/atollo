//! Atollo: portable development environment for Windows in one exe.
//!
//! Start order (docs/app/ARCHITECTURE.md, Startup): kit root → folders and settings →
//! environment of this process inside `data\` → command line or window →
//! one instance per kit → WebView2 → Job Object → main window.
#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod agents;
mod cli;
mod commands;
mod db;
mod detect;
mod editor;
mod env;
mod fsx;
mod github;
mod i18n;
mod install;
mod kit;
mod log;
mod net;
mod os;
mod ports;
mod projects;
mod router;
mod servers;
mod settings;
mod setup;
mod tray;
mod versions;

use std::path::Path;

use tauri::{Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
#[cfg(windows)]
use windows::core::HSTRING;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

use crate::cli::Cli;
use crate::commands::{AppState, MAIN};
use crate::env::{Host, KitEnv};
use crate::kit::Kit;
use crate::settings::Settings;

fn main() {
    std::process::exit(run());
}

/// Errors before (or instead of) the window, on macOS and Linux: stderr.
#[cfg(not(windows))]
fn fatal(_lang: &str, message: &str) {
    eprintln!("{message}");
}

/// Message box for errors before (or instead of) the window; also on
/// stderr, for the command line (a service session has no message box).
#[cfg(windows)]
fn fatal(lang: &str, message: &str) {
    eprintln!("{message}");
    let title = HSTRING::from(i18n::t(lang, "Atollo non può partire", &[]));
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
    let early_lang = i18n::resolve(None, host.get("ATOLLO_LANG"), windows_lang.as_deref());

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
    // macOS App Translocation: a read-only copy where a kit would be lost
    // (D-113); `None` elsewhere.
    if let Some(text) = kit::launch_problem(kit::Os::current(), &exe, early_lang) {
        fatal(early_lang, &text);
        return 1;
    }
    let kit = match Kit::from_exe(&exe) {
        // The host's TEMP, before the profile folders move into data\ (D-04,
        // D-90); CP_HOST_TEMP when started from the kit's own environment.
        Ok(kit) => kit.with_host(
            host.get("CP_HOST_TEMP")
                .or(host.get("TEMP"))
                .or(host.get("TMP"))
                .map(Path::new),
        ),
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
            // Windows keeps its text; macOS and Linux name a read-only disk.
            let text = kit::prepare_error_message(kit::Os::current(), &e, early_lang);
            fatal(early_lang, &text);
            return 1;
        }
    };

    log::init(&kit.logs());
    // Before the environment points at them: one folder per version.
    versions::migrate(&kit);
    // The versions side by side, known to Gradle, Maven and dotnet.
    versions::side_by_side::refresh(&kit);
    setup::self_update::remove_old(&exe);
    setup::self_update::record_version(&kit);
    settings::move_secrets(&kit);
    let loaded = Settings::load(&kit);
    let lang = i18n::resolve(
        loaded.settings.lang.as_deref(),
        host.get("ATOLLO_LANG"),
        windows_lang.as_deref(),
    );
    // A kit disk that runs no program (noexec, showexec) stops the app and
    // every command, `prepare` included (D-105); FAT32 and exFAT are only
    // logged. Nothing on Windows.
    if let Some(text) = disk_stop(os::volume::disk_notice(&kit, lang)) {
        fatal(lang, &text);
        return 1;
    }
    if let Some(problem) = &loaded.problem {
        log::warn(&i18n::t("en", "Le impostazioni in {0} non sono leggibili: uso quelle predefinite e non modifico il file.", &[problem]));
    }
    net::ca::refresh(&kit, &loaded.settings);
    let kit_env = KitEnv::build(&kit, &loaded.settings, lang, &host);
    if let Err(e) = kit_env.create_dirs() {
        log::warn(&format!("creating kit folders: {e}"));
    }
    env::apply_to_current_process(&kit_env, &kit);
    if let Err(e) = setup::kit_files::write_all(&kit) {
        log::warn(&format!("writing kit files: {e:#}"));
    }

    match command {
        Cli::Gui => {}
        Cli::Env { project } => {
            let env = match project {
                Some(p) => versions::project_env(&kit, &p, kit_env.clone()),
                None => kit_env.clone(),
            };
            return cli::print_env(&env, &mut *cli::output()).map_or(1, |()| 0);
        }
        Cli::Versions(None) => {
            return cli::print_versions(&kit, lang, &mut *cli::output()).map_or(1, |()| 0)
        }
        Cli::Versions(Some(id)) => {
            return version_lines(&kit, &kit_env, &loaded.settings, &host, &id)
        }
        Cli::Prepare(args) => return prepare(&kit, &kit_env, &loaded.settings, &host, lang, &args),
        Cli::Sync { project, verbose } => {
            let report = agents::sync::run(&kit, project.as_deref());
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
                agents::keyring::restore(&kit)
            } else {
                agents::keyring::save_after_run(&kit)
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
            return match projects::serve::run(&dir, port, lang) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("{e:#}");
                    1
                }
            };
        }
        Cli::Check => {
            let checks = setup::checkup::check_all(&kit, &kit_env);
            let ok = checks.iter().all(|c| c.ok);
            return cli::print_checks(&checks, &mut *cli::output()).map_or(1, |()| i32::from(!ok));
        }
        Cli::Components(target) => {
            let kit = target.map_or_else(|| kit.clone(), |t| kit.for_target(t));
            return cli::print_components(&kit, lang, &mut *cli::output()).map_or(1, |()| 0);
        }
        Cli::Version => {
            let _ = writeln!(cli::output(), "Atollo {}", env!("CARGO_PKG_VERSION"));
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

    let guard = match os::instance::acquire(&kit) {
        Ok(os::instance::Instance::Primary(guard)) => guard,
        Ok(os::instance::Instance::Secondary) => {
            if let Err(e) = os::instance::notify_primary(&kit, "focus") {
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

    // The projects' data in data\projects (D-89), as the one app of the kit
    // (D-08): a command line run or a second start would race the open app's
    // writes of passwords and ports. Before the Runner reads the ports.
    projects::migrate::run(&kit);
    // The pools and sites of Nginx become Caddy's (D-120), before the
    // pools and the projects are read.
    servers::migrate::run(&kit);
    // Leftovers of files in transit (D-90): only here, as the one app of the
    // kit (D-08); a command line run would race the open app's programs.
    os::temp::sweep(&kit);
    // The kept conda packages no list names any more (D-118); left for the
    // end of a preparation that runs now.
    let _ = setup::runner::while_idle(&kit, || install::conda::sweep(&kit));

    // WebView2 lookup sets a variable for the fixed runtime: still single-threaded here.
    let runtime = os::webview::locate(&kit);
    let Some(runtime) = runtime else {
        log::error("web view runtime not found");
        // WebView2 on Windows (same text as before), WebKitGTK on Linux
        // with the command of the distribution.
        let os_release = std::fs::read_to_string("/etc/os-release").ok();
        let text = os::webview::missing_message(&kit, os_release.as_deref(), lang);
        fatal(lang, &text);
        return 1;
    };
    let webview_desc = match &runtime {
        os::webview::Runtime::System(v) => format!("system {v}"),
        os::webview::Runtime::Fixed { folder, version } => {
            format!("fixed {version} ({})", folder.display())
        }
    };

    match os::jobs::init() {
        Ok(None) => {}
        Ok(Some(note)) => log::warn(&format!("child processes: {note}")),
        Err(e) => log::error(&format!("child processes: {e:#}")),
    }
    log::info(&format!(
        "start {} root={} platform={} lang={lang} webview={webview_desc} moved_from={:?}",
        env!("CARGO_PKG_VERSION"),
        kit.root().display(),
        kit.arch().platform(),
        moved_from
    ));

    let setup = std::sync::Arc::new(setup::task::SetupTask::new(
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
        services: std::sync::Arc::new(db::services::Services::new(kit.clone())),
        servers: std::sync::Arc::new(servers::WebServers::new(kit.clone())),
        env: std::sync::RwLock::new(kit_env),
        closing: agents::Closing::default(),
    };
    // The exe swap of a staged update happens on Exit (`run` may not return).
    let (update_kit, update_exe) = (kit.clone(), exe.clone());
    let result = build_app(tauri::Builder::default(), state, kit.clone())
        .build(context())
        .map(|app| {
            app.run(move |handle, event| {
                if let tauri::RunEvent::Exit = event {
                    // The app's terminal sessions (macOS and Linux) end with
                    // it: their programs run outside the kit's folder, where
                    // `stop_children` does not look. None on Windows.
                    handle.state::<editor::terminal::Consoles>().close_all();
                    // The programs' temp left on the PC goes (D-90): the
                    // programs are ended first, or they would still hold it.
                    if update_kit.system_temp().is_some() {
                        os::sessions::stop_children(&update_kit);
                        os::temp::cleanup_at_exit(&update_kit);
                    }
                    match setup::self_update::apply_staged(&update_kit, &update_exe) {
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

/// What the notice on the kit's disk means at start (D-105): the text
/// that stops the app, logged as an error; a warning is only logged.
fn disk_stop(notice: Option<os::volume::DiskNotice>) -> Option<String> {
    let notice = notice?;
    if notice.blocking {
        log::error(&format!("kit disk: {}", notice.text));
        Some(notice.text)
    } else {
        log::warn(&format!("kit disk: {}", notice.text));
        None
    }
}

/// Raises GitHub's API limit when the user has a token.
fn github_token(host: &Host) -> Option<String> {
    host.get("GITHUB_TOKEN")
        .or(host.get("GH_TOKEN"))
        .map(str::to_owned)
}

/// `Atollo.exe prepare`: 0 all good, 1 some component failed, 3 another
/// preparation is running.
fn prepare(
    kit: &Kit,
    kit_env: &KitEnv,
    settings: &Settings,
    host: &Host,
    lang: &str,
    args: &cli::PrepareArgs,
) -> i32 {
    let kit_scoped;
    let kit = match args.target {
        Some(t) => {
            kit_scoped = kit.for_target(t);
            &kit_scoped
        }
        None => kit,
    };
    let mut out = cli::output();
    if let Err(e) = os::jobs::init() {
        log::warn(&format!("child processes: {e:#}"));
    }
    let net = match net::Net::new(settings, github_token(host)) {
        Ok(net) => net.with_github_cache(kit.data().join("cache").join("github")),
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
    let mut printer =
        cli::Printer::new(&mut *out, lang).with_downloads_dir(setup::downloads::dir(kit));
    let results = match setup::runner::run(&ctx, &args.options(), &mut printer) {
        Ok(results) => results,
        Err(e) if e.is::<setup::runner::Busy>() => {
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
        .filter(|(_, o)| matches!(o, setup::runner::Outcome::Installed { .. }))
        .map(|(id, _)| id.as_str())
        .collect();
    let checks: Vec<_> = setup::catalog::Catalog::get()
        .all()
        .iter()
        .filter(|c| touched.contains(&c.id.as_str()))
        .filter_map(|c| setup::checkup::check(kit, kit_env, c))
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

/// Help text of the command line (Italian source; translated by the
/// catalog). `{0}` is the program's name ([`program_name`]), `{1}` the
/// kit's Antigravity command ([`help_text`]); the target ids
/// are those of `Target::ALL`.
const HELP: &str = "Uso: {0} [comando]\n\nComandi:\n  env                     mostra l'ambiente del kit\n      --project <nome>    quello dei programmi di un progetto\n  components              elenca linguaggi e strumenti, con quelli installati\n      --target <id>       per un altro sistema: win32-x64, win32-arm64,\n                          darwin-arm64, darwin-x64, linux-x64, linux-arm64\n  versions [id]           le versioni salvate; con un id, quelle da scaricare\n  check                   verifica che i programmi installati rispondano\n  prepare [opzioni]       installa e aggiorna i componenti scelti,\n                          rimuove quelli non scelti\n      --only a,b          solo questi e ciò che serve loro, senza rimuovere\n      --skip a,b          non questi\n      --update-all        aggiorna anche quelli già installati\n      --force             scarica di nuovo\n      --line id=x.y       salva questa versione accanto alla predefinita\n      --target <id>       prepara per un altro sistema (id come per components)\n  sync [opzioni]          allinea istruzioni, skill e server MCP degli agenti\n      --project <cartella> anche le istruzioni di questo progetto\n      --verbose           mostra ogni modifica\n  keyring restore|save    accesso di Antigravity (lo usa {1} del kit)\n  serve <cartella> <porta> server di file per un sito statico\n  version                 mostra la versione\n  help                    mostra questo aiuto\n\nSenza comando apre la finestra di Atollo.";

/// The program's name in the help: the file it was started from
/// (`Atollo-0.2.0-win32-x64.exe`, `Atollo.AppImage`, `Atollo`, ...); on
/// Windows only an `.exe` name, else `Atollo.exe`; elsewhere `Atollo`
/// when that is unknown.
fn program_name(os: kit::Os, exe: Option<&Path>) -> String {
    let name = exe
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .filter(|n| !n.is_empty());
    if os.is_windows() {
        // A Windows path read anywhere: `\` separates there, not on Unix.
        return name
            .and_then(|n| n.rsplit(['\\', '/']).next().map(str::to_owned))
            .filter(|n| n.to_ascii_lowercase().ends_with(".exe"))
            .unwrap_or_else(|| "Atollo.exe".to_owned());
    }
    name.unwrap_or_else(|| "Atollo".to_owned())
}

/// The help for `os`; `{1}` is the kit's Antigravity command (`agy.cmd`
/// on Windows, as always; `agy` elsewhere).
fn help_text(lang: &str, os: kit::Os, exe: Option<&Path>) -> String {
    let agy = if os.is_windows() { "agy.cmd" } else { "agy" };
    i18n::t(lang, HELP, &[&program_name(os, exe), agy])
}

/// `versions <id>`: the lines of `id` its source offers.
fn version_lines(kit: &Kit, kit_env: &KitEnv, settings: &Settings, host: &Host, id: &str) -> i32 {
    let mut out = cli::output();
    let result = (|| -> anyhow::Result<Vec<String>> {
        versions::check(id, None)?;
        let c = setup::catalog::Catalog::get()
            .find(id)
            .ok_or_else(|| anyhow::anyhow!("{id}"))?;
        let installer = install::installer(id).ok_or_else(|| anyhow::anyhow!("{id}"))?;
        let net = net::Net::new(settings, github_token(host))?
            .with_github_cache(kit.data().join("cache").join("github"));
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let ctx = install::Ctx {
            kit,
            net: &net,
            env: kit_env,
            cancel: &cancel,
        };
        let lines = installer.lines(&ctx, c)?;
        let _ = versions::save_offered(kit, id, &lines);
        Ok(lines)
    })();
    match result {
        Ok(lines) => {
            for l in lines {
                let _ = writeln!(out, "  {l}");
            }
            0
        }
        Err(e) => {
            let _ = writeln!(out, "  x {e:#}");
            1
        }
    }
}

/// The help for this program: [`kit::app_exe`] names the AppImage when the
/// app runs from one, not the inner program of its temporary mount, which
/// the user cannot start.
fn help(lang: &str) -> String {
    help_text(lang, kit::Os::current(), kit::app_exe().as_deref())
}

/// The app's context, expanded once for the app and its tests: on macOS
/// `generate_context!` embeds the Info.plist under a global symbol, so a
/// second expansion in the same binary does not link.
fn context<R: tauri::Runtime>() -> tauri::Context<R> {
    tauri::generate_context!()
}

/// The app: state, commands, main window, focus requests from a second start.
fn build_app<R: tauri::Runtime>(
    builder: tauri::Builder<R>,
    state: AppState,
    kit: Kit,
) -> tauri::Builder<R> {
    builder
        .manage(state)
        .manage(editor::terminal::Consoles::default())
        .manage(commands::Quitting::default())
        .invoke_handler(command_handler!())
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
                    commands::on_main_close(window, api);
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
                let handle = window.app_handle().clone();
                std::thread::spawn(move || {
                    servers.stop_all();
                    services.stop_all();
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
            // Then the server profiles marked "Avvia con Atollo".
            let (services, servers, env, lang) = {
                let state = app.state::<AppState>();
                (
                    std::sync::Arc::clone(&state.services),
                    std::sync::Arc::clone(&state.servers),
                    state.env.read().map(|e| e.clone()).unwrap_or_default(),
                    state.lang(),
                )
            };
            std::thread::spawn(move || {
                services.autostart(&env);
                servers.autostart(&env, lang);
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
                    let _ = events.emit_to(MAIN, "run", e);
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
                let (token, update_kit) =
                    (github_token(&app.state::<AppState>().host), kit.clone());
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
                let token = github_token(&app.state::<AppState>().host);
                std::thread::spawn(move || {
                    // After the start, not competing with it.
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    commands::versions::refresh_stale_lines(&lines_kit, &env, token);
                });
            }
            start_missing_required(app.handle(), &kit);
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
            os::instance::listen(&kit, move |message| {
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
fn start_missing_required<R: tauri::Runtime>(app: &tauri::AppHandle<R>, kit: &Kit) {
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
        .start(opts, true, commands::setup_notify(app.clone()))
    {
        log::warn(&format!("automatic setup: {e}"));
    }
}

fn open_main_window<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    data_dir: &Path,
    theme: Option<tauri::Theme>,
) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title("Atollo")
        .inner_size(1180.0, 760.0)
        .min_inner_size(820.0, 560.0)
        // The interface draws its own title bar (ui/src/titlebar.tsx).
        .decorations(false)
        .shadow(true)
        .data_directory(data_dir.to_path_buf())
        .theme(theme)
        .build()?;
    Ok(())
}

#[cfg(test)]
#[path = "../../test/rust/main.rs"]
mod tests;
