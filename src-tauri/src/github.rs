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

use std::collections::BTreeMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

use crate::env::KitEnv;
use crate::jobs;
use crate::kit::Kit;
use crate::net::{ApiError, Net};
use crate::projects::{self, Source};

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
    let system_root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    jobs::spawn_detached(
        Command::new(Path::new(&system_root).join("explorer.exe"))
            .arg(format!("https://github.com/{path}")),
        0,
    )
    .map(|_| ())
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

// --- Repositories -----------------------------------------------------------

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub full_name: String,
    pub name: String,
    pub owner: String,
    pub private: bool,
    pub fork: bool,
    pub archived: bool,
    pub description: String,
    pub language: String,
    pub stars: u64,
    pub pushed_at: Option<String>,
    pub html_url: String,
    /// The account administers it: it can rename and delete it.
    pub admin: bool,
    /// The project in `projects\` that is its clone.
    pub local: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repos {
    pub login: String,
    /// The account may delete repositories (`None`: the token does not say).
    pub can_delete: Option<bool>,
    pub repos: Vec<Repo>,
}

fn repo_from(v: &Value) -> Option<Repo> {
    let s = |k: &str| {
        v.get(k)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    let b = |k: &str| v.get(k).and_then(Value::as_bool).unwrap_or(false);
    let full_name = s("full_name");
    let (owner, name) = full_name.split_once('/')?;
    Some(Repo {
        owner: owner.to_owned(),
        name: name.to_owned(),
        private: b("private"),
        fork: b("fork"),
        archived: b("archived"),
        description: s("description"),
        language: s("language"),
        stars: v
            .get("stargazers_count")
            .and_then(Value::as_u64)
            .unwrap_or(0),
        pushed_at: v
            .get("pushed_at")
            .and_then(Value::as_str)
            .map(str::to_owned),
        html_url: s("html_url"),
        admin: v
            .pointer("/permissions/admin")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        local: None,
        full_name,
    })
}

/// The `rel="next"` address of a `Link` header.
fn next_page(link: &str) -> Option<String> {
    link.split(',').find_map(|part| {
        let (url, rel) = part.split_once(';')?;
        rel.contains("rel=\"next\"").then(|| {
            url.trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_owned()
        })
    })
}

/// `owner/repo` (lower case) of a github.com remote: https or ssh, with or
/// without `.git` and a user name.
fn github_full_name(url: &str) -> Option<String> {
    let url = url.trim().to_ascii_lowercase();
    let url = url.trim_end_matches('/').trim_end_matches(".git");
    let at = url
        .find("github.com/")
        .or_else(|| url.find("github.com:"))?;
    let before = &url[..at];
    if !(before.ends_with("//") || before.ends_with('@') || before.ends_with('.')) {
        return None;
    }
    let rest = &url[at + "github.com/".len()..];
    let (owner, name) = rest.split_once('/')?;
    (!owner.is_empty() && !name.is_empty() && !name.contains('/'))
        .then(|| format!("{owner}/{name}"))
}

/// The remotes' URLs in a clone's `.git\config`.
fn remote_urls(dir: &Path) -> Vec<String> {
    fs::read_to_string(dir.join(r".git\config"))
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, v) = l.split_once('=')?;
            (k.trim() == "url").then(|| v.trim().to_owned())
        })
        .collect()
}

/// The projects that are clones of a GitHub repository: `owner/repo` (lower
/// case) → project id. The first folder in name order wins.
pub fn clones(kit: &Kit) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for dir in projects::folders(kit) {
        let Some(id) = dir.file_name().map(|n| n.to_string_lossy().into_owned()) else {
            continue;
        };
        for url in remote_urls(&dir) {
            if let Some(full) = github_full_name(&url) {
                out.entry(full).or_insert_with(|| id.clone());
            }
        }
    }
    out
}

fn api_url(full_name: &str) -> Result<String> {
    match Source::parse(full_name) {
        Some(Source::GitHub(r)) if r == full_name => {
            Ok(format!("https://api.github.com/repos/{r}"))
        }
        _ => bail!("not a GitHub repository: {full_name}"),
    }
}

/// `owner/repo` as given by GitHub and safe for a URL and a command line.
pub fn valid_full_name(full_name: &str) -> bool {
    api_url(full_name).is_ok()
}

/// The account and its repositories (owner, collaborator, organizations),
/// each with its clone in `projects\` if there is one.
pub fn repos(kit: &Kit, net: &Net, token: &str) -> std::result::Result<Repos, ApiError> {
    let user = net.github_call("GET", "https://api.github.com/user", token, None)?;
    let login = user
        .json
        .get("login")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let can_delete = user
        .scopes
        .as_deref()
        .map(|s| s.split(',').any(|x| x.trim() == DELETE_SCOPE));
    let mut list = Vec::new();
    let mut url = Some(
        "https://api.github.com/user/repos?per_page=100&sort=pushed&affiliation=owner,collaborator,organization_member"
            .to_owned(),
    );
    // 100 pages = 10 000 repositories: enough, and never an endless loop.
    for _ in 0..100 {
        let Some(u) = url.take() else { break };
        let page = net.github_call("GET", &u, token, None)?;
        list.extend(
            page.json
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(repo_from),
        );
        url = next_page(&page.link).filter(|n| n.starts_with("https://api.github.com/"));
    }
    let local = clones(kit);
    for r in &mut list {
        r.local = local.get(&r.full_name.to_ascii_lowercase()).cloned();
    }
    Ok(Repos {
        login,
        can_delete,
        repos: list,
    })
}

/// Clones a repository of the account into `projects\<name>`, or
/// `projects\<owner>-<name>` when `<name>` is taken; an existing clone is
/// returned as it is. Returns the project id.
pub fn clone_repo(
    kit: &Kit,
    env: &KitEnv,
    full_name: &str,
    token: Option<&str>,
    lang: &str,
) -> Result<String> {
    api_url(full_name)?;
    if let Some(id) = clones(kit).get(&full_name.to_ascii_lowercase()) {
        return Ok(id.clone());
    }
    let (owner, name) = full_name.split_once('/').unwrap_or_default();
    let candidates = [name.to_owned(), format!("{owner}-{name}")];
    let Some(target) = candidates
        .iter()
        .find(|c| crate::kit::validate_project_name(c).is_ok() && !kit.projects().join(c).exists())
    else {
        bail!(
            "{}",
            tr(
                lang,
                "In projects esistono già le cartelle {0} e {1}.",
                &[&candidates[0], &candidates[1]]
            )
        );
    };
    projects::clone(kit, env, full_name, Some(target), lang, token)?;
    Ok(target.clone())
}

/// A repository name GitHub accepts.
pub fn valid_repo_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name.len() <= 100
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// `url` with its last path segment (the repository) renamed to `name`,
/// keeping `.git` and the protocol.
fn renamed_url(url: &str, name: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    let (base, git) = match trimmed.strip_suffix(".git") {
        Some(b) => (b, ".git"),
        None => (trimmed, ""),
    };
    let cut = base.rfind(['/', ':']).map_or(0, |i| i + 1);
    format!("{}{name}{git}", &base[..cut])
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Renamed {
    pub repo: Repo,
    /// The clone whose remotes now point to the new name.
    pub local: Option<String>,
}

/// Renames a repository on GitHub (GitHub redirects the old addresses);
/// the remotes of its clone follow. The project folder keeps its name: it
/// is the project's id (settings, ports, agent).
pub fn rename(
    kit: &Kit,
    env: &KitEnv,
    net: &Net,
    token: &str,
    full_name: &str,
    new_name: &str,
) -> std::result::Result<Renamed, ApiError> {
    let url = api_url(full_name).map_err(|e| ApiError {
        status: None,
        retry_after: None,
        detail: e.to_string(),
    })?;
    let answer = net.github_call("PATCH", &url, token, Some(&json!({ "name": new_name })))?;
    let mut repo = repo_from(&answer.json).ok_or_else(|| ApiError {
        status: Some(200),
        retry_after: None,
        detail: "unexpected answer from GitHub".into(),
    })?;
    let local = clones(kit).get(&full_name.to_ascii_lowercase()).cloned();
    if let Some(id) = &local {
        if let Err(e) = follow_rename(kit, env, &kit.projects().join(id), full_name, &repo.name) {
            crate::log::warn(&format!("github: remotes of {id}: {e:#}"));
        }
    }
    repo.local = clones(kit)
        .get(&repo.full_name.to_ascii_lowercase())
        .cloned();
    Ok(Renamed { repo, local })
}

fn follow_rename(kit: &Kit, env: &KitEnv, dir: &Path, old: &str, name: &str) -> Result<()> {
    let git = || {
        let mut cmd = jobs::kit_command(git_exe(kit), env);
        cmd.arg("-C").arg(dir).stdin(Stdio::null());
        cmd
    };
    let out = git()
        .args(["config", "--get-regexp", r"^remote\..*\.url$"])
        .output()
        .context("git config --get-regexp")?;
    let old = old.to_ascii_lowercase();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Some((key, url)) = line.trim().split_once(char::is_whitespace) else {
            continue;
        };
        let Some(remote) = key
            .strip_prefix("remote.")
            .and_then(|k| k.strip_suffix(".url"))
        else {
            continue;
        };
        if github_full_name(url).as_deref() != Some(old.as_str()) {
            continue;
        }
        let status = git()
            .args(["remote", "set-url", "--", remote])
            .arg(renamed_url(url, name))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .context("git remote set-url")?;
        if !status.success() {
            bail!("git remote set-url {remote}");
        }
    }
    Ok(())
}

/// Deletes a repository from GitHub; its clone, if any, stays.
pub fn delete(net: &Net, token: &str, full_name: &str) -> std::result::Result<(), ApiError> {
    let url = api_url(full_name).map_err(|e| ApiError {
        status: None,
        retry_after: None,
        detail: e.to_string(),
    })?;
    net.github_call("DELETE", &url, token, None).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn hosts_file() {
        let text = "github.com:\n    users:\n        user:\n            oauth_token: x\n        nabre:\n    git_protocol: https\n    user: nabre\n    oauth_token: gho_x\nexample.com:\n    user: other\n";
        assert_eq!(hosts_user(text).as_deref(), Some("nabre"));
        assert_eq!(hosts_user("example.com:\n    user: other\n"), None);
        assert_eq!(hosts_user(""), None);
    }

    #[test]
    fn code_of_the_sign_in() {
        assert_eq!(
            one_time_code("! First copy your one-time code: 1A2B-3C4D").as_deref(),
            Some("1A2B-3C4D")
        );
        assert_eq!(
            one_time_code(
                "Open this URL to continue in your web browser: https://github.com/login/device"
            ),
            None
        );
        assert_eq!(one_time_code("abcd-efgh"), None);
    }

    #[test]
    fn gh_helpers() {
        assert!(is_gh_helper(HELPER));
        assert!(is_gh_helper(
            r"!'C:\\Kit\\apps\\tools\\gh.exe' auth git-credential"
        ));
        assert!(!is_gh_helper("manager"));
        assert!(!is_gh_helper(""));
    }

    #[test]
    fn remotes_and_pages() {
        for url in [
            "https://github.com/Nabre/Dev-Kit.git",
            "https://x-access-token@github.com/nabre/dev-kit/",
            "git@github.com:nabre/dev-kit.git",
            "ssh://git@github.com/nabre/dev-kit",
        ] {
            assert_eq!(
                github_full_name(url).as_deref(),
                Some("nabre/dev-kit"),
                "{url}"
            );
        }
        for url in [
            "https://notgithub.com/a/b",
            "https://github.com/a",
            "https://github.com/a/b/c",
            "https://gitlab.com/a/b",
        ] {
            assert_eq!(github_full_name(url), None, "{url}");
        }
        assert_eq!(
            renamed_url("https://github.com/a/old.git", "new"),
            "https://github.com/a/new.git"
        );
        assert_eq!(
            renamed_url("git@github.com:a/old", "new"),
            "git@github.com:a/new"
        );
        let link = r#"<https://api.github.com/user/repos?page=2>; rel="next", <https://api.github.com/user/repos?page=5>; rel="last""#;
        assert_eq!(
            next_page(link).as_deref(),
            Some("https://api.github.com/user/repos?page=2")
        );
        assert_eq!(next_page(r#"<https://x>; rel="prev""#), None);
        assert_eq!(next_page(""), None);
    }

    #[test]
    fn names() {
        assert!(valid_full_name("nabre/dev-kit"));
        for bad in [
            "nabre",
            "a/b/c",
            "../x",
            "a/-b",
            "a/b c",
            "https://github.com/a/b",
        ] {
            assert!(!valid_full_name(bad), "{bad}");
        }
        assert!(valid_repo_name("dev-kit_2.0"));
        for bad in ["", ".", "..", "a b", "a/b", "é"] {
            assert!(!valid_repo_name(bad), "{bad}");
        }
    }

    #[test]
    fn repository_from_the_api() {
        let v = json!({
            "full_name": "nabre/dev-kit", "private": true, "fork": false,
            "description": null, "language": "Rust", "stargazers_count": 3,
            "pushed_at": "2026-10-14T08:00:00Z", "html_url": "https://github.com/nabre/dev-kit",
            "permissions": { "admin": true }
        });
        let r = repo_from(&v).unwrap();
        assert_eq!((r.owner.as_str(), r.name.as_str()), ("nabre", "dev-kit"));
        assert!(r.private && r.admin && !r.fork);
        assert_eq!(r.description, "");
        assert_eq!(r.stars, 3);
        assert!(repo_from(&json!({ "name": "x" })).is_none());
    }

    #[test]
    fn local_clones() {
        let tmp = TempDir::new("github-clones");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let repo = |dir: &str, url: &str| {
            let git = kit.projects().join(dir).join(".git");
            fs::create_dir_all(&git).unwrap();
            fs::write(
                git.join("config"),
                format!("[core]\n\tbare = false\n[remote \"origin\"]\n\turl = {url}\n"),
            )
            .unwrap();
        };
        repo("dev-kit", "https://github.com/nabre/dev-kit.git");
        repo("other", "https://gitlab.com/nabre/other.git");
        repo("nabre-dev-kit", "https://github.com/nabre/dev-kit");
        fs::create_dir_all(kit.projects().join("plain")).unwrap();
        let c = clones(&kit);
        assert_eq!(c.len(), 1);
        assert_eq!(c.get("nabre/dev-kit").map(String::as_str), Some("dev-kit"));
    }
}
