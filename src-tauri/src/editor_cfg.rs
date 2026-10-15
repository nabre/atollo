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
use crate::kit::Kit;

/// Profiles owned by the kit (rewritten) and the PC profiles hidden.
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

pub const DEFAULT_PROFILE: &str = "Atollo";

/// The kit's themes (editor_theme.rs) in place of VSCodium's own: a theme
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
        crate::editor_theme::DARK,
    ),
    (
        "workbench.preferredDarkColorTheme",
        &[
            "Dark Modern",
            "Default Dark Modern",
            "Dark+",
            "Default Dark+",
        ],
        crate::editor_theme::DARK,
    ),
    (
        "workbench.preferredLightColorTheme",
        &[
            "Light Modern",
            "Default Light Modern",
            "Light+",
            "Default Light+",
        ],
        crate::editor_theme::LIGHT,
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
    kit.data()
        .join(r"vscodium-server\data\Machine\settings.json")
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

/// The kit profiles for what is installed now.
pub fn kit_profiles(kit: &Kit, system_root: &Path) -> Map<String, Value> {
    let apps = kit.apps();
    let mut out = Map::new();
    let pwsh = apps.join(r"pwsh\pwsh.exe");
    if pwsh.is_file() {
        out.insert(
            "Atollo".into(),
            profile(&pwsh, &pwsh_args(kit), "terminal-powershell"),
        );
    }
    let bash = apps.join(r"git\bin\bash.exe");
    if bash.is_file() {
        let rc = kit.kit_files().join("bashrc").display().to_string();
        out.insert(
            "Atollo - Git Bash".into(),
            profile(
                &bash,
                &["--rcfile".into(), rc, "-i".into()],
                "terminal-bash",
            ),
        );
    }
    let cmd = system_root.join(r"System32\cmd.exe");
    out.insert(
        "Atollo - cmd".into(),
        profile(&cmd, &["/d".into(), "/k".into()], "terminal-cmd"),
    );
    for (name, rel, icon) in [
        ("Claude Code", r"claude\claude.exe", "sparkle"),
        ("Codex", r"codex\bin\codex.exe", "hubot"),
        ("Antigravity", r"antigravity\agy.exe", "rocket"),
    ] {
        let exe = apps.join(rel);
        if exe.is_file() {
            // Antigravity through the kit's agy.cmd: it carries the login.
            let path = if name == "Antigravity" {
                kit.kit_files().join(r"bin\agy.cmd")
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

/// Merges the kit's settings into `existing` (the parsed file).
pub fn merge(existing: &Map<String, Value>, kit: &Kit, system_root: &Path) -> Map<String, Value> {
    let mut out = existing.clone();
    let profiles_key = "terminal.integrated.profiles.windows";
    let mut profiles = Map::new();
    for name in HIDDEN {
        profiles.insert((*name).into(), Value::Null);
    }
    let kit_profiles = kit_profiles(kit, system_root);
    if let Some(Value::Object(old)) = existing.get(profiles_key) {
        for (name, value) in old {
            if !KIT_PROFILES.contains(&name.as_str()) && !HIDDEN.contains(&name.as_str()) {
                profiles.insert(name.clone(), value.clone());
            }
        }
    }
    let names: Vec<String> = kit_profiles.keys().cloned().collect();
    profiles.extend(kit_profiles);
    out.insert(profiles_key.into(), Value::Object(profiles.clone()));

    let default_key = "terminal.integrated.defaultProfile.windows";
    let keep_default = matches!(existing.get(default_key), Some(Value::String(d)) if profiles.get(d).is_some_and(|p| !p.is_null()));
    if !keep_default && names.iter().any(|n| n == DEFAULT_PROFILE) {
        out.insert(default_key.into(), json!(DEFAULT_PROFILE));
    }
    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
    if pwsh.is_file() {
        // Tasks and debug consoles use the kit's shell too.
        out.insert(
            "terminal.integrated.automationProfile.windows".into(),
            json!({ "path": pwsh.display().to_string(), "args": ["-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass"] }),
        );
    }
    out.insert("terminal.integrated.useWslProfiles".into(), json!(false));
    // VSCodium drops NODE_OPTIONS from the terminals' environment (one of its
    // "dangerous" variables): without `loopback.js` a dev server started in
    // the editor's terminal or a task listens on the network and Windows
    // Firewall asks for a consent that needs an administrator. The settings'
    // variables are added after that cleanup; the user's others are kept.
    let env_key = "terminal.integrated.env.windows";
    let mut vars = match existing.get(env_key) {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };
    vars.insert("NODE_OPTIONS".into(), json!(crate::env::node_options(kit)));
    out.insert(env_key.into(), Value::Object(vars));

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
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    fn kit_with(name: &str, installed: &[&str]) -> (TempDir, Kit) {
        let tmp = TempDir::new(name);
        let kit = Kit::at(tmp.0.join("k (1) & co"), Arch::X64);
        kit.prepare().unwrap();
        for rel in installed {
            let f = kit.apps().join(rel);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, b"").unwrap();
        }
        (tmp, kit)
    }

    fn read(kit: &Kit) -> Map<String, Value> {
        serde_json::from_str(&fs::read_to_string(settings_path(kit)).unwrap()).unwrap()
    }

    #[test]
    fn profiles_rewritten_user_ones_kept() {
        let (_t, kit) = kit_with("cfg-profiles", &[r"pwsh\pwsh.exe", r"git\bin\bash.exe"]);
        let path = settings_path(&kit);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{"terminal.integrated.profiles.windows":{"Mine":{"path":"x.exe"},"Atollo":{"path":"old"},"Codex":{"path":"gone"}},"editor.fontSize":15,"files.dialog.defaultPath":"D:\\mine"}"#,
        )
        .unwrap();
        assert_eq!(
            write(&kit, Path::new(r"C:\Windows")).unwrap(),
            Outcome::Written
        );
        let s = read(&kit);
        let p = s["terminal.integrated.profiles.windows"]
            .as_object()
            .unwrap();
        assert_eq!(p["Mine"]["path"], "x.exe");
        assert!(p["Atollo"]["path"]
            .as_str()
            .unwrap()
            .ends_with(r"pwsh\pwsh.exe"));
        assert!(p["Atollo - Git Bash"]["args"][1]
            .as_str()
            .unwrap()
            .ends_with(r"data\kit\bashrc"));
        assert!(
            p.get("Codex").is_none(),
            "agent not installed: profile removed"
        );
        assert!(p["PowerShell"].is_null() && p["Command Prompt"].is_null());
        assert_eq!(s["terminal.integrated.defaultProfile.windows"], "Atollo");
        assert!(s["terminal.integrated.env.windows"]["NODE_OPTIONS"]
            .as_str()
            .unwrap()
            .ends_with("/data/kit/loopback.js\""));
        assert_eq!(s["editor.fontSize"], 15);
        assert_eq!(s["files.dialog.defaultPath"], r"D:\mine", "user value kept");
        assert!(s["git.defaultCloneDirectory"]
            .as_str()
            .unwrap()
            .ends_with("projects"));
        assert_eq!(
            write(&kit, Path::new(r"C:\Windows")).unwrap(),
            Outcome::Unchanged
        );
    }

    #[test]
    fn user_default_profile_kept_when_valid() {
        let (_t, kit) = kit_with("cfg-default", &[r"pwsh\pwsh.exe", r"git\bin\bash.exe"]);
        let path = settings_path(&kit);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r#"{"terminal.integrated.defaultProfile.windows":"Atollo - Git Bash"}"#,
        )
        .unwrap();
        write(&kit, Path::new(r"C:\Windows")).unwrap();
        assert_eq!(
            read(&kit)["terminal.integrated.defaultProfile.windows"],
            "Atollo - Git Bash"
        );
        fs::write(
            &path,
            r#"{"terminal.integrated.defaultProfile.windows":"PowerShell"}"#,
        )
        .unwrap();
        write(&kit, Path::new(r"C:\Windows")).unwrap();
        assert_eq!(
            read(&kit)["terminal.integrated.defaultProfile.windows"],
            "Atollo",
            "hidden PC profile replaced"
        );
    }

    #[test]
    fn file_with_comments_left_alone() {
        let (_t, kit) = kit_with("cfg-comments", &[]);
        let path = settings_path(&kit);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let text = "{\n  // mine\n  \"a\": 1\n}";
        fs::write(&path, text).unwrap();
        assert!(matches!(
            write(&kit, Path::new(r"C:\Windows")).unwrap(),
            Outcome::LeftAlone(_)
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), text);
    }

    #[test]
    fn codium_themes_become_atollo_user_choice_kept() {
        let (_t, kit) = kit_with("cfg-colors", &[]);
        let path = settings_path(&kit);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            r##"{"workbench.colorCustomizations":{"editor.background":"#000000","[Default Dark Modern]":{"focusBorder":"#10B981"},"[Default Light Modern]":{}},"workbench.preferredDarkColorTheme":"Dark Modern","workbench.preferredLightColorTheme":"Solarized Light"}"##,
        )
        .unwrap();
        write(&kit, Path::new(r"C:\Windows")).unwrap();
        let s = read(&kit);
        let colors = s["workbench.colorCustomizations"].as_object().unwrap();
        assert_eq!(colors.len(), 1, "the user's own kept: {colors:?}");
        assert_eq!(colors["editor.background"], "#000000");
        assert_eq!(s["workbench.colorTheme"], "Atollo Dark");
        assert_eq!(s["workbench.experimental.modernUI"], true);
        assert_eq!(s["workbench.preferredDarkColorTheme"], "Atollo Dark");
        assert_eq!(s["workbench.preferredLightColorTheme"], "Solarized Light");

        fs::write(&path, r#"{"workbench.preferredLightColorTheme":"Default Light Modern","workbench.colorCustomizations":{"[Default Dark Modern]":{}}}"#).unwrap();
        write(&kit, Path::new(r"C:\Windows")).unwrap();
        let s = read(&kit);
        assert_eq!(s["workbench.preferredLightColorTheme"], "Atollo Light");
        assert!(s.get("workbench.colorCustomizations").is_none());
    }
}
