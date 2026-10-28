//! Agents: their state as the app shows it (this file), the alignment of
//! instructions, skills and MCP servers (`sync`), the paths rewritten when
//! the kit moves (`relocate`), Antigravity's login saved in the kit by an
//! earlier Atollo (`keyring`) and the usage of each agent (`stats`).
//!
//! Claude Code, Codex, Antigravity and Pi run only in a VM (D-179,
//! LEVELS.md §7): nothing of an agent runs on the PC, and the kit keeps
//! only Pi's and Antigravity's builds for the VM's Linux (Q11; Claude Code
//! and Codex come with their `sbx` sandboxes). Their state ([`status`]) is
//! the VM's: whether one can start now (the VM on), what the PC can tell of
//! its sign-in there (the Gemini key through `sbx`'s proxy, Pi on the
//! kit's local models; otherwise the agent signs in inside the VM at its
//! first start). A sign-in of the PC an earlier Atollo saved in the kit's
//! data stays there untouched until «Esci» removes it ([`sign_out`]): the
//! agents in the VM never get it (I5).

pub mod claude_token;
pub mod github_mcp;
pub mod keyring;
pub mod presets;
pub mod relocate;
pub mod stats;
pub mod sync;

use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::kit::Kit;
use crate::levels::{Agent, VmState};

pub const IDS: [&str; 4] = ["claude", "codex", "antigravity", "pi"];

/// Program of each agent, relative to `data\apps\<platform>` (the VM's
/// Linux: without `.exe`, `setup::kit_path`).
pub fn exe_of(id: &str) -> Option<&'static str> {
    match id {
        "claude" => Some(r"claude\claude.exe"),
        "codex" => Some(r"codex\bin\codex.exe"),
        "antigravity" => Some(r"antigravity\agy.exe"),
        "pi" => Some(r"pi\pi.exe"),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    /// It can be started in the VM: its build for the VM's Linux is in the
    /// kit (D-184: every agent runs in the project's sandbox).
    pub installed: bool,
    /// The version of its build for the VM's Linux.
    pub version: Option<String>,
    /// Signed in for the VM, as far as the PC can tell (LEVELS.md §13 item
    /// 12): Antigravity with the Gemini key of the settings, through
    /// `sbx`'s proxy; Pi on the kit's local models. Otherwise the agent
    /// signs in inside the VM, at its first start there.
    pub signed_in: bool,
    /// "api key" (Antigravity's Gemini key) or "local" (Pi on a local
    /// model).
    pub sign_in_kind: Option<String>,
    /// A sign-in of the PC an earlier Atollo saved in the kit's data:
    /// never given to the agents in the VM; «Esci» removes it.
    pub kit_login: bool,
    /// It can be started now: the VM is on (LEVELS.md §7).
    pub available: bool,
    /// Why not, when not: the VM's blockers and what to do.
    pub unavailable_reason: Option<String>,
}

fn json(path: &Path) -> Option<Value> {
    crate::common::fsx::read_json(path)
}

/// The Gemini API key of the settings (its own owner-only file), if any.
pub fn gemini_key(kit: &Kit) -> Option<String> {
    crate::common::settings::Settings::load(kit)
        .settings
        .gemini_api_key
        .filter(|k| !k.is_empty())
}

/// The variable Antigravity reads its Gemini API key from.
pub const GEMINI_VAR: &str = "GEMINI_API_KEY";

/// How agent `id` is signed in in the kit's own data (the kit's home on
/// the PC, `data/home`), if it is: what an earlier Atollo saved there, and
/// what the usage of the agents reads (`stats`).
pub(crate) fn sign_in(kit: &Kit, id: &str) -> Option<&'static str> {
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
        "antigravity" => keyring::has_login(kit).then_some("account"),
        "pi" => pi_sign_in(kit),
        _ => None,
    }
}

/// Pi: a provider of the cloud in `auth.json` (signed in with an account,
/// or an API key), else the local model the user chose for it.
fn pi_sign_in(kit: &Kit) -> Option<&'static str> {
    let auth = json(&sync::pi::dir(&kit.home()).join("auth.json"));
    let entries = auth.as_ref().and_then(Value::as_object);
    let kind = |k: &str| {
        entries
            .into_iter()
            .flat_map(|o| o.values())
            .any(|c| c.get("type").and_then(Value::as_str) == Some(k))
    };
    if kind("oauth") {
        Some("account")
    } else if kind("api_key") {
        Some("api key")
    } else {
        sync::pi::local_model(kit).map(|_| "local")
    }
}

/// What the PC can tell of `agent`'s sign-in for the VM: Antigravity's
/// Gemini key (through `sbx`'s proxy, LEVELS.md §9), Pi's local model
/// (D-178); `None`: it signs in inside the VM.
fn vm_sign_in(kit: &Kit, agent: Agent) -> Option<&'static str> {
    match agent {
        Agent::Antigravity => gemini_key(kit).map(|_| "api key"),
        Agent::Pi => sync::pi::local_model(kit).map(|_| "local"),
        Agent::Claude | Agent::Codex => None,
    }
}

/// A sign-in of the PC saved in the kit's data: an account or a key of
/// Claude Code, Codex or Pi, a Google login of Antigravity (Pi's local
/// model is no sign-in).
fn kit_login(kit: &Kit, agent: Agent) -> bool {
    sign_in(kit, agent.id()).is_some_and(|k| k != "local")
}

/// The agents as the app shows them, with the VM as `vm` says (LEVELS.md
/// §7): available only while it is on, else with the reason in `lang`.
pub fn status(kit: &Kit, vm: &VmState, lang: &str) -> Vec<AgentInfo> {
    let guest = crate::isolation::vm::guest_target(kit.target()).map(|t| kit.for_target(t));
    let reason = crate::levels::agents::off_reason(vm, lang);
    crate::levels::agents::catalog_agents()
        .map(|(c, agent)| {
            let kind = vm_sign_in(kit, agent);
            AgentInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                installed: guest.as_ref().is_some_and(|g| c.is_installed(g)),
                version: guest.as_ref().and_then(|g| c.installed_version(g)),
                signed_in: kind.is_some(),
                sign_in_kind: kind.map(str::to_owned),
                kit_login: kit_login(kit, agent),
                available: crate::levels::agents_enabled(vm),
                unavailable_reason: reason.clone(),
            }
        })
        .collect()
}

/// «Esci»: the sign-in of the PC saved in the kit's data is removed
/// (Claude's credentials and token, Codex's and Pi's `auth.json`,
/// Antigravity's saved login). Files only: no agent's program runs on the
/// PC (LEVELS.md E17). The account's sessions on the provider's site are
/// not revoked: do it from the account settings.
pub fn sign_out(kit: &Kit, id: &str) -> Result<()> {
    let home = kit.home();
    match id {
        "claude" => {
            remove_if_exists(&home.join(".claude").join(".credentials.json"))?;
            claude_token::remove(kit)
        }
        "codex" => remove_if_exists(&home.join(".codex").join("auth.json")),
        "antigravity" => remove_if_exists(&kit.data().join("antigravity").join("keyring.json")),
        // The providers of the cloud only: the local models need no login.
        "pi" => remove_if_exists(&sync::pi::dir(&home).join("auth.json")),
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

#[cfg(test)]
#[path = "../../../test/rust/agents/mod.rs"]
mod tests;
