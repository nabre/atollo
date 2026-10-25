//! The kit's terminals (terminals.rs): the default one and the user's, with
//! their tools and versions, opened in any folder of the PC; the sessions
//! of the app's terminal (`pty_*`).

use super::*;
use crate::editor::terminals::{self, Terminal, TerminalsView};
use tauri::{Emitter, Manager};

#[tauri::command]
pub async fn terminals_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<TerminalsView> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || terminals::list(&kit, lang))
        .await
        .or_fail("terminals")
}

/// Saves a new terminal (empty id) or the changes of one; returns its id.
#[tauri::command]
pub async fn terminal_save<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    terminal: Terminal,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (kit, lang) = (state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        terminals::save(&kit, terminal, lang).map_err(CmdError::from)
    })
    .await
    .or_fail("terminals")?
}

#[tauri::command]
pub async fn terminal_remove<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, host, env) = (state.kit.clone(), state.host.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || {
        terminals::remove(&kit, &id).or_fail_chain("terminals")?;
        // Its sandboxes, one per folder, and their disks go with it (D-169).
        let names = crate::isolation::terminal_sandboxes(&kit, &id);
        crate::isolation::remove_owned(&kit, &host, &env, &names);
        Ok(())
    })
    .await
    .or_fail("terminals")?
}

/// Opens the terminal `id` in `dir` (`None`: its own folder), installing
/// first what every console needs (the kit's PowerShell).
#[tauri::command]
pub async fn terminal_launch<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
    dir: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let host = state.host.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(terminal) = terminals::find(&kit, &id, lang) else {
            return Err(crate::common::i18n::t(lang, "Terminale non trovato: {0}", &[&id]).into());
        };
        let dir = terminals::start_dir(&kit, &terminal, dir.as_deref(), lang)?;
        let env = terminals::env(&kit, &terminal, ensure(&app, &task, &kit, lang, &[])?);
        // In a microVM of its own when the VM is on (D-167), with the kit's
        // tools for its Linux.
        let guest = crate::isolation::vm::guest_target(kit.target()).map(|t| kit.for_target(t));
        if let Some(guest) = guest.filter(|_| crate::isolation::vm_on(&kit, &host, &env)) {
            // Never a folder whose files the PC runs, writable from the VM;
            // from here on only the folder checked, canonical, is used.
            let Some(workspace) = crate::isolation::vm_workspace(&kit, &host, &dir) else {
                return Err(crate::common::i18n::t(
                    lang,
                    "La cartella {0} non si apre nella VM: è la radice di un disco, un link, una cartella di sistema, o è nel kit o nel tuo profilo (salvo i progetti del kit). Scegli la cartella di un progetto.",
                    &[&dir.display().to_string()],
                )
                .into());
            };
            let windows = kit.target().is_windows();
            let Some(guest_dir) = crate::isolation::vm::guest_path(windows, &workspace) else {
                return Err(crate::common::i18n::t(
                    lang,
                    "La cartella {0} non si apre nella VM: scegli una cartella di un disco del PC.",
                    &[&dir.display().to_string()],
                )
                .into());
            };
            let apps = guest.apps();
            if let Some(vm) =
                crate::isolation::kit_terminal_vm(&kit, &host, &env, &terminal.id, &workspace, &apps)
            {
                // The default terminal: the tools the kit chose, for the
                // VM's Linux; another: its own (D-170).
                let wanted = if terminal.id == terminals::DEFAULT_ID {
                    terminals::chosen_tools(&kit)
                } else {
                    terminal.tools.clone()
                };
                ensure_guest_tools(&app, &task, &guest, &wanted);
                std::fs::create_dir_all(&apps).or_fail("terminals")?;
                let base = KitEnv::build(&guest, &Settings::load(&kit).settings, lang, &host);
                let tools = crate::isolation::guest_tools(&kit, &guest, &terminal, base);
                vm.start(&env, lang).or_fail_chain("vm")?;
                let args = crate::isolation::vm::kit_terminal_args(&vm.name, &guest_dir, &tools);
                let launch = vm.launch(&kit, args, "VM");
                return terminal::open_launch(&app, &vm.session_env(&env), &workspace, launch)
                    .or_fail_chain("terminal");
            }
        }
        terminal::open_console(&app, &kit, &env, &dir).or_fail_chain("terminal")
    })
    .await
    .or_fail("terminals")?
}

/// The kit's tools `wanted` (id → the line chosen; `None`: the newest) for
/// the VM's Linux (`guest`, the kit scoped to its target), installed in
/// `data/apps/linux-…` when missing, waiting for the downloads (their
/// progress in the status bar): those with a build for it that this PC can
/// install, not those that finish on their own system
/// (`router::bootstrap`). A failure only goes to the log: the session
/// opens with what is there.
pub(crate) fn ensure_guest_tools<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    guest: &Kit,
    wanted: &std::collections::BTreeMap<String, Option<String>>,
) {
    let catalog = Catalog::get();
    let target = guest.target();
    let tools: Vec<(&crate::setup::Component, Option<&String>)> = wanted
        .iter()
        .filter_map(|(id, line)| Some((catalog.find(id)?, line.as_ref())))
        .filter(|(c, _)| {
            c.is_available_on(target)
                && install::installer(&c.id)
                    .is_some_and(|i| !crate::router::bootstrap::stages_for(i, c, target))
        })
        .collect();
    let run = |opts: Options| {
        while task.snapshot().running {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        if let Err(e) = task.start(opts, true, setup_notify(app.clone())) {
            crate::common::log::warn(&format!("tools for the VM: {e}"));
        }
        while task.snapshot().running {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
    };
    let missing: Vec<_> = tools
        .iter()
        .filter(|(c, _)| !c.is_installed(guest))
        .collect();
    if !missing.is_empty() {
        crate::common::log::info(&format!(
            "tools for the VM ({}): {:?}",
            target.as_str(),
            missing.iter().map(|(c, _)| &c.id).collect::<Vec<_>>()
        ));
        run(Options {
            target: Some(target),
            only: missing.iter().map(|(c, _)| c.id.clone()).collect(),
            at: missing
                .iter()
                .filter_map(|(c, line)| Some((c.id.clone(), (*line)?.clone())))
                .collect(),
            ..Options::default()
        });
    }
    // A line chosen, beside the one installed.
    let has_line = |c: &crate::setup::Component, line: &str| {
        crate::versions::default_line(guest, c).as_deref() == Some(line)
            || crate::versions::saved(guest, c)
                .iter()
                .any(|l| l.line == line)
    };
    let lines: Vec<(String, String)> = tools
        .iter()
        .filter_map(|(c, line)| Some((*c, (*line)?)))
        .filter(|(c, line)| c.is_installed(guest) && !has_line(c, line))
        .map(|(c, line)| (c.id.clone(), line.clone()))
        .collect();
    if !lines.is_empty() {
        run(Options {
            target: Some(target),
            lines,
            ..Options::default()
        });
    }
}

/// The databases of `project` (folder `dir`) for its VM (D-171): their
/// servers started on the PC (the kit's, never in the VM: their programs,
/// settings and data are out of its reach), the PC's ports its sandbox may
/// reach, and the file of the variables that point its programs there as
/// the project's user: the keys the kit writes in a `.env` like the
/// project's (`env_wanted`), with the VM's way to the PC, plus the MySQL
/// client's. A file readable by its owner only (`db.env` beside the
/// sandbox's kit), given with `--env-file`: the password is never on a
/// command line. The project's `.env` keeps its own lines: a variable of
/// the session comes first (Laravel, dotenv, Django).
pub(crate) fn project_vm_dbs(
    services: &crate::db::services::Services,
    kit: &Kit,
    project: &str,
    dir: &std::path::Path,
    env: &KitEnv,
) -> CmdResult<(Vec<u16>, Option<std::path::PathBuf>)> {
    let dbs = services.project_dbs(project);
    let file = crate::isolation::kit::dir(kit, &crate::isolation::vm::name(project)).join("db.env");
    if dbs.is_empty() {
        // A database removed takes its password out of the VM's file too.
        let _ = std::fs::remove_file(&file);
        return Ok((Vec::new(), None));
    }
    let text = crate::platform::safe_path::read_text(dir, &dir.join(".env")).unwrap_or_default();
    let mut ports = Vec::new();
    let mut lines = String::new();
    for db in dbs {
        let port = services
            .start_for(project, &db.engine, env)
            .or_fail_chain("database")?;
        ports.push(port);
        let in_vm = crate::isolation::db_in_vm(&db, port);
        for (k, v) in crate::db::auth::env_wanted(&text, &in_vm) {
            lines.push_str(&format!("{k}={v}\n"));
        }
        if crate::db::DbEngine::from_id(&db.engine) == Some(crate::db::DbEngine::MySql) {
            lines.push_str(&format!(
                "MYSQL_HOST={}\nMYSQL_TCP_PORT={port}\n",
                crate::isolation::vm::HOST
            ));
        }
    }
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent).or_fail("database")?;
    }
    crate::platform::files::replace_private(&file, &lines).or_fail_chain("database")?;
    Ok((ports, Some(file)))
}

/// `vm` of `project` (folder `dir`) ready for a session or an action: the
/// kit's tools for Linux at the project's lines (D-170), its databases on
/// the PC's servers and the ports to them (D-171). One way for the
/// terminal and the actions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn project_vm_ready<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    services: &crate::db::services::Services,
    kit: &Kit,
    host: &crate::env::Host,
    lang: &str,
    project: &str,
    dir: &std::path::Path,
    env: &KitEnv,
    vm: crate::isolation::Vm,
) -> CmdResult<crate::isolation::Vm> {
    let mut tools = project_vm_tools(app, task, kit, host, lang, project)?;
    let (ports, file) = project_vm_dbs(services, kit, project, dir, env)?;
    tools.env_file = file;
    Ok(vm.with_tools(tools).with_host_ports(ports))
}

/// The kit's tools for `project` in its VM (D-170): those the kit chose, at
/// the lines the project chose, installed for the VM's Linux when missing;
/// nothing where there is no VM's Linux.
pub(crate) fn project_vm_tools<R: Runtime>(
    app: &AppHandle<R>,
    task: &Arc<SetupTask>,
    kit: &Kit,
    host: &crate::env::Host,
    lang: &str,
    project: &str,
) -> CmdResult<crate::isolation::GuestTools> {
    let Some(guest) = crate::isolation::vm::guest_target(kit.target()).map(|t| kit.for_target(t))
    else {
        return Ok(crate::isolation::GuestTools::default());
    };
    let mut wanted = terminals::chosen_tools(kit);
    for (id, line) in crate::versions::project_choice(kit, project) {
        wanted.insert(id, Some(line));
    }
    ensure_guest_tools(app, task, &guest, &wanted);
    std::fs::create_dir_all(guest.apps()).or_fail("vm")?;
    let base = KitEnv::build(&guest, &Settings::load(kit).settings, lang, host);
    Ok(crate::isolation::project_guest_tools(
        kit, &guest, project, base,
    ))
}

/// The system's "Select folder" window, starting in `start`; `None` when it
/// was closed without a choice.
#[tauri::command]
pub async fn terminal_dir_pick<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    start: Option<String>,
) -> CmdResult<Option<String>> {
    ensure_main(&webview)?;
    let owner = crate::platform::desktop::owner(&webview.window()).or_fail("window")?;
    let title = crate::common::i18n::t(state.lang(), "Scegli la cartella del terminale", &[]);
    let projects = state.kit.projects();
    tauri::async_runtime::spawn_blocking(move || {
        let start = start.map(PathBuf::from).unwrap_or(projects);
        crate::platform::desktop::pick_folder(&owner, &title, Some(&start))
            .map(|d| d.map(|d| d.display().to_string()))
    })
    .await
    .or_fail("terminals")?
    .or_fail_chain("terminals")
}

// The app's terminal on every system (editor/pty.rs): its sessions are
// opened by `terminal_open`, `terminal_launch` and `agent_launch`, with the
// checks and the environment of each; the panel of the main window shows,
// types into, resizes and closes them, and so does the window of a
// detached session (`terminal-<id>`, D-149), for that session only.

/// The main window, or the window of session `id`.
/// A web link of session `id`'s output, clicked: the PC's browser, with
/// the PC's environment (`sbx login`'s page, D-169); `http` and `https`
/// only.
#[tauri::command]
pub fn pty_link_open<R: Runtime>(webview: Webview<R>, id: u64, url: String) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let url: tauri::Url = url.parse().or_fail("link")?;
    crate::editor::window::open_in_browser(&url).or_fail_chain("browser")
}

fn ensure_pty<R: Runtime>(webview: &Webview<R>, id: u64) -> CmdResult<()> {
    if webview.label() == MAIN || terminal::session_of(webview.label()) == Some(id) {
        Ok(())
    } else {
        Err("forbidden".into())
    }
}

/// The terminal sessions, running or ended but still shown; to the window
/// of a detached session, only its own.
#[tauri::command]
pub fn pty_sessions<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
) -> CmdResult<Vec<crate::editor::pty::PtyInfo>> {
    let ptys = &app.state::<terminal::Consoles>().ptys;
    if let Some(id) = terminal::session_of(webview.label()) {
        return Ok(ptys.info(id).into_iter().collect());
    }
    ensure_main(&webview)?;
    Ok(ptys.list())
}

/// The output kept by session `id`, up to the chunk number it carries.
#[tauri::command]
pub fn pty_backlog<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<crate::editor::pty::Chunk> {
    ensure_pty(&webview, id)?;
    app.state::<terminal::Consoles>()
        .ptys
        .backlog(id)
        .or_fail_chain("terminal")
}

/// What was typed in session `id`; queued, never blocking the window.
/// `binary`: bytes that are not text, one character per byte (xterm.js's
/// `onBinary`).
#[tauri::command]
pub fn pty_write<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    data: String,
    binary: Option<bool>,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let ptys = &app.state::<terminal::Consoles>().ptys;
    if binary == Some(true) {
        let Some(bytes) = crate::editor::pty::latin1(&data) else {
            return Err("terminal input: not bytes".to_owned().into());
        };
        return ptys.write_bytes(id, &bytes).or_fail_chain("terminal");
    }
    ptys.write(id, &data).or_fail_chain("terminal")
}

#[tauri::command]
pub fn pty_resize<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
    cols: u16,
    rows: u16,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    app.state::<terminal::Consoles>()
        .ptys
        .resize(id, cols, rows)
        .or_fail_chain("terminal")
}

/// Ends the program of session `id` (hang-up, then its tree after a
/// moment) and forgets the session; the panel drops its tab and its own
/// window, if any, closes.
#[tauri::command]
pub async fn pty_close<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    let ptys = app.state::<terminal::Consoles>().ptys.clone();
    tauri::async_runtime::spawn_blocking(move || {
        ptys.close(id, crate::editor::pty::GRACE)
            .or_fail_chain("terminal")
    })
    .await
    .or_fail("terminal")??;
    let _ = app.emit_to(MAIN, super::events::PTY_CLOSED, id);
    if let Some(window) = app.get_webview_window(&terminal::label(id)) {
        let _ = window.close();
    }
    Ok(())
}

/// Moves session `id` from the panel into a window of its own (D-149).
/// Async: a window is never built on the thread that runs the windows.
#[tauri::command]
pub async fn pty_detach<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    terminal::detach(&app, id).or_fail_chain("terminal")
}

/// Brings session `id` back to the panel and closes its window.
#[tauri::command]
pub async fn pty_dock<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    id: u64,
) -> CmdResult<()> {
    ensure_pty(&webview, id)?;
    terminal::dock(&app, id).or_fail_chain("terminal")
}
