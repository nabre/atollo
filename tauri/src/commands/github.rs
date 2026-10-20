//! GitHub page: the kit's GitHub account (github/, DECISIONS.md D-70).

use super::*;

pub(super) type GithubLogin<'a> = State<'a, Arc<crate::github::Login>>;

/// Sends the account's state to the window (`github` event).
pub(super) fn github_notify<R: Runtime>(app: AppHandle<R>) -> Arc<dyn Fn() + Send + Sync> {
    Arc::new(move || {
        use tauri::Manager;
        let state = app.state::<AppState>();
        let login = app.state::<Arc<crate::github::Login>>();
        let _ = app.emit_to(MAIN, "github", login.status(&state.kit, &state.env()));
    })
}

/// A refusal of GitHub's API as a short text.
pub(super) fn github_error(lang: &str, e: &crate::net::ApiError) -> String {
    crate::log::warn(&format!("github: {:?} {}", e.status, e.detail));
    match e.status {
        Some(401) => crate::i18n::t(lang, "GitHub ha rifiutato l'accesso: accedi di nuovo.", &[]),
        None => crate::i18n::t(lang, "GitHub non risponde: {0}", &[&e.detail]),
        Some(_) => crate::i18n::t(lang, "GitHub: {0}", &[&e.detail]),
    }
}

/// The account's token, or the text asking to sign in.
pub(super) fn github_token_of(kit: &Kit, env: &KitEnv, lang: &str) -> CmdResult<String> {
    crate::github::token(kit, env)
        .ok_or_else(|| crate::i18n::t(lang, "Accedi prima a GitHub.", &[]).into())
}

pub(super) fn github_net(kit: &Kit) -> CmdResult<crate::net::Net> {
    crate::net::Net::new(&Settings::load(kit).settings, None).or_fail_chain("github")
}

pub(super) fn check_full_name(full_name: &str) -> CmdResult<()> {
    if crate::github::valid_full_name(full_name) {
        Ok(())
    } else {
        Err(format!("not a GitHub repository: {full_name}").into())
    }
}

#[tauri::command]
pub async fn github_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    login: GithubLogin<'_>,
) -> CmdResult<crate::github::Status> {
    ensure_main(&webview)?;
    let (kit, env, login) = (state.kit.clone(), state.env(), Arc::clone(&login));
    tauri::async_runtime::spawn_blocking(move || login.status(&kit, &env))
        .await
        .or_fail("github")
}

/// Starts the sign-in (`delete`: with the permission to delete
/// repositories) and waits for the one-time code, opening the GitHub page
/// where it goes. The end arrives as a `github` event.
#[tauri::command]
pub async fn github_sign_in<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    login: GithubLogin<'_>,
    delete: bool,
) -> CmdResult<crate::github::Status> {
    ensure_main(&webview)?;
    let (task, kit, lang, login) = (
        Arc::clone(&state.setup),
        state.kit.clone(),
        state.lang(),
        Arc::clone(&login),
    );
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["gh"])?;
        login
            .start(&kit, &env, delete, github_notify(app.clone()))
            .or_fail_chain("github sign-in")?;
        let until = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let status = login.status(&kit, &env);
            if status.code.is_some() || !status.signing_in || std::time::Instant::now() > until {
                return Ok(status);
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    })
    .await
    .or_fail("github")?
}

#[tauri::command]
pub fn github_sign_in_cancel<R: Runtime>(
    webview: Webview<R>,
    login: GithubLogin<'_>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    login.cancel();
    Ok(())
}

#[tauri::command]
pub async fn github_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::github::sign_out(&kit, &env))
        .await
        .or_fail("github")?
        .or_fail_chain("github sign-out")
}

#[tauri::command]
pub async fn github_repos<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<crate::github::Repos> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        Ok(crate::github::repos(&kit, &github_net(&kit)?, &token)
            .map_err(|e| github_error(lang, &e))?)
    })
    .await
    .or_fail("github")?
}

/// Clones a repository of the account into `projects\`; returns the project.
/// A new clone is settled as one from an address (`project_clone`): the
/// same [`settle_project`], in the same blocking task.
#[tauri::command]
pub async fn github_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let servers = Arc::clone(&state.servers);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        let cloned = crate::github::clone_repo(&kit, &env, &full_name, token.as_deref(), lang)
            .or_fail_chain("clone")?;
        if cloned.new {
            let dir = kit.projects().join(&cloned.project);
            settle_project(&app, &task, &kit, lang, &cloned.project, &dir);
            // A new project: the pools are scanned again for its site.
            servers.invalidate();
        }
        Ok(cloned.project)
    })
    .await
    .or_fail("clone")?
}

#[tauri::command]
pub async fn github_rename<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
    name: String,
) -> CmdResult<crate::github::Renamed> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let name = name.trim().to_owned();
    if !crate::github::valid_repo_name(&name) {
        return Err(tr(
            &state,
            "Nome non valido: solo lettere, cifre, «.», «-» e «_».",
            &[],
        )
        .into());
    }
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        Ok(
            crate::github::rename(&kit, &env, &github_net(&kit)?, &token, &full_name, &name)
                .map_err(|e| github_error(lang, &e))?,
        )
    })
    .await
    .or_fail("github")?
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Deleted {
    Deleted,
    /// GitHub refused: the account lacks the permission to delete
    /// repositories (or is not an administrator of this one).
    NeedsPermission,
}

#[tauri::command]
pub async fn github_delete<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
) -> CmdResult<Deleted> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        match crate::github::delete(&github_net(&kit)?, &token, &full_name) {
            Ok(()) => {
                crate::log::info(&format!("github: {full_name} deleted"));
                Ok(Deleted::Deleted)
            }
            Err(e) if matches!(e.status, Some(403 | 404)) => Ok(Deleted::NeedsPermission),
            Err(e) => Err(github_error(lang, &e).into()),
        }
    })
    .await
    .or_fail("github")?
}

/// Opens a repository on github.com (`None`: the page where the sign-in
/// code goes) in the PC's browser.
#[tauri::command]
pub fn github_open<R: Runtime>(webview: Webview<R>, full_name: Option<String>) -> CmdResult<()> {
    ensure_main(&webview)?;
    let path = match full_name {
        Some(f) => {
            check_full_name(&f)?;
            f
        }
        None => crate::github::DEVICE_PATH.to_owned(),
    };
    crate::github::open_web(&path).or_fail_chain("browser")
}

/// Who signs the commits of the kit's Git (`None`: Git is not in the kit).
#[tauri::command]
pub async fn git_identity_get<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<Option<crate::github::Identity>> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::github::identity(&kit, &env))
        .await
        .or_fail("git")
}

/// Saves who signs the commits; an empty value is removed (the account's
/// come back at the next start while signed in to GitHub).
#[tauri::command]
pub async fn git_identity_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    identity: crate::github::Identity,
) -> CmdResult<crate::github::Identity> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        crate::github::set_identity(&kit, &env, &identity, lang)
    })
    .await
    .or_fail("git")?
    .map_err(|e| CmdError::from(format!("{e:#}")))
}
