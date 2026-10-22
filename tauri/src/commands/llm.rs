//! The local models (llm/, D-151): the server, the catalog and the models
//! on disk, the downloads with their progress (`llm-pull` events), the
//! Hugging Face token. One permission for all of them, `allow-local-models`
//! (permissions/llm.toml).

use std::sync::atomic::AtomicBool;

use super::*;
use crate::llm::catalog::CatalogModel;
use crate::llm::local::LocalModel;
use crate::llm::service::{Hardware, LlmInfo};

/// A model of the catalog as the page shows it: its description in the
/// interface's language.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    #[serde(flatten)]
    pub model: CatalogModel,
    pub bytes: u64,
}

/// Everything the page of the local models shows.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelsView {
    pub catalog: Vec<CatalogEntry>,
    pub local: Vec<LocalModel>,
    /// The downloads under way, by key (`<folder>/<id>`).
    pub pulling: Vec<String>,
    pub folder: String,
    /// The kit drive's file system (FAT32 holds no file over 4 GiB).
    pub fs: Option<String>,
    pub free: Option<u64>,
    pub hardware: Hardware,
    pub hf_token: bool,
}

/// One step of a download, for the page.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PullEvent {
    key: String,
    done: u64,
    total: u64,
    /// `running`, `done`, `failed`, `cancelled`.
    state: &'static str,
    error: Option<String>,
}

fn view(state: &AppState) -> ModelsView {
    let kit = &state.kit;
    let lang = state.lang();
    let root = crate::llm::models_dir(kit);
    let mut at = root.clone();
    while !at.exists() {
        match at.parent() {
            Some(p) => at = p.to_path_buf(),
            None => break,
        }
    }
    let volume = crate::os::volume::info(&at).ok();
    ModelsView {
        catalog: crate::llm::catalog::all()
            .iter()
            .map(|m| CatalogEntry {
                bytes: m.bytes(),
                model: CatalogModel {
                    desc: crate::i18n::t(lang, &m.desc, &[]),
                    ..m.clone()
                },
            })
            .collect(),
        local: crate::llm::local::list(kit),
        pulling: crate::llm::pull::pulling(),
        folder: root.display().to_string(),
        fs: volume.map(|v| v.fs.as_str().to_owned()),
        free: volume.map(|v| v.free),
        hardware: state.llm.hardware(&state.env()),
        hf_token: crate::llm::hf_token(kit).ok().flatten().is_some(),
    }
}

#[tauri::command]
pub async fn llm_status<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<LlmInfo> {
    ensure_main(&webview)?;
    let llm = Arc::clone(&state.llm);
    tauri::async_runtime::spawn_blocking(move || llm.status())
        .await
        .or_fail("llm")
}

#[tauri::command]
pub async fn llm_models<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
) -> CmdResult<ModelsView> {
    ensure_main(&webview)?;
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        view(&app.state::<AppState>())
    })
    .await
    .or_fail("llm")
}

/// Starts the server, installing llama.cpp first if needed.
#[tauri::command]
pub async fn llm_start<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<LlmInfo> {
    ensure_main(&webview)?;
    let (task, kit, lang) = (Arc::clone(&state.setup), state.kit.clone(), state.lang());
    let llm = Arc::clone(&state.llm);
    tauri::async_runtime::spawn_blocking(move || {
        let env = ensure(&app, &task, &kit, lang, &["llama"])?;
        let before = crate::llm::settings::load(&kit).port;
        let port = llm
            .start(&env, lang)
            .map_err(|e| CmdError::from(format!("{e:#}")))?;
        // A port chosen now: Pi's models.json points at it.
        if before != Some(port) {
            crate::agents::sync::run(&kit, None);
        }
        Ok(llm.status())
    })
    .await
    .or_fail("llm")?
}

#[tauri::command]
pub async fn llm_stop<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
) -> CmdResult<LlmInfo> {
    ensure_main(&webview)?;
    let llm = Arc::clone(&state.llm);
    tauri::async_runtime::spawn_blocking(move || {
        llm.stop();
        llm.status()
    })
    .await
    .or_fail("llm")
}

/// "Avvia con Atollo", and the model Pi starts with: an id on disk, or
/// an empty one for none (Pi then uses the first model on disk). The
/// agents are aligned after a change of Pi's model.
#[tauri::command]
pub async fn llm_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    autostart: Option<bool>,
    pi_model: Option<String>,
) -> CmdResult<LlmInfo> {
    ensure_main(&webview)?;
    let (kit, llm) = (state.kit.clone(), Arc::clone(&state.llm));
    tauri::async_runtime::spawn_blocking(move || {
        let pi_model = pi_model.map(|m| Some(m).filter(|m| !m.is_empty()));
        if let Some(Some(id)) = &pi_model {
            if crate::llm::local::find(&kit, id).is_none_or(|m| !m.complete) {
                return Err(fail("llm", format!("model {id} not found")));
            }
        }
        crate::llm::settings::update(&kit, |s| {
            if let Some(a) = autostart {
                s.autostart = a;
            }
            if let Some(m) = pi_model.clone() {
                s.pi_model = m;
            }
            Ok(())
        })
        .or_fail_chain("llm")?;
        if pi_model.is_some() {
            crate::agents::sync::run(&kit, None);
        }
        Ok(llm.status())
    })
    .await
    .or_fail("llm")?
}

/// The GGUF models of a Hugging Face repository (`owner/repo`).
#[tauri::command]
pub async fn llm_hf_inspect<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    repo: String,
) -> CmdResult<crate::llm::hf::RepoInfo> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let net =
            crate::net::Net::new(&Settings::load(&kit).settings, None).or_fail_chain("llm")?;
        let token = crate::llm::hf_token(&kit).or_fail_chain("llm")?;
        crate::llm::hf::inspect(&net, repo.trim(), token.as_deref()).or_fail_chain("llm")
    })
    .await
    .or_fail("llm")?
}

/// Downloads a model of the catalog (`catalog`) or the choice `choice` of
/// the repository `repo` (read again from Hugging Face: the files and their
/// checksums are never the window's). Progress as `llm-pull` events; the
/// server, when it runs, is started again on the new model. Returns the
/// model's id.
#[tauri::command]
pub async fn llm_pull<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    catalog: Option<String>,
    repo: Option<String>,
    choice: Option<String>,
) -> CmdResult<String> {
    ensure_main(&webview)?;
    let (kit, lang, llm) = (state.kit.clone(), state.lang(), Arc::clone(&state.llm));
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        let net =
            crate::net::Net::new(&Settings::load(&kit).settings, None).or_fail_chain("llm")?;
        let (plan, token) = match (catalog.as_deref(), repo.as_deref(), choice.as_deref()) {
            // Public: a token that cannot be read does not stop it.
            (Some(id), None, None) => (
                crate::llm::catalog::find(id)
                    .map(crate::llm::pull::Plan::from_catalog)
                    .ok_or_else(|| fail("llm", format!("unknown model {id}")))?,
                crate::llm::hf_token_if_readable(&kit),
            ),
            (None, Some(repo), Some(choice)) => {
                let token = crate::llm::hf_token(&kit).or_fail_chain("llm")?;
                let info = crate::llm::hf::inspect(&net, repo.trim(), token.as_deref())
                    .or_fail_chain("llm")?;
                let plan =
                    crate::llm::pull::Plan::of_choice(&kit, &info, choice).or_fail_chain("llm")?;
                (plan, token)
            }
            _ => return Err(fail("llm", "catalog, or repo and choice")),
        };
        let key = format!("{}/{}", crate::llm::local::folder_of(&plan.repo), plan.id);
        let cancel = Arc::new(AtomicBool::new(false));
        let emit = |done: u64, total: u64, state: &'static str, error: Option<String>| {
            let _ = app.emit_to(
                MAIN,
                "llm-pull",
                PullEvent {
                    key: key.clone(),
                    done,
                    total,
                    state,
                    error,
                },
            );
        };
        let total = plan.bytes();
        let mut last = std::time::Instant::now();
        let result = crate::llm::pull::pull(
            &kit,
            &net,
            &plan,
            token.as_deref(),
            lang,
            cancel,
            &mut |done, total| {
                if last.elapsed() >= std::time::Duration::from_millis(250) || done == total {
                    last = std::time::Instant::now();
                    emit(done, total, "running", None);
                }
            },
        );
        match result {
            Ok(id) => {
                emit(total, total, "done", None);
                let env = app.state::<AppState>().env();
                if let Err(e) = llm.reload(&env, lang) {
                    crate::log::warn(&format!("llm: restart after a download: {e:#}"));
                }
                crate::agents::sync::run(&kit, None);
                Ok(id)
            }
            // Paused: not an error, what arrived is kept for «Riprendi».
            Err(e) if e.is::<crate::net::Cancelled>() => {
                emit(0, total, "cancelled", None);
                Ok(String::new())
            }
            Err(e) => {
                emit(0, total, "failed", Some(format!("{e:#}")));
                Err(fail("llm", format!("{e:#}")))
            }
        }
    })
    .await
    .or_fail("llm")?
}

/// Stops the download `key`; what arrived is kept for the next time.
#[tauri::command]
pub fn llm_pull_cancel<R: Runtime>(webview: Webview<R>, key: String) -> CmdResult<()> {
    ensure_main(&webview)?;
    crate::llm::pull::cancel(&key);
    Ok(())
}

/// Deletes a model (or what was downloaded of it), with the server stopped
/// meanwhile: on Windows a model it has open cannot be deleted.
#[tauri::command]
pub async fn llm_remove<R: Runtime>(
    app: AppHandle<R>,
    webview: Webview<R>,
    state: State<'_, AppState>,
    id: String,
) -> CmdResult<ModelsView> {
    ensure_main(&webview)?;
    let (kit, lang, llm, env) = (
        state.kit.clone(),
        state.lang(),
        Arc::clone(&state.llm),
        state.env(),
    );
    let handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        // Refused while a download goes on in its folder (llm::local::remove).
        llm.without_server(&env, lang, || crate::llm::local::remove(&kit, &id))
            .or_fail_chain("llm")?;
        // Pi's model gone: Pi is set to none.
        if crate::llm::settings::load(&kit).pi_model.as_deref() == Some(id.as_str()) {
            let _ = crate::llm::settings::update(&kit, |s| {
                s.pi_model = None;
                Ok(())
            });
        }
        crate::agents::sync::run(&kit, None);
        Ok(view(&handle.state::<AppState>()))
    })
    .await
    .or_fail("llm")?
}

/// The Hugging Face token (gated models): saved owner-only, or removed
/// with an empty one.
#[tauri::command]
pub async fn llm_hf_token_set<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, AppState>,
    token: String,
) -> CmdResult<()> {
    ensure_main(&webview)?;
    let kit = state.kit.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let file = crate::llm::hf_token_file(&kit);
        let token = token.trim();
        if token.is_empty() {
            match std::fs::remove_file(&file) {
                Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                    return Err(fail("llm", e.to_string()))
                }
                _ => return Ok(()),
            }
        }
        if !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        {
            return Err(fail("llm", "not a Hugging Face token"));
        }
        crate::router::fs::write_private(&file, token).or_fail_chain("llm")
    })
    .await
    .or_fail("llm")?
}

/// The server's key, for the chat in the browser (its settings ask for it).
#[tauri::command]
pub fn llm_key<R: Runtime>(webview: Webview<R>, state: State<'_, AppState>) -> CmdResult<String> {
    ensure_main(&webview)?;
    crate::llm::settings::ensure_key(&state.kit).or_fail_chain("llm")
}
