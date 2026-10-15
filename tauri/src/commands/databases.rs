//! Databases of a project (D-74), backups, `.env`, content (D-79) and
//! Adminer (D-85).

use super::*;

/// The databases of a project (D-74): those created, those it seems to use,
/// and whether its `.env` is ignored by Git.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDatabases {
    pub dbs: Vec<crate::db::auth::ProjectDb>,
    /// `mariadb`, `mongodb`, `sqlite` found by `detect::requires`.
    pub detected: Vec<String>,
    pub env_exists: bool,
    pub env_ignored: bool,
}

#[tauri::command]
pub fn project_databases<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<ProjectDatabases, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    Ok(ProjectDatabases {
        dbs: state.services.project_dbs(&project),
        detected: crate::detect::requires(&dir)
            .into_iter()
            .filter(|id| matches!(id.as_str(), "mariadb" | "mongodb" | "sqlite"))
            .collect(),
        env_exists: dir.join(".env").is_file(),
        env_ignored: crate::db::auth::env_ignored(&dir),
    })
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
) -> Result<crate::db::auth::ProjectDb, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    if !crate::db::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("database", format!("unknown server {engine}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.as_str()])?;
        services
            .start(&engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        services
            .project_db_create(&project, &engine)
            .map_err(|e| fail("database", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("database", e))?
}

pub(super) fn project_db_of(
    state: &AppState,
    project: &str,
    engine: &str,
) -> Result<crate::db::auth::ProjectDb, String> {
    state
        .services
        .project_dbs(project)
        .into_iter()
        .find(|d| d.engine == engine)
        .ok_or_else(|| tr(state, "Crea prima il database del progetto.", &[]))
}

/// The lines of the project's `.env` that writing the connection changes.
#[tauri::command]
pub fn project_env_preview<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    engine: String,
) -> Result<Vec<crate::db::auth::EnvChange>, String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    Ok(crate::db::auth::env_changes(
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    let dir = project_dir(&state.kit, Some(&project), state.lang())?;
    let db = project_db_of(&state, &project, &engine)?;
    crate::db::auth::env_apply(&dir, &db).map_err(|e| fail(".env", format!("{e:#}")))
}

/// The copies of the project's databases (D-78), newest first.
#[tauri::command]
pub fn project_backups<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
) -> Result<Vec<crate::db::services::Backup>, String> {
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
) -> Result<String, String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    if !crate::db::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("database", format!("unknown server {engine}")));
    }
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine.as_str()])?;
        services
            .start(&engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        let file = services
            .export(&project, &engine)
            .map_err(|e| fail("export", format!("{e:#}")))?;
        Ok(file
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default())
    })
    .await
    .map_err(|e| fail("export", e))?
}

/// Puts a copy back into the project's database; starts the server if needed.
#[tauri::command]
pub async fn project_db_restore<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    project: String,
    file: String,
) -> Result<(), String> {
    ensure_main(&webview)?;
    project_dir(&state.kit, Some(&project), state.lang())?;
    let engine = if file.ends_with(".archive.gz") {
        "mongodb"
    } else {
        "mariadb"
    };
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &[engine])?;
        services
            .start(engine, &env)
            .map_err(|e| fail("database", format!("{e:#}")))?;
        services
            .restore(&project, &file)
            .map_err(|e| fail("restore", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("restore", e))?
}

/// The databases of a running server with their tables or collections
/// (D-79), read only.
#[tauri::command]
pub async fn db_browse_tree<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    engine: String,
) -> Result<Vec<crate::db::browse::Database>, String> {
    ensure_main(&webview)?;
    if !crate::db::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("browse", format!("unknown server {engine}")));
    }
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        crate::db::browse::tree(&services, &engine).map_err(|e| fail("browse", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("browse", e))?
}

/// One page of rows of a table or collection, from `offset` (D-79).
#[tauri::command]
pub async fn db_browse_rows<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    engine: String,
    database: String,
    table: String,
    offset: u64,
) -> Result<crate::db::browse::Rows, String> {
    ensure_main(&webview)?;
    if !crate::db::services::SERVERS.contains(&engine.as_str()) {
        return Err(fail("browse", format!("unknown server {engine}")));
    }
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        crate::db::browse::rows(&services, &engine, &database, &table, offset)
            .map_err(|e| fail("browse", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("browse", e))?
}

/// Adminer on the kit's MariaDB in the browser (D-85): Adminer (with PHP
/// and MariaDB) installed the first time, MariaDB started when it is off.
#[tauri::command]
pub async fn db_admin_open<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::db::services::ServiceInfo>, String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let services = Arc::clone(&state.services);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["mariadb", "adminer"])?;
        services
            .start("mariadb", &env)
            .map_err(|e| start_error(lang, &e))?;
        let url = services
            .adminer_url()
            .map_err(|e| fail("adminer", format!("{e:#}")))?;
        open_in_browser(&url)?;
        Ok(services.list())
    })
    .await
    .map_err(|e| fail("adminer", e))?
}
