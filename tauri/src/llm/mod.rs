//! Local language models (D-151): the GGUF files of `data/models`, shared
//! by every target of the kit (one copy on the drive for Windows, macOS
//! and Linux), and the llama.cpp server that answers for them.
//!
//! - `catalog`: the models Atollo offers (`catalog/models.json`), each at a
//!   fixed commit of Hugging Face with the SHA-256 of its files;
//! - `hf`: any other model of Hugging Face (`owner/repo`), its GGUF files
//!   grouped by quantization;
//! - `local`: what is on disk, one folder per repository
//!   (`data/models/gguf/<owner>--<repo>`) with its `atollo.json`;
//! - `pull`: the downloads, resumed after an interruption (`net::resume`),
//!   checked against the drive (FAT32, free space) before they start.
//!
//! Secrets in `data/app/secrets`, owner-only like the Gemini key (D-137):
//! the Hugging Face token for the gated models (`hf-token`).

pub mod catalog;
pub mod cli;
pub mod hf;
pub mod local;
pub mod pull;

use std::path::PathBuf;

use crate::kit::Kit;

/// `data/models/gguf`: one folder per repository of Hugging Face.
pub fn models_dir(kit: &Kit) -> PathBuf {
    kit.data().join("models").join("gguf")
}

/// The Hugging Face token (gated models), owner-only.
pub fn hf_token_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("secrets").join("hf-token")
}

/// The Hugging Face token, if the user gave one.
pub fn hf_token(kit: &Kit) -> anyhow::Result<Option<String>> {
    crate::settings::read_secret(&hf_token_file(kit))
}

/// A model id the kit makes from a file name: lowercase letters, digits
/// and `.`, `_`, `-` (the name of a section of the server's presets and of
/// a model for the agents).
pub fn slug(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        let c = c.to_ascii_lowercase();
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            out.push(c);
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches(|c| c == '-' || c == '.').to_owned()
}

/// A model id: what [`slug`] gives, not empty.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && slug(id) == id
}

#[cfg(test)]
#[path = "../../../test/rust/llm/mod.rs"]
mod tests;
