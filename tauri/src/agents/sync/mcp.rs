//! MCP servers: `data\agents\mcp.json` <-> Claude (`.claude.json`), and the
//! kit's servers in Antigravity's `mcp_config.json` (port of `mcp.js`;
//! Codex: `codex.rs`).

use std::collections::BTreeSet;

use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};

use super::{canonical, read_json, read_state, stamp, write_json, Paths, Report};

pub type Servers = Map<String, Value>;

/// `type`, or `http` when it has a url, else `stdio`.
pub fn server_type(s: &Value) -> String {
    s.get("type")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            if s.get("url").is_some() {
                "http".into()
            } else {
                "stdio".into()
            }
        })
}

/// No file, empty, or `{}`: no servers. `mcpServers` of the wrong type: an
/// error (an empty list would remove the servers from the agents).
pub fn load(p: &Paths) -> Result<Servers> {
    let Some(j) = read_json(&p.mcp_file())? else {
        return Ok(Map::new());
    };
    match j.get("mcpServers") {
        None => Ok(Map::new()),
        Some(Value::Object(m)) => Ok(m.clone()),
        Some(_) => bail!("data\\agents\\mcp.json: \"mcpServers\" is not an object"),
    }
}

pub fn save(p: &Paths, servers: &Servers) -> Result<()> {
    write_json(&p.mcp_file(), &json!({ "mcpServers": servers }))
}

/// Claude's user-level servers (`claude mcp add -s user`) in `.claude.json`.
/// Same rules as the files; when both changed a server, mcp.json wins and
/// Claude's version is saved in `.sync\conflicts`.
pub fn sync_claude(p: &Paths, shared: Servers, report: &mut Report) -> Result<Servers> {
    let state_file = p.state.join("mcp-claude.json");
    let base = read_state(&state_file);
    // Claude not started yet: its file appears at the first start.
    let Some(cfg) = read_json(&p.claude_json())? else {
        return Ok(shared);
    };
    let theirs = cfg
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let mut ours = shared;
    let mut for_claude: Vec<(String, Option<Value>)> = Vec::new();
    let mut shared_changed = false;

    let names: BTreeSet<String> = ours.keys().chain(theirs.keys()).cloned().collect();
    for n in names {
        let o = ours.get(&n).map(canonical);
        let th = theirs.get(&n).map(canonical);
        let b = base.get(&n).and_then(Value::as_str).map(str::to_owned);
        match (o, th) {
            (Some(o), Some(th)) => {
                if o == th {
                    continue;
                }
                let (o_changed, th_changed) = (Some(&o) != b.as_ref(), Some(&th) != b.as_ref());
                if th_changed && !o_changed {
                    ours.insert(n.clone(), theirs[&n].clone());
                    shared_changed = true;
                } else {
                    if th_changed {
                        let dest = p.state.join("conflicts").join(format!(
                            "{}-mcp-{}.json",
                            stamp(),
                            n.chars()
                                .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' {
                                    c
                                } else {
                                    '_'
                                })
                                .collect::<String>()
                        ));
                        write_json(
                            &dest,
                            &json!({ "mcpServers": { n.clone(): theirs[&n].clone() } }),
                        )?;
                        report.warn(format!("MCP {n} changed on both sides: keeping data\\agents\\mcp.json, the other is in {}", dest.display()));
                    }
                    for_claude.push((n.clone(), ours.get(&n).cloned()));
                }
            }
            (Some(o), None) => {
                if b.as_deref() == Some(o.as_str()) {
                    ours.remove(&n);
                    shared_changed = true;
                } else {
                    if b.is_some() {
                        report.warn(format!(
                            "MCP {n} deleted in Claude but changed elsewhere: restored"
                        ));
                    }
                    for_claude.push((n.clone(), ours.get(&n).cloned()));
                }
            }
            (None, Some(th)) => {
                if b.as_deref() == Some(th.as_str()) {
                    for_claude.push((n.clone(), None));
                } else {
                    if b.is_some() {
                        report.warn(format!(
                            "MCP {n} deleted elsewhere but changed in Claude: restored"
                        ));
                    }
                    ours.insert(n.clone(), theirs[&n].clone());
                    shared_changed = true;
                }
            }
            (None, None) => {}
        }
        report.note(format!("MCP server {n} aligned"));
    }

    if !for_claude.is_empty() {
        // Re-read just before writing: Claude may have saved meanwhile. Only
        // the servers decided here change.
        let mut fresh = read_json(&p.claude_json())?.unwrap_or_else(|| json!({}));
        let obj = fresh
            .as_object_mut()
            .context(".claude.json is not an object")?;
        let mut servers = obj
            .get("mcpServers")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        for (n, s) in for_claude {
            match s {
                Some(s) => {
                    servers.insert(n, s);
                }
                None => {
                    servers.remove(&n);
                }
            }
        }
        obj.insert("mcpServers".into(), Value::Object(servers));
        write_json(&p.claude_json(), &fresh)?;
    }
    if shared_changed {
        save(p, &ours)?;
    }
    let now: Map<String, Value> = ours
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(canonical(v))))
        .collect();
    write_json(&state_file, &Value::Object(now))?;
    Ok(ours)
}

/// One Claude server in Antigravity's format (`None` if it cannot use it).
fn antigravity_server(s: &Value) -> Option<Value> {
    let mut out = Map::new();
    if server_type(s) == "stdio" {
        out.insert("command".into(), s.get("command")?.clone());
        if let Some(args) = s
            .get("args")
            .and_then(Value::as_array)
            .filter(|a| !a.is_empty())
        {
            out.insert(
                "args".into(),
                Value::Array(
                    args.iter()
                        .map(|a| {
                            Value::String(a.as_str().map_or_else(|| a.to_string(), str::to_owned))
                        })
                        .collect(),
                ),
            );
        }
        if let Some(env) = s
            .get("env")
            .and_then(Value::as_object)
            .filter(|e| !e.is_empty())
        {
            out.insert("env".into(), Value::Object(env.clone()));
        }
        if let Some(cwd) = s.get("cwd") {
            out.insert("cwd".into(), cwd.clone());
        }
    } else {
        out.insert("serverUrl".into(), s.get("url")?.clone());
        if let Some(h) = s
            .get("headers")
            .and_then(Value::as_object)
            .filter(|h| !h.is_empty())
        {
            out.insert("headers".into(), Value::Object(h.clone()));
        }
    }
    Some(Value::Object(out))
}

/// Writes only the servers of mcp.json and remembers what it wrote: servers
/// added with `agy mcp add`, or kit servers changed there, are left alone.
pub fn sync_antigravity(p: &Paths, servers: &Servers, report: &mut Report) -> Result<()> {
    let file = p.gemini.join(r"config\mcp_config.json");
    let state_file = p.state.join("mcp-antigravity.json");
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
        let Some(want) = antigravity_server(s) else {
            continue;
        };
        let cur = theirs.get(n).map(canonical);
        let base_n = base.get(n).and_then(Value::as_str);
        if cur.is_some() && cur.as_deref() != base_n {
            continue; // defined or changed in Antigravity: theirs
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
        report.note("data\\home\\.gemini\\config\\mcp_config.json updated");
    }
    write_json(&state_file, &Value::Object(now))
}

#[cfg(test)]
#[path = "../../../../test/rust/agents/sync/mcp.rs"]
mod tests;
