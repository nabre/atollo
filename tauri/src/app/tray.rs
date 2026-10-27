//! The icon of Atollo in the notification area (Windows), the menu bar
//! (macOS) and the StatusNotifier area (Linux), D-145: show the window,
//! open a project's page, quit. Quitting goes through the window's own
//! close, so the interface asks first when something is open (D-55).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::ipc::{events, AppState, MAIN};

const TRAY: &str = "atollo";
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
        quit: tr(lang, "Chiudi Atollo"),
    }
}

/// A folder name as a menu label: `&` marks a mnemonic in the menus of
/// every platform (muda), doubled it is shown as it is.
pub fn label(name: &str) -> String {
    name.replace('&', "&&")
}

fn tr(lang: &str, source: &str) -> String {
    crate::common::i18n::t(lang, source, &[])
}

/// The menu of the icon, changed in place: replacing it while it is open
/// (the right button pressed refreshes it) would close it on Windows.
struct Shown<R: Runtime> {
    spec: Mutex<Spec>,
    show: MenuItem<R>,
    projects: Submenu<R>,
    quit: MenuItem<R>,
    /// A [`refresh`] reading the projects now, off the main thread.
    reading: AtomicBool,
    /// A [`refresh`] asked while one reads: that one reads once more.
    again: AtomicBool,
}

impl<R: Runtime> Shown<R> {
    fn new(app: &AppHandle<R>, spec: Spec) -> tauri::Result<(Shown<R>, Menu<R>)> {
        let show = MenuItem::with_id(app, SHOW, &spec.show, true, None::<&str>)?;
        let projects = Submenu::with_id(app, PROJECTS, &spec.projects, true)?;
        fill(app, &projects, &spec)?;
        let separator = PredefinedMenuItem::separator(app)?;
        let quit = MenuItem::with_id(app, QUIT, &spec.quit, true, None::<&str>)?;
        let menu = Menu::with_items(app, &[&show, &projects, &separator, &quit])?;
        let shown = Shown {
            spec: Mutex::new(spec),
            show,
            projects,
            quit,
            reading: AtomicBool::new(false),
            again: AtomicBool::new(false),
        };
        Ok((shown, menu))
    }

    /// From what it shows to `next`.
    fn update(&self, app: &AppHandle<R>, before: &Spec, next: &Spec) -> tauri::Result<()> {
        if before.show != next.show {
            self.show.set_text(&next.show)?;
        }
        if before.projects != next.projects {
            self.projects.set_text(&next.projects)?;
        }
        if before.quit != next.quit {
            self.quit.set_text(&next.quit)?;
        }
        if before.entries != next.entries || before.empty != next.empty {
            for item in self.projects.items()? {
                self.projects.remove(&item)?;
            }
            fill(app, &self.projects, next)?;
        }
        Ok(())
    }
}

/// The projects' items, or the disabled "no projects".
fn fill<R: Runtime>(app: &AppHandle<R>, projects: &Submenu<R>, spec: &Spec) -> tauri::Result<()> {
    if spec.entries.is_empty() {
        let none = MenuItem::with_id(app, NONE, &spec.empty, false, None::<&str>)?;
        projects.append(&none)?;
    }
    for (id, label) in &spec.entries {
        let item = MenuItem::with_id(app, id.as_str(), label, true, None::<&str>)?;
        projects.append(&item)?;
    }
    Ok(())
}

fn current<R: Runtime>(app: &AppHandle<R>) -> Spec {
    let state = app.state::<AppState>();
    spec(state.lang(), &crate::projects::recent(&state.kit))
}

/// The icon with its menu. Without it the app works the same: a failure
/// (a Linux desktop without the AppIndicator library) only goes to the log.
pub fn create<R: Runtime>(app: &AppHandle<R>) {
    if !crate::platform::desktop::tray_available() {
        crate::common::log::warn("tray: no AppIndicator library, no icon");
        return;
    }
    let (shown, menu) = match Shown::new(app, current(app)) {
        Ok(made) => made,
        Err(e) => {
            crate::common::log::warn(&format!("tray: menu: {e}"));
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
            app.manage(shown);
        }
        Err(e) => crate::common::log::warn(&format!("tray: {e}")),
    }
}

/// The menu again, if the projects or the language changed: after every
/// read of the projects' list (the interface reads it after creating,
/// cloning, renaming or deleting one), after saving the settings, when the
/// pointer reaches the icon and when the right button is pressed on it
/// (Windows, macOS).
///
/// The projects (a folder's listing and one settings file each) are read
/// off the main thread, only the menu is changed on it: the window does
/// not wait for the disk. One reading at a time; the refreshes asked
/// meanwhile make one more, never one each.
pub fn refresh<R: Runtime>(app: &AppHandle<R>) {
    let Some(shown) = app.try_state::<Shown<R>>() else {
        return;
    };
    if !start_reading(&shown.reading, &shown.again) {
        return;
    }
    let handle = app.clone();
    std::thread::spawn(move || loop {
        let Some(shown) = handle.try_state::<Shown<R>>() else {
            return;
        };
        shown.again.store(false, Ordering::SeqCst);
        let next = current(&handle);
        // One change at a time, on the thread that owns the menu.
        let menu = handle.clone();
        if let Err(e) = handle.run_on_main_thread(move || apply(&menu, next)) {
            crate::common::log::warn(&format!("tray: menu: {e}"));
        }
        if !read_again(&shown.reading, &shown.again) {
            return;
        }
    });
}

/// A refresh may read now (`reading` taken), or one reads already and
/// will read again for it (`again` set).
fn start_reading(reading: &AtomicBool, again: &AtomicBool) -> bool {
    if reading.swap(true, Ordering::SeqCst) {
        again.store(true, Ordering::SeqCst);
        return false;
    }
    true
}

/// A reading ended: true when it reads once more for a refresh asked
/// meanwhile (`reading` kept). A refresh asked right after `reading` is
/// let go starts its own reading instead.
fn read_again(reading: &AtomicBool, again: &AtomicBool) -> bool {
    reading.store(false, Ordering::SeqCst);
    again.load(Ordering::SeqCst) && !reading.swap(true, Ordering::SeqCst)
}

/// [`refresh`] on the main thread, read there too: the right button
/// pressed brings the menu up to date before it opens.
fn refresh_here<R: Runtime>(app: &AppHandle<R>) {
    apply(app, current(app));
}

/// The menu changed to `next` (on the main thread).
fn apply<R: Runtime>(app: &AppHandle<R>, next: Spec) {
    let Some(shown) = app.try_state::<Shown<R>>() else {
        return;
    };
    let mut spec = shown.spec.lock().unwrap_or_else(PoisonError::into_inner);
    if *spec == next {
        return;
    }
    if let Err(e) = shown.update(app, &spec, &next) {
        crate::common::log::warn(&format!("tray: menu: {e}"));
    }
    *spec = next;
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
                let _ = app.emit_to(MAIN, events::OPEN_PROJECT, project);
            }
        }
        // The window's close, as its title bar's «Chiudi»: with something
        // open the interface asks (`close-requested`), else the app ends.
        Some(Action::Quit) => {
            if let Some(window) = show_main(app) {
                if let Err(e) = window.close() {
                    crate::common::log::warn(&format!("tray: quit: {e}"));
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
        // On Windows the menu opens when the right button goes up: pressed,
        // the menu is brought up to date first (a few small files).
        TrayIconEvent::Click {
            button: MouseButton::Right,
            button_state: MouseButtonState::Down,
            ..
        } => refresh_here(app),
        // The pointer passing over the icon: read off the main thread,
        // usually done before the button is pressed.
        TrayIconEvent::Enter { .. } => refresh(app),
        _ => {}
    }
}

#[cfg(test)]
#[path = "../../../test/rust/app/tray.rs"]
mod tests;
