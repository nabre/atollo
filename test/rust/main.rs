use super::*;
use crate::kit::Arch;
use tauri::test::{get_ipc_response, mock_builder, INVOKE_KEY};
use tauri::webview::InvokeRequest;

fn state() -> AppState {
    let kit = Kit::at(std::env::temp_dir().join("atollo-ipc-test"), Arch::X64);
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

/// The URL of the app's own pages: `http://tauri.localhost` on Windows,
/// `tauri://localhost` on macOS and Linux.
const LOCAL: &str = if cfg!(windows) {
    "http://tauri.localhost"
} else {
    "tauri://localhost"
};

fn invoke(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    url: &str,
) -> Result<serde_json::Value, serde_json::Value> {
    invoke_cmd(webview, "app_info", url)
}

fn invoke_cmd(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    url: &str,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
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

/// WP-1.7: a blocking notice stops the start with its text; a warning
/// (FAT32, exFAT) and no notice let it go on.
#[test]
fn disk_stop_only_for_blocking_notices() {
    use os::volume::DiskNotice;
    let notice = |blocking| DiskNotice {
        blocking,
        text: "noexec".into(),
    };
    assert_eq!(disk_stop(Some(notice(true))), Some("noexec".to_owned()));
    assert_eq!(disk_stop(Some(notice(false))), None);
    assert_eq!(disk_stop(None), None);
}

/// The checks at start change nothing on Windows: no launch problem, no
/// notice on the kit's disk, the old texts (`kit::tests`, `os::webview`).
#[test]
fn start_checks_silent_on_windows() {
    let exe = std::path::Path::new(
        "/private/var/folders/x/AppTranslocation/1/d/Atollo.app/Contents/MacOS/Atollo",
    );
    assert_eq!(kit::launch_problem(kit::Os::Win32, exe, "it"), None);
    assert_eq!(kit::launch_problem(kit::Os::Linux, exe, "it"), None);
    assert!(kit::launch_problem(kit::Os::Darwin, exe, "it").is_some());
    if cfg!(windows) {
        let kit = Kit::at(std::env::temp_dir().join("atollo-ipc-test"), Arch::X64);
        assert_eq!(os::volume::disk_notice(&kit, "it"), None);
    }
}

#[test]
fn help_is_translated() {
    assert_ne!(i18n::t("en", HELP, &[]), HELP);
}

/// The help names the program as it is on each system: `Atollo.exe` on
/// Windows whatever the file is called, the started file elsewhere; it
/// documents `--target` with every target id, in every language.
#[test]
fn help_program_name_and_target_on_every_system() {
    let exe = Path::new("/media/usb/Kit/Atollo");
    for target in kit::Target::ALL {
        let os = target.os;
        let name = program_name(os, Some(exe));
        let unknown = program_name(os, None);
        if os.is_windows() {
            assert_eq!(name, "Atollo.exe", "{target}");
            assert_eq!(unknown, "Atollo.exe", "{target}");
        } else {
            assert_eq!(name, "Atollo", "{target}");
            assert_eq!(unknown, "Atollo", "{target}");
        }
        for lang in ["it", "en", "fr", "de"] {
            let text = help_text(lang, os, Some(exe));
            let first = text.lines().next().unwrap();
            assert!(
                first.contains(&format!(" {name} [")),
                "{target} {lang}: {first}"
            );
            assert!(!text.contains("{0}"), "{target} {lang}");
            if !os.is_windows() {
                assert!(!text.contains(".exe"), "{target} {lang}");
                assert!(!text.contains("agy.cmd"), "{target} {lang}");
            }
            // `--target` under both `components` and `prepare`.
            let lines: Vec<&str> = text.lines().collect();
            let at = |cmd: &str| {
                lines
                    .iter()
                    .position(|l| l.starts_with(&format!("  {cmd} ")))
                    .unwrap_or_else(|| panic!("{lang}: {cmd} missing"))
            };
            for cmd in ["components", "prepare"] {
                let start = at(cmd);
                let has = lines[start + 1..]
                    .iter()
                    .take_while(|l| l.starts_with("    "))
                    .any(|l| l.trim_start().starts_with("--target <id>"));
                assert!(has, "{lang}: --target missing under {cmd}");
            }
            for t in kit::Target::ALL {
                assert!(text.contains(t.as_str()), "{lang}: {t} missing");
            }
        }
    }
}

/// Windows keeps the help it has always printed, byte for byte, apart
/// from the new `--target` lines.
#[test]
fn help_on_windows_names_atollo_exe() {
    let exe = Path::new(r"D:\Kit\Atollo.exe");
    let text = help_text("it", kit::Os::Win32, Some(exe));
    assert!(text.starts_with("Uso: Atollo.exe [comando]\n\nComandi:\n"));
    assert_eq!(
        help_text("en", kit::Os::Win32, Some(exe)).lines().next(),
        Some("Usage: Atollo.exe [command]")
    );
}

/// DECISIONS.md D-14: an editor window (local server URL) cannot call
/// the app's commands; the main window can.
#[test]
fn editor_windows_cannot_invoke_commands() {
    let app = mock_builder()
        .manage(state())
        .invoke_handler(command_handler!())
        .build(context())
        .unwrap();
    let main = WebviewWindowBuilder::new(&app, MAIN, WebviewUrl::App("index.html".into()))
        .data_directory(std::env::temp_dir().join("atollo-ipc-test-webview"))
        .build()
        .unwrap();
    let r = invoke(&main, LOCAL);
    assert!(r.is_ok(), "main window must work: {r:?}");
    // The mount of the kit's disk (WP-1.7) is never sent on Windows.
    if cfg!(windows) {
        let info = r.as_ref().unwrap();
        assert_eq!(info["diskNoexec"], serde_json::Value::Null);
        assert_eq!(info["diskNote"], serde_json::Value::Null);
    }

    let editor_url = "http://127.0.0.1:7000/";
    let editor = WebviewWindowBuilder::new(
        &app,
        "editor-1",
        WebviewUrl::External(editor_url.parse().unwrap()),
    )
    .data_directory(std::env::temp_dir().join("atollo-ipc-test-webview"))
    .build()
    .unwrap();
    assert!(
        invoke(&editor, editor_url).is_err(),
        "editor window must be refused"
    );

    // Even a window labelled like an editor but loading app pages is refused.
    let other = WebviewWindowBuilder::new(&app, "editor-2", WebviewUrl::App("index.html".into()))
        .data_directory(std::env::temp_dir().join("atollo-ipc-test-webview"))
        .build()
        .unwrap();
    assert!(invoke(&other, LOCAL).is_err());
}

/// The app's terminal (editor/pty.rs): its commands answer the main window
/// on macOS and Linux (permissions/pty.toml), never an editor window, and
/// do not exist on Windows, which keeps its consoles.
#[test]
fn terminal_sessions_only_on_unix_and_only_for_main() {
    let app = mock_builder()
        .manage(state())
        .manage(editor::terminal::Consoles::default())
        .invoke_handler(command_handler!())
        .build(context())
        .unwrap();
    let data = std::env::temp_dir().join("atollo-ipc-test-webview-pty");
    let main = WebviewWindowBuilder::new(&app, MAIN, WebviewUrl::App("index.html".into()))
        .data_directory(data.clone())
        .build()
        .unwrap();
    let r = invoke_cmd(&main, "pty_sessions", LOCAL);
    if cfg!(unix) {
        assert_eq!(r, Ok(serde_json::json!([])));
    } else {
        assert!(r.is_err(), "no terminal sessions on Windows: {r:?}");
    }
    let editor_url = "http://127.0.0.1:7000/";
    let editor = WebviewWindowBuilder::new(
        &app,
        "editor-1",
        WebviewUrl::External(editor_url.parse().unwrap()),
    )
    .data_directory(data)
    .build()
    .unwrap();
    assert!(invoke_cmd(&editor, "pty_sessions", editor_url).is_err());
}
