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
//! - Who signs the commits (`user.name`, `user.email`): empty, they come
//!   from the account at sign-in and at start, with GitHub's `noreply`
//!   address so the real one stays private; the Settings change them
//!   (D-137).
//! - Git trusts the repositories inside the kit's folder only
//!   (`safe.directory = <kit>/*`, rewritten at every start as the kit
//!   moves), not every repository of the PC as with `*` before (D-137).
//!
//! Here the account, the sign-in and Git's settings; the repositories in
//! `repos`.

pub(crate) mod repos;

pub use repos::{
    clone_repo, delete, rename, repos, valid_full_name, valid_repo_name, Renamed, Repos,
};

use std::fs;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

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
    crate::setup::catalog::kit_path(kit, r"gh\bin\gh.exe")
}

fn git_exe(kit: &Kit) -> PathBuf {
    crate::setup::catalog::kit_path(kit, r"git\cmd\git.exe")
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

/// gh's `hosts.yml`, in the kit's `GH_CONFIG_DIR`.
pub(crate) fn hosts_file(kit: &Kit) -> PathBuf {
    kit.data().join("gh").join("hosts.yml")
}

fn login_name(kit: &Kit) -> Option<String> {
    fs::read_to_string(hosts_file(kit))
        .ok()
        .and_then(|t| hosts_user(&t))
}

/// Opens `https://github.com/<path>` in the PC's browser, outside the job.
/// `path`: [`DEVICE_PATH`] or an `owner/repo` already validated.
pub fn open_web(path: &str) -> Result<()> {
    jobs::open_url(&format!("https://github.com/{path}"))
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
    if let Err(e) = sync_safe_directory(kit, env) {
        crate::log::warn(&format!("github: safe.directory: {e:#}"));
    }
    if signed_in {
        if let Err(e) = fill_identity(kit, env) {
            crate::log::warn(&format!("github: git identity: {e:#}"));
        }
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
        let git = |args: &[&str]| git_config(kit, env, args);
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

/// `git config --global` with the kit's Git, its output not wanted.
fn git_config(kit: &Kit, env: &KitEnv, args: &[&str]) -> Result<std::process::ExitStatus> {
    jobs::kit_command(git_exe(kit), env)
        .args(["config", "--global"])
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("git config --global")
}

/// `safe.directory` for the kit's folder: `D:/kit/*` (Git 2.46 and later
/// read a trailing `/*` as "everything inside").
pub(crate) fn safe_pattern(root: &std::path::Path) -> String {
    let root = root.display().to_string().replace('\\', "/");
    format!("{}/*", root.trim_end_matches('/'))
}

/// A value of `safe.directory` written by a kit: `*` (before D-137) or the
/// folder of a kit, here or where it was before (it has `data/app`, or it
/// is not there any more). The user's own values stay.
pub(crate) fn kit_safe_value(value: &str) -> bool {
    value == "*"
        || value.strip_suffix("/*").is_some_and(|dir| {
            let dir = std::path::Path::new(dir);
            !dir.exists() || dir.join("data").join("app").is_dir()
        })
}

fn sync_safe_directory(kit: &Kit, env: &KitEnv) -> Result<()> {
    let want = safe_pattern(kit.root());
    let current = git_values(kit, env, "safe.directory");
    if current == [want.clone()] {
        return Ok(());
    }
    let keep: Vec<&String> = current.iter().filter(|v| !kit_safe_value(v)).collect();
    // Exit 5 when there was nothing to remove.
    git_config(kit, env, &["--unset-all", "safe.directory"])?;
    for value in keep.into_iter().chain([&want]) {
        if !git_config(kit, env, &["--add", "safe.directory", value])?.success() {
            bail!("git config --global --add safe.directory");
        }
    }
    Ok(())
}

// --- Git: who signs the commits ----------------------------------------------

/// `user.name` and `user.email` of the kit's Git (empty: not set).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub name: String,
    pub email: String,
}

/// GitHub's private address of an account: commits show the account
/// without the real address.
pub(crate) fn noreply(id: u64, login: &str) -> String {
    format!("{id}+{login}@users.noreply.github.com")
}

/// The identity as saved, or the Italian source of what is wrong.
pub(crate) fn clean_identity(input: &Identity) -> std::result::Result<Identity, &'static str> {
    let name = input.name.trim().to_owned();
    let email = input.email.trim().to_owned();
    if name.chars().any(char::is_control) || name.chars().count() > 100 {
        return Err("Nome non valido.");
    }
    let bad_email = !email.is_empty()
        && (email
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '<' || c == '>')
            || !email
                .split_once('@')
                .is_some_and(|(user, host)| !user.is_empty() && host.contains('.')));
    if bad_email {
        return Err("Indirizzo e-mail non valido.");
    }
    Ok(Identity { name, email })
}

/// The identity of the kit's Git; `None` when Git is not in the kit.
pub fn identity(kit: &Kit, env: &KitEnv) -> Option<Identity> {
    if !git_exe(kit).is_file() {
        return None;
    }
    let one = |key: &str| git_values(kit, env, key).pop().unwrap_or_default();
    Some(Identity {
        name: one("user.name"),
        email: one("user.email"),
    })
}

/// Saves `id` (an empty value is removed); returns it as saved.
pub fn set_identity(kit: &Kit, env: &KitEnv, id: &Identity, lang: &str) -> Result<Identity> {
    let id = clean_identity(id).map_err(|source| anyhow::anyhow!(tr(lang, source, &[])))?;
    if !git_exe(kit).is_file() {
        bail!("{}", tr(lang, "Git non è nel kit.", &[]));
    }
    for (key, value) in [("user.name", &id.name), ("user.email", &id.email)] {
        git_config(kit, env, &["--unset-all", key])?;
        if !value.is_empty() && !git_config(kit, env, &[key, value])?.success() {
            bail!("git config --global {key}");
        }
    }
    Ok(id)
}

#[derive(Deserialize)]
struct AccountUser {
    login: String,
    id: u64,
    #[serde(default)]
    name: Option<String>,
}

/// Name and `noreply` address of the signed-in account, when neither is
/// set: one call to GitHub's API (`gh api user`), then never again.
fn fill_identity(kit: &Kit, env: &KitEnv) -> Result<()> {
    let Some(current) = identity(kit, env) else {
        return Ok(());
    };
    if !current.name.is_empty() || !current.email.is_empty() || !gh_exe(kit).is_file() {
        return Ok(());
    }
    let out = gh(kit, env)
        .args(["api", "user"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .context("gh api user")?;
    if !out.status.success() {
        bail!("gh api user failed");
    }
    let user: AccountUser = serde_json::from_slice(&out.stdout).context("gh api user")?;
    let name = user
        .name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| user.login.clone());
    let id = Identity {
        name,
        email: noreply(user.id, &user.login),
    };
    set_identity(kit, env, &id, "en").map(drop)
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
