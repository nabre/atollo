//! Claude Code's token an earlier Atollo saved in the kit (D-115, macOS):
//! `data/agents/claude/oauth-token`, written by «Collega con un token»
//! (`claude setup-token` on the PC) and given to Claude Code on the PC.
//!
//! No agent runs on the PC since D-179 (LEVELS.md §7, E16): the token is
//! no longer made nor given to anything, and never to a VM (I5); Claude
//! Code signs in inside its `sbx` sandbox. A file still there is kit data:
//! the agents' page says so, «Esci» deletes it ([`remove`]), the
//! diagnostics leave it out ([`path`]).

use std::path::PathBuf;

use anyhow::Result;

use crate::kit::Kit;

/// What starts every token `claude setup-token` printed.
const PREFIX: &str = "sk-ant-oat";

/// `data/agents/claude/oauth-token`.
pub fn path(kit: &Kit) -> PathBuf {
    kit.data().join("agents").join("claude").join("oauth-token")
}

fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// A token as `claude setup-token` prints it.
fn is_token(text: &str) -> bool {
    text.starts_with(PREFIX) && text.len() >= PREFIX.len() + 32 && text.chars().all(token_char)
}

/// The saved token, if any.
pub fn read(kit: &Kit) -> Option<String> {
    let text = std::fs::read_to_string(path(kit)).ok()?;
    let token = text.trim();
    is_token(token).then(|| token.to_owned())
}

/// «Esci»: the token is forgotten (no file is not an error).
pub fn remove(kit: &Kit) -> Result<()> {
    match std::fs::remove_file(path(kit)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/agents/claude_token.rs"]
mod tests;
