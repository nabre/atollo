//! The desktop around the app's windows: the system's "Select folder" and
//! "Open" windows, the message box of an error before any window, and
//! whether the system can show the app's icon in its tray.
//!
//! The dialogs are made from Rust only: the interface gets no dialog
//! permission, and no webview gets a script of a dialog plugin
//! (tauri-plugin-dialog would replace `window.alert` and `window.confirm`
//! in every one, the editor's too). Windows shows IFileOpenDialog
//! (`os/windows/desktop.rs`); macOS and Linux NSOpenPanel and GTK through
//! `rfd` (`os/unix/desktop.rs`).

use std::path::{Path, PathBuf};

use anyhow::Result;
use tauri::{Runtime, Window};

use crate::os::current::desktop as imp;

/// The window a dialog shows over, taken on the command's thread, for the
/// blocking task that shows the dialog.
pub struct Owner<R: Runtime>(imp::Owner<R>);

/// The owner of a dialog over `window`.
pub fn owner<R: Runtime>(window: &Window<R>) -> Result<Owner<R>> {
    imp::owner(window).map(Owner)
}

/// A folder, over `owner`, starting in `start`; `None` when closed without
/// a choice. Blocks: never on the main thread.
pub fn pick_folder<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    start: Option<&Path>,
) -> Result<Option<PathBuf>> {
    imp::pick_folder(&owner.0, title, start)
}

/// Several files, over `owner`; empty when closed without a choice. With
/// `downloads` the window starts in the user's Downloads folder (Windows
/// always offers it first). Blocks: never on the main thread.
pub fn pick_files<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    downloads: bool,
) -> Result<Vec<PathBuf>> {
    imp::pick_files(&owner.0, title, downloads)
}

/// The error that stops the app before (or instead of) the window: a
/// message box on Windows, where an app started from the Explorer has no
/// console; nothing elsewhere (the caller writes it to stderr too).
pub fn error_box(title: &str, message: &str) {
    imp::error_box(title, message)
}

/// The system can show the app's icon in its tray: on Linux only with
/// libayatana-appindicator (or libappindicator), loaded at run time, or the
/// icon would end the app.
pub fn tray_available() -> bool {
    imp::tray_available()
}
