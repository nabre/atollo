//! The abstract layer between the engine and the operating system: what
//! the app needs of the system, the same API on Windows, macOS and Linux.
//! The rest of the engine calls only this; each function hands over to the
//! module of the same name of `crate::os::current` (`os/windows/`,
//! `os/macos/`, `os/linux/`, with what macOS and Linux share in
//! `os/unix/`), which every system implements with the same signature: the
//! build of each system checks it. The logic that is the same everywhere
//! (which processes are the kit's, the notice for a disk, the folders in
//! transit) stays here; nothing here looks at which system it is.
//!
//! - `jobs`: the programs the kit starts, none outliving the app (D-07);
//! - `processes`, `sessions`: the PC's processes, the kit's programs open,
//!   other kits, "Rimuovi l'unità";
//! - `console`, `desktop`: the command line's output; the dialogs, the
//!   message box, the tray;
//! - `files`: owner-only files and folders, the executable bit;
//! - `host`: what the app knows of the machine (processor, language...);
//! - `instance`: one app per kit (D-08);
//! - `lock`: the kit's locks across its processes;
//! - `volume`: the disk of a folder (D-90, D-105);
//! - `temp`: the folders in transit (D-90);
//! - `safe_path`: reading inside a project without following links (D-77);
//! - `memory`: the PC's RAM;
//! - `random`: random bytes of the system, for tokens;
//! - `signature`: Authenticode signatures (Windows);
//! - `pty`: a program on a pseudo terminal, for the app's terminal;
//! - `webview`: the web view of the windows (D-05).

pub mod console;
pub mod desktop;
pub mod files;
pub mod host;
pub mod instance;
pub mod jobs;
pub mod lock;
pub mod memory;
pub mod processes;
pub mod pty;
pub mod random;
pub mod safe_path;
pub mod sessions;
pub mod signature;
pub mod temp;
pub mod volume;
pub mod webview;

use crate::kit::Os;
use crate::os::current as sys;

/// The system the app is built for.
pub const OS: Os = sys::OS;

/// `text` as one word for a POSIX shell, for the commands the user is told
/// to paste in a Terminal: always in single quotes, where nothing expands
/// (`$(...)`, backquotes), each `'` written as `'\''`. A volume label or a
/// bundle name is chosen by whoever made the disk.
pub fn sh_quote(text: &str) -> String {
    format!("'{}'", text.replace('\'', r"'\''"))
}
