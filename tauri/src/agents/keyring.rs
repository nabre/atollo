//! Antigravity's Google login an earlier Atollo saved in the kit
//! (`data/antigravity/keyring.json`, D-115): the kit's launcher of `agy`
//! moved it into the system's credential store around every run on the PC
//! and back.
//!
//! No agent runs on the PC since D-179 (LEVELS.md §7): nothing moves a
//! login any more, and `agy` signs in inside the project's VM (§9). The
//! file is kit data: read for the usage of the agents ([`saved_secrets`])
//! and told on the agents' page ([`has_login`]), removed by «Esci». A
//! login an agy run of an earlier Atollo left in the system's store
//! without ending cleanly stays there (declared, LEVELS.md §9).

use serde::Deserialize;

use crate::kit::Kit;

/// One credential of the file. `blob`: base64 of the secret.
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    #[serde(default)]
    blob: Option<String>,
}

/// The credentials of `path` (one object or a list), none when it is
/// missing or no JSON.
fn read(path: &std::path::Path) -> Vec<Item> {
    crate::common::fsx::read_text(path)
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(t.trim()).ok())
        .map(|v| {
            if v.is_array() {
                v
            } else {
                serde_json::Value::Array(vec![v])
            }
        })
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// The secrets (base64) of the login saved in `dir` (`data\antigravity`),
/// read by `stats` for the quotas.
pub fn saved_secrets(dir: &std::path::Path) -> Vec<String> {
    read(&dir.join("keyring.json"))
        .into_iter()
        .filter_map(|i| i.blob)
        .collect()
}

/// A Google login saved in the kit.
pub fn has_login(kit: &Kit) -> bool {
    !read(&kit.data().join("antigravity").join("keyring.json")).is_empty()
}

#[cfg(test)]
#[path = "../../../test/rust/agents/keyring.rs"]
mod tests;
