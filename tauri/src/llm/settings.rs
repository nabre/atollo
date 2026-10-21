//! The server's settings, `data/app/llm.json`: its port (chosen once, kept
//! for the agents that point at it), whether it starts with the app, and
//! the model Pi uses. Its key, `data/app/secrets/llama-key`: made at the
//! first start, owner-only, given to the server as a file
//! (`--api-key-file`), never on its command line.

use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::kit::Kit;

/// The ports the server can have: llama.cpp's own (9931) and the next
/// ones, outside those of the pools, the projects and the editor.
pub const PORTS: RangeInclusive<u16> = 9931..=9999;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmSettings {
    #[serde(default)]
    pub port: Option<u16>,
    /// Started with the app.
    #[serde(default)]
    pub autostart: bool,
    /// The model Pi starts with (its id), when the user chose one.
    #[serde(default)]
    pub pi_model: Option<String>,
}

pub fn path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("llm.json")
}

/// The settings; the defaults without a file. A file that cannot be read is
/// never overwritten ([`update`] fails).
pub fn load(kit: &Kit) -> LlmSettings {
    crate::fsx::read_json(&path(kit)).unwrap_or_default()
}

/// One read-change-write at a time.
static LOCK: Mutex<()> = Mutex::new(());

pub fn update<T>(kit: &Kit, change: impl FnOnce(&mut LlmSettings) -> Result<T>) -> Result<T> {
    let _turn = LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut settings: LlmSettings = crate::fsx::try_read_json(&path(kit))?.unwrap_or_default();
    let out = change(&mut settings)?;
    crate::fsx::write_json(&path(kit), &settings)?;
    Ok(out)
}

/// The server's key, owner-only.
pub fn key_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("secrets").join("llama-key")
}

/// The key, made the first time.
pub fn ensure_key(kit: &Kit) -> Result<String> {
    if let Some(key) = crate::settings::read_secret(&key_file(kit))? {
        return Ok(key);
    }
    let key = crate::editor::random_token()?;
    crate::router::fs::write_private(&key_file(kit), &key)?;
    Ok(key)
}

/// The key, if it was made.
pub fn key(kit: &Kit) -> Option<String> {
    crate::settings::read_secret(&key_file(kit)).ok().flatten()
}

#[cfg(test)]
#[path = "../../../test/rust/llm/settings.rs"]
mod tests;
