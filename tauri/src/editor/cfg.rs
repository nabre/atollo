//! Settings of the integrated editor, written before every start.
//!
//! File: `data\vscodium-server\data\Machine\settings.json`, the server-side
//! settings that VSCodium applies to every window of the server (the user's
//! own settings stay in the editor):
//! - only Atollo's terminal profiles, rewritten at every start (D-179,
//!   LEVELS.md §13 item 10): the terminals level 1, the kit's shells on the
//!   PC; the agents (level 3) and the tasks (level 2, `automationProfile`)
//!   through the request client `launch.ps1`/`launch.sh` (§4.3); the PC's
//!   profiles hidden, any other profile removed;
//! - a few defaults set only when missing;
//! - a file with comments is read without them; one that is no JSON is
//!   moved aside and written anew.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

use crate::common::fsx::write_atomic;
use crate::kit::{Kit, Os};

/// The profiles VSCodium detects on Windows, hidden: only Atollo's are
/// offered (D-179).
const HIDDEN: &[&str] = &[
    "PowerShell",
    "Windows PowerShell",
    "Command Prompt",
    "Git Bash",
    "JavaScript Debug Terminal",
];

/// The shells VSCodium detects on macOS and Linux: those of /etc/shells,
/// named after their file (macOS lists csh, dash, ksh and tcsh too).
const HIDDEN_UNIX: &[&str] = &[
    "bash",
    "zsh",
    "fish",
    "tmux",
    "pwsh",
    "sh",
    "csh",
    "tcsh",
    "ksh",
    "dash",
    "rbash",
    "JavaScript Debug Terminal",
];

pub const DEFAULT_PROFILE: &str = "Atollo";
/// The default on macOS and Linux without the kit's PowerShell.
pub const DEFAULT_PROFILE_UNIX: &str = "Atollo - Bash";
/// The shells of macOS and Linux the kit starts (never one of the user's
/// PATH): bash for the kit's `bashrc`, zsh on macOS.
pub const BASH: &str = "/bin/bash";
pub const ZSH: &str = "/bin/zsh";

/// The editor's and its terminal's font: Geist Mono, which the editor
/// window loads itself (`window.rs`), with the system's monospace behind.
pub const EDITOR_FONT: &str = "'Geist Mono', ui-monospace, Consolas, 'Courier New', monospace";

/// The suffix of the editor's per-system settings
/// (`terminal.integrated.profiles.<os>`).
fn os_key(os: Os) -> &'static str {
    match os {
        Os::Win32 => "windows",
        Os::Darwin => "osx",
        Os::Linux => "linux",
    }
}

fn hidden(os: Os) -> &'static [&'static str] {
    if os.is_windows() {
        HIDDEN
    } else {
        HIDDEN_UNIX
    }
}

/// The kit's themes (editor/theme.rs) in place of VSCodium's own: a theme
/// setting missing or on one of these becomes Atollo's; any other theme is
/// the user's choice and stays. VSCodium 1.140 calls its themes "Dark
/// Modern" and "Light Modern" (older ids: "Default ...").
const THEMES: &[(&str, &[&str], &str)] = &[
    (
        "workbench.colorTheme",
        &[
            "Dark Modern",
            "Default Dark Modern",
            "Dark+",
            "Default Dark+",
        ],
        crate::editor::theme::DARK,
    ),
    (
        "workbench.preferredDarkColorTheme",
        &[
            "Dark Modern",
            "Default Dark Modern",
            "Dark+",
            "Default Dark+",
        ],
        crate::editor::theme::DARK,
    ),
    (
        "workbench.preferredLightColorTheme",
        &[
            "Light Modern",
            "Default Light Modern",
            "Light+",
            "Default Light+",
        ],
        crate::editor::theme::LIGHT,
    ),
];

/// Color blocks the kit once wrote over VSCodium's themes, before the
/// Atollo themes (`workbench.colorCustomizations`): removed, the user's
/// other colors stay.
const OLD_BLOCKS: &[&str] = &["[Default Dark Modern]", "[Default Light Modern]"];

fn set_themes(existing: &Map<String, Value>, out: &mut Map<String, Value>) {
    for (setting, codium, atollo) in THEMES {
        let own = match existing.get(*setting) {
            Some(Value::String(t)) => codium.contains(&t.as_str()),
            _ => true,
        };
        if own {
            out.insert((*setting).into(), json!(atollo));
        }
    }
    let key = "workbench.colorCustomizations";
    if let Some(Value::Object(colors)) = existing.get(key) {
        let mut colors = colors.clone();
        for block in OLD_BLOCKS {
            colors.remove(*block);
        }
        if colors.is_empty() {
            out.remove(key);
        } else {
            out.insert(key.into(), Value::Object(colors));
        }
    }
}

pub fn settings_path(kit: &Kit) -> PathBuf {
    crate::common::fsx::join_rel(kit.data(), "vscodium-server/data/Machine/settings.json")
}

/// How the kit's PowerShell is started (editor, console windows, tasks).
pub fn pwsh_args(kit: &Kit) -> Vec<String> {
    vec![
        "-NoLogo".into(),
        "-NoProfile".into(),
        "-NoExit".into(),
        "-ExecutionPolicy".into(),
        "Bypass".into(),
        "-File".into(),
        kit.kit_files().join("profile.ps1").display().to_string(),
    ]
}

fn profile(path: &Path, args: &[String], icon: &str) -> Value {
    json!({ "path": path.display().to_string(), "args": args, "icon": icon, "overrideName": true })
}

/// How the editor runs a request's client (LEVELS.md §4.3): the kit's
/// PowerShell with `launch.ps1`, or a shell with `launch.sh` where the
/// kit's PowerShell is missing (Git Bash of the kit on Windows, the
/// system's `/bin/sh` elsewhere).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Client {
    Pwsh(PathBuf),
    Sh(PathBuf),
}

/// The client of `kit`; `None` on Windows with neither the kit's PowerShell
/// nor its Git Bash: then no task and no agent of the editor runs (E18).
pub fn client(kit: &Kit) -> Option<Client> {
    let pwsh = crate::setup::kit_path(kit, r"pwsh\pwsh.exe");
    if pwsh.is_file() {
        return Some(Client::Pwsh(pwsh));
    }
    if kit.os().is_windows() {
        let bash = crate::setup::kit_path(kit, r"git\bin\bash.exe");
        return bash.is_file().then_some(Client::Sh(bash));
    }
    Some(Client::Sh(PathBuf::from(SH)))
}

/// The system's sh that runs `launch.sh` on macOS and Linux.
pub const SH: &str = "/bin/sh";

impl Client {
    /// The program of the profile.
    pub fn program(&self) -> &Path {
        match self {
            Client::Pwsh(p) | Client::Sh(p) => p,
        }
    }

    /// Its arguments for `origin` (the words of `launch.ps1`, e.g.
    /// `["-Origin", "editorAgentProfile", "-Agent", "claude"]`, given to
    /// `launch.sh` without the names: `["editorAgentProfile", "claude"]`).
    pub fn args(&self, kit: &Kit, origin: &[(&str, &str)]) -> Vec<String> {
        match self {
            Client::Pwsh(_) => {
                let mut args: Vec<String> = [
                    "-NoLogo",
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ]
                .map(String::from)
                .into();
                args.push(kit.kit_files().join("launch.ps1").display().to_string());
                for (name, value) in origin {
                    args.push(format!("-{name}"));
                    args.push((*value).to_owned());
                }
                args
            }
            Client::Sh(_) => {
                let mut args = vec![kit.kit_files().join("launch.sh").display().to_string()];
                args.extend(origin.iter().map(|(_, v)| (*v).to_owned()));
                args
            }
        }
    }
}

/// The arguments of bash with the kit's `bashrc`.
pub fn bash_args(kit: &Kit) -> Vec<String> {
    let rc = kit.kit_files().join("bashrc").display().to_string();
    vec!["--rcfile".into(), rc, "-i".into()]
}

/// The agents' profiles: name, agent id, icon.
const AGENT_PROFILES: [(&str, &str, &str); 4] = [
    ("Claude Code", "claude", "sparkle"),
    ("Codex", "codex", "hubot"),
    ("Antigravity", "antigravity", "rocket"),
    ("Pi", "pi", "symbol-constant"),
];

/// True when `agent` can start in the VM (LEVELS.md §7, Q11): Claude Code
/// and Codex always (the `sbx` images bring them), Pi and Antigravity once
/// their build for the VM's Linux is in the kit. What the PC's folder has
/// does not count: no agent runs on the PC.
fn has_agent(kit: &Kit, agent: &str) -> bool {
    let Some(agent) = crate::levels::Agent::from_id(agent) else {
        return false;
    };
    if agent.runner() == crate::levels::AgentRunner::SbxAgent {
        return true;
    }
    crate::isolation::vm::guest_target(kit.target()).is_some_and(|t| {
        crate::setup::Catalog::get()
            .find(agent.id())
            .is_some_and(|c| c.is_installed(&kit.for_target(t)))
    })
}

/// The kit profiles for what is installed now, for the kit's system
/// (D-179): the terminals are level 1, the kit's shells on the PC as they
/// are (the editor server's environment is `environment(Pc{Manual})`, the
/// kit's profile applies the project's lines of the folder); the agents
/// are level 3, through the request client ([`client`]) to their VM, and
/// only while `agents` (`levels::agents_enabled`, LEVELS.md §7: absent
/// without the VM, I6); without a client no agent profile.
pub fn kit_profiles(kit: &Kit, system_root: &Path, agents: bool) -> Map<String, Value> {
    let os = kit.os();
    let app = |rel: &str| crate::setup::kit_path(kit, rel);
    let mut out = Map::new();
    let pwsh = app(r"pwsh\pwsh.exe");
    if pwsh.is_file() {
        out.insert(
            "Atollo".into(),
            profile(&pwsh, &pwsh_args(kit), "terminal-powershell"),
        );
    }
    if os.is_windows() {
        let bash = app(r"git\bin\bash.exe");
        if bash.is_file() {
            out.insert(
                "Atollo - Git Bash".into(),
                profile(&bash, &bash_args(kit), "terminal-bash"),
            );
        }
        let cmd = system_root.join(r"System32\cmd.exe");
        out.insert(
            "Atollo - cmd".into(),
            profile(&cmd, &["/d".into(), "/k".into()], "terminal-cmd"),
        );
    } else {
        // The system's bash with the kit's bashrc (bash 3.2 on macOS reads
        // it too, D-105); the environment is the kit's, from the server.
        out.insert(
            "Atollo - Bash".into(),
            profile(Path::new(BASH), &bash_args(kit), "terminal-bash"),
        );
        if os.is_macos() {
            // Interactive, not a login shell: path_helper of
            // /etc/zprofile would put the system's folders first.
            out.insert(
                "Atollo - zsh".into(),
                profile(Path::new(ZSH), &["-i".into()], "terminal"),
            );
        }
    }
    if let Some(client) = client(kit).filter(|_| agents) {
        for (name, id, icon) in AGENT_PROFILES {
            if has_agent(kit, id) {
                let args = client.args(kit, &[("Origin", "editorAgentProfile"), ("Agent", id)]);
                out.insert(name.into(), profile(client.program(), &args, icon));
            }
        }
    }
    out
}

/// The editor's tasks (`automationProfile`, LEVELS.md §6.3): through the
/// request client, which VSCodium gives `-Command <line>` (PowerShell on
/// Windows) or `-c <line>`. Without a client (Windows with neither the
/// kit's PowerShell nor its Git Bash) a task never runs on the PC: Windows
/// PowerShell says why and fails, the task's line after a `#` (E18).
fn tasks_profile(kit: &Kit, system_root: &Path) -> Value {
    match client(kit) {
        Some(client) => json!({
            "path": client.program().display().to_string(),
            "args": client.args(kit, &[("Origin", "editorTask")]),
        }),
        None => {
            let ps = system_root
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe");
            let say = format!(
                "[Console]::Error.WriteLine('{}'); exit 1 #",
                NO_CLIENT.replace('\'', "''")
            );
            json!({
                "path": ps.display().to_string(),
                "args": ["-NoLogo", "-NoProfile", "-Command", say],
            })
        }
    }
}

/// What a task says without a request client (its `'` doubled in
/// PowerShell's quotes).
pub const NO_CLIENT: &str =
    "Atollo: senza il PowerShell del kit i task e gli agenti dell'editor non partono: installalo da Strumenti.";

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Written,
    Unchanged,
    /// Invalid JSON, even without comments: moved aside to this file, and
    /// the settings written anew.
    Replaced(PathBuf),
}

/// Merges the kit's settings into `existing` (the parsed file). The
/// terminal settings are those of the kit's system (`.windows`, `.osx`,
/// `.linux`): the same file serves a kit used on several systems, each
/// writes its own and leaves the others. Only Atollo's profiles (D-179):
/// the system's detected ones hidden, any other profile removed; the
/// default kept only when it is one of Atollo's (an agent's, without the
/// VM, falls back to «Atollo»). `agents`: the agents' profiles too.
pub fn merge(
    existing: &Map<String, Value>,
    kit: &Kit,
    system_root: &Path,
    agents: bool,
) -> Map<String, Value> {
    let os = kit.os();
    let key = |name: &str| format!("terminal.integrated.{name}.{}", os_key(os));
    let mut out = existing.clone();
    let profiles_key = key("profiles");
    let mut profiles = Map::new();
    for name in hidden(os) {
        profiles.insert((*name).into(), Value::Null);
    }
    let kit_profiles = kit_profiles(kit, system_root, agents);
    let names: Vec<String> = kit_profiles.keys().cloned().collect();
    profiles.extend(kit_profiles);
    out.insert(profiles_key, Value::Object(profiles));

    let default_key = key("defaultProfile");
    let keep_default =
        matches!(existing.get(&default_key), Some(Value::String(d)) if names.contains(d));
    // Without the kit's PowerShell, bash on macOS and Linux.
    let default = [DEFAULT_PROFILE]
        .into_iter()
        .chain((!os.is_windows()).then_some(DEFAULT_PROFILE_UNIX))
        .find(|d| names.iter().any(|n| n == d));
    match (keep_default, default) {
        (true, _) => {}
        (false, Some(default)) => {
            out.insert(default_key, json!(default));
        }
        (false, None) => {
            out.remove(&default_key);
        }
    }
    out.insert(key("automationProfile"), tasks_profile(kit, system_root));
    if os.is_windows() {
        out.insert("terminal.integrated.useWslProfiles".into(), json!(false));
    }
    // VSCodium drops NODE_OPTIONS from the terminals' environment (one of its
    // "dangerous" variables): without `loopback.js` a dev server started in
    // the editor's terminal or a task listens on the network and Windows
    // Firewall asks for a consent that needs an administrator. The settings'
    // variables are added after that cleanup; the user's others are kept.
    // macOS and Linux get it too: the servers stay on 127.0.0.1 there as well.
    let env_key = key("env");
    let mut vars = match existing.get(&env_key) {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    vars.insert("NODE_OPTIONS".into(), json!(crate::env::node_options(kit)));
    out.insert(env_key, Value::Object(vars));

    set_themes(existing, &mut out);

    let projects = kit.projects().display().to_string();
    for (key, value) in [
        ("files.dialog.defaultPath", json!(projects)),
        ("git.defaultCloneDirectory", json!(projects)),
        ("workbench.iconTheme", json!("vscode-icons")),
        ("workbench.startupEditor", json!("none")),
        ("telemetry.telemetryLevel", json!("off")),
        ("window.autoDetectColorScheme", json!(true)),
        // VSCodium 1.140's floating parts, as the app's pages (off by default).
        ("workbench.experimental.modernUI", json!(true)),
        ("update.mode", json!("none")),
        ("extensions.autoUpdate", json!(false)),
        // Atollo's monospace (MARKETING.md, section 6). Ligatures off, as
        // VSCodium's default and the app's own views
        // (--vscode-editor-font-feature-settings): code reads as typed.
        ("editor.fontFamily", json!(EDITOR_FONT)),
        ("editor.fontLigatures", json!(false)),
        ("terminal.integrated.fontFamily", json!(EDITOR_FONT)),
    ] {
        out.entry(key).or_insert(value);
    }
    out
}

/// `text` without the comments (`//`, `/* */`) and the trailing commas
/// VSCodium's settings may have (JSON with comments), strings left as they
/// are.
pub fn strip_jsonc(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut in_string = false;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if c == '\\' && i + 1 < chars.len() {
                out.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match (c, chars.get(i + 1)) {
            ('"', _) => {
                in_string = true;
                out.push(c);
                i += 1;
            }
            ('/', Some('/')) => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            ('/', Some('*')) => {
                i += 2;
                while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                    i += 1;
                }
                i += 2;
            }
            (',', _) => {
                let next = chars[i + 1..].iter().find(|c| !c.is_whitespace());
                if !matches!(next, Some('}') | Some(']')) {
                    out.push(c);
                }
                i += 1;
            }
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Writes the settings file at every start of the editor, the keys Atollo
/// owns rewritten (§13 item 10); the agents' profiles while `agents` (the
/// VM on, LEVELS.md §7: a VM that goes on or off changes them at the
/// editor's next start). A file with comments or trailing commas is read
/// without them; one that is still no JSON object is moved aside to
/// `settings.json.bak-<seconds>` and the settings are written anew
/// ([`Outcome::Replaced`]).
pub fn write(kit: &Kit, system_root: &Path, agents: bool) -> Result<Outcome> {
    let path = settings_path(kit);
    let mut moved = None;
    let existing = match fs::read_to_string(&path) {
        Ok(text) => {
            let text = crate::common::fsx::strip_bom(&text);
            if text.trim().is_empty() {
                Map::new()
            } else {
                match serde_json::from_str::<Value>(&strip_jsonc(text)) {
                    Ok(Value::Object(map)) => map,
                    _ => {
                        let secs = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or_default();
                        let aside = path.with_file_name(format!("settings.json.bak-{secs}"));
                        fs::rename(&path, &aside).with_context(|| aside.display().to_string())?;
                        moved = Some(aside);
                        Map::new()
                    }
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(e).with_context(|| path.display().to_string()),
    };
    let merged = merge(&existing, kit, system_root, agents);
    if moved.is_none() && merged == existing && path.exists() {
        return Ok(Outcome::Unchanged);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&Value::Object(merged))?;
    write_atomic(&path, text.as_bytes()).with_context(|| path.display().to_string())?;
    Ok(match moved {
        Some(aside) => Outcome::Replaced(aside),
        None => Outcome::Written,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/editor/cfg.rs"]
mod tests;
