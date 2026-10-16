//! Files of the kit on disk: the few rules every module follows when it
//! reads or writes its own state (`data\app\*.json` and the like).
//!
//! - a text file may start with a BOM (Notepad, PowerShell 5) and may hold
//!   bytes that are not UTF-8: it is read lossy, without the BOM;
//! - a missing or empty JSON file is "nothing saved yet", never an error;
//! - every write goes through a temporary file renamed over the target, so
//!   a crash never leaves a half-written file; the folder is created first.
//!
//! Files of other programs edited as `serde_json::Value` with their own rules
//! (the editor's and Windows Terminal's settings, the agents' files) use
//! [`read_text`] and [`write_atomic`] directly.

use std::fs;
use std::io;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

/// `text` without the byte order mark some Windows editors put first.
pub fn strip_bom(text: &str) -> &str {
    text.trim_start_matches('\u{feff}')
}

/// The text of `path`, lossy UTF-8, without the BOM.
pub fn read_text(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(strip_bom(&String::from_utf8_lossy(&bytes)).to_owned())
}

/// Missing or empty file: `None`. Unreadable or damaged: an error that says
/// which file and why.
pub fn try_read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    let text = match read_text(path) {
        Ok(t) => t,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(anyhow!("{}: {e}", path.display())),
    };
    if text.trim().is_empty() {
        return Ok(None);
    }
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|e| anyhow!("{}: {e}", path.display()))
}

/// The value saved in `path`, `None` when missing or damaged: for state the
/// kit can rebuild (caches, remembered choices).
pub fn read_json<T: DeserializeOwned>(path: &Path) -> Option<T> {
    try_read_json(path).ok().flatten()
}

/// The value saved in `path` or the default, with the problem to show when
/// the file exists but cannot be used (the caller decides whether to write
/// it again: a damaged settings file is never overwritten).
pub fn load_json<T: DeserializeOwned + Default>(path: &Path) -> (T, Option<String>) {
    match try_read_json(path) {
        Ok(v) => (v.unwrap_or_default(), None),
        Err(e) => (T::default(), Some(e.to_string())),
    }
}

/// Writes through a temporary file in the same folder, then renames it over
/// the target, so a crash never leaves a half-written file.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    static TMP_SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let seq = TMP_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = path.with_file_name(format!("{name}.tmp-{}-{seq}", std::process::id()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path).inspect_err(|_| {
        let _ = fs::remove_file(&tmp);
    })
}

/// `value` as indented JSON in `path`, creating its folder, atomically.
pub fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| dir.display().to_string())?;
    }
    let text = serde_json::to_string_pretty(value)?;
    write_atomic(path, text.as_bytes()).with_context(|| path.display().to_string())
}

#[cfg(test)]
#[path = "../../test/rust/fsx.rs"]
mod tests;
