//! Projects' actions started by the kit (projects/run.rs): start, stop, log, ports.

use super::*;
use crate::projects::run::{site_mode, SiteMode};

#[tauri::command]
pub fn run_list<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
) -> CmdResult<Vec<crate::projects::run::RunInfo>> {
    ensure_main(&webview)?;
    // A site served by a pool stopped (or dead) since: stopped too.
    runner.reconcile_pools(|id| state.servers.is_running(id));
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
            // A fresh scan: the cached one may still name the pool the site
            // was moved from.
            if let Some((pool, _)) = servers.pool_of(&project, &a.id, lang) {
                let mut wanted = vec![pool.engine.as_str()];
                if pool.php_line.is_some() {
                    wanted.push("php");
                }
                let env = ensure(&app, &task, &kit, lang, &wanted)?;
                // The site joins the pool (started, or reloaded when it was
                // left out of it); the run follows the pool while it runs.
                let (pool_id, port, server_name) = servers
                    .attach(&project, &a.id, &env, lang)
                    .or_fail_chain("run")?;
                if servers.running_port(&pool_id) != Some(port) {
                    return Err(crate::i18n::t(
                        lang,
                        "Il server condiviso {0} non è partito: guarda la pagina Server Web.",
                        &[&pool_id],
                    )
                    .into());
                }
                let url = format!("http://{server_name}:{port}");
                return Ok(runner.attach_pool(&project, a, &pool_id, port, &url, lang));
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

/// Stops an action. A site served by a pool leaves it (a reload, or the
/// pool stops when no project is left in it: PLAN.md §13.1), off the IPC
/// thread.
#[tauri::command]
pub async fn run_stop<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    runner: State<'_, Arc<Runner>>,
    key: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let pooled = runner.pooled(&key);
    runner.stop(&key);
    let Some((pool, project, action)) = pooled else {
        return Ok(());
    };
    let (servers, env, lang) = (Arc::clone(&state.servers), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        servers
            .detach(&pool, &project, &action, &env, lang)
            .or_fail_chain("run")
    })
    .await
    .or_fail("run")?
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
