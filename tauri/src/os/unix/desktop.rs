//! The desktop around the app's windows on macOS and Linux
//! (`platform::desktop`): the system's "Select folder" and "Open" windows
//! (NSOpenPanel, GTK) through `rfd`; the tray is each system's
//! (`os/macos/desktop.rs`, `os/linux/desktop.rs`).

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

/// The window itself: the dialog is made on the main thread, over it.
pub struct Owner<R: Runtime>(Window<R>);

pub fn owner<R: Runtime>(window: &Window<R>) -> Result<Owner<R>> {
    Ok(Owner(window.clone()))
}

/// A folder, over `owner`, starting in `start`; `None` when closed without
/// a choice. Blocks: never on the main thread.
pub fn pick_folder<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    start: Option<&Path>,
) -> Result<Option<PathBuf>> {
    let picked = ask(&owner.0, title, start, AsyncFileDialog::pick_folder)?;
    Ok(picked.as_ref().map(path_of))
}

/// Several files, over `owner`, starting in the user's Downloads folder
/// with `downloads`; empty when closed without a choice. Blocks: never on
/// the main thread.
pub fn pick_files<R: Runtime>(
    owner: &Owner<R>,
    title: &str,
    downloads: bool,
) -> Result<Vec<PathBuf>> {
    let home = std::env::var_os("HOME").map(PathBuf::from);
    let start = downloads.then(|| downloads_dir(home.as_deref())).flatten();
    let picked = ask(
        &owner.0,
        title,
        start.as_deref(),
        AsyncFileDialog::pick_files,
    )?;
    Ok(picked.unwrap_or_default().iter().map(path_of).collect())
}

/// No message box: the error goes to stderr only.
pub fn error_box(_title: &str, _message: &str) {}

/// Where the "Open" window for a download starts: the user's Downloads
/// folder when there is one, else the home folder.
fn downloads_dir(home: Option<&Path>) -> Option<PathBuf> {
    let home = home?;
    let downloads = home.join("Downloads");
    Some(if downloads.is_dir() {
        downloads
    } else {
        home.to_path_buf()
    })
}
