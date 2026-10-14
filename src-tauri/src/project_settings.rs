//! The kit's settings of each project, kept in
//! `data\app\projects\<project>\dev-settings.json` and never in the project's
//! folder, so the project stays as its repository has it. Today: the agent
//! last opened in it (the default of the agent choice) and when the editor,
//! a console or an agent last opened it (the order of the projects).
//! Keys written by a newer version of the app are kept as they are.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::kit::{validate_project_name, write_atomic, Kit};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    /// Id of the agent last opened in the project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Seconds since 1970 of the last editor, console or agent opened in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_opened: Option<u64>,
    #[serde(flatten)]
    pub other: serde_json::Map<String, serde_json::Value>,
}

/// The folder of the kit's files of `project` (`None`: not a valid name).
pub fn dir(kit: &Kit, project: &str) -> Option<PathBuf> {
    validate_project_name(project).ok()?;
    Some(kit.app_dir().join("projects").join(project))
}

pub fn load(kit: &Kit, project: &str) -> ProjectSettings {
    dir(kit, project)
        .and_then(|d| std::fs::read_to_string(d.join("dev-settings.json")).ok())
        .and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default()
}

/// Changes the settings of `project` and saves them; a failure only goes to
/// the log (the settings help, they never stop an action).
pub fn update(kit: &Kit, project: &str, change: impl FnOnce(&mut ProjectSettings)) {
    let Some(d) = dir(kit, project) else { return };
    let mut s = load(kit, project);
    change(&mut s);
    let result = std::fs::create_dir_all(&d).and_then(|()| {
        let text = serde_json::to_string_pretty(&s).unwrap_or_else(|_| "{}".into());
        write_atomic(&d.join("dev-settings.json"), text.as_bytes())
    });
    if let Err(e) = result {
        crate::log::warn(&format!("project settings of {project}: {e}"));
    }
}

/// The project was opened now (editor, console, agent).
pub fn opened(kit: &Kit, project: &str) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    update(kit, project, |s| s.last_opened = Some(now));
}

/// The project's folder is gone: its settings go with it.
pub fn remove(kit: &Kit, project: &str) {
    if let Some(d) = dir(kit, project) {
        let _ = std::fs::remove_dir_all(d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn saved_in_data_and_kept() {
        let tmp = TempDir::new("project-settings");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        assert_eq!(load(&kit, "app"), ProjectSettings::default());
        update(&kit, "app", |s| s.agent = Some("codex".into()));
        opened(&kit, "app");
        let file = kit.data().join(r"app\projects\app\dev-settings.json");
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("\"agent\": \"codex\"") && text.contains("lastOpened"));
        // a key of a newer version stays
        std::fs::write(&file, r#"{"agent":"claude","future":{"x":1}}"#).unwrap();
        opened(&kit, "app");
        let s = load(&kit, "app");
        assert_eq!(s.agent.as_deref(), Some("claude"));
        assert!(s.other.contains_key("future") && s.last_opened.is_some());
        // never outside data\app\projects
        for bad in ["..", r"..\x", "a/b", "con"] {
            update(&kit, bad, |s| s.agent = Some("x".into()));
            assert!(dir(&kit, bad).is_none(), "{bad}");
        }
        remove(&kit, "app");
        assert!(!file.exists());
    }
}
