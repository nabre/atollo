//! The app log, `data\app\logs\app.log`: one line per event, secrets masked
//! (`env::mask_secrets`), rotated to `app.1.log` past 1 MB.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::env::mask_secrets;

const MAX_BYTES: u64 = 1024 * 1024;

static FILE: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn init(logs: &Path) {
    let path = logs.join("app.log");
    if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_BYTES) {
        let _ = fs::rename(&path, logs.join("app.1.log"));
    }
    if let Ok(mut file) = FILE.lock() {
        *file = Some(path);
    }
}

fn write(level: &str, message: &str) {
    let Ok(guard) = FILE.lock() else { return };
    let Some(path) = guard.as_ref() else { return };
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!(
        "{secs} {level} {}\n",
        mask_secrets(message).replace('\n', " | ")
    );
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(line.as_bytes());
    }
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
