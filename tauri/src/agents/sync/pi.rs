//! Pi (D-152): what the kit keeps in its folder, `data\home\.pi\agent`
//! (`PI_CODING_AGENT_DIR`), next to the shared instructions and skills:
//!
//! - `APPEND_SYSTEM.md`: the kit rule in a marked block, the user's own
//!   text around it kept;
//! - `models.json`: the provider `atollo`, the server of the local models
//!   (`llm/`) with every model on disk; the user's other providers kept.
//!   Pi asks the router for a model and the router loads it; the key is in
//!   the file (owner-only), so `pi` works from any terminal of the kit;
//! - `settings.json`: on Windows the kit's Git Bash as `shellPath` (Pi
//!   finds no Git in Program Files), `defaultProjectTrust: "ask"` unless
//!   the user set it, and the local model as the default while the user
//!   has no other provider chosen;
//! - `mcp.json`: the kit's MCP servers (stdio and HTTP; SSE is refused by
//!   Pi and left out with a warning), the user's own kept;
//! - `trust.json`: a project the user opens Pi in from Atollo is trusted
//!   (SEC-V6-02), as for Claude Code and Codex.
//!
//! Every key the kit writes is written again at each sync: the absolute
//! path of `shellPath` follows the kit when it moves.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

use super::mcp::{server_type, Servers};
use super::{canonical, read_json, read_state, write_json, Paths, Report};
use crate::kit::Kit;

/// The provider of the local models in Pi's `models.json`.
pub const PROVIDER: &str = "atollo";

/// `<home>\.pi\agent`, Pi's folder in the kit.
pub fn dir(home: &Path) -> PathBuf {
    home.join(".pi").join("agent")
}

/// The local model Pi uses: the one chosen for it when it is on disk;
/// without a choice, while Pi has no provider of the cloud, the first
/// complete model (the catalog's order, then the others).
pub fn local_model(kit: &Kit) -> Option<String> {
    let models: Vec<_> = crate::llm::local::list(kit)
        .into_iter()
        .filter(|m| m.complete)
        .collect();
    if let Some(id) = crate::llm::settings::load(kit).pi_model {
        return models.iter().any(|m| m.id == id).then_some(id);
    }
    let auth = crate::common::fsx::read_json::<Value>(&dir(&kit.home()).join("auth.json"));
    if auth
        .as_ref()
        .and_then(Value::as_object)
        .is_some_and(|o| !o.is_empty())
    {
        return None;
    }
    // The catalog's models that call tools first, in its order; then the
    // others; last the catalog's models for chat only.
    let order = |id: &str| {
        let catalog = crate::llm::catalog::all();
        let at = catalog.iter().position(|c| {
            models
                .iter()
                .any(|m| m.id == id && m.catalog.as_deref() == Some(&c.id))
        });
        match at {
            Some(i) if catalog[i].tools => i,
            Some(i) => 2 * catalog.len() + i,
            None => catalog.len(),
        }
    };
    let mut ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    ids.sort_by_key(|id| order(id));
    ids.first().map(|id| (*id).to_owned())
}

const BEGIN: &str = "<!-- Atollo: begin (written again at every start, do not edit) -->";
const END: &str = "<!-- Atollo: end -->";

/// `text` with the kit's block (`block`) in place of the old one, or added
/// at the end; the rest as it was.
pub fn with_block(text: &str, block: &str) -> String {
    let marked = format!("{BEGIN}\n{block}\n{END}");
    let begin = text.find(BEGIN);
    let end = begin.and_then(|a| text[a..].find(END).map(|b| a + b));
    match (begin, end) {
        (Some(a), Some(b)) => format!("{}{marked}{}", &text[..a], &text[b + END.len()..]),
        _ if text.trim().is_empty() => format!("{marked}\n"),
        _ => format!("{}\n\n{marked}\n", text.trim_end()),
    }
}

/// The kit rule in `APPEND_SYSTEM.md`.
pub fn sync_rule(p: &Paths) -> Result<()> {
    let file = p.pi.join("APPEND_SYSTEM.md");
    let old = match fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e.into()),
    };
    let new = with_block(&old, super::KIT_RULE_TEXT);
    if new != old {
        fs::create_dir_all(&p.pi)?;
        crate::common::fsx::write_atomic(&file, new.as_bytes())?;
    }
    Ok(())
}

/// The provider `atollo` for `models` on `port` with `key`: Pi's
/// OpenAI-compatible client on the router's `/v1`.
pub fn provider(port: u16, key: &str, models: &[crate::llm::local::LocalModel]) -> Value {
    let list: Vec<Value> = models
        .iter()
        .map(|m| {
            let mut input = vec!["text"];
            if m.mmproj.is_some() {
                input.push("image");
            }
            json!({
                "id": m.id,
                "name": m.name,
                "contextWindow": m.ctx.unwrap_or(crate::llm::presets::DEFAULT_CTX),
                "maxTokens": 8192,
                "input": input,
            })
        })
        .collect();
    json!({
        "name": "Atollo (modelli locali)",
        "baseUrl": format!("{}/v1", crate::llm::service::url_of(port)),
        "api": "openai-completions",
        "apiKey": key,
        "models": list,
    })
}

/// `models.json`: the provider of the local models, or none without a
/// model on disk.
pub fn sync_models(kit: &Kit, p: &Paths) -> Result<()> {
    let file = p.pi.join("models.json");
    let mut cfg = read_json(&file)?.unwrap_or_else(|| json!({}));
    let obj = cfg
        .as_object_mut()
        .context("models.json is not an object")?;
    let models: Vec<_> = crate::llm::local::list(kit)
        .into_iter()
        .filter(|m| m.complete)
        .collect();
    let providers = obj
        .entry("providers")
        .or_insert_with(|| Value::Object(Map::new()));
    if !providers.is_object() {
        *providers = Value::Object(Map::new());
    }
    let list = providers.as_object_mut().context("providers")?;
    let before = list.get(PROVIDER).cloned();
    if models.is_empty() {
        list.remove(PROVIDER);
    } else {
        let port = crate::llm::settings::load(kit)
            .port
            .unwrap_or(*crate::llm::settings::PORTS.start());
        let key = crate::llm::settings::ensure_key(kit)?;
        list.insert(PROVIDER.into(), provider(port, &key, &models));
    }
    if list.get(PROVIDER) == before.as_ref() {
        return Ok(());
    }
    let text = serde_json::to_string_pretty(&cfg)?;
    // The server's key is in it: owner-only, as auth.json.
    crate::platform::files::write_private(&file, &text)
}

/// `settings.json`: the keys the kit manages (see the module).
pub fn sync_settings(kit: &Kit, p: &Paths) -> Result<()> {
    let file = p.pi.join("settings.json");
    let state_file = p.state.join("pi-settings.json");
    let written = read_state(&state_file);
    let mut cfg = read_json(&file)?.unwrap_or_else(|| json!({}));
    let obj = cfg
        .as_object_mut()
        .context("settings.json is not an object")?;
    let before = obj.clone();
    let mut now = Map::new();
    // A key is the kit's while it holds what the kit wrote last.
    let ours =
        |obj: &Map<String, Value>, k: &str| obj.get(k).is_none() || obj.get(k) == written.get(k);
    if kit.os().is_windows() {
        let bash = crate::setup::kit_path(kit, "git/bin/bash.exe");
        if bash.is_file() && ours(obj, "shellPath") {
            let v = Value::String(bash.display().to_string());
            obj.insert("shellPath".into(), v.clone());
            now.insert("shellPath".into(), v);
        }
    }
    if !obj.contains_key("defaultProjectTrust") {
        obj.insert("defaultProjectTrust".into(), json!("ask"));
    }
    let local = local_model(kit);
    let user_provider = obj
        .get("defaultProvider")
        .and_then(Value::as_str)
        .is_some_and(|v| v != PROVIDER);
    match local {
        Some(id) if !user_provider => {
            for (k, v) in [
                ("defaultProvider", json!(PROVIDER)),
                ("defaultModel", json!(id)),
            ] {
                obj.insert(k.into(), v.clone());
                now.insert(k.into(), v);
            }
        }
        None if !user_provider => {
            obj.remove("defaultProvider");
            obj.remove("defaultModel");
        }
        _ => {}
    }
    if *obj != before {
        write_json(&file, &cfg)?;
    }
    write_json(&state_file, &Value::Object(now))
}

/// One server of the kit in Pi's format (`None`: Pi cannot use it, SSE).
fn pi_server(s: &Value) -> Option<Value> {
    match server_type(s).as_str() {
        "sse" => None,
        "stdio" => {
            let mut out = Map::new();
            out.insert("command".into(), s.get("command")?.clone());
            for k in ["args", "env", "cwd"] {
                if let Some(v) = s.get(k) {
                    out.insert(k.into(), v.clone());
                }
            }
            Some(Value::Object(out))
        }
        _ => {
            let mut out = Map::new();
            out.insert("url".into(), s.get("url")?.clone());
            if let Some(h) = s.get("headers") {
                out.insert("headers".into(), h.clone());
            }
            Some(Value::Object(out))
        }
    }
}

/// The kit's MCP servers in `mcp.json`; the servers added in Pi (`pi mcp
/// add`), or the kit's changed there, are left alone (as Antigravity's).
pub fn sync_mcp(p: &Paths, servers: &Servers, report: &mut Report) -> Result<()> {
    let file = p.pi.join("mcp.json");
    let state_file = p.state.join("mcp-pi.json");
    let base = read_state(&state_file);
    let mut cfg = read_json(&file)?.unwrap_or_else(|| json!({}));
    let mut theirs = cfg
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut now = Map::new();
    let mut changed = false;
    for (n, s) in servers {
        let Some(want) = pi_server(s) else {
            report.warn(format!(
                "MCP {n}: Pi does not use SSE servers, left out of Pi"
            ));
            continue;
        };
        let cur = theirs.get(n).map(canonical);
        let base_n = base.get(n).and_then(Value::as_str);
        if cur.is_some() && cur.as_deref() != base_n {
            continue;
        }
        let want_c = canonical(&want);
        if cur.as_deref() != Some(want_c.as_str()) {
            theirs.insert(n.clone(), want);
            changed = true;
        }
        now.insert(n.clone(), Value::String(want_c));
    }
    if let Some(base) = base.as_object() {
        for (n, b) in base {
            if !now.contains_key(n) && theirs.get(n).map(canonical).as_deref() == b.as_str() {
                theirs.remove(n);
                changed = true;
            }
        }
    }
    if changed {
        if let Some(obj) = cfg.as_object_mut() {
            obj.insert("mcpServers".into(), Value::Object(theirs));
        }
        write_json(&file, &cfg)?;
        report.note("data\\home\\.pi\\agent\\mcp.json updated");
    }
    write_json(&state_file, &Value::Object(now))
}

/// The key of `dir` in `trust.json`: the folder as the system names it
/// (Pi resolves it with `realpath`).
fn trust_key(dir: &Path) -> String {
    let real = fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let text = real.display().to_string();
    text.strip_prefix(r"\\?\")
        .map_or(text.clone(), str::to_owned)
}

/// `dir` trusted in `trust.json`, unless the user decided for it already.
pub(super) fn trust(p: &Paths, dir: &Path) -> Result<()> {
    let file = p.pi.join("trust.json");
    let mut cfg = read_json(&file)?.unwrap_or_else(|| json!({}));
    let obj = cfg.as_object_mut().context("trust.json is not an object")?;
    let key = trust_key(dir);
    if obj.get(&key).is_some_and(|v| !v.is_null()) {
        return Ok(());
    }
    obj.insert(key, Value::Bool(true));
    write_json(&file, &cfg)
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/pi.rs"]
mod tests;
