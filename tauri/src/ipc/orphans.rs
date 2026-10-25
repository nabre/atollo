//! Data of the kit without a project (D-89): the folders of
//! `data\projects\` whose project is gone from `projects\`, what each one
//! holds, and giving them to a project. Never deleted automatically:
//! deleting one is the deletion of a project, with its confirmation.

use super::*;
use crate::projects::state::{self as project_state, Holdings};

/// The orphans, with what each one holds (no password).
#[tauri::command]
pub fn projects_orphans<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<Holdings>> {
    ensure_main(&webview)?;
    let kit = &state.kit;
    Ok(project_state::orphans(kit)
        .iter()
        .filter_map(|p| project_state::holdings(kit, p))
        .collect())
}

/// Gives the files of `orphan` to `project` (renamed outside the app);
/// refused when `project` has data of the kit of its own besides its
/// settings (ports, servers, databases, copies).
#[tauri::command]
pub fn project_orphan_adopt<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    orphan: String,
    project: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    project_state::adopt(&state.kit, state.lang(), &orphan, &project).or_fail("adopt")?;
    runner.rename_project(&orphan, &project);
    crate::log::info(&format!("data of {orphan} given to {project}"));
    Ok(())
}
