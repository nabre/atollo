//! The kit's GitHub account: one sign-in for the app, the editor, the
//! consoles and the agents (DECISIONS.md D-70). Port of nabre/dev-kit
//! `setup/extensions/github-repos` (the VSCodium side view) as the app's
//! GitHub page.
//!
//! - The account is GitHub CLI's, saved in `data\gh` (`--insecure-storage`):
//!   it travels with the kit and never reaches the Credential Manager.
//! - Sign-in: `gh auth login --web` without a console prints a one-time code
//!   and waits; the app shows the code and opens github.com/login/device.
//! - The kit's Git asks `gh` for github.com (`credential.helper` in the
//!   kit's `.gitconfig`, without the path of `gh`, so it survives moving the
//!   kit): push and pull in the editor, the consoles and the agents use the
//!   same account. Signed out, the helper goes and Git is as before.
//! - The page: the repositories of the account (API with the account's
//!   token), clone into `projects\`, rename and delete on GitHub.
//!
//! Here the account, the sign-in and Git's helper; the repositories in
//! `repos`.

mod repos;

pub use repos::{
    clone_repo, delete, rename, repos, valid_full_name, valid_repo_name, Renamed, Repos,
};

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs;

/// Where the one-time code of the sign-in is typed.
pub const DEVICE_PATH: &str = "login/device";
/// Beyond gh's own (`repo`, `read:org`, `gist`): pushing GitHub Actions
/// workflows, as `gh auth setup-git` asks.
const SCOPES: &str = "workflow";
/// Deleting a repository: asked only when the user deletes one.
pub const DELETE_SCOPE: &str = "delete_repo";
/// Git's helper for github.com: the kit's `gh`, found in the kit's PATH.
const HELPER: &str = "!gh auth git-credential";
const HELPER_HOSTS: [&str; 2] = ["https://github.com", "https://gist.github.com"];

fn tr(lang: &str, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang, source, args)
}

pub fn gh_exe(kit: &Kit) -> PathBuf {
    kit.apps().join(r"tools\gh.exe")
}

fn git_exe(kit: &Kit) -> PathBuf {
    kit.apps().join(r"git\cmd\git.exe")
}

/// `gh` with the kit's environment and never a question.
fn gh(kit: &Kit, env: &KitEnv) -> Command {
    let mut cmd = jobs::kit_command(gh_exe(kit), env);
    cmd.env("GH_PROMPT_DISABLED", "1")
        .env("GH_NO_UPDATE_NOTIFIER", "1")
        .env("GH_SPINNER_DISABLED", "1")
        .stdin(Stdio::null());
    cmd
}

/// The account's token, from `gh` (`None`: not signed in).
pub fn token(kit: &Kit, env: &KitEnv) -> Option<String> {
    if !gh_exe(kit).is_file() {
        return None;
    }
    let out = gh(kit, env)
        .args(["auth", "token", "--hostname", "github.com"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let token = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    (out.status.success() && !token.is_empty() && !token.contains(char::is_whitespace))
        .then_some(token)
}

/// The active user of github.com in gh's `hosts.yml` (no network).
fn hosts_user(text: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        if !line.starts_with([' ', '\t']) {
            inside = line.trim_end() == "github.com:";
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        if inside && indent == 4 {
            if let Some(user) = line.trim().strip_prefix("user:") {
                let user = user.trim().trim_matches(['"', '\'']);
                if !user.is_empty() {
                    return Some(user.to_owned());
                }
            }
        }
    }
    None
}

fn login_name(kit: &Kit) -> Option<String> {
    fs::read_to_string(kit.data().join(r"gh\hosts.yml"))
        .ok()
        .and_then(|t| hosts_user(&t))
}

/// Opens `https://github.com/<path>` in the PC's browser, outside the job.
/// `path`: [`DEVICE_PATH`] or an `owner/repo` already validated.
pub fn open_web(path: &str) -> Result<()> {
    jobs::open_with_explorer(format!("https://github.com/{path}"))
}

// --- Git: the account for github.com ----------------------------------------

fn git_values(kit: &Kit, env: &KitEnv, key: &str) -> Vec<String> {
    jobs::kit_command(git_exe(kit), env)
        .args(["config", "--global", "--get-all", key])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// A helper of GitHub CLI: the kit's, or the one `gh auth setup-git` writes
/// with the full path of gh.
fn is_gh_helper(value: &str) -> bool {
    value.starts_with('!') && value.ends_with("auth git-credential") && value.contains("gh")
}

/// The kit's Git uses the GitHub account for github.com while signed in
/// (the helpers inherited from the system, Git Credential Manager, are
/// reset for those hosts only); signed out, gh's helper goes. A helper the
/// user wrote is left alone.
pub fn sync_git(kit: &Kit, env: &KitEnv, signed_in: bool) -> Result<()> {
    if !git_exe(kit).is_file() {
        return Ok(());
    }
    for host in HELPER_HOSTS {
        let key = format!("credential.{host}.helper");
        let current = git_values(kit, env, &key);
        let want: &[&str] = if signed_in { &["", HELPER] } else { &[] };
        if current.iter().map(String::as_str).eq(want.iter().copied()) {
            continue;
        }
        if !current.is_empty() && !current.iter().any(|v| is_gh_helper(v)) {
            continue;
        }
        let git = |args: &[&str]| {
            jobs::kit_command(git_exe(kit), env)
                .args(["config", "--global"])
                .args(args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .context("git config --global")
        };
        // Exit 5 when there was nothing to remove.
        git(&["--unset-all", &key])?;
        for value in want {
            if !git(&["--add", &key, value])?.success() {
                bail!("git config --global --add {key}");
            }
        }
    }
    Ok(())
}

// --- Sign-in ----------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// GitHub CLI is in the kit.
    pub installed: bool,
    pub signed_in: bool,
    pub login: Option<String>,
    /// A sign-in is waiting for the user.
    pub signing_in: bool,
    /// The one-time code to type at github.com/login/device.
    pub code: Option<String>,
    /// Why the last sign-in failed.
    pub error: Option<String>,
}

#[derive(Default)]
struct Pending {
    child: Option<Child>,
    running: bool,
    cancelled: bool,
    code: Option<String>,
    error: Option<String>,
}

/// The sign-in in progress (one at a time); in the app's job, so it ends
/// with the app.
#[derive(Default)]
pub struct Login {
    pending: Arc<Mutex<Pending>>,
}

/// The one-time code in a line of `gh auth login --web`
/// (`! First copy your one-time code: 1A2B-3C4D`).
fn one_time_code(line: &str) -> Option<String> {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .find(|w| {
            w.len() == 9
                && w.as_bytes()[4] == b'-'
                && w.chars()
                    .filter(|c| *c != '-')
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
        })
        .map(str::to_owned)
}

impl Login {
    fn lock(&self) -> std::sync::MutexGuard<'_, Pending> {
        self.pending.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn status(&self, kit: &Kit, env: &KitEnv) -> Status {
        let installed = gh_exe(kit).is_file();
        let signed_in = installed && token(kit, env).is_some();
        let p = self.lock();
        Status {
            installed,
            signed_in,
            login: signed_in.then(|| login_name(kit)).flatten(),
            signing_in: p.running,
            code: p.code.clone(),
            error: p.error.clone(),
        }
    }

    /// Starts `gh auth login --web` (with the permission to delete
    /// repositories when `delete`); `notify` runs when the code arrives and
    /// when the sign-in ends. A sign-in already running is kept.
    pub fn start(
        &self,
        kit: &Kit,
        env: &KitEnv,
        delete: bool,
        notify: Arc<dyn Fn() + Send + Sync>,
    ) -> Result<()> {
        let mut p = self.lock();
        if p.running {
            return Ok(());
        }
        let scopes = if delete {
            format!("{SCOPES},{DELETE_SCOPE}")
        } else {
            SCOPES.to_owned()
        };
        let mut child = jobs::spawn(
            gh(kit, env)
                .args([
                    "auth",
                    "login",
                    "--hostname",
                    "github.com",
                    "--git-protocol",
                    "https",
                    "--web",
                    "--insecure-storage",
                    "--scopes",
                    &scopes,
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::piped()),
        )?;
        let stderr = child.stderr.take().context("gh auth login: no stderr")?;
        *p = Pending {
            child: Some(child),
            running: true,
            ..Pending::default()
        };
        drop(p);
        crate::log::info("github: sign-in started");
        let (pending, kit, env) = (Arc::clone(&self.pending), kit.clone(), env.clone());
        std::thread::spawn(move || {
            let lock = || pending.lock().unwrap_or_else(|e| e.into_inner());
            let mut tail = String::new();
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                let line = line.trim().to_owned();
                if line.is_empty() {
                    continue;
                }
                if let Some(code) = one_time_code(&line).filter(|_| lock().code.is_none()) {
                    lock().code = Some(code);
                    if let Err(e) = open_web(DEVICE_PATH) {
                        crate::log::warn(&format!("github: browser: {e:#}"));
                    }
                    notify();
                } else {
                    tail = line;
                }
            }
            let child = lock().child.take();
            let ok = child
                .map(|mut c| c.wait().is_ok_and(|s| s.success()))
                .unwrap_or(false);
            let cancelled = lock().cancelled;
            if ok {
                if let Err(e) = sync_git(&kit, &env, true) {
                    crate::log::warn(&format!("github: git helper: {e:#}"));
                }
            }
            crate::log::info(&format!(
                "github: sign-in {}",
                if ok {
                    "done"
                } else if cancelled {
                    "cancelled"
                } else {
                    "failed"
                }
            ));
            {
                let mut p = lock();
                p.running = false;
                p.code = None;
                p.error = if ok || cancelled {
                    None
                } else if tail.is_empty() {
                    Some("gh auth login".to_owned())
                } else {
                    Some(tail)
                };
            }
            notify();
        });
        Ok(())
    }

    pub fn cancel(&self) {
        let mut p = self.lock();
        let p = &mut *p;
        if let Some(child) = p.child.as_mut() {
            p.cancelled = true;
            let _ = child.kill();
        }
    }
}

/// Signs the kit out of GitHub: gh forgets the account, Git stops using it.
pub fn sign_out(kit: &Kit, env: &KitEnv) -> Result<()> {
    if gh_exe(kit).is_file() && token(kit, env).is_some() {
        let mut cmd = gh(kit, env);
        cmd.args(["auth", "logout", "--hostname", "github.com"]);
        // With several accounts gh asks which one: the active one.
        if let Some(user) = login_name(kit) {
            cmd.args(["--user", &user]);
        }
        let out = cmd
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .context("gh auth logout")?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            bail!("{}", err.lines().last().unwrap_or("gh auth logout").trim());
        }
    }
    sync_git(kit, env, token(kit, env).is_some())
}

#[cfg(test)]
#[path = "../../../test/rust/github/mod.rs"]
mod tests;
