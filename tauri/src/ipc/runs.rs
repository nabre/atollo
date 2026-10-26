//! Projects' actions started by the kit (projects/run/): start, stop, log, ports.

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
    let (host, kit_env) = (state.host.clone(), state.env());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let detected = crate::detect::detect(&dir, lang);
        let Some(a) = detected.actions.iter().find(|a| a.id == action) else {
            return Err(crate::common::i18n::t(lang, "Azione non trovata: premi Aggiorna", &[]).into());
        };
        // Level 2 (D-179): `decide` places the action or the site.
        let req = action_request(&kit, &dir, a);
        let facts = super::levels::facts_now(&kit, &host, &kit_env, &req);
        let placement = super::levels::admitted(&req, &facts, lang)?;
        if let crate::levels::Placement::Vm { .. } = placement {
            // In the project's microVM (R16, R17): the action there, its
            // web server a container of the VM, never a pool of the PC; the
            // kit's tools and its databases on the PC (D-170, D-171). The
            // VM's path as before D-179 (step 4 of LEVELS.md §12.2 moves it
            // to `levels::prepare`).
            let env = crate::levels::env::pc_environment(
                &kit,
                kit_env.clone(),
                crate::levels::Level::Project,
                Some(&project),
                None,
            );
            let vm = crate::isolation::vm_for(&kit, &host, &env, &project, &dir)
                .ok_or_else(|| super::editing::no_sbx(lang))?;
            let vm = super::terminals::project_vm_ready(
                &app, &task, &services, &kit, &host, lang, &project, &dir, &kit_env, vm,
            )?;
            return runner
                .start_in(&project, &dir, a, &env, lang, Some(&vm))
                .or_fail_chain("run");
        }
        // On the PC without the VM (R5), sealed.
        // An Apache/Caddy site is started in a shared pool: the profile it
        // was assigned to (GAP-RUN), else the automatic pool of its lines
        // (D-101). A profile gone since: the pool `list` puts it in,
        // or, not in any, its own server as before.
        let mode = site_mode(a);
        let mut fallback = None;
        if mode != SiteMode::Dedicated {
            // A fresh scan: the cached one may still name the pool the site
            // was moved from.
            if let Some((pool, _)) = servers.pool_of(&project, &a.id, lang) {
                let mut wanted = vec![pool.engine.as_str()];
                if pool.php_line.is_some() {
                    wanted.push("php");
                }
                let base = ensure(&app, &task, &kit, lang, &wanted)?;
                // The pool itself is placed too (R6; R2 with the VM on, not
                // reached: the site went to the VM first).
                let env = super::servers::pool_env(&kit, &host, &base, &pool.id, lang)?;
                // The site joins the pool (started, or reloaded when it was
                // left out of it); the run follows the pool while it runs.
                let attached = servers.attach(&project, &a.id, &env, lang).and_then(
                    |(pool_id, port, server_name)| {
                        if servers.running_port(&pool_id) == Some(port) {
                            Ok((pool_id, port, server_name))
                        } else {
                            Err(anyhow::anyhow!(crate::common::i18n::t(
                                lang,
                                "Il server condiviso {0} non è partito: guarda la pagina Server Web.",
                                &[&pool_id],
                            )))
                        }
                    },
                );
                match attached {
                    Ok((pool_id, port, server_name)) => {
                        let url = format!("http://{server_name}:{port}");
                        return Ok(runner.attach_pool(&project, a, &pool_id, port, &url, lang));
                    }
                    // The pool the user chose: its error, never another server.
                    Err(e) if mode != SiteMode::Auto => return Err(e).or_fail_chain("run"),
                    // The automatic pool cannot start (D-101): the site's own
                    // server, saying why in its log.
                    Err(e) => {
                        crate::common::log::warn(&format!("run {project} {}: pool: {e:#}", a.id));
                        fallback = Some(crate::common::i18n::t(
                            lang,
                            "Il server condiviso {0} non è partito ({1}): il sito usa un server tutto suo.",
                            &[&pool.id, &format!("{e:#}")],
                        ));
                    }
                }
            }
        }
        let wanted: Vec<&str> = crate::setup::needs::of_action(a).into_iter().collect();
        let base = ensure(&app, &task, &kit, lang, &wanted)?;
        let env = super::levels::pc_env(&kit, &base, &placement, None, lang, &host)?;
        let info = runner
            .start(&project, &dir, a, &env, lang)
            .or_fail_chain("run")?;
        if let Some(text) = fallback {
            runner.note(&info.key, &text);
        }
        Ok(info)
    })
    .await
    .or_fail("run")?
}

/// The request of the action `a` of the project in `dir` (E4–E6): a
/// `Site` when a web server of the kit serves it (static, Apache, Caddy),
/// else an `Action`, `web` when it listens on a port.
pub(crate) fn action_request(
    kit: &Kit,
    dir: &std::path::Path,
    a: &crate::detect::Action,
) -> crate::levels::Request {
    let served = a.static_dir.is_some() || a.apache_dir.is_some() || a.caddy_dir.is_some();
    let origin = if served {
        crate::levels::Origin::Site {
            action: a.id.clone(),
        }
    } else {
        crate::levels::Origin::Action {
            action: a.id.clone(),
            web: a.kind == crate::detect::Kind::Web,
        }
    };
    crate::levels::Request {
        origin,
        place: crate::levels::place::place_of(kit, dir),
    }
}

/// Stops an action. A site served by a pool leaves it (a reload, or the
/// pool stops when no project is left in it: D-101), off the IPC
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
) -> CmdResult<crate::projects::run::RunLog> {
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
