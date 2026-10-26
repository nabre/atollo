//! Settings of the integrated editor, written before every start.
//!
//! File: `data\vscodium-server\data\Machine\settings.json`, the server-side
//! settings that VSCodium applies to every window of the server (the user's
//! own settings stay in the editor):
//! - terminal profiles of the kit rewritten each time, PC profiles hidden,
//!   profiles added by the user kept (DECISIONS.md D-23), those an earlier
//!   kit left under other names removed (D-126);
//! - a few defaults set only when missing;
//! - a file with comments or invalid JSON is left alone and reported.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};

use crate::common::fsx::write_atomic;
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
    "Pi",
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
    "Pi",
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

/// The kit's PowerShell running `vm.ps1` with `extra` (D-175): every
/// terminal, task and agent of the editor goes through it, to the
/// project's VM when the VM is on (`isolation::session`), else to the
/// kit's own on the PC; `interactive` keeps the shell open after it.
pub fn vm_args(kit: &Kit, interactive: bool, extra: &[String]) -> Vec<String> {
    let mut args: Vec<String> = vec!["-NoLogo".into(), "-NoProfile".into()];
    if interactive {
        args.push("-NoExit".into());
    }
    args.extend([
        "-ExecutionPolicy".to_owned(),
        "Bypass".to_owned(),
        "-File".to_owned(),
        kit.kit_files().join("vm.ps1").display().to_string(),
    ]);
    args.extend(extra.iter().cloned());
    args
}

/// `-Shell <kind>` and, for another program than the kit's PowerShell,
/// `-Exe <its path>`, for [`vm_args`].
fn shell_extra(kind: &str, exe: Option<&Path>) -> Vec<String> {
    let mut extra = vec!["-Shell".to_owned(), kind.to_owned()];
    if let Some(exe) = exe {
        extra.push("-Exe".into());
        extra.push(exe.display().to_string());
    }
    extra
}

/// The arguments of bash with the kit's `bashrc`.
pub fn bash_args(kit: &Kit) -> Vec<String> {
    let rc = kit.kit_files().join("bashrc").display().to_string();
    vec!["--rcfile".into(), rc, "-i".into()]
}

/// The kit profiles for what is installed now, for the kit's system.
pub fn kit_profiles(kit: &Kit, system_root: &Path) -> Map<String, Value> {
    let os = kit.os();
    let app = |rel: &str| crate::setup::kit_path(kit, rel);
    let mut out = Map::new();
    let pwsh = app(r"pwsh\pwsh.exe");
    // Through `vm.ps1` when the kit's PowerShell is there (D-175): `kind`
    // of shell, `exe` its program; else the program itself with `args`.
    let has_pwsh = pwsh.is_file();
    let shell = |kind: &str, exe: &Path, args: Vec<String>, icon: &str| {
        if has_pwsh {
            let extra = shell_extra(kind, Some(exe));
            profile(&pwsh, &vm_args(kit, false, &extra), icon)
        } else {
            profile(exe, &args, icon)
        }
    };
    if has_pwsh {
        out.insert(
            "Atollo".into(),
            profile(
                &pwsh,
                &vm_args(kit, true, &shell_extra("pwsh", None)),
                "terminal-powershell",
            ),
        );
    }
    if os.is_windows() {
        let bash = app(r"git\bin\bash.exe");
        if bash.is_file() {
            out.insert(
                "Atollo - Git Bash".into(),
                shell("bash", &bash, bash_args(kit), "terminal-bash"),
            );
        }
        let cmd = system_root.join(r"System32\cmd.exe");
        out.insert(
            "Atollo - cmd".into(),
            shell("cmd", &cmd, vec!["/d".into(), "/k".into()], "terminal-cmd"),
        );
    } else {
        // The system's bash with the kit's bashrc (bash 3.2 on macOS reads
        // it too, D-105); the environment is the kit's, from the server.
        out.insert(
            "Atollo - Bash".into(),
            shell("bash", Path::new(BASH), bash_args(kit), "terminal-bash"),
        );
        if os.is_macos() {
            // Interactive, not a login shell: path_helper of
            // /etc/zprofile would put the system's folders first.
            out.insert(
                "Atollo - zsh".into(),
                shell("zsh", Path::new(ZSH), vec!["-i".into()], "terminal"),
            );
        }
    }
    for (name, id, rel, icon) in [
        ("Claude Code", "claude", r"claude\claude.exe", "sparkle"),
        ("Codex", "codex", r"codex\bin\codex.exe", "hubot"),
        (
            "Antigravity",
            "antigravity",
            r"antigravity\agy.exe",
            "rocket",
        ),
        ("Pi", "pi", r"pi\pi.exe", "symbol-constant"),
    ] {
        let exe = app(rel);
        if exe.is_file() {
            // Antigravity through the kit's agy.cmd (agy, sh, on macOS and
            // Linux): it carries the login and the Gemini key. Claude Code
            // on macOS through the kit's `claude` (sh): it gives the token
            // saved in the kit to Claude Code only (SEC-V6-02).
            let bin = kit.kit_files().join(crate::setup::kit_files::AGENTS_DIR);
            let path = match name {
                "Antigravity" => bin.join(crate::router::bins::cmd_name(os, "agy")),
                "Claude Code" if crate::agents::claude_token::offered(os) => bin.join("claude"),
                _ => exe,
            };
            // In the VM, Claude Code and Codex in `sbx`'s sandboxes, the
            // others refused; without it, the agent on the PC (D-175).
            let entry = if has_pwsh {
                let extra = [
                    "-Agent".to_owned(),
                    id.to_owned(),
                    "-Exe".to_owned(),
                    path.display().to_string(),
                ];
                profile(&pwsh, &vm_args(kit, false, &extra), icon)
            } else {
                profile(&path, &[], icon)
            };
            out.insert(name.into(), entry);
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

/// A profile the kit wrote under a name it no longer uses: one of today's
/// kit profiles under another name, or, with the kit's `overrideName`, a
/// program in the kit. Kept, it would point at a versioned PowerShell or
/// Git folder that an update removes, and a default on it would make
/// VSCodium open the PC's shell.
fn left_by_the_kit(profile: &Value, kit: &Kit, today: &Map<String, Value>) -> bool {
    today.values().any(|p| p == profile)
        || profile["overrideName"] == json!(true)
            && profile["path"]
                .as_str()
                .is_some_and(|p| Path::new(p).starts_with(kit.data()))
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
            if !owned.contains(&name.as_str())
                && !hidden.contains(&name.as_str())
                && !left_by_the_kit(value, kit, &kit_profiles)
            {
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
    let pwsh = crate::setup::kit_path(kit, r"pwsh\pwsh.exe");
    // Tasks and debug consoles through `vm.ps1` too (D-175): in the
    // project's VM with `sh -lc` when the VM is on; else the kit's shell on
    // Windows and, on macOS and Linux, the system's bash (tasks written for
    // sh and bash; never the user's login shell, whose profile could put
    // other tools first).
    if os.is_windows() {
        if pwsh.is_file() {
            let args = vm_args(kit, false, &shell_extra("pwsh", None));
            out.insert(
                key("automationProfile"),
                json!({ "path": pwsh.display().to_string(), "args": args }),
            );
        }
        out.insert("terminal.integrated.useWslProfiles".into(), json!(false));
    } else if pwsh.is_file() {
        let args = vm_args(kit, false, &shell_extra("bash", Some(Path::new(BASH))));
        out.insert(
            key("automationProfile"),
            json!({ "path": pwsh.display().to_string(), "args": args }),
        );
    } else {
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

/// Writes the settings file. `Ok(LeftAlone)` when the user's file cannot be
/// parsed as plain JSON.
pub fn write(kit: &Kit, system_root: &Path) -> Result<Outcome> {
    let path = settings_path(kit);
    let existing = match fs::read_to_string(&path) {
        Ok(text) => {
            let text = crate::common::fsx::strip_bom(&text);
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
