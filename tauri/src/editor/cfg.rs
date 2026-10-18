//! Settings of the integrated editor, written before every start.
//!
//! File: `data\vscodium-server\data\Machine\settings.json`, the server-side
//! settings that VSCodium applies to every window of the server (the user's
//! own settings stay in the editor). Port of nabre/atollo
//! `setup/extensions/atollo/kit.js:163-183` and `ext/startup.js:36-51`:
//! - terminal profiles of the kit rewritten each time, PC profiles hidden,
//!   profiles added by the user kept (DECISIONS.md D-23);
//! - a few defaults set only when missing;
//! - a file with comments or invalid JSON is left alone and reported.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

use crate::fsx::write_atomic;
use crate::kit::{Kit, Os};

/// Profiles owned by the kit (rewritten) and the PC profiles hidden, on
/// Windows.
pub const KIT_PROFILES: &[&str] = &[
    "Atollo",
    "Atollo - Git Bash",
    "Atollo - cmd",
    "Claude Code",
    "Codex",
    "Antigravity",
];
const HIDDEN: &[&str] = &[
    "PowerShell",
    "Windows PowerShell",
    "Command Prompt",
    "Git Bash",
    "JavaScript Debug Terminal",
];

/// The same on macOS and Linux: the system's bash (Git of the kit has no
/// shell there, D-109) and, on macOS, zsh; never cmd.
pub const KIT_PROFILES_UNIX: &[&str] = &[
    "Atollo",
    "Atollo - Bash",
    "Atollo - zsh",
    "Claude Code",
    "Codex",
    "Antigravity",
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

/// The suffix of the editor's per-system settings
/// (`terminal.integrated.profiles.<os>`).
fn os_key(os: Os) -> &'static str {
    match os {
        Os::Win32 => "windows",
        Os::Darwin => "osx",
        Os::Linux => "linux",
    }
}

fn kit_profile_names(os: Os) -> &'static [&'static str] {
    if os.is_windows() {
        KIT_PROFILES
    } else {
        KIT_PROFILES_UNIX
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
            "Atollo Dark",
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
            "Atollo Dark",
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
            "Atollo Light",
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
    crate::fsx::join_rel(kit.data(), "vscodium-server/data/Machine/settings.json")
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

/// The arguments of bash with the kit's `bashrc`.
pub fn bash_args(kit: &Kit) -> Vec<String> {
    let rc = kit.kit_files().join("bashrc").display().to_string();
    vec!["--rcfile".into(), rc, "-i".into()]
}

/// The kit profiles for what is installed now, for the kit's system.
pub fn kit_profiles(kit: &Kit, system_root: &Path) -> Map<String, Value> {
    let os = kit.os();
    let app = |rel: &str| crate::setup::catalog::kit_path(kit, rel);
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
    for (name, rel, icon) in [
        ("Claude Code", r"claude\claude.exe", "sparkle"),
        ("Codex", r"codex\bin\codex.exe", "hubot"),
        ("Antigravity", r"antigravity\agy.exe", "rocket"),
    ] {
        let exe = app(rel);
        if exe.is_file() {
            // Antigravity through the kit's agy.cmd (agy, sh, on macOS and
            // Linux): it carries the login.
            let path = if name == "Antigravity" {
                kit.kit_files()
                    .join("bin")
                    .join(crate::router::bins::cmd_name(os, "agy"))
            } else {
                exe
            };
            out.insert(name.into(), profile(&path, &[], icon));
        }
    }
    out
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Written,
    Unchanged,
    /// Comments or invalid JSON: not touched.
    LeftAlone(String),
}

/// Merges the kit's settings into `existing` (the parsed file). The
/// terminal settings are those of the kit's system (`.windows`, `.osx`,
/// `.linux`): the same file serves a kit used on several systems, each
/// writes its own and leaves the others.
pub fn merge(existing: &Map<String, Value>, kit: &Kit, system_root: &Path) -> Map<String, Value> {
    let os = kit.os();
    let key = |name: &str| format!("terminal.integrated.{name}.{}", os_key(os));
    let (owned, hidden) = (kit_profile_names(os), hidden(os));
    let mut out = existing.clone();
    let profiles_key = key("profiles");
    let mut profiles = Map::new();
    for name in hidden {
        profiles.insert((*name).into(), Value::Null);
    }
    let kit_profiles = kit_profiles(kit, system_root);
    if let Some(Value::Object(old)) = existing.get(&profiles_key) {
        for (name, value) in old {
            if !owned.contains(&name.as_str()) && !hidden.contains(&name.as_str()) {
                profiles.insert(name.clone(), value.clone());
            }
        }
    }
    let names: Vec<String> = kit_profiles.keys().cloned().collect();
    profiles.extend(kit_profiles);
    out.insert(profiles_key, Value::Object(profiles.clone()));

    let default_key = key("defaultProfile");
    let keep_default = matches!(existing.get(&default_key), Some(Value::String(d)) if profiles.get(d).is_some_and(|p| !p.is_null()));
    // Without the kit's PowerShell, bash on macOS and Linux.
    let default = [DEFAULT_PROFILE]
        .into_iter()
        .chain((!os.is_windows()).then_some(DEFAULT_PROFILE_UNIX))
        .find(|d| names.iter().any(|n| n == d));
    if let (false, Some(default)) = (keep_default, default) {
        out.insert(default_key, json!(default));
    }
    let pwsh = crate::setup::catalog::kit_path(kit, r"pwsh\pwsh.exe");
    if os.is_windows() {
        if pwsh.is_file() {
            // Tasks and debug consoles use the kit's shell too.
            out.insert(
                key("automationProfile"),
                json!({ "path": pwsh.display().to_string(), "args": ["-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass"] }),
            );
        }
        out.insert("terminal.integrated.useWslProfiles".into(), json!(false));
    } else {
        // Tasks written for sh and bash: the system's bash, never the
        // user's login shell (its profile could put other tools first).
        out.insert(key("automationProfile"), json!({ "path": BASH }));
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
    ] {
        out.entry(key).or_insert(value);
    }
    out
}

/// Writes the settings file. `Ok(LeftAlone)` when the user's file cannot be
/// parsed as plain JSON.
pub fn write(kit: &Kit, system_root: &Path) -> Result<Outcome> {
    let path = settings_path(kit);
    let existing = match fs::read_to_string(&path) {
        Ok(text) => {
            let text = crate::fsx::strip_bom(&text);
            if text.trim().is_empty() {
                Map::new()
            } else {
                match serde_json::from_str::<Value>(text) {
                    Ok(Value::Object(map)) => map,
                    Ok(_) => {
                        return Ok(Outcome::LeftAlone(format!(
                            "{}: not a JSON object",
                            path.display()
                        )))
                    }
                    Err(e) => return Ok(Outcome::LeftAlone(format!("{}: {e}", path.display()))),
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Map::new(),
        Err(e) => return Err(e).with_context(|| path.display().to_string()),
    };
    let merged = merge(&existing, kit, system_root);
    if merged == existing && path.exists() {
        return Ok(Outcome::Unchanged);
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let text = serde_json::to_string_pretty(&Value::Object(merged))?;
    write_atomic(&path, text.as_bytes()).with_context(|| path.display().to_string())?;
    Ok(Outcome::Written)
}

#[cfg(test)]
#[path = "../../../test/rust/editor/cfg.rs"]
mod tests;
