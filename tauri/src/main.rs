//! Atollo: portable development environment for Windows in one exe.
//!
//! Start order (docs/app/ARCHITECTURE.md, Startup): kit root → folders and settings →
//! environment of this process inside `data\` → command line or window →
//! one instance per kit → WebView2 → Job Object → main window.
#![cfg_attr(not(test), windows_subsystem = "windows")]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used))]

mod agents;
mod app;
mod common;
mod db;
mod detect;
mod diagnostics;
mod editor;
mod env;
mod github;
mod install;
mod ipc;
mod isolation;
mod kit;
mod llm;
mod net;
mod os;
mod platform;
mod projects;
mod router;
mod servers;
mod setup;
mod versions;

use std::path::Path;

use tauri::Manager;

use crate::app::cli::{self, help, models, prepare, version_lines, Cli};
use crate::app::{build_app, context};
use crate::common::settings::{self, Settings};
use crate::common::{i18n, log};
use crate::diagnostics::health;
use crate::env::{Host, KitEnv};
use crate::ipc::AppState;
use crate::kit::{backup, Kit};

fn main() {
    std::process::exit(run());
}

/// Errors before (or instead of) the window: on stderr, for the command
/// line (a service session has no message box), and in a message box on
/// Windows.
fn fatal(lang: &str, message: &str) {
    eprintln!("{message}");
    platform::desktop::error_box(&i18n::t(lang, "Atollo non può partire", &[]), message);
}

fn run() -> i32 {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let command = cli::parse(&args);
    let host = Host::capture();
    platform::jobs::set_browser_env(&host);
    let windows_lang = platform::host::locale();
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
        // On macOS and Linux only a folder of this user that nobody else
        // can write in: a shared `/tmp` would let another account prepare
        // `Atollo/<id>` before the app.
        Ok(kit) => kit.with_host(
            host.temp()
                .map(Path::new)
                .filter(|t| platform::temp::private_base(t)),
        ),
        Err(e) => {
            fatal(
                early_lang,
                &i18n::t(early_lang, "Errore imprevisto: {0}", &[&e.to_string()]),
            );
            return 1;
        }
    };
    // A copy of a kit imported (N-12): in place before anything of `data\`
    // is opened, only by the app's window and only when no other app of
    // this kit runs (its guard, taken here for the swap).
    let mut imported = None;
    if matches!(command, cli::Cli::Gui) && backup::pending(kit.root()) {
        if let Ok(platform::instance::Instance::Primary(guard)) = platform::instance::acquire(&kit)
        {
            let result = backup::apply_pending(kit.root());
            drop(guard);
            match result {
                Ok(aside) => imported = aside,
                Err(e) => {
                    fatal(
                        early_lang,
                        &i18n::t(
                            early_lang,
                            "Importazione della copia del kit non riuscita: {0}",
                            &[&format!("{e:#}")],
                        ),
                    );
                    return 1;
                }
            }
        }
    }
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
    if let Some(aside) = &imported {
        log::info(&format!(
            "kit copy imported; what it replaced: {}",
            aside.display()
        ));
    }
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
    if let Some(text) = disk_stop(platform::volume::disk_notice(&kit, lang)) {
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
        Cli::SecretGithub => {
            // Only for the kit's Docker Sandboxes, which runs it on the PC
            // when its proxy needs the value: never for an agent or a
            // project's program that runs the app (D-172, SEC-V7-01).
            let all = platform::processes::all();
            let above = platform::processes::ancestors(&all, std::process::id(), 3);
            let by_sbx = isolation::kit_sbx(&kit)
                .is_some_and(|sbx| isolation::secrets::asked_by_sbx(&above, &sbx));
            if !by_sbx {
                eprintln!("secret github: only for the kit's Docker Sandboxes");
                return 1;
            }
            // Standard output is the token only; nothing else is written.
            return match github::token(&kit, &kit_env) {
                Some(token) => {
                    println!("{token}");
                    0
                }
                None => 1,
            };
        }
        Cli::McpGithub => {
            // Standard output is the agent's MCP channel: errors on stderr.
            return match agents::github_mcp::run(&kit, &kit_env, &loaded.settings) {
                Ok(()) => 0,
                Err(e) => {
                    log::warn(&format!("mcp github: {e:#}"));
                    eprintln!("{e:#}");
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
        Cli::Check { kit: true } => {
            let findings = health::check(&kit, lang);
            let bad = findings.iter().any(|f| f.level == health::Level::Err);
            return cli::print_health(&findings, &mut *cli::output())
                .map_or(1, |()| i32::from(bad));
        }
        Cli::CheckFiles => {
            let report = setup::integrity::check_all(&kit);
            return cli::print_integrity(&report, lang, &mut *cli::output()).map_or(1, i32::from);
        }
        Cli::Check { kit: false } => {
            let checks = setup::checkup::check_all(&kit, &kit_env);
            let ok = checks.iter().all(|c| c.ok);
            return cli::print_checks(&checks, &mut *cli::output()).map_or(1, |()| i32::from(!ok));
        }
        Cli::Components(target) => {
            let kit = target.map_or_else(|| kit.clone(), |t| kit.for_target(t));
            return cli::print_components(&kit, lang, &mut *cli::output()).map_or(1, |()| 0);
        }
        Cli::Models(cmd) => return models(&kit, &loaded.settings, &host, lang, cmd),
        Cli::Llm(action) => {
            let mut out = cli::output();
            let result = match action.as_str() {
                "start" => llm::cli::start(&kit, &kit_env, lang, &mut *out),
                "stop" => llm::cli::stop(&kit, lang, &mut *out),
                _ => llm::cli::status(&kit, lang, &mut *out),
            };
            return match result {
                Ok(()) => 0,
                Err(e) => {
                    let _ = writeln!(out, "  x {e:#}");
                    1
                }
            };
        }
        Cli::Diagnostics => {
            let mut out = cli::output();
            return match diagnostics::export(&kit, &kit_env, lang) {
                Ok(path) => {
                    let _ = writeln!(out, "  √ {}", path.display());
                    0
                }
                Err(e) => {
                    let _ = writeln!(out, "  x {e:#}");
                    1
                }
            };
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

    let guard = match platform::instance::acquire(&kit) {
        Ok(platform::instance::Instance::Primary(guard)) => guard,
        Ok(platform::instance::Instance::Secondary) => {
            if let Err(e) = platform::instance::notify_primary(&kit, "focus") {
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
    platform::temp::sweep(&kit);
    // The kept conda packages no list names any more (D-118); left for the
    // end of a preparation that runs now.
    let _ = setup::runner::while_idle(&kit, || install::conda::sweep(&kit));

    // WebView2 lookup sets a variable for the fixed runtime: still single-threaded here.
    let runtime = platform::webview::locate(&kit);
    let Some(runtime) = runtime else {
        log::error("web view runtime not found");
        // WebView2 on Windows (same text as before), WebKitGTK on Linux
        // with the command of the distribution.
        let os_release = platform::host::os_release();
        let text = platform::webview::missing_message(&kit, os_release.as_deref(), lang);
        fatal(lang, &text);
        return 1;
    };
    let webview_desc = match &runtime {
        platform::webview::Runtime::System(v) => format!("system {v}"),
        platform::webview::Runtime::Fixed { folder, version } => {
            format!("fixed {version} ({})", folder.display())
        }
    };

    match platform::jobs::init() {
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
        llm: std::sync::Arc::new(llm::service::Llm::new(kit.clone())),
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
                    // An editor start in progress gives up before the sweep
                    // below (the windows are gone: waiting freezes nothing).
                    handle.state::<AppState>().editor.stop_and_wait();
                    // The programs' temp left on the PC goes (D-90): the
                    // programs are ended first, or they would still hold it.
                    if update_kit.system_temp().is_some() {
                        platform::sessions::stop_children(&update_kit);
                        platform::temp::cleanup_at_exit(&update_kit);
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
fn disk_stop(notice: Option<platform::volume::DiskNotice>) -> Option<String> {
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

#[cfg(test)]
#[path = "../../test/rust/support.rs"]
mod test_support;

#[cfg(test)]
#[path = "../../test/rust/main.rs"]
mod tests;
