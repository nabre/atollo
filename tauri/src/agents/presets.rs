//! Ready MCP servers (report v6/04, N-7, DECISIONS.md D-156): «Aggiungi»
//! puts one in the kit's `data\agents\mcp.json` (every agent, every
//! project) or in a project's `.mcp.json`, as if written by hand.
//!
//! - no secret in the files (`.mcp.json` is in the repository): a server
//!   reads the GitHub token or the project's database from a variable,
//!   `KEY: ${KEY}` (Codex passes it on as `env_vars`, Claude Code expands it),
//!   that the app sets when it starts the agent ([`launch_vars`]);
//! - their programs (`npx` → Node, `uvx` → uv) are installed when the agent
//!   starts, as for any server (`setup::needs::of_agent`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::db::auth::ProjectDb;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::projects::mcp::McpServer;

/// The kit account's GitHub token, for the GitHub server.
pub const GITHUB_VAR: &str = "GITHUB_PERSONAL_ACCESS_TOKEN";
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

/// The ready servers, in the order of the page.
pub fn all() -> Vec<Preset> {
    let preset = |id, scope, engine, server| Preset {
        id,
        scope,
        engine,
        server,
    };
    vec![
        // GitHub's own server, on GitHub (the local one is a program of its
        // own to download); the kit's account, never a token in the file.
        preset(
            "github",
            "kit",
            None,
            McpServer {
                name: "github".into(),
                kind: "http".into(),
                command: None,
                args: Vec::new(),
                url: Some("https://api.githubcopilot.com/mcp/".into()),
                env: BTreeMap::new(),
                headers: BTreeMap::from([(
                    "Authorization".to_owned(),
                    format!("Bearer ${{{GITHUB_VAR}}}"),
                )]),
            },
        ),
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
    ]
}

/// Which of the variables of the ready servers `texts` (the MCP files the
/// agent reads) use as `${NAME}`.
pub fn used_vars(texts: &[String]) -> Vec<&'static str> {
    [GITHUB_VAR, MYSQL_VAR, MONGODB_VAR]
        .into_iter()
        .filter(|v| {
            let var = format!("${{{v}}}");
            texts.iter().any(|t| t.contains(&var))
        })
        .collect()
}

/// The variables to give the agent: those of the kit's servers, and the
/// project's servers' but the GitHub token.
pub fn wanted_vars(kit: &[String], project: &[String]) -> Vec<&'static str> {
    let mut wanted = used_vars(kit);
    for v in used_vars(project) {
        if v != GITHUB_VAR && !wanted.contains(&v) {
            wanted.push(v);
        }
    }
    wanted
}

/// The values of the variables the agent's servers use, to set when the
/// agent starts in `dir`: the GitHub token (when the kit's account is
/// signed in), the project's databases (when created). One already set
/// stays. The token only for the kit's own servers: a cloned repository's
/// `.mcp.json` asking for it would send it wherever it likes.
pub fn launch_vars(
    kit: &Kit,
    env: &KitEnv,
    dir: &Path,
    dbs: impl FnOnce() -> Vec<ProjectDb>,
) -> Vec<(&'static str, String)> {
    let read = |f: &Path| {
        std::fs::read_to_string(f)
            .ok()
            .into_iter()
            .collect::<Vec<_>>()
    };
    let mut wanted = wanted_vars(
        &read(&crate::agents::sync::Paths::of(kit).mcp_file()),
        &read(&dir.join(".mcp.json")),
    );
    wanted.retain(|v| env.get(v).is_none());
    let mut out = Vec::new();
    if wanted.contains(&GITHUB_VAR) {
        if let Some(token) = crate::github::token(kit, env) {
            out.push((GITHUB_VAR, token));
        }
    }
    if wanted.contains(&MYSQL_VAR) || wanted.contains(&MONGODB_VAR) {
        for db in dbs() {
            let var = match db.engine.as_str() {
                "mysql" => MYSQL_VAR,
                "mongodb" => MONGODB_VAR,
                _ => continue,
            };
            if wanted.contains(&var) {
                out.push((var, db.url));
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../test/rust/agents/presets.rs"]
mod tests;
