//! The models Atollo offers (`catalog/models.json`, embedded): a few GGUF
//! files of Hugging Face at a fixed commit, each with its SHA-256 and size,
//! chosen for the agents (they call tools) from 8 GB of memory up.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

/// One file of a model, as Hugging Face has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    /// Its path in the repository (the file name in the model's folder).
    pub name: String,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    /// `owner/repo` on Hugging Face.
    pub repo: String,
    pub commit: String,
    /// The first part first when the model is in several.
    pub files: Vec<ModelFile>,
    pub quant: String,
    /// The context window the server gives it.
    pub ctx: u32,
    /// The memory it needs with that context, about.
    pub ram_gb: u32,
    /// It calls tools: an agent can use it.
    pub tools: bool,
    pub license: String,
    /// Italian; translated by the catalogs.
    pub desc: String,
}

impl CatalogModel {
    pub fn bytes(&self) -> u64 {
        self.files.iter().map(|f| f.bytes).sum()
    }
}

#[derive(Deserialize)]
struct File {
    models: Vec<CatalogModel>,
}

/// The catalog's models, in its order.
pub fn all() -> &'static [CatalogModel] {
    static MODELS: OnceLock<Vec<CatalogModel>> = OnceLock::new();
    MODELS.get_or_init(|| {
        serde_json::from_str::<File>(include_str!("../../catalog/models.json"))
            .map(|f| f.models)
            .unwrap_or_default()
    })
}

pub fn find(id: &str) -> Option<&'static CatalogModel> {
    all().iter().find(|m| m.id == id)
}

#[cfg(test)]
#[path = "../../../test/rust/llm/catalog.rs"]
mod tests;
