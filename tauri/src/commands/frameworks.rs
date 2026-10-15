//! New project from frameworks (projects/frameworks.rs, DECISIONS.md D-83).

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
) -> Result<FrameworksView, String> {
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
) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::projects::frameworks::save(&state.kit, pieces).map_err(|p| problem_text(state.lang(), p))
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
) -> Result<crate::projects::frameworks::Plan, String> {
    ensure_main(&webview)?;
    pieces
        .iter_mut()
        .for_each(crate::projects::frameworks::clean);
    crate::projects::frameworks::plan(&pieces, &base, &addons)
        .map_err(|p| problem_text(state.lang(), p))
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

/// Creates `projects\<name>` from a base and its additions: installs the
/// kit's tools they need, runs the commands with the kit's PowerShell,
/// writes the chosen variables in its `.env` (`env`, as the wizard shows
/// them), saves their actions in the project. An existing project is used when it
/// is empty, or when it has files but no Git: then the framework is made in
/// `data	mprameworks` and its files are added, those already there stay.
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
) -> Result<String, String> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    crate::projects::frameworks::check_env(&env).map_err(|p| problem_text(lang, p))?;
    let wanted_env = env;
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
        std::fs::create_dir_all(kit.projects()).map_err(|e| fail("frameworks", e))?;
        // A project with files: the framework is made aside, then added to it.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let staging = kit.data().join(r"tmp\frameworks").join(stamp.to_string());
        let make_at = if existing == Existing::Files {
            std::fs::create_dir_all(&staging).map_err(|e| fail("frameworks", e))?;
            staging.clone()
        } else {
            kit.projects()
        };
        // An empty project: its folder is made again by the creation.
        if existing == Existing::Empty {
            std::fs::remove_dir(&dir).map_err(|e| fail("frameworks", e))?;
        }
        let no_folder = || {
            crate::i18n::t(
                lang,
                "Il comando di creazione non ha fatto la cartella {0}.",
                &[&name],
            )
        };
        let run = |step: usize, line: &str, at: &std::path::Path| -> Result<(), String> {
            if crate::projects::frameworks::cancelled() {
                return Err(crate::i18n::t(lang, "Creazione annullata.", &[]));
            }
            let command = crate::projects::frameworks::fill(line, &name);
            emit(step, Some(command.clone()), None);
            let out = |l: &str| emit(step, None, Some(l.to_owned()));
            crate::projects::frameworks::run_line(&kit, &env, at, &command, &out).map_err(|last| {
                if crate::projects::frameworks::cancelled() {
                    return crate::i18n::t(lang, "Creazione annullata.", &[]);
                }
                crate::log::warn(&format!("frameworks: {command}: {last}"));
                crate::i18n::t(lang, "Comando non riuscito: {0}\n{1}", &[&command, &last])
            })
        };
        let result = (|| -> Result<(), String> {
            for (i, line) in plan.create.iter().enumerate() {
                run(i + 1, line, &make_at)?;
            }
            if existing == Existing::Files {
                let made = staging.join(&name);
                if !made.is_dir() {
                    return Err(no_folder());
                }
                let mut kept = Vec::new();
                crate::projects::frameworks::merge_into(&made, &dir, &mut kept)
                    .map_err(|e| fail("frameworks", e))?;
                for k in kept {
                    let text = crate::i18n::t(lang, "Già nel progetto, lasciato com'è: {0}", &[&k]);
                    emit(plan.create.len(), None, Some(text));
                }
            }
            if !dir.is_dir() {
                return Err(no_folder());
            }
            if let Some(engine) = &plan.database {
                let text = crate::i18n::t(lang, "Creo il database del progetto in {0}…", &[engine]);
                emit(plan.create.len(), None, Some(text));
                let db = services
                    .start(engine, &env)
                    .and_then(|_port| services.project_db_create(&name, engine))
                    .map_err(|e| fail("database", format!("{e:#}")))?;
                crate::db_auth::env_apply(&dir, &db).map_err(|e| fail(".env", format!("{e:#}")))?;
            }
            let vars: Vec<(String, String)> = wanted_env
                .iter()
                .filter(|v| !v.key.is_empty())
                .map(|v| {
                    (
                        v.key.clone(),
                        crate::projects::frameworks::env_value(&v.value, &name),
                    )
                })
                .collect();
            if !vars.is_empty() {
                let keys: Vec<&str> = vars.iter().map(|(k, _)| k.as_str()).collect();
                let text = crate::i18n::t(lang, "Scrivo nel file .env: {0}", &[&keys.join(", ")]);
                emit(plan.create.len(), None, Some(text));
                crate::db_auth::env_set(&dir, &vars).map_err(|e| fail(".env", format!("{e:#}")))?;
            }
            for (i, line) in plan.steps.iter().enumerate() {
                run(plan.create.len() + i + 1, line, &dir)?;
            }
            Ok(())
        })();
        if existing == Existing::Files {
            let _ = std::fs::remove_dir_all(&staging);
        }
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
    .map_err(|e| fail("frameworks", e))?
}

/// The projects' folders and what is in them, for the wizard's name.
#[tauri::command]
pub fn frameworks_targets<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<Vec<crate::projects::frameworks::Target>, String> {
    ensure_main(&webview)?;
    Ok(crate::projects::frameworks::targets(&state.kit))
}

/// Stops the creation running now.
#[tauri::command]
pub fn frameworks_cancel<R: Runtime>(webview: Webview<R>) -> Result<(), String> {
    ensure_main(&webview)?;
    crate::projects::frameworks::cancel();
    Ok(())
}
