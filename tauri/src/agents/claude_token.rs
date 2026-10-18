//! Claude Code's sign-in on macOS kept in the kit (WP-4.2).
//!
//! On macOS Claude Code keeps its login in the Keychain, not in
//! `.claude/.credentials.json` of the kit's home: a kit taken to another
//! Mac is signed out there. "Collega con un token" (Agents page, macOS)
//! runs `claude setup-token` in a session of the app's terminal; the
//! long-lived token it prints is read from the session's output and saved
//! in `data/agents/claude/oauth-token` (600, in a 700 folder). While that
//! file exists Claude Code gets it as `CLAUDE_CODE_OAUTH_TOKEN`: when
//! started from the Agents page ([`super::launch`]), and from the kit's
//! terminals and editor once the kit's environment calls [`apply`] on
//! macOS (`env.rs`). "Esci" deletes it. The token is never logged: only
//! "saved" or "not found".
//!
//! The reading of the output is plain code, tested on every system; only
//! the terminal session is Unix code.
#![cfg_attr(not(any(unix, test)), allow(dead_code))]

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::env::KitEnv;
use crate::kit::{Kit, Os};

/// The variable Claude Code reads its token from.
pub const VAR: &str = "CLAUDE_CODE_OAUTH_TOKEN";

/// What starts every token `claude setup-token` prints.
pub const PREFIX: &str = "sk-ant-oat";

/// "Collega con un token" is offered on macOS only: elsewhere Claude Code
/// keeps its login in the kit's home already.
pub const fn offered(os: Os) -> bool {
    os.is_macos()
}

/// `data/agents/claude/oauth-token`.
pub fn path(kit: &Kit) -> PathBuf {
    path_in(&kit.data())
}

pub fn path_in(data: &Path) -> PathBuf {
    data.join("agents").join("claude").join("oauth-token")
}

fn token_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// A token as `claude setup-token` prints it.
pub fn is_token(text: &str) -> bool {
    text.starts_with(PREFIX) && text.len() >= PREFIX.len() + 32 && text.chars().all(token_char)
}

/// The saved token, if any.
pub fn read(kit: &Kit) -> Option<String> {
    read_in(&kit.data())
}

pub fn read_in(data: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path_in(data)).ok()?;
    let token = text.trim();
    is_token(token).then(|| token.to_owned())
}

/// Saves `token` (owner-only file).
pub fn save(kit: &Kit, token: &str) -> Result<()> {
    let token = token.trim();
    if !is_token(token) {
        bail!("not a Claude Code token");
    }
    crate::router::fs::write_private(&path(kit), token)
}

/// "Esci": the token is forgotten (no file is not an error).
pub fn remove(kit: &Kit) -> Result<()> {
    match std::fs::remove_file(path(kit)) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
        _ => Ok(()),
    }
}

/// Claude Code's environment: the saved token when there is one.
pub fn apply(kit: &Kit, env: &mut KitEnv) {
    apply_in(&kit.data(), env);
}

pub fn apply_in(data: &Path, env: &mut KitEnv) {
    if let Some(token) = read_in(data) {
        env.set(VAR, token);
    }
}

/// The text of a terminal's output without its escape sequences (CSI,
/// OSC, the two-byte ones) and with every line end as `\n`.
pub fn plain_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\u{1b}' => match chars.next() {
                // CSI: parameters, then one final byte in @..~.
                Some('[') => {
                    for d in chars.by_ref() {
                        if ('@'..='~').contains(&d) {
                            break;
                        }
                    }
                }
                // OSC: up to BEL or ESC \.
                Some(']') => {
                    while let Some(d) = chars.next() {
                        if d == '\u{7}' {
                            break;
                        }
                        if d == '\u{1b}' {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => {}
            },
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    out.push('\n');
                }
            }
            '\u{7}' => {}
            c => out.push(c),
        }
    }
    out
}

/// Spaces and box borders around a line of the token (Ink draws boxes).
fn trim_frame(line: &str) -> &str {
    line.trim_matches(|c: char| {
        c.is_whitespace() || c == '|' || ('\u{2500}'..='\u{257f}').contains(&c)
    })
}

/// The last token in `text` (plain text), joined back when the terminal
/// wrapped it on several lines: a token that reaches the end of its line
/// goes on with the next lines made only of token characters.
pub fn find_token(text: &str) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut found = None;
    for (i, line) in lines.iter().enumerate() {
        let mut from = 0;
        while let Some(at) = line[from..].find(PREFIX) {
            let start = from + at;
            let run: String = line[start..]
                .chars()
                .take_while(|c| token_char(*c))
                .collect();
            from = start + run.len();
            let mut token = run;
            if trim_frame(&line[from..]).is_empty() {
                for next in &lines[i + 1..] {
                    let next = trim_frame(next);
                    if next.is_empty() || !next.chars().all(token_char) {
                        break;
                    }
                    token.push_str(next);
                }
            }
            if is_token(&token) {
                found = Some(token);
            }
        }
    }
    found
}

/// The output of `claude setup-token`, kept until it ends (at most the
/// last [`TokenScan::CAP`] bytes).
#[derive(Default)]
pub struct TokenScan {
    raw: String,
}

impl TokenScan {
    pub const CAP: usize = 64 * 1024;

    pub fn push(&mut self, chunk: &str) {
        self.raw.push_str(chunk);
        if self.raw.len() > Self::CAP {
            let mut cut = self.raw.len() - Self::CAP;
            while !self.raw.is_char_boundary(cut) {
                cut += 1;
            }
            self.raw.drain(..cut);
        }
    }

    pub fn token(&self) -> Option<String> {
        find_token(&plain_text(&self.raw))
    }
}

/// Never the token itself in a message.
impl std::fmt::Debug for TokenScan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TokenScan({} bytes)", self.raw.len())
    }
}

/// "Collega con un token": `claude setup-token` in a session of the app's
/// terminal in the kit's home; when it ends, the token it printed is saved
/// and `on_saved` runs. Errors in `lang`.
// Called by the command `agent_token_link` (commands/agents.rs, added by
// the coordinator): unused until then.
#[allow(dead_code)]
#[cfg(unix)]
pub fn link<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    lang: &str,
    on_saved: impl FnOnce() + Send + 'static,
) -> Result<()> {
    use std::sync::{Arc, Mutex, PoisonError};
    use tauri::{Emitter, Manager};

    use crate::editor::pty;

    let exe = crate::setup::catalog::kit_path(kit, super::exe_of("claude").unwrap_or_default());
    if !exe.is_file() {
        bail!(
            "{}",
            crate::i18n::t(lang, "{0} non è installato.", &["Claude Code"])
        );
    }
    let dir = kit.home();
    let mut launch = pty::command(&exe, &dir);
    launch.args.push("setup-token".into());
    launch.title = "Claude Code: setup-token".into();
    // A new token is asked for: not the saved one, nor an API key.
    let mut env = env.clone();
    env.remove(VAR);
    let scan = Arc::new(Mutex::new(TokenScan::default()));
    let (fed, out, end, after) = (Arc::clone(&scan), app.clone(), app.clone(), kit.clone());
    let main = crate::commands::MAIN;
    app.state::<crate::editor::terminal::Consoles>().ptys.open(
        &launch,
        &env,
        &dir,
        lang,
        |info| {
            crate::log::info(&format!("terminal {}: {}", info.id, info.title));
            let _ = app.emit_to(main, pty::OPENED_EVENT, info);
        },
        move |id, chunk| {
            fed.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .push(&chunk.data);
            let _ = out.emit_to(main, &pty::output_event(id), chunk);
        },
        move |info| {
            let token = scan.lock().unwrap_or_else(PoisonError::into_inner).token();
            match token.map(|t| save(&after, &t)) {
                Some(Ok(())) => {
                    crate::log::info("claude: token saved in the kit");
                    on_saved();
                }
                Some(Err(e)) => crate::log::warn(&format!("claude: token not saved: {e:#}")),
                None => crate::log::warn("claude: setup-token ended without a token"),
            }
            let _ = end.emit_to(main, pty::EXIT_EVENT, &info);
        },
    )?;
    Ok(())
}

/// Windows keeps Claude Code's login in the kit's home already.
#[allow(dead_code)]
#[cfg(not(unix))]
pub fn link<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    _kit: &Kit,
    _env: &KitEnv,
    _lang: &str,
    _on_saved: impl FnOnce() + Send + 'static,
) -> Result<()> {
    bail!("claude setup-token: only on macOS")
}

#[cfg(test)]
#[path = "../../../test/rust/agents/claude_token.rs"]
mod tests;
