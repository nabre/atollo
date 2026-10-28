//! The kit's settings of each project, kept in
//! `data\projects\<project>\dev-settings.json` and never in the project's
//! folder, so the project stays as its repository has it. Today: the agent
//! last opened in it (recorded only, D-146) and when the editor,
//! a console or an agent last opened it (the order of the projects), the
//! editor extensions added to or removed from those of its languages, the
//! versions of its languages, whether it is among the favorites of the
//! Projects home.
//! Keys written by a newer version of the app are kept as they are.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::kit::Kit;
use crate::projects::state::ProjectState;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    /// Id of the agent last opened in the project. Only recorded: no choice
    /// starts from it any more, every installed agent has its own button
    /// in the projects (D-146); kept so a project's file stays readable by
    /// older versions of the app and as a trace of the last launch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Seconds since 1970 of the last editor, console or agent opened in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_opened: Option<u64>,
    /// Among the favorites, first on the Projects home.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pinned: bool,
    /// Editor extensions changed from those of its languages (`editor/ext.rs`).
    #[serde(default, skip_serializing_if = "Extensions::is_empty")]
    pub extensions: Extensions,
    /// Lines of the languages chosen for the project, by component id
    /// (`versions/mod.rs`); missing = the kit's.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub versions: BTreeMap<String, String>,
    /// The kit's GitHub access in the project's VM (D-172): the proxy adds
    /// it to the VM's requests to GitHub, so the project's code can act as
    /// the user there. Off unless the user turns it on for this project.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub github_in_vm: bool,
    /// The account of each agent (its id) the project uses in its sandbox
    /// (`agents::accounts`, D-185); missing = the default one.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub agent_accounts: BTreeMap<String, String>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// Lower case `publisher.name` ids, sorted.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
    /// Wanted besides those of its languages.
    #[serde(default)]
    pub added: Vec<String>,
    /// Of its languages, not wanted.
    #[serde(default)]
    pub removed: Vec<String>,
}

impl Extensions {
    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty()
    }
}

pub fn load(kit: &Kit, project: &str) -> ProjectSettings {
    ProjectState::of(kit, project)
        .and_then(|s| crate::common::fsx::read_json(&s.settings_file()))
        .unwrap_or_default()
}

/// Changes the settings of `project` and saves them, one change at a time
/// (pinning, the agent opened, extensions and versions can come together);
/// a failure only goes to the log (the settings help, they never stop an
/// action).
pub fn update(kit: &Kit, project: &str, change: impl FnOnce(&mut ProjectSettings)) {
    static ONE: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let Some(state) = ProjectState::of(kit, project) else {
        return;
    };
    let _one = ONE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut s = load(kit, project);
    change(&mut s);
    if let Err(e) = crate::common::fsx::write_json(&state.settings_file(), &s) {
        crate::common::log::warn(&format!("project settings of {project}: {e}"));
    }
}

/// The project was opened now (editor, console, agent).
pub fn opened(kit: &Kit, project: &str) {
    let now = crate::common::time::now_secs();
    update(kit, project, |s| s.last_opened = Some(now));
}

/// The project's folder is gone: its settings go with it, and everything
/// else the kit keeps for it (`data\projects\<project>\`, passwords and
/// database copies included: the caller drops the databases first).
pub fn remove(kit: &Kit, project: &str) -> std::io::Result<()> {
    match ProjectState::of(kit, project) {
        Some(state) => state.remove(),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "../../../test/rust/projects/settings.rs"]
mod tests;
