//! Trusted folders of Claude Code and Codex (SEC-V6-02). A project is
//! trusted when the user launches an agent in it from Atollo: that launch
//! is the answer to the question the agent would ask. Nothing is trusted
//! before: neither `projects\` as a whole nor a project just cloned, whose
//! hooks and MCP servers would otherwise load unasked.
//!
//! The trust is written as the agents write it when the user accepts:
//! `projects["D:/kit/projects/x"].hasTrustDialogAccepted` in Claude's
//! `.claude.json`, a `[projects.'d:\kit\projects\x']` table with
//! `trust_level = "trusted"` in Codex's `config.toml`, outside the kit's
//! blocks. What is already there for that folder is left as it is (also a
//! Codex table the user set to "untrusted").
//!
//! Earlier versions trusted `projects\` and each folder in it at every
//! sync. Codex's block of those is dropped by `codex::sync_config`; in
//! Claude's file only the entry of `projects\` itself is taken back, once
//! (a parent's trust covers the folders in it); the entries of single
//! projects cannot be told from the user's and stay.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use super::{read_json, write_json, Paths};
use crate::fsx::write_atomic;

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

fn project_entry<'a>(cfg: &'a Value, key: &str) -> Option<&'a Value> {
    cfg.get("projects").and_then(|p| p.get(key))
}

/// Claude: `hasTrustDialogAccepted` for `dir`. Re-read just before writing.
pub(super) fn claude(p: &Paths, dir: &Path) -> Result<()> {
    let file = p.claude_json();
    let current = read_json(&file)?;
    let todo: Vec<String> = claude_keys(dir)
        .into_iter()
        .filter(|k| {
            current
                .as_ref()
                .and_then(|c| project_entry(c, k))
                .and_then(|v| v.get("hasTrustDialogAccepted"))
                != Some(&Value::Bool(true))
        })
        .collect();
    if todo.is_empty() {
        return Ok(());
    }
    let mut fresh = read_json(&file)?.unwrap_or_else(|| Value::Object(Default::default()));
    let obj = fresh
        .as_object_mut()
        .context(".claude.json is not an object")?;
    let projects = obj
        .entry("projects")
        .or_insert_with(|| Value::Object(Default::default()));
    if !projects.is_object() {
        *projects = Value::Object(Default::default());
    }
    if let Some(map) = projects.as_object_mut() {
        for k in todo {
            let entry = map
                .entry(k)
                .or_insert_with(|| Value::Object(Default::default()));
            if !entry.is_object() {
                *entry = Value::Object(Default::default());
            }
            if let Some(e) = entry.as_object_mut() {
                e.insert("hasTrustDialogAccepted".into(), Value::Bool(true));
            }
        }
    }
    write_json(&file, &fresh)
}

/// Codex's key of `dir`: lower case on Windows, where paths ignore case;
/// elsewhere the path as it is (PORT-V6-02).
fn codex_key(p: &Paths, dir: &Path) -> String {
    let path = dir.display().to_string();
    if p.os.is_windows() {
        path.to_lowercase()
    } else {
        path
    }
}

/// Codex: a `[projects.<dir>]` table, unless one is there already.
pub(super) fn codex(p: &Paths, dir: &Path) -> Result<()> {
    let file = p.codex.join("config.toml");
    let old = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let path = codex_key(p, dir);
    let literal = format!("'{path}'");
    let basic = Value::String(path.clone()).to_string();
    let fold = |s: &str| {
        if p.os.is_windows() {
            s.to_lowercase()
        } else {
            s.to_owned()
        }
    };
    let text = fold(&old);
    let has = |key: &str| {
        text.lines().any(|l| {
            l.trim()
                .strip_prefix("[projects.")
                .and_then(|r| r.strip_suffix(']'))
                .is_some_and(|k| k.trim() == fold(key))
        })
    };
    if has(&literal) || has(&basic) {
        return Ok(());
    }
    let key = if path.contains('\'') { basic } else { literal };
    let mut out = old.clone();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("\n[projects.{key}]\ntrust_level = \"trusted\"\n"));
    fs::create_dir_all(&p.codex)?;
    write_atomic(&file, out.as_bytes()).with_context(|| file.display().to_string())
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
