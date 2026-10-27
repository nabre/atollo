//! Trusted folders of Claude Code (SEC-V6-02). Atollo trusts no project
//! now: no agent starts on the PC since D-178, and the agents in the VM
//! have their own settings.
//!
//! Earlier versions trusted `projects\` and each folder in it at every
//! sync. Codex's block of those is dropped by `codex::sync_config`; in
//! Claude's file only the entry of `projects\` itself is taken back, once
//! (a parent's trust covers the folders in it); the entries of single
//! projects cannot be told from the user's and stay.

use std::fs;
use std::path::Path;

use anyhow::Result;
use serde_json::Value;

use super::{read_json, write_json, Paths};
use crate::common::fsx::write_atomic;

/// The key of `dir` in Claude's `projects`: `/` separators, the drive letter
/// in both cases (the kit's windows write "D:/...", VSCodium "d:/...").
fn claude_keys(dir: &Path) -> Vec<String> {
    let key = dir.display().to_string().replace('\\', "/");
    let mut chars = key.chars();
    let first = chars.next().unwrap_or_default();
    let rest: String = chars.collect();
    let mut keys = vec![
        format!("{}{rest}", first.to_ascii_uppercase()),
        format!("{}{rest}", first.to_ascii_lowercase()),
    ];
    keys.dedup();
    keys
}
/// Once: takes back the trust earlier versions gave Claude for the whole
/// `projects\` folder. Done when the mark in `.sync` exists.
pub(super) fn forget_pretrust(p: &Paths) -> Result<()> {
    let mark = p.state.join("trust-on-launch");
    if mark.exists() {
        return Ok(());
    }
    let file = p.claude_json();
    if let Some(mut cfg) = read_json(&file)? {
        let mut changed = false;
        if let Some(map) = cfg.get_mut("projects").and_then(Value::as_object_mut) {
            for k in claude_keys(&p.projects) {
                if let Some(e) = map.get_mut(&k).and_then(Value::as_object_mut) {
                    changed |= e.remove("hasTrustDialogAccepted").is_some();
                }
            }
        }
        if changed {
            write_json(&file, &cfg)?;
        }
    }
    fs::create_dir_all(&p.state)?;
    write_atomic(
        &mark,
        b"projects are trusted when an agent is launched in them\n",
    )?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/trust.rs"]
mod tests;
