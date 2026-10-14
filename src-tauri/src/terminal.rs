//! Consoles outside the editor: the kit's portable Windows Terminal
//! (DECISIONS.md D-13, D-23), or the kit's PowerShell in a classic console
//! when Windows Terminal cannot run (Windows 10 before 2004). Never a shell of
//! the PC.
//!
//! Settings live in `data\apps\<platform>\terminal\settings\settings.json`
//! (portable mode). The app owns the "Atollo" profiles and the VSCodium
//! color schemes and rewrites them at every start; the rest of the file is
//! the user's. `compatibility.reloadEnvironmentVariables` is off: otherwise
//! new tabs would rebuild the environment from the PC profile.

use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{json, Map, Value};
use windows::Win32::System::Threading::CREATE_NEW_CONSOLE;

use crate::editor_cfg::pwsh_args;
use crate::env::KitEnv;
use crate::jobs;
use crate::kit::{write_atomic, Kit};

const PROFILE_PWSH: &str = "{6e0f3c1a-4b7d-4d39-9a41-5f2d0c6b7a01}";
const PROFILE_BASH: &str = "{6e0f3c1a-4b7d-4d39-9a41-5f2d0c6b7a02}";
pub const SCHEME_DARK: &str = "VSCodium Dark Modern";
pub const SCHEME_LIGHT: &str = "VSCodium Light Modern";

/// Terminal colors of VSCodium's "Default Dark Modern" / "Default Light
/// Modern" themes, so consoles look like the editor and the app.
fn schemes() -> Vec<Value> {
    vec![
        json!({
            "name": SCHEME_DARK,
            "background": "#181818", "foreground": "#CCCCCC", "cursorColor": "#CCCCCC", "selectionBackground": "#264F78",
            "black": "#000000", "red": "#CD3131", "green": "#0DBC79", "yellow": "#E5E510",
            "blue": "#2472C8", "purple": "#BC3FBC", "cyan": "#11A8CD", "white": "#E5E5E5",
            "brightBlack": "#666666", "brightRed": "#F14C4C", "brightGreen": "#23D18B", "brightYellow": "#F5F543",
            "brightBlue": "#3B8EEA", "brightPurple": "#D670D6", "brightCyan": "#29B8DB", "brightWhite": "#E5E5E5"
        }),
        json!({
            "name": SCHEME_LIGHT,
            "background": "#F8F8F8", "foreground": "#3B3B3B", "cursorColor": "#3B3B3B", "selectionBackground": "#ADD6FF",
            "black": "#000000", "red": "#CD3131", "green": "#107C10", "yellow": "#949800",
            "blue": "#0451A5", "purple": "#BC05BC", "cyan": "#0598BC", "white": "#555555",
            "brightBlack": "#666666", "brightRed": "#CD3131", "brightGreen": "#14CE14", "brightYellow": "#B5BA00",
            "brightBlue": "#0451A5", "brightPurple": "#BC05BC", "brightCyan": "#0598BC", "brightWhite": "#A5A5A5"
        }),
    ]
}

fn terminal_exe(kit: &Kit) -> PathBuf {
    kit.apps().join(r"terminal\WindowsTerminal.exe")
}

pub fn settings_path(kit: &Kit) -> PathBuf {
    kit.apps().join(r"terminal\settings\settings.json")
}

fn quote(arg: &str) -> String {
    if arg.is_empty() || arg.contains([' ', '\t', '"', '&', '(', ')', ';']) {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_owned()
    }
}

fn kit_profiles(kit: &Kit) -> Vec<Value> {
    let mut out = Vec::new();
    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
    if pwsh.is_file() {
        let line = std::iter::once(pwsh.display().to_string())
            .chain(pwsh_args(kit))
            .map(|a| quote(&a))
            .collect::<Vec<_>>()
            .join(" ");
        out.push(json!({
            "guid": PROFILE_PWSH, "name": "Atollo", "commandline": line,
            "icon": pwsh.display().to_string(), "startingDirectory": kit.projects().display().to_string(),
            "colorScheme": { "dark": SCHEME_DARK, "light": SCHEME_LIGHT }, "hidden": false
        }));
    }
    let bash = kit.apps().join(r"git\bin\bash.exe");
    if bash.is_file() {
        let rc = kit.kit_files().join("bashrc").display().to_string();
        out.push(json!({
            "guid": PROFILE_BASH, "name": "Atollo - Git Bash",
            "commandline": format!("{} --rcfile {} -i", quote(&bash.display().to_string()), quote(&rc)),
            "icon": kit.apps().join(r"git\git-bash.exe").display().to_string(),
            "startingDirectory": kit.projects().display().to_string(),
            "colorScheme": { "dark": SCHEME_DARK, "light": SCHEME_LIGHT }, "hidden": false
        }));
    }
    out
}

/// Merges the kit's part into the user's settings (parsed).
pub fn merge(existing: &Map<String, Value>, kit: &Kit) -> Map<String, Value> {
    let mut out = existing.clone();
    let ours = kit_profiles(kit);
    let our_guids: Vec<&str> = [PROFILE_PWSH, PROFILE_BASH].to_vec();

    let mut profiles = match out.remove("profiles") {
        Some(Value::Object(p)) => p,
        Some(Value::Array(list)) => Map::from_iter([("list".to_owned(), Value::Array(list))]),
        _ => Map::new(),
    };
    let mut list: Vec<Value> = match profiles.remove("list") {
        Some(Value::Array(l)) => l,
        _ => Vec::new(),
    };
    list.retain(|p| {
        !p.get("guid")
            .and_then(Value::as_str)
            .is_some_and(|g| our_guids.contains(&g))
    });
    // PC shells (Windows PowerShell, cmd, Azure...) stay but are hidden.
    for p in &mut list {
        if p.get("source").is_some()
            || matches!(
                p.get("name").and_then(Value::as_str),
                Some("Windows PowerShell" | "Command Prompt" | "Prompt dei comandi")
            )
        {
            if let Some(obj) = p.as_object_mut() {
                obj.insert("hidden".into(), json!(true));
            }
        }
    }
    let mut new_list = ours.clone();
    new_list.extend(list);
    profiles.insert("list".into(), Value::Array(new_list));
    let defaults = profiles
        .entry("defaults")
        .or_insert_with(|| json!({ "font": { "face": "Cascadia Mono", "size": 11 } }));
    if let Some(d) = defaults.as_object_mut() {
        // A profile setting since Windows Terminal 1.21 (it moves it there).
        d.insert(
            "compatibility.reloadEnvironmentVariables".into(),
            json!(false),
        );
    }
    out.insert("profiles".into(), Value::Object(profiles));
    out.remove("compatibility.reloadEnvironmentVariables");

    let mut schemes_list: Vec<Value> = match out.remove("schemes") {
        Some(Value::Array(l)) => l,
        _ => Vec::new(),
    };
    schemes_list.retain(|s| {
        !matches!(
            s.get("name").and_then(Value::as_str),
            Some(SCHEME_DARK | SCHEME_LIGHT)
        )
    });
    schemes_list.extend(schemes());
    out.insert("schemes".into(), Value::Array(schemes_list));

    if !ours.is_empty() {
        let default_ok = out
            .get("defaultProfile")
            .and_then(Value::as_str)
            .is_some_and(|d| our_guids.contains(&d));
        if !default_ok {
            out.insert("defaultProfile".into(), json!(PROFILE_PWSH));
        }
    }
    out.entry("theme").or_insert(json!("system"));
    out.entry("disabledProfileSources").or_insert(json!([
        "Windows.Terminal.Wsl",
        "Windows.Terminal.Azure",
        "Windows.Terminal.PowershellCore",
        "Windows.Terminal.VisualStudio"
    ]));
    out.insert(
        "$help".into(),
        json!("Atollo rewrites the Atollo profiles and the VSCodium schemes at every start."),
    );
    out
}

/// Writes the settings; a file that is not plain JSON is left alone.
pub fn write_settings(kit: &Kit) -> Result<bool> {
    if !terminal_exe(kit).is_file() {
        return Ok(false);
    }
    let path = settings_path(kit);
    let existing = match fs::read_to_string(&path) {
        Ok(text) => match serde_json::from_str::<Value>(text.trim_start_matches('\u{feff}')) {
            Ok(Value::Object(map)) => map,
            _ => {
                crate::log::warn(&format!("terminal settings left alone: {}", path.display()));
                return Ok(false);
            }
        },
        Err(_) => Map::new(),
    };
    let merged = merge(&existing, kit);
    if merged != existing {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        write_atomic(
            &path,
            serde_json::to_string_pretty(&Value::Object(merged))?.as_bytes(),
        )
        .with_context(|| path.display().to_string())?;
    }
    Ok(true)
}

/// Opens a kit console in `dir`: Windows Terminal with the Atollo profile,
/// or the kit's PowerShell in a classic console. Inside the job.
pub fn open_console(kit: &Kit, env: &KitEnv, dir: &Path) -> Result<()> {
    let wt = terminal_exe(kit);
    if wt.is_file() && write_settings(kit).unwrap_or(false) {
        let mut cmd = jobs::kit_command(&wt, env);
        cmd.args([
            "--window",
            "new",
            "--profile",
            PROFILE_PWSH,
            "--startingDirectory",
        ])
        .arg(dir)
        .current_dir(dir)
        .creation_flags(0);
        jobs::spawn(&mut cmd)?;
        return Ok(());
    }
    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
    // Never the shells of the PC: without the kit's PowerShell, no console.
    anyhow::ensure!(pwsh.is_file(), "PowerShell 7 del kit non è installato");
    let mut cmd = jobs::kit_command(&pwsh, env);
    cmd.args(pwsh_args(kit))
        .current_dir(dir)
        .creation_flags(CREATE_NEW_CONSOLE.0);
    jobs::spawn(&mut cmd)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn merge_keeps_user_parts() {
        let tmp = TempDir::new("wt");
        let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64);
        kit.prepare().unwrap();
        let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
        fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
        fs::write(&pwsh, b"").unwrap();
        let existing: Map<String, Value> = serde_json::from_str(&format!(
            r##"{{"profiles":{{"list":[{{"guid":"{PROFILE_PWSH}","name":"old"}},{{"guid":"{{x}}","name":"Mine"}},{{"name":"Windows PowerShell"}}]}},"schemes":[{{"name":"{SCHEME_DARK}","background":"#000"}},{{"name":"Mine"}}],"copyOnSelect":true}}"##
        ))
        .unwrap();
        let m = merge(&existing, &kit);
        let list = m["profiles"]["list"].as_array().unwrap();
        assert_eq!(list[0]["name"], "Atollo");
        assert!(
            list[0]["commandline"]
                .as_str()
                .unwrap()
                .contains("\"-File\"")
                || list[0]["commandline"].as_str().unwrap().contains("-File")
        );
        assert!(
            list[0]["commandline"].as_str().unwrap().contains("\""),
            "path with spaces is quoted"
        );
        assert!(list.iter().any(|p| p["name"] == "Mine"));
        assert_eq!(list.iter().filter(|p| p["guid"] == PROFILE_PWSH).count(), 1);
        assert_eq!(
            list.iter()
                .find(|p| p["name"] == "Windows PowerShell")
                .unwrap()["hidden"],
            true
        );
        let schemes = m["schemes"].as_array().unwrap();
        assert_eq!(
            schemes.iter().filter(|s| s["name"] == SCHEME_DARK).count(),
            1
        );
        assert_eq!(
            schemes.iter().find(|s| s["name"] == SCHEME_DARK).unwrap()["background"],
            "#181818"
        );
        assert!(schemes.iter().any(|s| s["name"] == "Mine"));
        assert_eq!(m["copyOnSelect"], true);
        assert_eq!(m["defaultProfile"], PROFILE_PWSH);
        assert_eq!(
            m["profiles"]["defaults"]["compatibility.reloadEnvironmentVariables"],
            false
        );
        assert_eq!(merge(&m, &kit), m, "stable");
    }
}
