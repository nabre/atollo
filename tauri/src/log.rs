//! The app log, `data\app\logs\app.log`: one line per event, secrets masked
//! (`env::mask_secrets`), rotated to `app.1.log` past 1 MB, at start and
//! during the session (BUG-V6-17).

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::env::mask_secrets;

const MAX_BYTES: u64 = 1024 * 1024;

static FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(logs: &Path) {
    let path = logs.join("app.log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES) {
        rotate(&path);
    }
    if let Ok(mut file) = FILE.lock() {
        *file = Some(path);
    }
}

/// `app.log` becomes `app.1.log` (the previous one is replaced).
fn rotate(path: &Path) {
    let _ = fs::rename(path, path.with_extension("1.log"));
}

/// Appends `line` to `path`, rotating it first when the line would take it
/// past `max` bytes.
fn append(path: &Path, line: &str, max: u64) {
    if fs::metadata(path).is_ok_and(|m| m.len() + line.len() as u64 > max) {
        rotate(path);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
}

fn write(level: &str, message: &str) {
    let Ok(guard) = FILE.lock() else { return };
    let Some(path) = guard.as_ref() else { return };
    let secs = crate::time::now_secs();
    let line = format!(
        "{secs} {level} {}\n",
        mask_secrets(message).replace('\n', " | ")
    );
    append(path, &line, MAX_BYTES);
}

pub fn info(message: &str) {
    write("INFO", message);
}
pub fn warn(message: &str) {
    write("WARN", message);
}
pub fn error(message: &str) {
    write("ERROR", message);
}

#[cfg(test)]
#[path = "../../test/rust/log.rs"]
mod tests;
