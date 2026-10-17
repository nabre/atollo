//! New project from frameworks (projects/frameworks/, DECISIONS.md D-83).

use super::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworksView {
    /// In use: the official ones with the changes, then the user's own.
    pub pieces: Vec<crate::projects::frameworks::Piece>,
    /// The official instructions (to restore a piece).
    pub official: &'static [crate::projects::frameworks::Piece],
    /// Descriptions and notes of the official pieces in the app's language,
    /// by Italian source.
    pub texts: std::collections::BTreeMap<String, String>,
    pub problem: Option<String>,
}

pub(super) fn problem_text(
    lang: &str,
    (source, args): crate::projects::frameworks::Problem,
) -> String {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    crate::i18n::t(lang, source, &args)
}

#[tauri::command]
pub fn frameworks_get<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<FrameworksView> {
    ensure_main(&webview)?;
    let lang = state.lang();
    let loaded = crate::projects::frameworks::load(&state.kit);
    let official = crate::projects::frameworks::official();
    let mut texts = std::collections::BTreeMap::new();
    for p in official {
        let notes = p.on.values().filter_map(|v| v.note.as_ref());
        for s in std::iter::once(&p.desc).chain(&p.note).chain(notes) {
            texts.insert(s.clone(), crate::i18n::t(lang, s, &[]));
        }
    }
    Ok(FrameworksView {
        pieces: loaded.pieces,
        official,
        texts,
        problem: loaded.problem,
    })
}

#[tauri::command]
pub fn frameworks_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    pieces: Vec<crate::projects::frameworks::Piece>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    Ok(crate::projects::frameworks::save(&state.kit, pieces)
        .map_err(|p| problem_text(state.lang(), p))?)
}

/// What a base with its additions would run (`pieces`: as edited, even if
/// not saved yet).
#[tauri::command]
pub fn frameworks_plan<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    mut pieces: Vec<crate::projects::frameworks::Piece>,
    base: String,
    addons: Vec<String>,
) -> CmdResult<crate::projects::frameworks::Plan> {
    ensure_main(&webview)?;
    pieces
        .iter_mut()
        .for_each(crate::projects::frameworks::clean);
    Ok(crate::projects::frameworks::plan(&pieces, &base, &addons)
        .map_err(|p| problem_text(state.lang(), p))?)
}

/// Progress of a creation (`framework` event): a command starting, or a
/// line of its output.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameworkEvent {
    /// 1-based number of the command; 0 = installing the kit's tools.
    pub step: usize,
    pub total: usize,
    pub command: Option<String>,
    pub line: Option<String>,
}

/// The action id of a framework action: `fw-` and its name in lower case.
pub(super) fn framework_action_id(name: &str, i: usize) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        format!("fw-{i}")
    } else {
        format!("fw-{}", slug.chars().take(30).collect::<String>())
    }
}

/// The connection to the project's database for the commands of a
/// creation, by name, whatever the framework calls it (TYPO3's and
/// Drupal's installers take it as options).
pub(super) fn db_vars(db: &crate::db::auth::ProjectDb) -> Vec<(String, String)> {
    [
        ("ATOLLO_DB_HOST", db.host.clone()),
        ("ATOLLO_DB_PORT", db.port.to_string()),
        ("ATOLLO_DB_NAME", db.database.clone()),
        ("ATOLLO_DB_USER", db.user.clone()),
        ("ATOLLO_DB_PASSWORD", db.password.clone()),
        ("ATOLLO_DB_URL", db.url.clone()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect()
}

/// Adds `/.env` to the project's `.gitignore` when the project has a
/// `.env` that Git would take: the kit writes passwords there (the
/// database's, a CMS administrator's). TYPO3 and Drupal do not list it.
pub(super) fn ignore_env(dir: &std::path::Path) -> anyhow::Result<()> {
    let path = dir.join(".gitignore");
    if !dir.join(".env").is_file()
        || crate::db::auth::env_ignored(dir)
        || crate::os::safe_path::is_link(&path)
    {
        return Ok(());
    }
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let sep = if text.is_empty() || text.ends_with('\n') {
        ""
    } else {
        newline
    };
    crate::fsx::write_atomic(&path, format!("{text}{sep}/.env{newline}").as_bytes())?;
    Ok(())
}

/// `merge_into` for a framework made on another disk, where a rename fails:
/// what `to` already has stays (in `kept`), the rest is copied. Links are
/// skipped at the levels walked here; inside a new subfolder
/// `archive::copy_tree` makes them again on Unix when they stay inside (a
/// copy where the file system has none).
fn merge_copy(
    from: &std::path::Path,
    to: &std::path::Path,
    rel: &std::path::Path,
    kept: &mut Vec<String>,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        let (src, dst) = (entry.path(), to.join(entry.file_name()));
        let rel = rel.join(entry.file_name());
        match std::fs::symlink_metadata(&dst) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                if kind.is_dir() {
                    crate::net::archive::copy_tree(&src, &dst)?;
                } else {
                    std::fs::copy(&src, &dst)?;
                }
            }
            Err(e) => return Err(e),
            Ok(m) if m.is_dir() && !crate::os::safe_path::is_reparse(&m) && kind.is_dir() => {
                merge_copy(&src, &dst, &rel, kept)?;
            }
            Ok(_) => kept.push(rel.to_string_lossy().into_owned()),
        }
    }
    Ok(())
}

/// Creates `projects\<name>` from a base and its additions: installs the
/// kit's tools they need, runs the commands with the kit's PowerShell,
/// writes the chosen variables in its `.env` (`env`, as the wizard shows
/// them), saves their actions in the project. An existing project is used when it
/// is empty, or when it has files but no Git: then the framework is made in
/// the PC's temporary folder (`frameworks\<stamp>`, D-90) and its files are
/// added, those already there stay.
/// A failed command stops it and leaves the folder as it is, to look at.
#[tauri::command]
pub async fn frameworks_create<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    base: String,
    addons: Vec<String>,
    name: String,
    env: Vec<crate::projects::frameworks::EnvVar>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    crate::projects::frameworks::check_env(&env).map_err(|p| problem_text(lang, p))?;
    let wanted_env = env;
    // `{password}` of `.env`: one new password for the creation.
    let password = crate::projects::frameworks::password_of(
        &crate::editor::random_token().or_fail("frameworks")?,
    );
    let services = Arc::clone(&state.services);
    let busy = crate::projects::frameworks::begin()
        .ok_or_else(|| tr(&state, "È già in corso la creazione di un progetto.", &[]))?;
    let name = name.trim().to_owned();
    let existing =
        crate::projects::frameworks::existing(&kit, &name).map_err(|p| problem_text(lang, p))?;
    let dir = kit.projects().join(&name);
    let pieces = crate::projects::frameworks::load(&kit).pieces;
    let plan = crate::projects::frameworks::plan(&pieces, &base, &addons)
        .map_err(|p| problem_text(lang, p))?;
    tauri::async_runtime::spawn_blocking(move || {
        use crate::projects::frameworks::Existing;
        let _busy = busy;
        let total = plan.create.len() + plan.steps.len();
        let emit = |step: usize, command: Option<String>, line: Option<String>| {
            let event = FrameworkEvent {
                step,
                total,
                command,
                line,
            };
            let _ = app.emit_to(MAIN, "framework", event);
        };
        emit(0, None, None);
        let needs: Vec<&str> = plan.needs.iter().map(String::as_str).collect();
        let env = ensure(&app, &task, &kit, lang, &needs)?;
        std::fs::create_dir_all(kit.projects()).or_fail("frameworks")?;
        // A project with files: the framework is made aside, in the PC's
        // temporary folder (D-90; removed when `staging` is dropped), then
        // added to it.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let staging = if existing == Existing::Files {
            let rel = format!("frameworks/{stamp}");
            Some(crate::os::temp::TempWork::new(&kit, &rel).or_fail("frameworks")?)
        } else {
            None
        };
        let make_at = staging
            .as_ref()
            .map_or_else(|| kit.projects(), |s| s.path().to_path_buf());
        // An empty project: its folder is made again by the creation.
        if existing == Existing::Empty {
            std::fs::remove_dir(&dir).or_fail("frameworks")?;
        }
        let no_folder = || {
            crate::i18n::t(
                lang,
                "Il comando di creazione non ha fatto la cartella {0}.",
                &[&name],
            )
        };
        let run = |step: usize,
                   line: &str,
                   at: &std::path::Path,
                   vars: &[(String, String)]|
         -> CmdResult<()> {
            if crate::projects::frameworks::cancelled() {
                return Err(crate::i18n::t(lang, "Creazione annullata.", &[]).into());
            }
            let command = crate::projects::frameworks::fill(line, &name);
            emit(step, Some(command.clone()), None);
            let out = |l: &str| emit(step, None, Some(l.to_owned()));
            crate::projects::frameworks::run_line(&kit, &env, vars, at, &command, &out).map_err(
                |last| {
                    if crate::projects::frameworks::cancelled() {
                        return crate::i18n::t(lang, "Creazione annullata.", &[]).into();
                    }
                    crate::log::warn(&format!("frameworks: {command}: {last}"));
                    crate::i18n::t(lang, "Comando non riuscito: {0}\n{1}", &[&command, &last])
                        .into()
                },
            )
        };
        let result = (|| -> CmdResult<()> {
            for (i, line) in plan.create.iter().enumerate() {
                run(i + 1, line, &make_at, &[])?;
            }
            if existing == Existing::Files {
                let made = make_at.join(&name);
                if !made.is_dir() {
                    return Err(no_folder().into());
                }
                let mut kept = Vec::new();
                // Moved on the same disk; copied from the PC's disk.
                if crate::os::volume::same_volume(&made, &dir) {
                    crate::projects::frameworks::merge_into(&made, &dir, &mut kept)
                } else {
                    merge_copy(&made, &dir, std::path::Path::new(""), &mut kept)
                }
                .or_fail("frameworks")?;
                for k in kept {
                    let text = crate::i18n::t(lang, "Già nel progetto, lasciato com'è: {0}", &[&k]);
                    emit(plan.create.len(), None, Some(text));
                }
            }
            if !dir.is_dir() {
                return Err(no_folder().into());
            }
            // What the steps get in their environment: the database's
            // connection (`ATOLLO_DB_*`) and the variables of `.env`.
            let mut step_vars: Vec<(String, String)> = Vec::new();
            if let Some(engine) = &plan.database {
                let text = crate::i18n::t(lang, "Creo il database del progetto in {0}…", &[engine]);
                emit(plan.create.len(), None, Some(text));
                let db = services
                    .start_for(&name, engine, &env)
                    .and_then(|_port| services.project_db_create(&name, engine))
                    .or_fail_chain("database")?;
                crate::db::auth::env_apply(&dir, &db).or_fail_chain(".env")?;
                step_vars.extend(db_vars(&db));
            }
            let vars: Vec<(String, String)> = wanted_env
                .iter()
                .filter(|v| !v.key.is_empty())
                .map(|v| {
                    (
                        v.key.clone(),
                        crate::projects::frameworks::env_value(&v.value, &name, &password),
                    )
                })
                .collect();
            step_vars.extend(wanted_env.iter().filter(|v| !v.key.is_empty()).map(|v| {
                (
                    v.key.clone(),
                    crate::projects::frameworks::env_raw(&v.value, &name, &password),
                )
            }));
            if !vars.is_empty() {
                let keys: Vec<&str> = vars.iter().map(|(k, _)| k.as_str()).collect();
                let text = crate::i18n::t(lang, "Scrivo nel file .env: {0}", &[&keys.join(", ")]);
                emit(plan.create.len(), None, Some(text));
                crate::db::auth::env_set(&dir, &vars).or_fail_chain(".env")?;
            }
            for (i, line) in plan.steps.iter().enumerate() {
                run(plan.create.len() + i + 1, line, &dir, &step_vars)?;
            }
            if let Err(e) = ignore_env(&dir) {
                crate::log::warn(&format!("{name}: .gitignore: {e:#}"));
            }
            Ok(())
        })();
        drop(staging);
        if existing == Existing::Empty && !dir.exists() {
            let _ = std::fs::create_dir(&dir);
        }
        result?;
        for (i, a) in plan.actions.iter().enumerate() {
            let edit = crate::detect::ActionEdit {
                id: framework_action_id(&a.name, i),
                label: a.name.clone(),
                command: a.command.clone(),
                kind: "command".into(),
                folder: None,
                url: None,
                env: std::collections::BTreeMap::new(),
            };
            if let Err(e) = crate::detect::save_action(&dir, &edit, lang) {
                crate::log::warn(&format!("{name}: action {}: {e:#}", a.name));
            }
        }
        crate::versions::pin_newest(&kit, &name);
        settle_project(&app, &task, &kit, lang, &name, &dir);
        Ok(name)
    })
    .await
    .or_fail("frameworks")?
}

/// The projects' folders and what is in them, for the wizard's name.
#[tauri::command]
pub fn frameworks_targets<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Vec<crate::projects::frameworks::Target>> {
    ensure_main(&webview)?;
    Ok(crate::projects::frameworks::targets(&state.kit))
}

/// Stops the creation running now.
#[tauri::command]
pub fn frameworks_cancel<R: Runtime>(webview: Webview<R>) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::projects::frameworks::cancel();
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/commands/frameworks.rs"]
mod tests;
