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
    crate::common::i18n::t(lang, source, &args)
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
            texts.insert(s.clone(), crate::common::i18n::t(lang, s, &[]));
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

/// What a creation installs first: the plan's needs, each once. The
/// database itself goes to the project's server (`Services::start_for`).
pub(super) fn plan_needs(plan: &crate::projects::frameworks::Plan) -> Vec<&str> {
    let mut out: Vec<&str> = Vec::new();
    for n in plan.needs.iter().map(String::as_str) {
        if !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

/// Writes the wizard's variables in the project's `.env`, then the
/// connection to its new database: the generated credentials win over a
/// `DB_*` of the base, even an empty one (BUG-V4-10). Returns the keys the
/// database set.
pub(super) fn write_env(
    dir: &std::path::Path,
    vars: &[(String, String)],
    db: Option<&crate::db::auth::ProjectDb>,
) -> anyhow::Result<Vec<String>> {
    if !vars.is_empty() {
        crate::db::auth::env_set(dir, vars)?;
    }
    let Some(db) = db else {
        return Ok(Vec::new());
    };
    let keys = crate::db::auth::env_wanted(&crate::db::auth::env_source(dir), db)
        .into_iter()
        .map(|(k, _)| k)
        .collect();
    crate::db::auth::env_apply(dir, db)?;
    Ok(keys)
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
        || crate::platform::safe_path::is_link(&path)
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
    crate::common::fsx::write_atomic(&path, format!("{text}{sep}/.env{newline}").as_bytes())?;
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
            Ok(m) if m.is_dir() && !crate::platform::safe_path::is_reparse(&m) && kind.is_dir() => {
                merge_copy(&src, &dst, &rel, kept)?;
            }
            Ok(_) => kept.push(rel.to_string_lossy().into_owned()),
        }
    }
    Ok(())
}

/// The folder of the new project where a creation in its VM makes the
/// framework (the `create` lines make `{name}` in it), then moved into the
/// project: the VM reaches only the project's folder, never `projects/`.
pub(crate) const VM_STAGING: &str = ".atollo-new";

/// The request of the creation of a project from the base `base` in the
/// new project's folder `dir` (E9, `Origin::Scaffold`): `dir` exists (made
/// empty by the caller), so the place is the new project.
pub(crate) fn scaffold_request(
    kit: &Kit,
    base: &str,
    dir: &std::path::Path,
) -> crate::levels::Request {
    crate::levels::Request {
        origin: crate::levels::Origin::Scaffold {
            base: base.to_owned(),
        },
        place: crate::levels::place::place_of(kit, dir),
    }
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
    let host = state.host.clone();
    crate::projects::frameworks::check_env(&env).map_err(|p| problem_text(lang, p))?;
    let wanted_env = env;
    // `{password}` of `.env`: one new password for the creation.
    let password = crate::projects::frameworks::password_of(
        &crate::editor::random_token().or_fail("frameworks")?,
    );
    let (services, servers) = (Arc::clone(&state.services), Arc::clone(&state.servers));
    let fx = super::levels::AppEffects::of(&app, &state, state.env());
    let busy = crate::projects::frameworks::begin()
        .ok_or_else(|| tr(&state, "È già in corso la creazione di un progetto.", &[]))?;
    let name = name.trim().to_owned();
    let existing =
        crate::projects::frameworks::existing(&kit, &name).map_err(|p| problem_text(lang, p))?;
    let dir = kit.projects().join(&name);
    let pieces = crate::projects::frameworks::load(&kit).pieces;
    let plan = crate::projects::frameworks::plan(&pieces, &base, &addons)
        .map_err(|p| problem_text(lang, p))?;
    // A base of «Le mie basi» (N-10): its folder, copied.
    let template = pieces
        .iter()
        .find(|p| p.id == base)
        .and_then(|p| p.template.as_deref())
        .map(|t| {
            crate::projects::frameworks::mine::template_dir(&kit, t)
                .filter(|d| d.is_dir())
                .ok_or_else(|| {
                    tr(
                        &state,
                        "La cartella della base non c'è più: salvala di nuovo dal progetto.",
                        &[],
                    )
                })
        })
        .transpose()?;
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
            let _ = app.emit_to(MAIN, super::events::FRAMEWORK, event);
        };
        emit(0, None, None);
        // The creation runs the packages' own scripts (`npm create`,
        // `composer create-project`): project code (level 2, D-179),
        // placed by `decide` in the new project's folder, made empty first
        // so that it is a project's (LEVELS.md §3.3).
        let base_env = ensure(&app, &task, &kit, lang, &[])?;
        std::fs::create_dir_all(kit.projects()).or_fail("frameworks")?;
        let made_now = existing == Existing::New;
        if made_now {
            std::fs::create_dir(&dir).or_fail("frameworks")?;
        }
        let undo = || {
            if made_now {
                let _ = std::fs::remove_dir(&dir);
            }
        };
        let req = scaffold_request(&kit, &base, &dir);
        let facts = super::levels::facts_now(&kit, &host, &base_env, &req);
        let placement = super::levels::admitted(&req, &facts, lang).inspect_err(|_| undo())?;
        let needs: Vec<String> = plan_needs(&plan).into_iter().map(str::to_owned).collect();
        let in_vm = matches!(placement, crate::levels::Placement::Vm { .. });
        let (mut ready, db_env) = if in_vm {
            // With the VM (R15): in the new project's sandbox, made ready by
            // `levels::prepare` with the plan's tools for the VM's Linux
            // and the kit's PowerShell, whose lines these are. The
            // project's folder, its workspace, stays.
            let ready =
                super::levels::ready(&fx, &kit, &base_env, &req, &facts, &needs, lang, &host)
                    .inspect_err(|_| undo())?;
            let db_env = crate::levels::env::pc_environment(
                &kit,
                base_env.clone(),
                crate::levels::Level::Project,
                None,
                None,
            );
            (ready, db_env)
        } else {
            // Without the VM (R5): on the PC, sealed like a project's
            // programs (D-167), never with the kit's MySQL root, the user's
            // SSH keys or an agent's command; the plan's tools installed.
            let base_env =
                ensure(&app, &task, &kit, lang, &plan_needs(&plan)).inspect_err(|_| undo())?;
            let env = super::levels::pc_env(&kit, &base_env, &placement, None, lang, &host)
                .inspect_err(|_| undo())?;
            (
                crate::levels::prepare::Ready::on_the_pc(placement, env.clone()),
                env,
            )
        };
        // In the VM the framework is made in a folder of the project (its
        // workspace: the VM reaches nothing else of the PC), then added to
        // it. On the PC a project with files: aside, in the PC's temporary
        // folder (D-90; removed when `staging` is dropped), then added.
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos());
        let staging = if existing == Existing::Files && !in_vm {
            let rel = format!("frameworks/{stamp}");
            Some(crate::platform::temp::TempWork::new(&kit, &rel).or_fail("frameworks")?)
        } else {
            None
        };
        let make_at = if in_vm {
            let at = dir.join(VM_STAGING);
            std::fs::create_dir_all(&at).or_fail("frameworks")?;
            at
        } else {
            staging
                .as_ref()
                .map_or_else(|| kit.projects(), |s| s.path().to_path_buf())
        };
        let added = in_vm || existing == Existing::Files;
        // On the PC an empty project, or the folder made above for the
        // decision: made again by the creation.
        if !in_vm && (existing == Existing::Empty || made_now) {
            std::fs::remove_dir(&dir).or_fail("frameworks")?;
        }
        let no_folder = || {
            crate::common::i18n::t(
                lang,
                "Il comando di creazione non ha fatto la cartella {0}.",
                &[&name],
            )
        };
        let run = |ready: &crate::levels::prepare::Ready,
                   step: usize,
                   line: &str,
                   at: &std::path::Path,
                   vars: &[(String, String)]|
         -> CmdResult<()> {
            if crate::projects::frameworks::cancelled() {
                return Err(crate::common::i18n::t(lang, "Creazione annullata.", &[]).into());
            }
            let command = crate::projects::frameworks::fill(line, &name);
            emit(step, Some(command.clone()), None);
            let out = |l: &str| emit(step, None, Some(l.to_owned()));
            let cmd = crate::levels::start::scaffold_command(&kit, ready, &command, at, vars)
                .or_fail_chain("frameworks")?;
            crate::projects::frameworks::run_command(cmd, &out).map_err(|last| {
                if crate::projects::frameworks::cancelled() {
                    return crate::common::i18n::t(lang, "Creazione annullata.", &[]).into();
                }
                crate::common::log::warn(&format!("frameworks: {command}: {last}"));
                crate::common::i18n::t(lang, "Comando non riuscito: {0}\n{1}", &[&command, &last])
                    .into()
            })
        };
        let result = (|| -> CmdResult<()> {
            if let Some(from) = &template {
                emit(0, Some(from.display().to_string()), None);
                crate::net::archive::copy_tree(from, &make_at.join(&name)).or_fail("frameworks")?;
            }
            for (i, line) in plan.create.iter().enumerate() {
                run(&ready, i + 1, line, &make_at, &[])?;
            }
            if added {
                let made = make_at.join(&name);
                if !made.is_dir() {
                    return Err(no_folder().into());
                }
                let mut kept = Vec::new();
                // Moved on the same disk; copied from the PC's disk.
                if crate::platform::volume::same_volume(&made, &dir) {
                    crate::projects::frameworks::merge_into(&made, &dir, &mut kept)
                } else {
                    merge_copy(&made, &dir, std::path::Path::new(""), &mut kept)
                }
                .or_fail("frameworks")?;
                if in_vm {
                    std::fs::remove_dir_all(&make_at).or_fail("frameworks")?;
                }
                for k in kept {
                    let text = crate::common::i18n::t(
                        lang,
                        "Già nel progetto, lasciato com'è: {0}",
                        &[&k],
                    );
                    emit(plan.create.len(), None, Some(text));
                }
            }
            if !dir.is_dir() {
                return Err(no_folder().into());
            }
            // What the steps get in their environment: the database's
            // connection (`ATOLLO_DB_*`) and the variables of `.env`.
            let mut step_vars: Vec<(String, String)> = Vec::new();
            let mut db = None;
            if let Some(engine) = &plan.database {
                let text = crate::common::i18n::t(
                    lang,
                    "Creo il database del progetto in {0}…",
                    &[engine],
                );
                emit(plan.create.len(), None, Some(text));
                let made = services
                    .start_for(&name, engine, &db_env)
                    .and_then(|_port| services.project_db_create(&name, engine))
                    .or_fail_chain("database")?;
                if in_vm {
                    // Prepared again: the new database's port opened for the
                    // sandbox and its variables in `db.env` (D-171); the
                    // steps reach it through the VM's way to the PC.
                    ready = super::levels::ready(
                        &fx, &kit, &base_env, &req, &facts, &needs, lang, &host,
                    )?;
                    step_vars.extend(db_vars(&crate::isolation::db_in_vm(&made, made.port)));
                } else {
                    step_vars.extend(db_vars(&made));
                }
                db = Some(made);
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
                let text =
                    crate::common::i18n::t(lang, "Scrivo nel file .env: {0}", &[&keys.join(", ")]);
                emit(plan.create.len(), None, Some(text));
            }
            let db_keys = write_env(&dir, &vars, db.as_ref()).or_fail_chain(".env")?;
            // The steps read those keys from `.env`, not a value of the base.
            step_vars.retain(|(k, _)| !db_keys.contains(k));
            for (i, line) in plan.steps.iter().enumerate() {
                run(&ready, plan.create.len() + i + 1, line, &dir, &step_vars)?;
            }
            if let Err(e) = ignore_env(&dir) {
                crate::common::log::warn(&format!("{name}: .gitignore: {e:#}"));
            }
            Ok(())
        })();
        crate::levels::start::scaffold_end(&kit, &ready);
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
                crate::common::log::warn(&format!("{name}: action {}: {e:#}", a.name));
            }
        }
        crate::versions::pin_newest(&kit, &name);
        settle_project(&app, &task, &kit, lang, &name, &dir);
        // A new project: the pools are scanned again for its site.
        servers.invalidate();
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
#[path = "../../../test/rust/ipc/frameworks.rs"]
mod tests;
