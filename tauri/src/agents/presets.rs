//! Ready MCP servers (report v6/04, N-7, DECISIONS.md D-156): «Aggiungi»
//! puts one in the kit's `data\agents\mcp.json` (every agent, every
//! project) or in a project's `.mcp.json`, as if written by hand.
//!
//! - no secret in the files (`.mcp.json` is in the repository): a server
//!   reads the project's database from a variable, `KEY: ${KEY}` (Codex
//!   passes it on as `env_vars`, Claude Code expands it);
//! - the GitHub token is in no file and in no environment of the agent
//!   (SEC-V7-01): the github server is the app itself by its absolute
//!   path, `<app> mcp github` (`agents::github_mcp`), which reads the token
//!   itself and sends it to GitHub only. In the agent's environment the MCP
//!   servers of a cloned project, which inherit it, would read it;
//! - their programs (`npx` → Node, `uvx` → uv) are not installed by the
//!   app: no agent starts on the PC since D-178 (the agents in the VM:
//!   D-175).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::projects::mcp::McpServer;

/// The project's MySQL database, for DBHub.
pub const MYSQL_VAR: &str = "DSN";
/// The project's MongoDB database, for MongoDB's server.
pub const MONGODB_VAR: &str = "MDB_MCP_CONNECTION_STRING";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: &'static str,
    /// `kit` (data\agents\mcp.json) or `project` (its .mcp.json).
    pub scope: &'static str,
    /// The project's database it reads (`mysql`, `mongodb`): offered only
    /// to a project that has one.
    pub engine: Option<&'static str>,
    pub server: McpServer,
}

fn stdio(name: &str, command: &str, args: &[&str], env: &[&str]) -> McpServer {
    McpServer {
        name: name.to_owned(),
        kind: "stdio".into(),
        command: Some(command.to_owned()),
        args: args.iter().map(|a| (*a).to_owned()).collect(),
        url: None,
        env: env
            .iter()
            .map(|k| ((*k).to_owned(), format!("${{{k}}}")))
            .collect(),
        headers: BTreeMap::new(),
    }
}

/// The github server: the app itself by its absolute path, `<app> mcp
/// github` (`agents::github_mcp`). Never a name looked up when the agent
/// starts it: the agent runs in the project's folder, and Windows (cmd,
/// libuv) looks there before the PATH, so a cloned repository's
/// `atollo-mcp.cmd` or `cmd.exe` would run as the kit's trusted server
/// (SEC-V7-01). The app is inside the kit's root, so when the kit moves
/// `agents::relocate` rewrites the path in `mcp.json` and the agents'
/// files like every other path of the kit.
fn github(exe: &Path) -> McpServer {
    stdio("github", &exe.to_string_lossy(), &["mcp", "github"], &[])
}

/// The ready servers, in the order of the page; github with this app's path.
pub fn all() -> Vec<Preset> {
    all_for(crate::kit::app_exe().as_deref())
}

/// The ready servers with `exe` as the app (no github without it).
pub fn all_for(exe: Option<&Path>) -> Vec<Preset> {
    let preset = |id, scope, engine, server| Preset {
        id,
        scope,
        engine,
        server,
    };
    let mut out = Vec::new();
    // GitHub's own server, on GitHub (the local one is a program of its own
    // to download), through the app: the kit's account, the token neither
    // in the file nor in the agent's environment.
    if let Some(exe) = exe.filter(|e| e.is_absolute()) {
        out.push(preset("github", "kit", None, github(exe)));
    }
    out.extend([
        preset(
            "playwright",
            "kit",
            None,
            stdio("playwright", "npx", &["-y", "@playwright/mcp@latest"], &[]),
        ),
        preset(
            "fetch",
            "kit",
            None,
            stdio("fetch", "uvx", &["mcp-server-fetch"], &[]),
        ),
        // `.`: the agent runs in the project's folder, so the server sees
        // that folder only, wherever the kit is.
        preset(
            "filesystem",
            "project",
            None,
            stdio(
                "filesystem",
                "npx",
                &["-y", "@modelcontextprotocol/server-filesystem", "."],
                &[],
            ),
        ),
        preset(
            "mysql",
            "project",
            Some("mysql"),
            stdio(
                "mysql",
                "npx",
                &["-y", "@bytebase/dbhub", "--transport", "stdio"],
                &[MYSQL_VAR],
            ),
        ),
        preset(
            "mongodb",
            "project",
            Some("mongodb"),
            stdio(
                "mongodb",
                "npx",
                &["-y", "mongodb-mcp-server@latest"],
                &[MONGODB_VAR],
            ),
        ),
    ]);
    out
}

#[cfg(test)]
#[path = "../../../test/rust/agents/presets.rs"]
mod tests;
