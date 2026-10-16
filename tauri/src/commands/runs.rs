//! Projects' actions started by the kit (projects/run.rs): start, stop, log, ports.

use super::*;
use crate::projects::run::{site_mode, SiteMode};

#[tauri::command]
pub fn run_list<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> CmdResult<Vec<crate::projects::run::RunInfo>> {
    ensure_main(&webview)?;
    Ok(runner.list())
}

/// Starts an action of a project (detected again: panel.json may have changed).
#[tauri::command]
pub async fn run_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    project: String,
    action: String,
) -> CmdResult<crate::projects::run::RunInfo> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let (runner, servers) = (Arc::clone(&runner), Arc::clone(&state.servers));
    tauri::async_runtime::spawn_blocking(move || {
        let detected = crate::detect::detect(&dir, lang);
        let Some(a) = detected.actions.iter().find(|a| a.id == action) else {
            return Err(crate::i18n::t(lang, "Azione non trovata: premi Aggiorna", &[]).into());
        };
        // A site assigned to a server profile: started in that shared pool
        // (GAP-RUN). A profile gone since: the pool `list` puts it in, or,
        // not in any, its own server as before.
        if let SiteMode::Pool(_) = site_mode(a) {
            if let Some((pool, _)) = servers.pool_of(&project, &a.id, lang) {
                let mut wanted = vec![pool.engine.as_str()];
                if pool.php_line.is_some() {
                    wanted.push("php");
                }
                let env = ensure(&app, &task, &kit, lang, &wanted)?;
                servers.start(&pool.id, &env, lang).or_fail_chain("run")?;
                let Some((pool, site)) = servers.pool_of(&project, &a.id, lang) else {
                    return Err(
                        crate::i18n::t(lang, "Azione non trovata: premi Aggiorna", &[]).into(),
                    );
                };
                return Ok(runner.attach_pool(&project, a, &pool.id, pool.port, &site.url, lang));
            }
        }
        let wanted: Vec<&str> = crate::setup::needs::of_action(a).into_iter().collect();
        let env =
            crate::versions::project_env(&kit, &project, ensure(&app, &task, &kit, lang, &wanted)?);
        runner
            .start(&project, &dir, a, &env, lang)
            .or_fail_chain("run")
    })
    .await
    .or_fail("run")?
}

#[tauri::command]
pub fn run_stop<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    runner.stop(&key);
    Ok(())
}

#[tauri::command]
pub fn run_log<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> CmdResult<Vec<String>> {
    ensure_main(&webview)?;
    Ok(runner.log(&key))
}

#[tauri::command]
pub fn run_clear<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    runner.clear(&key);
    Ok(())
}

/// The ports remembered for the web actions, and the locked ones.
#[tauri::command]
pub fn run_ports<R: Runtime>(
    webview: Webview<R>,
    runner: State<'_, Arc<Runner>>,
) -> CmdResult<crate::projects::run::Ports> {
    ensure_main(&webview)?;
    Ok(runner.ports())
}

/// The user's port for an action (`None`: automatic again).
#[tauri::command]
pub fn run_set_port<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    key: String,
    port: Option<u16>,
) -> CmdResult<crate::projects::run::Ports> {
    ensure_main(&webview)?;
    Ok(runner.set_port(&key, port, state.lang())?)
}
