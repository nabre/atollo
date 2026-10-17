//! The system's "Select folder" and "Open" windows on macOS and Linux
//! (NSOpenPanel, GTK or the XDG portal), through `tauri-plugin-dialog`,
//! called from Rust only: the interface gets no dialog permission. Windows
//! keeps IFileOpenDialog (`terminals::pick_dir`, `setup::downloads::pick`).

use std::path::{Path, PathBuf};

use tauri::{Runtime, Window};
use tauri_plugin_dialog::DialogExt;

/// A folder, over `window`, starting in `start`; `None` when closed without
/// a choice. Blocks: never on the main thread.
pub fn folder<R: Runtime>(
    window: &Window<R>,
    title: &str,
    start: Option<&Path>,
) -> Option<PathBuf> {
    let mut dialog = window.dialog().file().set_title(title).set_parent(window);
    if let Some(start) = start.filter(|s| s.is_dir()) {
        dialog = dialog.set_directory(start);
    }
    dialog.blocking_pick_folder()?.into_path().ok()
}

/// Several files, over `window`, starting in `start`; empty when closed
/// without a choice. Blocks: never on the main thread.
pub fn files<R: Runtime>(window: &Window<R>, title: &str, start: Option<&Path>) -> Vec<PathBuf> {
    let mut dialog = window.dialog().file().set_title(title).set_parent(window);
    if let Some(start) = start.filter(|s| s.is_dir()) {
        dialog = dialog.set_directory(start);
    }
    dialog
        .blocking_pick_files()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|f| f.into_path().ok())
        .collect()
}

/// Where the "Open" window for a download starts: the user's Downloads
/// folder when there is one, else the home folder.
pub fn downloads_dir(home: Option<&Path>) -> Option<PathBuf> {
    let home = home?;
    let downloads = home.join("Downloads");
    Some(if downloads.is_dir() {
        downloads
    } else {
        home.to_path_buf()
    })
}
