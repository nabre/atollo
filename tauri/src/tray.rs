//! The icon of Atollo in the notification area (Windows), the menu bar
//! (macOS) and the StatusNotifier area (Linux), D-145: show the window,
//! open a project's page, quit. Quitting goes through the window's own
//! close, so the interface asks first when something is open (D-55).

use std::sync::{Mutex, PoisonError};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::commands::{AppState, MAIN};

const TRAY: &str = "atollo";
/// The event that opens a project's page in the interface (its id).
pub const OPEN_PROJECT: &str = "open-project";
/// The most recent projects in the menu; the others are a Ctrl+P away.
pub const MAX_PROJECTS: usize = 20;

const SHOW: &str = "tray:show";
const PROJECTS: &str = "tray:projects";
const NONE: &str = "tray:none";
const QUIT: &str = "tray:quit";
const PROJECT: &str = "tray:project:";

/// What an item of the menu asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Show,
    Open(String),
    Quit,
}

/// The action of a menu id; none for the disabled "no projects" item.
pub fn action(id: &str) -> Option<Action> {
    match id {
        SHOW => Some(Action::Show),
        QUIT => Some(Action::Quit),
        _ => id
            .strip_prefix(PROJECT)
            .filter(|p| !p.is_empty())
            .map(|p| Action::Open(p.to_owned())),
    }
}

/// The menu's texts and project entries, made without a window system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec {
    pub show: String,
    pub projects: String,
    /// Menu id and label, the most recent first; none: one disabled item.
    pub entries: Vec<(String, String)>,
    pub empty: String,
    pub quit: String,
}

/// The menu in `lang` for the projects of [`crate::projects::recent`].
pub fn spec(lang: &str, recent: &[String]) -> Spec {
    Spec {
        show: tr(lang, "Mostra Atollo"),
        projects: tr(lang, "Progetti"),
        entries: recent
            .iter()
            .take(MAX_PROJECTS)
            .map(|id| (format!("{PROJECT}{id}"), label(id)))
            .collect(),
        empty: tr(lang, "Nessun progetto"),
        quit: tr(lang, "Esci da Atollo"),
    }
}

/// A folder name as a menu label: `&` marks a mnemonic in the menus of
/// every platform (muda), doubled it is shown as it is.
pub fn label(name: &str) -> String {
    name.replace('&', "&&")
}

fn tr(lang: &str, source: &str) -> String {
    crate::i18n::t(lang, source, &[])
}

/// The menu shown now: rebuilt only when it changes.
#[derive(Default)]
struct Shown(Mutex<Option<Spec>>);

fn menu<R: Runtime>(app: &AppHandle<R>, spec: &Spec) -> tauri::Result<Menu<R>> {
    let show = MenuItem::with_id(app, SHOW, &spec.show, true, None::<&str>)?;
    let projects = Submenu::with_id(app, PROJECTS, &spec.projects, true)?;
    if spec.entries.is_empty() {
        projects.append(&MenuItem::with_id(
            app,
            NONE,
            &spec.empty,
            false,
            None::<&str>,
        )?)?;
    }
    for (id, label) in &spec.entries {
        projects.append(&MenuItem::with_id(
            app,
            id.as_str(),
            label,
            true,
            None::<&str>,
        )?)?;
    }
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, QUIT, &spec.quit, true, None::<&str>)?;
    Menu::with_items(app, &[&show, &projects, &separator, &quit])
}

fn current<R: Runtime>(app: &AppHandle<R>) -> Spec {
    let state = app.state::<AppState>();
    spec(state.lang(), &crate::projects::recent(&state.kit))
}

/// The icon with its menu. Without it the app works the same: a failure
/// (a Linux desktop without the AppIndicator library) only goes to the log.
pub fn create<R: Runtime>(app: &AppHandle<R>) {
    if !available() {
        crate::log::warn("tray: no AppIndicator library, no icon");
        return;
    }
    let spec = current(app);
    let menu = match menu(app, &spec) {
        Ok(menu) => menu,
        Err(e) => {
            crate::log::warn(&format!("tray: menu: {e}"));
            return;
        }
    };
    let mut builder = TrayIconBuilder::with_id(TRAY)
        .tooltip("Atollo")
        .menu(&menu)
        // The left button shows the window (Windows, macOS); the right
        // one the menu. Linux has the menu only.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| on_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| on_icon(tray.app_handle(), &event));
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    match builder.build(app) {
        Ok(_) => {
            app.manage(Shown(Mutex::new(Some(spec))));
        }
        Err(e) => crate::log::warn(&format!("tray: {e}")),
    }
}

/// The menu again, if the projects or the language changed: after every
/// read of the projects' list (the interface reads it after creating,
/// cloning, renaming or deleting one), after saving the settings and when
/// the pointer reaches the icon (Windows, macOS).
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let (Some(tray), Some(shown)) = (app.tray_by_id(TRAY), app.try_state::<Shown>()) else {
        return;
    };
    let spec = current(app);
    {
        // Not held while the menu is built: that waits for the main thread,
        // which may be refreshing too.
        let mut last = shown.0.lock().unwrap_or_else(PoisonError::into_inner);
        if last.as_ref() == Some(&spec) {
            return;
        }
        *last = Some(spec.clone());
    }
    if let Err(e) = menu(app, &spec).and_then(|m| tray.set_menu(Some(m))) {
        crate::log::warn(&format!("tray: menu: {e}"));
        *shown.0.lock().unwrap_or_else(PoisonError::into_inner) = None;
    }
}

/// The main window shown, restored and in front.
pub fn show_main<R: Runtime>(app: &AppHandle<R>) -> Option<WebviewWindow<R>> {
    let window = app.get_webview_window(MAIN)?;
    let _ = window.unminimize();
    let _ = window.show();
    let _ = window.set_focus();
    Some(window)
}

fn on_menu<R: Runtime>(app: &AppHandle<R>, id: &str) {
    match action(id) {
        Some(Action::Show) => {
            show_main(app);
        }
        Some(Action::Open(project)) => {
            if show_main(app).is_some() {
                let _ = app.emit_to(MAIN, OPEN_PROJECT, project);
            }
        }
        // The window's close, as its title bar's «Chiudi»: with something
        // open the interface asks (`close-requested`), else the app ends.
        Some(Action::Quit) => {
            if let Some(window) = show_main(app) {
                if let Err(e) = window.close() {
                    crate::log::warn(&format!("tray: quit: {e}"));
                }
            }
        }
        None => {}
    }
}

fn on_icon<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    match event {
        TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } => {
            show_main(app);
        }
        TrayIconEvent::Enter { .. } => {
            let app = app.clone();
            std::thread::spawn(move || refresh(&app));
        }
        _ => {}
    }
}

/// The icon of Linux needs libayatana-appindicator (or libappindicator),
/// loaded at run time: without it the icon would end the app.
#[cfg(all(unix, not(target_os = "macos")))]
fn available() -> bool {
    [
        c"libayatana-appindicator3.so.1",
        c"libappindicator3.so.1",
        c"libayatana-appindicator3.so",
        c"libappindicator3.so",
    ]
    .iter()
    .any(|name| {
        // SAFETY: a NUL-terminated name; the handle is closed at once (the
        // tray opens the library again).
        let handle = unsafe { libc::dlopen(name.as_ptr(), libc::RTLD_LAZY) };
        if handle.is_null() {
            return false;
        }
        // SAFETY: a handle dlopen just returned.
        unsafe { libc::dlclose(handle) };
        true
    })
}

#[cfg(not(all(unix, not(target_os = "macos"))))]
fn available() -> bool {
    true
}

#[cfg(test)]
#[path = "../../test/rust/tray.rs"]
mod tests;
