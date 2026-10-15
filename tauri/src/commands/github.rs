//! GitHub page: the kit's GitHub account (github.rs, DECISIONS.md D-70).

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
pub(super) fn github_token_of(kit: &Kit, env: &KitEnv, lang: &str) -> Result<String, String> {
    crate::github::token(kit, env)
        .ok_or_else(|| crate::i18n::t(lang, "Accedi prima a GitHub.", &[]))
}

pub(super) fn github_net(kit: &Kit) -> Result<crate::net::Net, String> {
    crate::net::Net::new(&Settings::load(kit).settings, None)
        .map_err(|e| fail("github", format!("{e:#}")))
}

pub(super) fn check_full_name(full_name: &str) -> Result<(), String> {
    if crate::github::valid_full_name(full_name) {
        Ok(())
    } else {
        Err(format!("not a GitHub repository: {full_name}"))
    }
}

#[tauri::command]
pub async fn github_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    login: GithubLogin<'_>,
) -> Result<crate::github::Status, String> {
    ensure_main(&webview)?;
    let (kit, env, login) = (state.kit.clone(), state.env(), Arc::clone(&login));
    tauri::async_runtime::spawn_blocking(move || login.status(&kit, &env))
        .await
        .map_err(|e| fail("github", e))
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
) -> Result<crate::github::Status, String> {
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
            .map_err(|e| fail("github sign-in", format!("{e:#}")))?;
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
    .map_err(|e| fail("github", e))?
}

#[tauri::command]
pub fn github_sign_in_cancel<R: Runtime>(
    webview: Webview<R>,
    login: GithubLogin<'_>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    login.cancel();
    Ok(())
}

#[tauri::command]
pub async fn github_sign_out<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let (kit, env) = (state.kit.clone(), state.env());
    tauri::async_runtime::spawn_blocking(move || crate::github::sign_out(&kit, &env))
        .await
        .map_err(|e| fail("github", e))?
        .map_err(|e| fail("github sign-out", format!("{e:#}")))
}

#[tauri::command]
pub async fn github_repos<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> Result<crate::github::Repos, String> {
    ensure_main(&webview)?;
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        crate::github::repos(&kit, &github_net(&kit)?, &token).map_err(|e| github_error(lang, &e))
    })
    .await
    .map_err(|e| fail("github", e))?
}

/// Clones a repository of the account into `projects\`; returns the project.
#[tauri::command]
pub async fn github_clone<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
) -> Result<String, String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["git"])?;
        let token = crate::github::token(&kit, &env);
        crate::github::clone_repo(&kit, &env, &full_name, token.as_deref(), lang)
            .map_err(|e| fail("clone", format!("{e:#}")))
    })
    .await
    .map_err(|e| fail("clone", e))?
}

#[tauri::command]
pub async fn github_rename<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    full_name: String,
    name: String,
) -> Result<crate::github::Renamed, String> {
    ensure_main(&webview)?;
    check_full_name(&full_name)?;
    let name = name.trim().to_owned();
    if !crate::github::valid_repo_name(&name) {
        return Err(tr(
            &state,
            "Nome non valido: solo lettere, cifre, «.», «-» e «_».",
            &[],
        ));
    }
    let (kit, env, lang) = (state.kit.clone(), state.env(), state.lang());
    tauri::async_runtime::spawn_blocking(move || {
        let token = github_token_of(&kit, &env, lang)?;
        crate::github::rename(&kit, &env, &github_net(&kit)?, &token, &full_name, &name)
            .map_err(|e| github_error(lang, &e))
    })
    .await
    .map_err(|e| fail("github", e))?
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
) -> Result<Deleted, String> {
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
            Err(e) => Err(github_error(lang, &e)),
        }
    })
    .await
    .map_err(|e| fail("github", e))?
}

/// Opens a repository on github.com (`None`: the page where the sign-in
/// code goes) in the PC's browser.
#[tauri::command]
pub fn github_open<R: Runtime>(
    webview: Webview<R>,
    full_name: Option<String>,
) -> Result<(), String> {
    ensure_main(&webview)?;
    let path = match full_name {
        Some(f) => {
            check_full_name(&f)?;
            f
        }
        None => crate::github::DEVICE_PATH.to_owned(),
    };
    crate::github::open_web(&path).map_err(|e| fail("browser", format!("{e:#}")))
}
