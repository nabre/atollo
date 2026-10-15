//! A project's own MCP servers (DECISIONS.md D-78), besides the kit's ones
//! in `data\agents\mcp.json` that every project has.
//!
//! - kept in the project's `.mcp.json` (`mcpServers`), the file Claude Code
//!   reads for a project (it asks before using them); in the repository,
//!   like the actions in panel.json;
//! - Codex reads a trusted project's `.codex\config.toml` (checked in its
//!   own help, 14/10/2026): the app writes the same servers there, in a
//!   block of its own; the rest of that file is the user's;
//! - Antigravity has no per-project MCP file (only its global
//!   `mcp_config.json`): the page says so;
//! - every other key of `.mcp.json` and of each server stays; never through
//!   a link (D-77).

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::agents::sync::codex::{codex_server, strip_block};
use crate::fsx::write_atomic;

const CODEX_BEGIN: &str =
    "# >>> Atollo: server MCP del progetto da .mcp.json (non modificare: usa la pagina del progetto) >>>";
const CODEX_END: &str = "# <<< Atollo: server MCP del progetto <<<";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub name: String,
    /// `stdio` (a program) or `http` (an address).
    pub kind: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpView {
    pub servers: Vec<McpServer>,
    /// `.mcp.json` is not plain JSON: shown empty, never written.
    pub read_only: bool,
}

fn mcp_path(dir: &Path) -> std::path::PathBuf {
    dir.join(".mcp.json")
}

fn read_object(dir: &Path) -> Result<Option<Map<String, Value>>> {
    match crate::os::safe_path::read_text(dir, &mcp_path(dir)) {
        None => Ok(None),
        Some(text) => match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(o)) => Ok(Some(o)),
            _ => bail!(".mcp.json is not plain JSON: left alone"),
        },
    }
}

fn server_of(name: &str, v: &Value) -> McpServer {
    let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_owned);
    McpServer {
        name: name.to_owned(),
        kind: crate::agents::sync::mcp::server_type(v),
        command: s("command"),
        args: v
            .get("args")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default(),
        url: s("url"),
        env: v
            .get("env")
            .and_then(Value::as_object)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_owned())))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

pub fn get(dir: &Path) -> McpView {
    match read_object(dir) {
        Err(_) => McpView {
            servers: Vec::new(),
            read_only: true,
        },
        Ok(o) => McpView {
            servers: o
                .as_ref()
                .and_then(|o| o.get("mcpServers"))
                .and_then(Value::as_object)
                .map(|m| m.iter().map(|(n, v)| server_of(n, v)).collect())
                .unwrap_or_default(),
            read_only: false,
        },
    }
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 60
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn check(s: &McpServer) -> Result<()> {
    if !valid_name(&s.name) {
        bail!("name: letters, digits, - and _");
    }
    let one_line = |v: &str| !v.contains(['\n', '\r']);
    match s.kind.as_str() {
        "stdio" => {
            if !s
                .command
                .as_deref()
                .is_some_and(|c| !c.trim().is_empty() && one_line(c))
            {
                bail!("{}: a program is needed", s.name);
            }
        }
        "http" => {
            if !s.url.as_deref().is_some_and(|u| {
                (u.starts_with("https://") || u.starts_with("http://")) && one_line(u)
            }) {
                bail!("{}: an http(s) address is needed", s.name);
            }
        }
        other => bail!("{}: unknown kind {other}", s.name),
    }
    if s.args.iter().chain(s.env.values()).any(|v| !one_line(v)) {
        bail!("{}: one line per value", s.name);
    }
    Ok(())
}

/// The server as `.mcp.json` has it, keeping the keys the page does not edit.
fn to_value(s: &McpServer, old: Option<&Value>) -> Value {
    let mut o = old.and_then(Value::as_object).cloned().unwrap_or_default();
    for k in ["type", "command", "args", "url", "env"] {
        o.remove(k);
    }
    if s.kind == "http" {
        o.insert("type".into(), Value::String("http".into()));
        o.insert(
            "url".into(),
            Value::String(s.url.clone().unwrap_or_default()),
        );
    } else {
        o.insert(
            "command".into(),
            Value::String(s.command.clone().unwrap_or_default().trim().to_owned()),
        );
        if !s.args.is_empty() {
            o.insert(
                "args".into(),
                serde_json::to_value(&s.args).unwrap_or_default(),
            );
        }
    }
    if !s.env.is_empty() {
        o.insert(
            "env".into(),
            serde_json::to_value(&s.env).unwrap_or_default(),
        );
    }
    Value::Object(o)
}

/// Writes the servers to `.mcp.json` and the same ones to Codex's project
/// file. An empty list removes `mcpServers` (and the Codex block).
pub fn set(dir: &Path, servers: &[McpServer]) -> Result<()> {
    for s in servers {
        check(s)?;
    }
    let mut seen = std::collections::BTreeSet::new();
    if servers.iter().any(|s| !seen.insert(s.name.as_str())) {
        bail!("two servers with the same name");
    }
    for p in [
        mcp_path(dir),
        dir.join(".codex"),
        dir.join(".codex").join("config.toml"),
    ] {
        if crate::os::safe_path::is_link(&p) {
            bail!("{}: link, left alone", p.display());
        }
    }
    let mut o = read_object(dir)?.unwrap_or_default();
    let old = o
        .get("mcpServers")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let new: Map<String, Value> = servers
        .iter()
        .map(|s| (s.name.clone(), to_value(s, old.get(&s.name))))
        .collect();
    if new.is_empty() {
        o.remove("mcpServers");
    } else {
        o.insert("mcpServers".into(), Value::Object(new.clone()));
    }
    if o.is_empty() {
        if crate::os::safe_path::is_file(dir, &mcp_path(dir)) {
            std::fs::remove_file(mcp_path(dir))?;
        }
    } else {
        let json = serde_json::to_string_pretty(&Value::Object(o))? + "\n";
        write_atomic(&mcp_path(dir), json.as_bytes())?;
    }
    write_codex(dir, &new)
}

fn write_codex(dir: &Path, servers: &Map<String, Value>) -> Result<()> {
    let file = dir.join(".codex").join("config.toml");
    let old = crate::os::safe_path::read_text(dir, &file).unwrap_or_default();
    let mut text = strip_block(&old, CODEX_BEGIN, CODEX_END);
    let tables: Vec<String> = servers
        .iter()
        .filter_map(|(n, v)| codex_server(n, v))
        .collect();
    if !tables.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&format!(
            "{CODEX_BEGIN}\n{}\n{CODEX_END}\n",
            tables.join("\n\n")
        ));
    }
    if text.trim().is_empty() {
        if crate::os::safe_path::is_file(dir, &file) {
            std::fs::remove_file(&file)?;
        }
        return Ok(());
    }
    if text != old {
        std::fs::create_dir_all(dir.join(".codex"))?;
        write_atomic(&file, text.as_bytes())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::tests::TempDir;
    use std::fs;

    fn stdio(name: &str) -> McpServer {
        McpServer {
            name: name.into(),
            kind: "stdio".into(),
            command: Some("npx".into()),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-filesystem".into(),
                ".".into(),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn servers_in_mcp_json_and_codex() {
        let tmp = TempDir::new("project-mcp");
        let d = tmp.0.join("p");
        fs::create_dir_all(d.join(".codex")).unwrap();
        fs::write(d.join(".codex").join("config.toml"), "model = \"x\"\n").unwrap();
        fs::write(
            d.join(".mcp.json"),
            r#"{"other":1,"mcpServers":{"fs":{"command":"old","timeout":30}}}"#,
        )
        .unwrap();
        assert_eq!(get(&d).servers[0].command.as_deref(), Some("old"));
        let web = McpServer {
            name: "docs".into(),
            kind: "http".into(),
            url: Some("https://example.com/mcp".into()),
            ..Default::default()
        };
        set(&d, &[stdio("fs"), web.clone()]).unwrap();
        let o: Value =
            serde_json::from_str(&fs::read_to_string(d.join(".mcp.json")).unwrap()).unwrap();
        assert_eq!(o["other"], 1, "other keys stay");
        assert_eq!(o["mcpServers"]["fs"]["command"], "npx");
        assert_eq!(
            o["mcpServers"]["fs"]["timeout"], 30,
            "keys of a server stay"
        );
        assert_eq!(o["mcpServers"]["docs"]["type"], "http");
        let toml = fs::read_to_string(d.join(".codex").join("config.toml")).unwrap();
        assert!(toml.starts_with("model = \"x\"\n"), "{toml}");
        assert!(
            toml.contains("[mcp_servers.\"fs\"]") || toml.contains("[mcp_servers.fs]"),
            "{toml}"
        );
        assert_eq!(get(&d).servers.len(), 2);
        // Removed: the Codex block goes, the user's line stays.
        set(&d, &[]).unwrap();
        let toml = fs::read_to_string(d.join(".codex").join("config.toml")).unwrap();
        assert_eq!(toml, "model = \"x\"\n");
        let o: Value =
            serde_json::from_str(&fs::read_to_string(d.join(".mcp.json")).unwrap()).unwrap();
        assert!(o.get("mcpServers").is_none());
        // Refused.
        assert!(set(
            &d,
            &[McpServer {
                name: "a b".into(),
                ..stdio("x")
            }]
        )
        .is_err());
        assert!(set(&d, &[stdio("x"), stdio("x")]).is_err());
        assert!(set(
            &d,
            &[McpServer {
                url: Some("ftp://x".into()),
                ..web
            }]
        )
        .is_err());
    }
}
