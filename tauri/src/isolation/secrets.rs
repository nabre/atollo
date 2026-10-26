//! The kit's secrets for the VMs (D-172): what the proxy of Docker
//! Sandboxes adds to a VM's requests (`sbx secret`), never inside the VM.
//!
//! `sbx` keeps its secrets on the PC, not in the kit (Windows' Credential
//! Manager, `com.docker.sandboxes:…`: tried with sbx 0.49.2). So the kit
//! gives it no copy: it registers a command of its own (`--command`),
//! which `sbx` runs on the PC when it needs the value: `Atollo.exe secret
//! github` prints the token the kit's `gh` holds, and only when the kit's
//! `sbx` asks ([`asked_by_sbx`]). Signed out in the kit, the command fails
//! and nothing is added. On another PC, or after the kit moved, the
//! command is registered again. The Gemini key of the settings goes the
//! same way to the project's sandbox Antigravity runs in (`secret gemini`,
//! `sbx`'s `google` service, D-179).
//!
//! Never global: a VM runs the project's code, which could act as the
//! user on GitHub through the proxy. The secret is given to the sandbox of
//! a project the user allowed (`github_in_vm`), that one only
//! (`--sandbox`), and taken back when it is turned off. A secret `sbx` has
//! that Atollo did not register (the user's own) is never touched: Atollo
//! notes, per PC and sandbox, the commands it registered
//! (`data\app\sandboxes\secrets.json`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::kit::Kit;

/// True when `sbx` (the kit's) is among the processes that started this
/// one (`ancestors`, nearest first): `secret github` answers it only, not
/// an agent or a project's program on the PC that runs the app (D-172).
pub fn asked_by_sbx(ancestors: &[&crate::platform::processes::Proc], sbx: &Path) -> bool {
    let key = crate::platform::processes::path_key(&sbx.display().to_string());
    ancestors
        .iter()
        .any(|p| crate::platform::processes::path_key(&p.path.display().to_string()) == key)
}

/// What Atollo registered on one PC for one service, for one sandbox
/// (empty: global, a note of a first version, only ever taken back).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Noted {
    pub computer: String,
    #[serde(default)]
    pub sandbox: String,
    pub command: String,
}

fn file(kit: &Kit) -> std::path::PathBuf {
    super::kit::root(kit).join("secrets.json")
}

/// The commands Atollo registered, by service: every entry that can be
/// read, one that cannot left out (never the whole file).
pub fn noted(kit: &Kit) -> BTreeMap<String, Vec<Noted>> {
    let Some(value) = std::fs::read_to_string(file(kit))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
    else {
        return BTreeMap::new();
    };
    let Some(map) = value.as_object() else {
        return BTreeMap::new();
    };
    map.iter()
        .map(|(service, list)| {
            let entries = list
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| serde_json::from_value::<Noted>(e.clone()).ok())
                .collect();
            (service.clone(), entries)
        })
        .collect()
}

/// A nonce for one registration: 32 random bytes in hex.
pub fn new_nonce() -> anyhow::Result<String> {
    crate::editor::random_token()
}

/// A kit secret the proxy of `sbx` adds to a sandbox's requests, never
/// given to the sandbox itself (LEVELS.md §11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Secret {
    /// The kit's GitHub token, for a project that chose GitHub in its VM
    /// (D-172): the `github` service of `sbx`.
    Github,
    /// The Gemini API key of the settings, for the project's sandbox
    /// Antigravity runs in (D-179, LEVELS.md §9): the `google` service of
    /// `sbx` (`GEMINI_API_KEY`, `GOOGLE_API_KEY`).
    Gemini,
}

impl Secret {
    /// The service of `sbx secret`, also the key of its notes.
    pub fn service(self) -> &'static str {
        match self {
            Secret::Github => "github",
            Secret::Gemini => "google",
        }
    }

    /// The word of the app's command `secret <word> <nonce>`.
    pub fn word(self) -> &'static str {
        match self {
            Secret::Github => "github",
            Secret::Gemini => "gemini",
        }
    }

    /// The secret of the app's command `secret <word>`.
    pub fn from_word(word: &str) -> Option<Secret> {
        [Secret::Github, Secret::Gemini]
            .into_iter()
            .find(|s| s.word() == word)
    }
}

/// True when `nonce` is in a command Atollo registered for `secret`
/// (D-172): `secret <word>` answers only a command `sbx` has from Atollo.
pub fn nonce_known(kit: &Kit, secret: Secret, nonce: &str) -> bool {
    let well_formed = nonce.len() == 64 && nonce.bytes().all(|b| b.is_ascii_hexdigit());
    let end = format!(" secret {} {nonce}", secret.word());
    well_formed
        && noted(kit)
            .get(secret.service())
            .into_iter()
            .flatten()
            .any(|n| n.command.ends_with(&end))
}

/// What Atollo noted for `service` on `computer` for `sandbox`.
pub fn mine(kit: &Kit, service: &str, computer: &str, sandbox: &str) -> Option<Noted> {
    noted(kit)
        .remove(service)?
        .into_iter()
        .find(|n| n.computer == computer && n.sandbox == sandbox)
}

/// Notes `command` registered for `service` on `computer` for `sandbox`
/// (one entry each), or with `None` that it was taken back.
pub fn note(
    kit: &Kit,
    service: &str,
    computer: &str,
    sandbox: &str,
    command: Option<&str>,
) -> anyhow::Result<()> {
    // One writer at a time: two notes read and written together would
    // lose one, and a registration without its note is never taken back.
    static WRITE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _one = WRITE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut all = noted(kit);
    let list = all.entry(service.to_owned()).or_default();
    list.retain(|n| !(n.computer == computer && n.sandbox == sandbox));
    if let Some(command) = command {
        list.push(Noted {
            computer: computer.to_owned(),
            sandbox: sandbox.to_owned(),
            command: command.to_owned(),
        });
    }
    let path = file(kit);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    // Readable by its owner only: its nonces open `secret github`.
    crate::platform::files::replace_private(&path, &(serde_json::to_string_pretty(&all)? + "\n"))
}

/// The start of the command `sbx` runs for `secret`: the app with `secret
/// <word>`, its path quoted as `sbx` splits the command (a path with
/// spaces unquoted fails: tried), for the PC's system.
pub fn prefix(exe: &Path, windows: bool, secret: Secret) -> String {
    let path = exe.display().to_string();
    let quoted = if windows {
        format!("\"{path}\"")
    } else {
        crate::platform::sh_quote(&path)
    };
    format!("{quoted} secret {}", secret.word())
}

/// The whole command: [`prefix`] and the registration's `nonce`.
pub fn command(exe: &Path, windows: bool, secret: Secret, nonce: &str) -> String {
    format!("{} {nonce}", prefix(exe, windows, secret))
}

/// `sbx secret set` of `secret` for the sandbox `sandbox` only, through
/// `command` (Atollo's, [`command`]): `sbx` runs it on the PC when its
/// proxy needs the value, which is never an argument here nor in the VM.
pub fn set_args(secret: Secret, sandbox: &str, command: &str) -> Vec<String> {
    [
        "secret",
        "set",
        secret.service(),
        "--sandbox",
        sandbox,
        "--command",
        command,
    ]
    .map(String::from)
    .into()
}

/// `sbx secret rm` of `secret` for the sandbox `sandbox` (`--force`: no
/// question on a terminal).
pub fn rm_args(secret: Secret, sandbox: &str) -> Vec<String> {
    [
        "secret",
        "rm",
        secret.service(),
        "--sandbox",
        sandbox,
        "--force",
    ]
    .map(String::from)
    .into()
}

/// What to do with `service`, from whether `sbx` lists it (`listed`) and
/// what Atollo noted for this PC (`mine`) against the start of the command
/// now (`prefix`, [`prefix`]: the nonce differs at each
/// registration).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Nothing: registered already, or the user's own.
    Keep,
    /// Registered for the first time.
    Register,
    /// Registered again over Atollo's own (the kit moved).
    Replace,
}

pub fn step(listed: bool, mine: Option<&Noted>, prefix: &str) -> Step {
    match (listed, mine) {
        (false, _) => Step::Register,
        (true, Some(n)) if !n.command.starts_with(&format!("{prefix} ")) => Step::Replace,
        (true, _) => Step::Keep,
    }
}

/// True when `sbx secret ls` lists the secret of `service` for `scope`
/// (a sandbox's name, or `(global)`).
pub fn listed(ls: &str, service: &str, scope: &str) -> bool {
    ls.lines().any(|l| {
        let words: Vec<&str> = l.split_whitespace().collect();
        words.first() == Some(&scope) && words.contains(&service)
    })
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/secrets.rs"]
mod tests;
