//! Claude Code, Codex and Antigravity: state, launch, sign-out.
//!
//! - signed in: Claude `.claude\.credentials.json` (OAuth token) or
//!   `ANTHROPIC_API_KEY`; Codex `.codex\auth.json` (token or API key);
//!   Antigravity a login saved in the kit (`keyring.rs`) or `GEMINI_API_KEY`
//!   (same sources as nabre/dev-kit `core/stats/*.js`);
//! - launch: the agents are aligned first, the agent opens in a console
//!   window of the app in the chosen folder, and they are aligned again when it
//!   ends (`_agent.cmd` of the kit). Antigravity runs through
//!   `data\kit\bin\agy.cmd`, which carries the login in and out of Windows;
//! - a watcher thread aligns when an agent of the kit ends anywhere (editor
//!   terminals, consoles) and saves Antigravity's login left in Windows.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::Value;

use crate::catalog::Catalog;
use crate::env::KitEnv;
use crate::kit::Kit;

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
    /// How: "account", "api key".
    pub sign_in_kind: Option<String>,
    pub running: bool,
}

fn json(path: &Path) -> Option<Value> {
    serde_json::from_str(
        fs::read_to_string(path)
            .ok()?
            .trim_start_matches('\u{feff}'),
    )
    .ok()
}

fn sign_in(kit: &Kit, env: &KitEnv, id: &str) -> Option<&'static str> {
    let home = kit.home();
    let has = |v: Option<&str>| v.is_some_and(|s| !s.is_empty());
    match id {
        "claude" => {
            let creds = json(&home.join(r".claude\.credentials.json"));
            if has(creds
                .as_ref()
                .and_then(|c| c.pointer("/claudeAiOauth/accessToken"))
                .and_then(Value::as_str))
            {
                Some("account")
            } else if has(env.get("ANTHROPIC_API_KEY")) {
                Some("api key")
            } else {
                None
            }
        }
        "codex" => {
            let auth = json(&home.join(r".codex\auth.json"));
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
                || has(env.get("OPENAI_API_KEY"))
            {
                Some("api key")
            } else {
                None
            }
        }
        "antigravity" => {
            if crate::keyring::has_login(kit) {
                Some("account")
            } else if has(env.get("GEMINI_API_KEY")) {
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
    let procs = crate::processes::running_under(&kit.apps());
    IDS.iter()
        .filter(|id| {
            let exe = kit.apps().join(exe_of(id).unwrap_or_default());
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

pub fn status(kit: &Kit, env: &KitEnv) -> Vec<AgentInfo> {
    let catalog = Catalog::get();
    let now = running(kit);
    IDS.iter()
        .filter_map(|id| catalog.find(id))
        .map(|c| {
            let kind = sign_in(kit, env, &c.id);
            AgentInfo {
                id: c.id.clone(),
                name: c.name.clone(),
                installed: c.is_installed(kit),
                version: c.installed_version(kit),
                signed_in: kind.is_some(),
                sign_in_kind: kind.map(str::to_owned),
                running: now.contains(&c.id),
            }
        })
        .collect()
}

/// The command that starts `id` (Antigravity through the kit's `agy.cmd`).
fn command_of(kit: &Kit, id: &str) -> Result<PathBuf> {
    let Some(rel) = exe_of(id) else {
        bail!("unknown agent: {id}")
    };
    let exe = kit.apps().join(rel);
    if !exe.is_file() {
        bail!("{id} is not installed");
    }
    Ok(if id == "antigravity" {
        kit.kit_files().join(r"bin\agy.cmd")
    } else {
        exe
    })
}

/// Aligns the agents, then opens `id` in a kit console in `dir`; aligns
/// again when that console ends.
pub fn launch<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    kit: &Kit,
    env: &KitEnv,
    id: &str,
    dir: &Path,
) -> Result<()> {
    let command = command_of(kit, id)?;
    let report = crate::agents_sync::run(kit, Some(dir));
    for w in &report.warnings {
        crate::log::warn(&format!("before {id}: {w}"));
    }
    let (after, after_dir) = (kit.clone(), dir.to_path_buf());
    crate::terminal::open_command(app, kit, env, dir, &command, move || {
        let _ = crate::agents_sync::run(&after, Some(&after_dir));
    })
}

/// Signs out in the kit: the saved login is removed (Claude's and Codex's
/// files, Antigravity's saved login). The account's sessions on the
/// provider's site are not revoked: do it from the account settings.
pub fn sign_out(kit: &Kit, env: &KitEnv, id: &str) -> Result<()> {
    let home = kit.home();
    match id {
        "claude" => remove_if_exists(&home.join(r".claude\.credentials.json")),
        "codex" => {
            let codex = kit.apps().join(r"codex\bin\codex.exe");
            if codex.is_file() {
                let status = crate::jobs::kit_command(&codex, env)
                    .arg("logout")
                    .status()
                    .context("codex logout")?;
                if status.success() {
                    return Ok(());
                }
            }
            remove_if_exists(&home.join(r".codex\auth.json"))
        }
        "antigravity" => {
            crate::keyring::save(kit)?;
            remove_if_exists(&kit.data().join(r"antigravity\keyring.json"))
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

/// Background: when an agent of the kit ends anywhere, align the agents and
/// save Antigravity's login left in Windows.
pub fn watch(kit: Kit) {
    std::thread::spawn(move || {
        let mut before = BTreeSet::new();
        loop {
            std::thread::sleep(Duration::from_secs(3));
            let now = running(&kit);
            let ended: Vec<&String> = before.difference(&now).collect();
            if !ended.is_empty() {
                crate::log::info(&format!("agents ended: {ended:?}"));
                let _ = crate::agents_sync::run(&kit, None);
            }
            // Antigravity's login still in Windows and no agy of the kit
            // running: saved now (save waits for a fresh "starting" mark).
            if crate::keyring::pending(&kit) && !now.contains("antigravity") {
                if let Err(e) = crate::keyring::save(&kit) {
                    crate::log::warn(&format!("antigravity login: {e:#}"));
                }
            }
            before = now;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn sign_in_detection() {
        let tmp = TempDir::new("agents");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let env = KitEnv::default();
        assert!(status(&kit, &env)
            .iter()
            .all(|a| !a.signed_in && !a.installed));
        fs::create_dir_all(kit.home().join(".claude")).unwrap();
        fs::write(
            kit.home().join(r".claude\.credentials.json"),
            r#"{"claudeAiOauth":{"accessToken":"t"}}"#,
        )
        .unwrap();
        fs::create_dir_all(kit.home().join(".codex")).unwrap();
        fs::write(
            kit.home().join(r".codex\auth.json"),
            r#"{"OPENAI_API_KEY":"k"}"#,
        )
        .unwrap();
        let s = status(&kit, &env);
        assert_eq!(s[0].sign_in_kind.as_deref(), Some("account"));
        assert_eq!(s[1].sign_in_kind.as_deref(), Some("api key"));
        assert!(!s[2].signed_in);
        sign_out(&kit, &env, "claude").unwrap();
        assert!(!status(&kit, &env)[0].signed_in);
    }
}
