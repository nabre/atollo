//! The kit's GitHub MCP server (D-156, SEC-V7-01): `Atollo mcp github`,
//! started by the agent through `data\kit\bin\atollo-mcp(.cmd)`.
//!
//! The agent talks MCP over stdio with this program, which passes every
//! message to GitHub's own remote server (`URL`, MCP's Streamable HTTP)
//! with the token of the kit's account, read here from the kit's `gh` and
//! never put in the agent's environment: the agent, its commands and the
//! MCP servers of a cloned project (which inherit the agent's environment)
//! never see it.
//!
//! One message per line in, one per line out: a request is answered with
//! the server's JSON or with the events of its stream; a failed request
//! gets a JSON-RPC error, so the agent never waits for an answer that will
//! not come. The messages go one at a time, in order (`initialize` first,
//! then the rest with its session).

use std::io::{BufRead, BufReader, Write};

use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::settings::Settings;

/// GitHub's own MCP server.
pub const URL: &str = "https://api.githubcopilot.com/mcp/";

/// The largest answer kept (a JSON answer or one event).
const LIMIT: u64 = 32 * 1024 * 1024;

/// `Atollo mcp github`: the bridge on standard input and output.
pub fn run(kit: &Kit, env: &KitEnv, settings: &Settings) -> Result<()> {
    let token = crate::github::token(kit, env)
        .context("GitHub: the kit is not signed in (Settings > GitHub)")?;
    let net = crate::net::Net::new(settings, None)?;
    bridge(
        net.agent(),
        URL,
        &token,
        std::io::stdin().lock(),
        std::io::stdout().lock(),
    )
}

/// What the bridge remembers between the messages.
#[derive(Default)]
struct Session {
    /// `Mcp-Session-Id` of the server's answer to `initialize`.
    id: Option<String>,
    /// The protocol version agreed at `initialize` (`MCP-Protocol-Version`).
    version: Option<String>,
}

/// Every line of `input` to `url` with `token`, the answers to `output`.
/// Ends with `input`; the session is closed at the server then.
pub fn bridge(
    agent: &ureq::Agent,
    url: &str,
    token: &str,
    input: impl BufRead,
    mut output: impl Write,
) -> Result<()> {
    let mut session = Session::default();
    for line in input.lines() {
        let line = line.context("reading the agent's message")?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(message) = serde_json::from_str::<Value>(line) else {
            emit(
                &mut output,
                &error(Value::Null, -32700, "not a JSON message"),
                &mut session,
            )?;
            continue;
        };
        // A request (method and id) waits for its answer; notifications and
        // answers to the server do not.
        let id = message.get("method").and(message.get("id")).cloned();
        if let Err(e) = post(agent, url, token, line, &mut session, &mut output) {
            if let Some(id) = id {
                emit(
                    &mut output,
                    &error(id, -32603, &format!("GitHub MCP (Atollo): {e:#}")),
                    &mut session,
                )?;
            }
        }
    }
    if let Some(id) = &session.id {
        // Best effort: the server forgets the session anyway.
        let _ = agent
            .delete(url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Mcp-Session-Id", id)
            .call();
    }
    Ok(())
}

/// One message to the server, its answers to `output`.
fn post(
    agent: &ureq::Agent,
    url: &str,
    token: &str,
    message: &str,
    session: &mut Session,
    output: &mut impl Write,
) -> Result<()> {
    let mut request = agent
        .post(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream");
    if let Some(id) = &session.id {
        request = request.header("Mcp-Session-Id", id);
    }
    if let Some(v) = &session.version {
        request = request.header("MCP-Protocol-Version", v);
    }
    let response = request
        .config()
        .http_status_as_error(false)
        .build()
        .send(message)
        .context("no answer from GitHub")?;
    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        let hint = match status {
            401 | 403 => ": sign in to GitHub again in the kit (Settings > GitHub)",
            _ => "",
        };
        anyhow::bail!("HTTP {status}{hint}");
    }
    if let Some(id) = response
        .headers()
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
    {
        session.id = Some(id.to_owned());
    }
    let stream = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|t| t.trim_start().starts_with("text/event-stream"));
    if stream {
        let reader = BufReader::new(response.into_body().into_reader());
        events(reader, |data| relay(output, &data, session))
    } else {
        let text = response
            .into_body()
            .with_config()
            .limit(LIMIT)
            .read_to_string()
            .context("reading GitHub's answer")?;
        relay(output, &text, session)
    }
}

/// The `data` of each event of a stream (Server-Sent Events): its `data:`
/// lines joined by a line break, given when the blank line ends the event
/// (or the stream ends).
pub fn events(reader: impl BufRead, mut each: impl FnMut(String) -> Result<()>) -> Result<()> {
    let mut data: Option<String> = None;
    let mut size = 0u64;
    for line in reader.split(b'\n') {
        let line = line.context("reading GitHub's stream")?;
        let line = String::from_utf8_lossy(&line);
        let line = line.strip_suffix('\r').unwrap_or(&line);
        if line.is_empty() {
            if let Some(d) = data.take() {
                each(d)?;
            }
            size = 0;
            continue;
        }
        let Some(value) = line.strip_prefix("data:") else {
            continue; // `event:`, `id:`, `retry:`, comments
        };
        let value = value.strip_prefix(' ').unwrap_or(value);
        size += value.len() as u64;
        anyhow::ensure!(size <= LIMIT, "an event of GitHub's stream is too large");
        match &mut data {
            Some(d) => {
                d.push('\n');
                d.push_str(value);
            }
            None => data = Some(value.to_owned()),
        }
    }
    if let Some(d) = data {
        each(d)?;
    }
    Ok(())
}

/// The messages of an answer (one, or a batch) to `output`, one per line.
fn relay(output: &mut impl Write, text: &str, session: &mut Session) -> Result<()> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(());
    }
    let value: Value = serde_json::from_str(text).context("GitHub's answer is not JSON")?;
    match value {
        Value::Array(batch) => {
            for m in &batch {
                emit(output, m, session)?;
            }
            Ok(())
        }
        m => emit(output, &m, session),
    }
}

/// One message on one line; the protocol version of `initialize`'s answer
/// is kept for the next requests.
fn emit(output: &mut impl Write, message: &Value, session: &mut Session) -> Result<()> {
    if let Some(v) = message
        .get("result")
        .and_then(|r| r.get("protocolVersion"))
        .and_then(Value::as_str)
    {
        session.version = Some(v.to_owned());
    }
    let mut line = serde_json::to_string(message)?;
    line.push('\n');
    output
        .write_all(line.as_bytes())
        .and_then(|()| output.flush())
        .context("writing to the agent")
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

#[cfg(test)]
#[path = "../../../test/rust/agents/github_mcp.rs"]
mod tests;
