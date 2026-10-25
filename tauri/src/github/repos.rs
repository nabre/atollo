//! The account's repositories on the GitHub page: listed with the API,
//! cloned into `projects\`, renamed and deleted on GitHub.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Stdio;

use anyhow::{bail, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

use super::{git_exe, tr, DELETE_SCOPE};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::net::{ApiError, Net};
use crate::platform::jobs;
use crate::projects::{self, Source};

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
pub(crate) fn remote_urls(dir: &Path) -> Vec<String> {
    fs::read_to_string(dir.join(".git").join("config"))
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
/// returned as it is.
pub fn clone_repo(
    kit: &Kit,
    env: &KitEnv,
    full_name: &str,
    token: Option<&str>,
    lang: &str,
) -> Result<Cloned> {
    api_url(full_name)?;
    if let Some(id) = clones(kit).get(&full_name.to_ascii_lowercase()) {
        return Ok(Cloned {
            project: id.clone(),
            new: false,
        });
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
    Ok(Cloned {
        project: target.clone(),
        new: true,
    })
}

/// The project of a repository cloned by [`clone_repo`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cloned {
    pub project: String,
    /// Copied now; `false`: a clone that was already there.
    pub new: bool,
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
            crate::common::log::warn(&format!("github: remotes of {id}: {e:#}"));
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
#[path = "../../../test/rust/github/repos.rs"]
mod tests;
