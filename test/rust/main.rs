use super::*;
use crate::kit::Arch;
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;

fn state() -> AppState {
    let kit = Kit::at(std::env::temp_dir().join("devkit-ipc-test"), Arch::X64);
    let editor = std::sync::Arc::new(editor::Editor::new(kit.clone(), KitEnv::default()));
    let setup = std::sync::Arc::new(setup::task::SetupTask::new(
        kit.clone(),
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    AppState {
        setup,
        editor,
        services: std::sync::Arc::new(db::services::Services::new(kit.clone())),
        servers: std::sync::Arc::new(servers::WebServers::new(kit.clone())),
        kit,
        lang: std::sync::RwLock::new("it"),
        moved_from: None,
        webview: "test".into(),
        env: std::sync::RwLock::new(KitEnv::default()),
        host: env::Host::default(),
    }
}

fn invoke(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    url: &str,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: "app_info".into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: url.parse().unwrap(),
            body: tauri::ipc::InvokeBody::default(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize::<serde_json::Value>().unwrap())
}

#[test]
fn help_is_translated() {
    assert_ne!(i18n::t("en", HELP, &[]), HELP);
}

/// DECISIONS.md D-14: an editor window (local server URL) cannot call
/// the app's commands; the main window can.
#[test]
fn editor_windows_cannot_invoke_commands() {
    let app = mock_builder()
        .manage(state())
        .invoke_handler(command_handler!())
        .build(tauri::generate_context!())
        .unwrap();
    let main = WebviewWindowBuilder::new(&app, MAIN, WebviewUrl::App("index.html".into()))
        .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
        .build()
        .unwrap();
    let r = invoke(&main, "http://tauri.localhost");
    assert!(r.is_ok(), "main window must work: {r:?}");

    let editor_url = "http://127.0.0.1:7000/";
    let editor = WebviewWindowBuilder::new(
        &app,
        "editor-1",
        WebviewUrl::External(editor_url.parse().unwrap()),
    )
    .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
    .build()
    .unwrap();
    assert!(
        invoke(&editor, editor_url).is_err(),
        "editor window must be refused"
    );

    // Even a window labelled like an editor but loading app pages is refused.
    let other = WebviewWindowBuilder::new(&app, "editor-2", WebviewUrl::App("index.html".into()))
        .data_directory(std::env::temp_dir().join("devkit-ipc-test-webview"))
        .build()
        .unwrap();
    assert!(invoke(&other, "http://tauri.localhost").is_err());
}
