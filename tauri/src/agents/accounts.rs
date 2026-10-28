//! The agents' accounts (D-185): named sign-ins kept in the kit, in
//! `data/accounts/<agent>/<account>/`, each the agent's own folder of
//! settings and credentials. A project picks one per agent (its settings,
//! `agentAccounts`); the project's sandbox mounts the chosen folder and the
//! agent's session points at it (`levels::prepare`), so the sign-in made
//! inside the sandbox lands there: it survives «Ricrea» and is shared by
//! every project that picks the same account.

use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::Serialize;

use crate::kit::Kit;
use crate::levels::Agent;

/// The account a project uses until it picks another: made on first use.
pub const DEFAULT: &str = "default";

/// The variable that moves `agent`'s folder of settings and credentials,
/// the same the kit gives it on the PC (`env::agent_vars`); `None` for
/// Antigravity, whose CLI has none [unverified]: it keeps the sandbox's
/// home.
pub fn env_var(agent: Agent) -> Option<&'static str> {
    match agent {
        Agent::Claude => Some("CLAUDE_CONFIG_DIR"),
        Agent::Codex => Some("CODEX_HOME"),
        Agent::Pi => Some("PI_CODING_AGENT_DIR"),
        Agent::Antigravity => None,
    }
}

/// The file of `agent`'s sign-in in its folder, removed by «Esci».
fn credentials(agent: Agent) -> &'static str {
    match agent {
        Agent::Claude => ".credentials.json",
        Agent::Codex | Agent::Pi | Agent::Antigravity => "auth.json",
    }
}

/// `data/accounts/<agent>`.
fn agent_root(kit: &Kit, agent: Agent) -> PathBuf {
    kit.data().join("accounts").join(agent.id())
}

/// The folder of `agent`'s account `name`.
pub fn dir(kit: &Kit, agent: Agent, name: &str) -> PathBuf {
    agent_root(kit, agent).join(name)
}

/// `name` is a valid account's: the rules of a project's name (a folder on
/// every system).
pub fn valid_name(name: &str) -> bool {
    crate::kit::validate_project_name(name).is_ok()
}

/// An account as the interface lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub name: String,
    /// Its credentials' file is there: signed in once, in some sandbox.
    pub signed_in: bool,
    /// The projects that use it.
    pub used_by: Vec<String>,
}

/// The accounts of one agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentAccounts {
    pub agent: Agent,
    /// False for an agent whose folder cannot be moved (Antigravity).
    pub supported: bool,
    pub accounts: Vec<Account>,
}

/// The project `project` uses for `agent`: its choice, else [`DEFAULT`].
pub fn chosen(kit: &Kit, project: &str, agent: Agent) -> String {
    crate::projects::settings::load(kit, project)
        .agent_accounts
        .get(agent.id())
        .cloned()
        .unwrap_or_else(|| DEFAULT.to_owned())
}

/// The projects that use `agent`'s account `name` (the default one counted
/// for every project that picked none).
pub fn used_by(kit: &Kit, agent: Agent, name: &str) -> Vec<String> {
    crate::projects::folders(kit)
        .iter()
        .filter_map(|d| d.file_name()?.to_str().map(str::to_owned))
        .filter(|p| chosen(kit, p, agent) == name)
        .collect()
}

/// Every agent's accounts, sorted, [`DEFAULT`] always among them.
pub fn list(kit: &Kit) -> Vec<AgentAccounts> {
    Agent::ALL
        .into_iter()
        .map(|agent| {
            let mut names: Vec<String> = fs::read_dir(agent_root(kit, agent))
                .into_iter()
                .flatten()
                .flatten()
                .filter(|e| e.path().is_dir())
                .filter_map(|e| e.file_name().to_str().map(str::to_owned))
                .filter(|n| valid_name(n))
                .collect();
            if !names.iter().any(|n| n == DEFAULT) {
                names.push(DEFAULT.to_owned());
            }
            names.sort();
            AgentAccounts {
                agent,
                supported: env_var(agent).is_some(),
                accounts: names
                    .into_iter()
                    .map(|name| Account {
                        signed_in: dir(kit, agent, &name).join(credentials(agent)).is_file(),
                        used_by: used_by(kit, agent, &name),
                        name,
                    })
                    .collect(),
            }
        })
        .collect()
}

fn check(agent: Agent, name: &str) -> Result<()> {
    if env_var(agent).is_none() {
        bail!("{}: no accounts", agent.id());
    }
    if !valid_name(name) {
        bail!("{name:?}: not a valid account name");
    }
    Ok(())
}

/// Makes `agent`'s account `name` (its empty folder); one there already is
/// fine. Returns its folder.
pub fn create(kit: &Kit, agent: Agent, name: &str) -> Result<PathBuf> {
    check(agent, name)?;
    let d = dir(kit, agent, name);
    fs::create_dir_all(&d).with_context(|| d.display().to_string())?;
    Ok(d)
}

/// Renames `agent`'s account `from` to `to`, and the projects that used
/// it follow. [`DEFAULT`] keeps its name; `to` must be free.
pub fn rename(kit: &Kit, agent: Agent, from: &str, to: &str) -> Result<()> {
    check(agent, from)?;
    check(agent, to)?;
    if from == DEFAULT {
        bail!("the default account keeps its name");
    }
    let (old, new) = (dir(kit, agent, from), dir(kit, agent, to));
    if new.exists() {
        bail!("{to}: an account with this name exists");
    }
    let users = used_by(kit, agent, from);
    fs::rename(&old, &new).with_context(|| old.display().to_string())?;
    for project in users {
        set(kit, &project, agent, to);
    }
    Ok(())
}

/// Why `agent`'s account `name` cannot be deleted, if it cannot: the
/// default one, or one a project uses.
pub fn delete_refused(kit: &Kit, agent: Agent, name: &str) -> Option<Vec<String>> {
    if name == DEFAULT {
        return Some(Vec::new());
    }
    let users = used_by(kit, agent, name);
    (!users.is_empty()).then_some(users)
}

/// Deletes `agent`'s account `name` and its sign-in; refused for the
/// default one and for one a project uses ([`delete_refused`]).
pub fn delete(kit: &Kit, agent: Agent, name: &str) -> Result<()> {
    check(agent, name)?;
    if let Some(users) = delete_refused(kit, agent, name) {
        bail!("{name}: in use ({})", users.join(", "));
    }
    let d = dir(kit, agent, name);
    match fs::remove_dir_all(&d) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| d.display().to_string())
        }
        _ => Ok(()),
    }
}

/// `project` uses `agent`'s account `name` from its next session.
pub fn set(kit: &Kit, project: &str, agent: Agent, name: &str) {
    let id = agent.id().to_owned();
    crate::projects::settings::update(kit, project, |s| {
        if name == DEFAULT {
            s.agent_accounts.remove(&id);
        } else {
            s.agent_accounts.insert(id, name.to_owned());
        }
    });
}

/// «Esci» for `agent`'s account `name`: its credentials' file removed. The
/// sessions on the provider's site are not revoked.
pub fn sign_out(kit: &Kit, agent: Agent, name: &str) -> Result<()> {
    check(agent, name)?;
    let file = dir(kit, agent, name).join(credentials(agent));
    match fs::remove_file(&file) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
            Err(e).with_context(|| file.display().to_string())
        }
        _ => Ok(()),
    }
}

/// The folder of the account `project` uses for `agent`, made if missing,
/// with the variable that points the agent at it; `None` for Antigravity.
pub fn for_session(
    kit: &Kit,
    project: &str,
    agent: Agent,
) -> Result<Option<(PathBuf, &'static str)>> {
    let Some(var) = env_var(agent) else {
        return Ok(None);
    };
    let name = chosen(kit, project, agent);
    let name = if valid_name(&name) {
        name
    } else {
        DEFAULT.to_owned()
    };
    Ok(Some((create(kit, agent, &name)?, var)))
}

#[cfg(test)]
#[path = "../../../test/rust/agents/accounts.rs"]
mod tests;
