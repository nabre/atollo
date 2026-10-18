//! The system's "Select folder" and "Open" windows on macOS and Linux
//! (NSOpenPanel, GTK), through `rfd`, called from Rust only: the interface
//! gets no dialog permission, and no webview gets a script of a dialog
//! plugin (tauri-plugin-dialog would replace `window.alert` and
//! `window.confirm` in every one, the editor's too). Windows keeps
//! IFileOpenDialog (`terminals::pick_dir`, `setup::downloads::pick`).

use std::future::Future;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rfd::{AsyncFileDialog, FileHandle};
use tauri::{Runtime, Window};

/// Shows a dialog over `window`, made on the main thread (as macOS and GTK
/// want), and waits on this thread for what `pick` returns.
fn ask<R, T, F>(
    window: &Window<R>,
    title: &str,
    start: Option<&Path>,
    pick: impl FnOnce(AsyncFileDialog) -> F + Send + 'static,
) -> Result<T>
where
    R: Runtime,
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let parent = window.clone();
    let title = title.to_owned();
    let start = start.filter(|s| s.is_dir()).map(Path::to_path_buf);
    window
        .run_on_main_thread(move || {
            let mut dialog = AsyncFileDialog::new().set_title(title).set_parent(&parent);
            if let Some(start) = start {
                dialog = dialog.set_directory(start);
            }
            let picked = pick(dialog);
            std::thread::spawn(move || {
                let _ = tx.send(tauri::async_runtime::block_on(picked));
            });
        })
        .context("system dialog")?;
    rx.recv().context("system dialog closed without an answer")
}

fn path_of(file: &FileHandle) -> PathBuf {
    file.path().to_path_buf()
}

/// A folder, over `window`, starting in `start`; `None` when closed without
/// a choice. Blocks: never on the main thread.
pub fn folder<R: Runtime>(
    window: &Window<R>,
    title: &str,
    start: Option<&Path>,
) -> Result<Option<PathBuf>> {
    let picked = ask(window, title, start, AsyncFileDialog::pick_folder)?;
    Ok(picked.as_ref().map(path_of))
}

/// Several files, over `window`, starting in `start`; empty when closed
/// without a choice. Blocks: never on the main thread.
pub fn files<R: Runtime>(
    window: &Window<R>,
    title: &str,
    start: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    let picked = ask(window, title, start, AsyncFileDialog::pick_files)?;
    Ok(picked.unwrap_or_default().iter().map(path_of).collect())
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
