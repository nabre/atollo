//! The main window's end: what is open now, «Remove the drive» (eject),
//! quitting and the confirmation on close.

use super::*;

/// What is still open in this kit: "Rimuovi l'unità" and the close question.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenNow {
    pub processes: Vec<crate::os::sessions::KitProcess>,
    pub editor: bool,
    pub terminals: usize,
    pub actions: usize,
    pub setup: bool,
    /// Database servers started by the app.
    pub databases: usize,
    /// The server of the local models, when it runs (0 or 1).
    pub llm: usize,
    /// Downloads of local models under way.
    pub downloads: usize,
}

impl OpenNow {
    pub fn any(&self) -> bool {
        !self.processes.is_empty()
            || self.editor
            || self.terminals > 0
            || self.actions > 0
            || self.setup
            || self.databases > 0
            || self.llm > 0
            || self.downloads > 0
    }
}

pub fn open_now<R: Runtime>(app: &AppHandle<R>) -> OpenNow {
    use tauri::Manager;
    let state = app.state::<AppState>();
    let actions =
        app.try_state::<Arc<Runner>>().map_or(0, |r| r.running()) + state.servers.running_count();
    OpenNow {
        processes: crate::os::sessions::foreign_processes(&state.kit),
        editor: state.editor.status().running,
        terminals: app.state::<terminal::Consoles>().count(),
        actions,
        setup: state.setup.snapshot().running,
        databases: state.services.running(),
        llm: state.llm.running(),
        downloads: crate::llm::pull::pulling().len(),
    }
}

#[tauri::command]
pub fn eject_list<R: Runtime>(app: AppHandle<R>, webview: Webview<R>) -> CmdResult<OpenNow> {
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
) -> CmdResult<OpenNow> {
    ensure_main(&webview)?;
    use tauri::Manager;
    // First: the agents' background alignments and saves stop now, before
    // the consoles and agents they react to are ended (BUG-V6-15).
    state.closing.begin();
    state.setup.cancel(&setup_notify(app.clone()));
    if let Some(r) = app.try_state::<Arc<Runner>>() {
        r.stop_all();
    }
    state.servers.stop_all();
    state.llm.stop_all();
    for (label, window) in app.webview_windows() {
        if label.starts_with(editor::window::LABEL_PREFIX) {
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
    let closing = state.closing.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // The downloads of models let go of their files on the drive (what
        // arrived is kept for later).
        crate::llm::pull::cancel_all(std::time::Duration::from_secs(10));
        // Asked to shut down before the kit's processes are ended.
        services.stop_all();
        editor.stop();
        // The app's terminal sessions hung up first, as at the app's exit;
        // what is left of them is stopped below by their pids.
        handle.state::<terminal::Consoles>().close_all();
        crate::os::sessions::stop(&consoles);
        let pids: Vec<u32> = crate::os::sessions::kit_processes(&kit)
            .iter()
            .map(|p| p.pid)
            .collect();
        crate::os::sessions::stop(&pids);
        // The installation ends at the end of its step: waited a little.
        for _ in 0..100 {
            if !setup.snapshot().running {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
        // An alignment or a save started before the flag was raised ends
        // before the drive is said to be free.
        if !closing.wait_idle(std::time::Duration::from_secs(60)) {
            crate::log::warn("eject: an agents' alignment still running after 60 s");
        }
        if let Err(e) = crate::agents::keyring::save_after_run(&kit) {
            crate::log::warn(&format!("eject, antigravity login: {e:#}"));
        }
        crate::log::info("eject: programs of the kit stopped");
        let _ = handle.emit_to(MAIN, "editor-status", editor.status());
        Ok(open_now(&handle))
    })
    .await
    .or_fail("eject")?
}

/// Closes the app after the user confirmed (close question, eject).
#[tauri::command]
pub fn app_quit<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    quit: State<'_, Quitting>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    quit.0.store(true, std::sync::atomic::Ordering::SeqCst);
    use tauri::Manager;
    if let Some(r) = app.try_state::<Arc<Runner>>() {
        r.stop_all();
    }
    app.state::<AppState>().servers.stop_all();
    app.state::<AppState>().llm.stop_all();
    // The databases shut down cleanly first (D-73), off this thread.
    let services = Arc::clone(&app.state::<AppState>().services);
    let handle = app.clone();
    std::thread::spawn(move || {
        crate::llm::pull::cancel_all(std::time::Duration::from_secs(10));
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
