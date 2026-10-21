//! Agents: their state, launch and sign-out (this file), the alignment of
//! instructions, skills and MCP servers (`sync`), the paths rewritten when
//! the kit moves (`relocate`), Antigravity's login (`keyring`) and the
//! usage of each agent (`stats`).
//!
//! Claude Code, Codex and Antigravity: state, launch, sign-out.
//!
//! - signed in: Claude `.claude\.credentials.json` (OAuth token), the
//!   token of `claude setup-token` in `data/agents/claude/oauth-token`
//!   (macOS, `claude_token`, given as `CLAUDE_CODE_OAUTH_TOKEN`); Codex
//!   `.codex\auth.json` (token, or the API key of `codex login
//!   --with-api-key`);
//!   Antigravity a login saved in the kit (`agents/keyring.rs`) or `GEMINI_API_KEY`
//!   (same sources as nabre/dev-kit `core/stats/*.js`);
//! - launch: the agents are aligned first, the agent opens in a console
//!   window of the app in the chosen folder (on macOS and Linux a session of
//!   the app's terminal, `editor/pty.rs`), and they are aligned again when it
//!   ends (`_agent.cmd` of the kit). Antigravity runs through
//!   `data\kit\bin\agy.cmd` (`agy`, sh, on macOS and Linux), which carries
//!   the login in and out of the system's store;
//! - a watcher thread aligns when an agent of the kit ends anywhere (editor
//!   terminals, consoles) and saves Antigravity's login left in Windows.

pub mod claude_token;
pub mod keyring;
pub mod relocate;
pub mod stats;
pub mod sync;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::setup::catalog::Catalog;

pub const IDS: [&str; 3] = ["claude", "codex", "antigravity"];

/// Program of each agent, relative to `data\apps\<platform>`.
pub fn exe_of(id: &str) -> Option<&'static str> {
    match id {
        "claude" => Some(r"claude\claude.exe"),
        "codex" => Some(r"codex\bin\codex.exe"),
        "antigravity" => Some(r"antigravity\agy.exe"),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub version: Option<String>,
    pub signed_in: bool,
    /// How: "account", "token" (Claude Code on macOS) or "api key" (Codex's
    /// auth.json, Antigravity's Gemini key).
    pub sign_in_kind: Option<String>,
    pub running: bool,
    /// "Collega con un token" is offered (Claude Code on macOS).
    pub token_link: bool,
}

fn json(path: &Path) -> Option<Value> {
    crate::fsx::read_json(path)
}

/// The Gemini API key of the settings (its own owner-only file), if any.
pub fn gemini_key(kit: &Kit) -> Option<String> {
    crate::settings::Settings::load(kit)
        .settings
        .gemini_api_key
        .filter(|k| !k.is_empty())
}

/// The environment agent `id` is started with: the kit's (`env`, or a
/// project's) plus the secrets only that agent gets, never the other
/// programs of the kit (SEC-V6-02): Claude Code's token saved in the kit
/// (D-115), Antigravity's Gemini key. The agent hands them to what it
/// starts itself (its MCP servers, its commands).
pub fn launch_env(kit: &Kit, env: &KitEnv, id: &str) -> KitEnv {
    let mut env = env.clone();
    match id {
        "claude" => claude_token::apply_in(&kit.data(), &mut env),
        "antigravity" => {
            if let Some(key) = gemini_key(kit) {
                env.set(GEMINI_VAR, key);
            }
        }
        _ => {}
    }
    env
}

/// The variable Antigravity reads its Gemini API key from.
pub const GEMINI_VAR: &str = "GEMINI_API_KEY";

fn sign_in(kit: &Kit, id: &str) -> Option<&'static str> {
    let home = kit.home();
    let has = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
    match id {
        "claude" => {
            let creds = json(&home.join(".claude").join(".credentials.json"));
            if has(creds
                .as_ref()
                .and_then(|c| c.pointer("/claudeAiOauth/accessToken"))
                .and_then(Value::as_str))
            {
                Some("account")
            } else if claude_token::read(kit).is_some() {
                Some("token")
            } else {
                None
            }
        }
        "codex" => {
            let auth = json(&home.join(".codex").join("auth.json"));
            if has(auth
                .as_ref()
                .and_then(|a| a.pointer("/tokens/access_token"))
                .and_then(Value::as_str))
            {
                Some("account")
            } else if has(auth
                .as_ref()
                .and_then(|a| a.get("OPENAI_API_KEY"))
                .and_then(Value::as_str))
            {
                // `codex login --with-api-key`: the key in auth.json.
                Some("api key")
            } else {
                None
            }
        }
        "antigravity" => {
            if keyring::has_login(kit) {
                Some("account")
            } else if gemini_key(kit).is_some() {
                Some("api key")
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Ids of the agents of this kit running now.
pub fn running(kit: &Kit) -> BTreeSet<String> {
    let procs = crate::os::processes::running_under(&kit.apps());
    IDS.iter()
        .filter(|id| {
            let exe = crate::setup::catalog::kit_path(kit, exe_of(id).unwrap_or_default());
            procs.iter().any(|p| {
                p.path
                    .display()
                    .to_string()
                    .eq_ignore_ascii_case(&exe.display().to_string())
            })
        })
        .map(|id| (*id).to_owned())
        .collect()
}

pub fn status(kit: &Kit) -> Vec<AgentInfo> {
    let catalog = Catalog::get();
    let now = running(kit);
    IDS.iter()
        .filter_map(|id| catalog.find(id))
        .map(|c| {
            let kind = sign_in(kit, &c.id);
            AgentInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                installed: c.is_installed(kit),
                version: c.installed_version(kit),
                signed_in: kind.is_some(),
                sign_in_kind: kind.map(str::to_owned),
                running: now.contains(&c.id),
                token_link: c.id == "claude" && claude_token::offered(crate::kit::Os::current()),
            }
        })
        .collect()
}

/// The command that starts `id` (Antigravity through the kit's `agy(.cmd)`).
fn command_of(kit: &Kit, id: &str, lang: &str) -> Result<PathBuf> {
    let Some(rel) = exe_of(id) else {
        bail!("unknown agent: {id}")
    };
    let exe = crate::setup::catalog::kit_path(kit, rel);
    if !exe.is_file() {
        let name = Catalog::get().find(id).map_or(id, |c| c.name.as_str());
        bail!("{}", crate::i18n::t(lang, "{0} non è installato.", &[name]));
    }
    Ok(if id == "antigravity" {
        crate::router::bins::agent_launcher(kit, id).unwrap_or(exe)
    } else {
        exe
    })
}

/// Aligns the agents, then opens `id` in a kit console in `dir` (Windows)
/// or a session of the app's terminal with the agent as its program (macOS
/// and Linux, D-109); aligns again when that console or session ends.
pub fn launch<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    id: &str,
    dir: &Path,
    lang: &str,
) -> Result<()> {
    let command = command_of(kit, id, lang)?;
    let report = sync::run(kit, Some(dir));
    for w in &report.warnings {
        crate::log::warn(&format!("before {id}: {w}"));
    }
    let (after, after_dir) = (kit.clone(), dir.to_path_buf());
    // Claude Code's token or the Gemini key, in this agent's environment
    // only (SEC-V6-02).
    let env = launch_env(kit, env, id);
    // Not aligned again when the console ends because the drive is being
    // removed (BUG-V6-15).
    let closing = closing_of(app);
    crate::editor::terminal::open_command(app, kit, &env, dir, &command, move || {
        let _ = closing.run(|| sync::run(&after, Some(&after_dir)));
    })
}

/// Signs out in the kit: the saved login is removed (Claude's and Codex's
/// files, Antigravity's saved login). The account's sessions on the
/// provider's site are not revoked: do it from the account settings.
pub fn sign_out(kit: &Kit, env: &KitEnv, id: &str) -> Result<()> {
    let home = kit.home();
    match id {
        "claude" => {
            remove_if_exists(&home.join(".claude").join(".credentials.json"))?;
            claude_token::remove(kit)
        }
        "codex" => {
            let codex = crate::setup::catalog::kit_path(kit, "codex/bin/codex.exe");
            if codex.is_file() {
                let status = crate::os::jobs::kit_command(&codex, env)
                    .arg("logout")
                    .status()
                    .context("codex logout")?;
                if status.success() {
                    return Ok(());
                }
            }
            remove_if_exists(&home.join(".codex").join("auth.json"))
        }
        "antigravity" => {
            keyring::save(kit)?;
            remove_if_exists(&kit.data().join("antigravity").join("keyring.json"))
        }
        _ => bail!("unknown agent: {id}"),
    }
}

fn remove_if_exists(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| path.display().to_string())
        }
        _ => Ok(()),
    }
}

/// "Rimuovi l'unità" under way (BUG-V6-15): the alignments and saves the
/// app starts by itself (the watcher, the end of an agent's console) are
/// skipped from the moment it begins, and it waits for the one already
/// running, so nothing writes to the drive once the interface says it can
/// be removed. One per app (`AppState::closing`), shared by clones.
#[derive(Debug, Default, Clone)]
pub struct Closing(std::sync::Arc<ClosingState>);

#[derive(Debug, Default)]
struct ClosingState {
    /// Background work running now; `closing` is read and set under it.
    busy: std::sync::Mutex<Busy>,
    idle: std::sync::Condvar,
}

#[derive(Debug, Default)]
struct Busy {
    running: usize,
    closing: bool,
}

/// Counts one background work until dropped (also on a panic).
struct Running<'a>(&'a ClosingState);

impl Drop for Running<'_> {
    fn drop(&mut self) {
        let mut busy = self.0.lock();
        busy.running -= 1;
        if busy.running == 0 {
            self.0.idle.notify_all();
        }
    }
}

impl ClosingState {
    fn lock(&self) -> std::sync::MutexGuard<'_, Busy> {
        self.busy
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Closing {
    /// Runs `work` (an alignment, a save of the login) unless the drive is
    /// being removed; `None` when skipped.
    pub fn run<T>(&self, work: impl FnOnce() -> T) -> Option<T> {
        {
            let mut busy = self.0.lock();
            if busy.closing {
                return None;
            }
            busy.running += 1;
        }
        let _running = Running(&self.0);
        Some(work())
    }

    /// From now on background work is skipped.
    pub fn begin(&self) {
        self.0.lock().closing = true;
    }

    /// Waits for the background work already running, at most `limit`;
    /// `false` if some is still running then.
    pub fn wait_idle(&self, limit: Duration) -> bool {
        let busy = self.0.lock();
        let (busy, _) = self
            .0
            .idle
            .wait_timeout_while(busy, limit, |b| b.running > 0)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        busy.running == 0
    }

    /// The user goes on working after "Rimuovi l'unità" (an agent started
    /// again): background work resumes.
    pub fn reopen(&self) {
        self.0.lock().closing = false;
    }
}

/// The `Closing` of the app, if it runs (`None` in the command line).
fn closing_of<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Closing {
    use tauri::Manager;
    app.try_state::<crate::commands::AppState>()
        .map(|s| s.closing.clone())
        .unwrap_or_default()
}

/// Background: when an agent of the kit ends anywhere, align the agents and
/// save Antigravity's login left in Windows; nothing while the drive is
/// being removed (`closing`).
pub fn watch(kit: Kit, closing: Closing) {
    std::thread::spawn(move || {
        let mut before = BTreeSet::new();
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let now = running(&kit);
            let ended: Vec<&String> = before.difference(&now).collect();
            if !ended.is_empty() {
                crate::log::info(&format!("agents ended: {ended:?}"));
                let _ = closing.run(|| sync::run(&kit, None));
            }
            // Antigravity's login still in Windows and no agy of the kit
            // running: saved now (save waits for a fresh "starting" mark).
            if keyring::pending(&kit) && !now.contains("antigravity") {
                if let Some(Err(e)) = closing.run(|| keyring::save(&kit)) {
                    crate::log::warn(&format!("antigravity login: {e:#}"));
                }
            }
            before = now;
        }
    });
}

#[cfg(test)]
#[path = "../../../test/rust/agents/mod.rs"]
mod tests;
