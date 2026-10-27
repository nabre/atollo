//! Databases of a project (D-74), backups, `.env`, phpMyAdmin (D-107) and
//! mongo-express (D-147).

use super::*;
use crate::db::DbEngine;

/// The engine the interface names (`mysql`, `mongodb`), or the error of an
/// unknown one, logged under `context`.
fn engine_of(engine: &str, context: &str) -> CmdResult<DbEngine> {
    DbEngine::from_id(engine).ok_or_else(|| fail(context, format!("unknown server {engine}")))
}

/// The servers `detect::requires` found (`mysql`, `mongodb`, `sqlite`),
/// each once.
fn detected_as(found: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for id in found {
        if DbEngine::from_id(&id).is_none() && id != "sqlite" {
            continue;
        }
        if !out.contains(&id) {
            out.push(id);
        }
    }
    out
}

/// The databases of a project (D-74): those created, those it seems to use,
/// and whether its `.env` is ignored by Git.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDatabases {
    pub dbs: Vec<crate::db::auth::ProjectDb>,
    /// `mysql`, `mongodb`, `sqlite` found by `detect::requires`.
    pub detected: Vec<String>,
    pub env_exists: bool,
    pub env_ignored: bool,
    /// By engine (`mysql`, `mongodb`): the version its database is on and
    /// those installed, of the server it reaches.
    pub versions: std::collections::BTreeMap<String, DbVersions>,
}

/// The versions of a server for a project (D-102).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DbVersions {
    /// Chosen for the project; None: the one the server runs (`server`).
    pub chosen: Option<String>,
    pub server: Option<String>,
    /// Installed, newest first.
    pub lines: Vec<String>,
}

fn db_versions(kit: &Kit, project: &str) -> std::collections::BTreeMap<String, DbVersions> {
    DbEngine::ALL
        .into_iter()
        .map(|engine| {
            let id = engine.id();
            let chosen = crate::db::auth::project_line(kit, project, id)
                .ok()
                .flatten();
            let versions = DbVersions {
                chosen,
                server: crate::db::services::server_line(kit, id),
                lines: crate::db::services::server_lines(kit, id),
            };
            (id.to_owned(), versions)
        })
        .collect()
}

/// The project's databases. Off the window's thread: what its code needs
/// is a walk of its files (`detect::requires`).
#[tauri::command]
pub async fn project_databases<R: Runtime>(
    webview: Webview<R>,
    project: String,
) -> CmdResult<ProjectDatabases> {
    ensure_main(&webview)?;
    off_main(&webview, "databases", move |_, state| {
        let dir = project_dir(&state.kit, Some(&project), state.lang())?;
        Ok(ProjectDatabases {
            dbs: state.services.project_dbs(&project),
            detected: detected_as(crate::detect::requires(&dir)),
            env_exists: dir.join(".env").is_file(),
            env_ignored: crate::db::auth::env_ignored(&dir),
            versions: db_versions(&state.kit, &project),
        })
    })
    .await
}

/// The version of a server for the project's database (null: the one the
/// server runs), each on its own port (D-102). A database already
/// created is created on that version too: the server is installed and
/// started for it.
#[tauri::command]
pub async fn project_db_line<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
    line: Option<String>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let kind = engine_of(&engine, "database")?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[kind.component()])?;
        services
            .project_db_line(&project, kind.id(), line.as_deref(), &env)
            .map_err(|e| super::services::start_error(lang, &e))?;
        Ok(())
    })
    .await
    .or_fail("database")?
}

/// Creates the project's database and user: installs and starts the server
/// first if needed.
#[tauri::command]
pub async fn project_db_create<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> CmdResult<crate::db::auth::ProjectDb> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let kind = engine_of(&engine, "database")?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[kind.component()])?;
        services
            .start_for(&project, kind.id(), &env)
            .map_err(|e| start_error(lang, &e))?;
        services
            .project_db_create(&project, kind.id())
            .or_fail_chain("database")
    })
    .await
    .or_fail("database")?
}

pub(super) fn project_db_of(
    state: &AppState,
    project: &str,
    engine: &str,
) -> CmdResult<crate::db::auth::ProjectDb> {
    state
        .services
        .project_dbs(project)
        .into_iter()
        .find(|d| d.engine == engine)
        .ok_or_else(|| tr(state, "Crea prima il database del progetto.", &[]).into())
}

/// The lines of the project's `.env` that writing the connection changes.
#[tauri::command]
pub fn project_env_preview<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> CmdResult<Vec<crate::db::auth::EnvChange>> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    Ok(crate::db::auth::env_changes(
        &crate::db::auth::env_source(&dir),
        &db,
    ))
}

/// The connection as `.env` lines, for «Copia blocco .env»: Laravel's
/// `DB_*` when the project's `.env` uses them.
#[tauri::command]
pub fn project_env_block<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    Ok(crate::db::auth::env_block(
        &crate::db::auth::env_source(&dir),
        &db,
    ))
}

/// Writes the connection into the project's `.env` (from `.env.example`
/// when it has none); only the lines shown by the preview change.
#[tauri::command]
pub fn project_env_apply<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    crate::db::auth::env_apply(&dir, &db).or_fail_chain(".env")
}

/// The copies of the project's databases (D-78), newest first.
#[tauri::command]
pub fn project_backups<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> CmdResult<Vec<crate::db::services::Backup>> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(state.services.backups(&project))
}

/// A copy of the project's database; starts the server if needed.
#[tauri::command]
pub async fn project_db_export<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let kind = engine_of(&engine, "database")?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[kind.component()])?;
        services
            .start_for(&project, kind.id(), &env)
            .map_err(|e| start_error(lang, &e))?;
        let file = services
            .export(&project, kind.id())
            .or_fail_chain("export")?;
        Ok(file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default())
    })
    .await
    .or_fail("export")?
}

/// Puts a copy back into the project's database; starts the server if needed.
#[tauri::command]
pub async fn project_db_restore<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    file: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    // A name that is no copy goes to MySQL, and its restore refuses it.
    let engine = DbEngine::of_backup(&file).unwrap_or(DbEngine::MySql);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.component()])?;
        services
            .start_for(&project, engine.id(), &env)
            .map_err(|e| start_error(lang, &e))?;
        services.restore(&project, &file).or_fail_chain("restore")
    })
    .await
    .or_fail("restore")?
}

/// The web admin of `engine` (D-107, D-147): phpMyAdmin for MySQL,
/// mongo-express for MongoDB.
pub fn admin_component(engine: DbEngine) -> &'static str {
    match engine {
        DbEngine::MySql => "phpmyadmin",
        DbEngine::MongoDb => "mongo-express",
    }
}

/// The kit's database in the browser: phpMyAdmin on MySQL (D-107, D-110),
/// mongo-express on MongoDB (D-147). The tool (with what it needs: PHP or
/// Node.js, and the server) installed the first time with the progress of
/// the preparation task, the server started when it is off. `engine`:
/// `mysql` (null too) or `mongodb`. `line`: another installed version, on
/// its own port (D-102); null: the version the server runs.
#[tauri::command]
pub async fn db_admin_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    line: Option<String>,
    engine: Option<String>,
) -> CmdResult<Vec<crate::db::services::ServiceInfo>> {
    ensure_main(&webview)?;
    let engine = match engine.as_deref() {
        Some(id) => engine_of(id, "db-admin")?,
        None => DbEngine::MySql,
    };
    let tool = admin_component(engine);
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.component(), tool])?;
        services
            .start_engine(engine, line.as_deref(), &env)
            .map_err(|e| start_error(lang, &e))?;
        let url = match engine {
            DbEngine::MySql => services.phpmyadmin_url(engine, line.as_deref(), lang),
            DbEngine::MongoDb => services.mongo_express_url(line.as_deref()),
        }
        .or_fail_chain(tool)?;
        open_in_browser(&url)?;
        Ok(services.list())
    })
    .await
    .or_fail(tool)?
}

#[cfg(test)]
#[path = "../../../test/rust/ipc/databases.rs"]
mod tests;
