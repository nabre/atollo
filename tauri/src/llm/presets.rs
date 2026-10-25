//! The server's presets, `data/app/llm/presets.ini`, written at every start
//! from the models on disk: one section per complete model, named with its
//! id (what the agents ask for), its file (the first part of a model in
//! several), its projector and its context. Written again at every start
//! because the paths are absolute: the kit may have moved.
//!
//! The server runs in router mode on these presets only: it loads a model
//! when it is asked for, one at a time (`--models-max 1`).

use std::path::{Path, PathBuf};

use anyhow::Result;

use super::local::LocalModel;
use crate::kit::Kit;

/// The context of a model the catalog does not describe: enough for an
/// agent, far below the native one of many models (which would not fit).
pub const DEFAULT_CTX: u32 = 16384;

pub fn path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("llm").join("presets.ini")
}

/// The text of the presets of `models` in `root` (`data/models/gguf`).
pub fn text(root: &Path, models: &[LocalModel]) -> String {
    let mut out = String::from("; Atollo: written at every start of the server (llm/presets.rs)\n");
    for m in models.iter().filter(|m| m.complete && !m.files.is_empty()) {
        let dir = root.join(&m.folder);
        let file = |name: &str| dir.join(name).display().to_string();
        out.push_str(&format!("\n[{}]\nmodel = {}\n", m.id, file(&m.files[0])));
        if let Some(mmproj) = &m.mmproj {
            out.push_str(&format!("mmproj = {}\n", file(mmproj)));
        }
        out.push_str(&format!("ctx-size = {}\n", m.ctx.unwrap_or(DEFAULT_CTX)));
    }
    out
}

/// Writes the presets of the models on disk; their ids.
pub fn write(kit: &Kit) -> Result<Vec<String>> {
    let models = super::local::list(kit);
    let text = text(&super::models_dir(kit), &models);
    let file = path(kit);
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    crate::common::fsx::write_atomic(&file, text.as_bytes())?;
    Ok(models
        .into_iter()
        .filter(|m| m.complete)
        .map(|m| m.id)
        .collect())
}

#[cfg(test)]
#[path = "../../../test/rust/llm/presets.rs"]
mod tests;
