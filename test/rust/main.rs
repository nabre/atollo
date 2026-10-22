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
        llm: std::sync::Arc::new(llm::service::Llm::new(kit.clone())),
        servers: std::sync::Arc::new(servers::WebServers::new(kit.clone())),
        kit,
        lang: std::sync::RwLock::new("it"),
        moved_from: None,
        webview: "test".into(),
        env: std::sync::RwLock::new(KitEnv::default()),
        host: env::Host::default(),
        closing: agents::Closing::default(),
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

/// D-105: a blocking notice stops the start with its text; a warning
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

/// The help names the program as it is on each system: the started file
/// (on Windows only an `.exe`, else `Atollo.exe`); it documents `--target`
/// with every target id, in every language.
#[test]
fn help_program_name_and_target_on_every_system() {
    // A Windows path, read on any system the tests run on.
    for release in [
        "/media/usb/Kit/Atollo-0.2.0-win32-x64.exe",
        r"D:\Kit\Atollo-0.2.0-win32-x64.exe",
    ] {
        assert_eq!(
            program_name(kit::Os::Win32, Some(Path::new(release))),
            "Atollo-0.2.0-win32-x64.exe"
        );
    }
    assert_eq!(
        program_name(kit::Os::Win32, Some(Path::new(r"D:\Kit\Atollo.exe"))),
        "Atollo.exe"
    );
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

/// The help printed before `--target` was documented, on Windows (with
/// prepare's removals and `--only`'s needs spelled out since): the
/// current text must be exactly this one with the `--target` lines added.
const OLD_HELP_IT: &str = "Uso: Atollo.exe [comando]\n\nComandi:\n  env                     mostra l'ambiente del kit\n      --project <nome>    quello dei programmi di un progetto\n  components              elenca linguaggi e strumenti, con quelli installati\n  versions [id]           le versioni salvate; con un id, quelle da scaricare\n  check                   verifica che i programmi installati rispondano\n  prepare [opzioni]       installa e aggiorna i componenti scelti,\n                          rimuove quelli non scelti\n      --only a,b          solo questi e ciò che serve loro, senza rimuovere\n      --skip a,b          non questi\n      --update-all        aggiorna anche quelli già installati\n      --force             scarica di nuovo\n      --line id=x.y       salva questa versione accanto alla predefinita\n  sync [opzioni]          allinea istruzioni, skill e server MCP degli agenti\n      --project <cartella> anche le istruzioni di questo progetto\n      --verbose           mostra ogni modifica\n  keyring restore|save    accesso di Antigravity (lo usa agy.cmd del kit)\n  serve <cartella> <porta> server di file per un sito statico\n  version                 mostra la versione\n  help                    mostra questo aiuto\n\nSenza comando apre la finestra di Atollo.";

const OLD_HELP_EN: &str = "Usage: Atollo.exe [command]\n\nCommands:\n  env                     show the kit environment\n      --project <name>    the one of a project's programs\n  components              list languages and tools, with the installed ones\n  versions [id]           the saved versions; with an id, those to download\n  check                   check that the installed programs answer\n  prepare [options]       install and update the chosen components,\n                          remove the ones not chosen\n      --only a,b          only these and what they need, removing nothing\n      --skip a,b          not these\n      --update-all        also update those already installed\n      --force             download again\n      --line id=x.y       save this version next to the default\n  sync [options]          align the agents' instructions, skills and MCP servers\n      --project <folder>  also the instructions of this project\n      --verbose           show every change\n  keyring restore|save    Antigravity sign-in (used by the kit's agy.cmd)\n  serve <folder> <port>   file server for a static site\n  version                 show the version\n  help                    show this help\n\nWithout a command, the Atollo window opens.";

const OLD_HELP_FR: &str = "Utilisation : Atollo.exe [commande]\n\nCommandes :\n  env                     affiche l'environnement du kit\n      --project <nom>     celui des programmes d'un projet\n  components              liste les langages et outils, avec ceux installés\n  versions [id]           les versions gardées ; avec un id, celles à télécharger\n  check                   vérifie que les programmes installés répondent\n  prepare [options]       installe et met à jour les composants choisis,\n                          supprime ceux qui ne le sont pas\n      --only a,b          ceux-ci et ce dont ils ont besoin, sans rien supprimer\n      --skip a,b          pas ceux-ci\n      --update-all        met aussi à jour ceux déjà installés\n      --force             télécharge à nouveau\n      --line id=x.y       garde cette version à côté de celle par défaut\n  sync [options]          aligne les instructions, skills et serveurs MCP des agents\n      --project <dossier> aussi les instructions de ce projet\n      --verbose           affiche chaque modification\n  keyring restore|save    connexion d'Antigravity (utilisé par agy.cmd du kit)\n  serve <dossier> <port>  serveur de fichiers pour un site statique\n  version                 affiche la version\n  help                    affiche cette aide\n\nSans commande, la fenêtre d'Atollo s'ouvre.";

const OLD_HELP_DE: &str = "Verwendung: Atollo.exe [Befehl]\n\nBefehle:\n  env                     zeigt die Umgebung des Kits\n      --project <Name>    die der Programme eines Projekts\n  components              listet Sprachen und Werkzeuge, mit den installierten\n  versions [id]           die behaltenen Versionen; mit einer id die herunterladbaren\n  check                   prüft, ob die installierten Programme antworten\n  prepare [Optionen]      installiert und aktualisiert die gewählten Komponenten,\n                          entfernt die nicht gewählten\n      --only a,b          nur diese und was sie brauchen, ohne zu entfernen\n      --skip a,b          diese nicht\n      --update-all        aktualisiert auch die bereits installierten\n      --force             lädt erneut herunter\n      --line id=x.y       behält diese Version neben der Standardversion\n  sync [Optionen]         gleicht Anweisungen, Skills und MCP-Server der Agenten ab\n      --project <Ordner>  auch die Anweisungen dieses Projekts\n      --verbose           zeigt jede Änderung\n  keyring restore|save    Anmeldung von Antigravity (verwendet von agy.cmd des Kits)\n  serve <Ordner> <Port>   Dateiserver für eine statische Website\n  version                 zeigt die Version\n  help                    zeigt diese Hilfe\n\nOhne Befehl öffnet sich das Fenster von Atollo.";

/// `old` with `components_lines` inserted after the line of `components`,
/// `prepare_line` after the line of `--line` and the lines of the local
/// models (D-151) after the line of `serve`.
fn splice_target_lines(old: &str, components_lines: &[&str], prepare_line: &str) -> String {
    let mut out = Vec::new();
    let lang = if old.starts_with("Uso") {
        "it"
    } else if old.starts_with("Usage") {
        "en"
    } else if old.starts_with("Utilisation") {
        "fr"
    } else {
        "de"
    };
    for line in old.split('\n') {
        out.push(line);
        if line.starts_with("  components ") {
            out.extend_from_slice(components_lines);
        }
        if line.starts_with("      --line ") {
            out.push(prepare_line);
        }
        if line.starts_with("  serve ") {
            out.extend_from_slice(models_lines(lang));
        }
    }
    out.join("\n")
}

/// The lines of `models`, `llm` and `diagnostics` in `lang`.
fn models_lines(lang: &str) -> &'static [&'static str] {
    match lang {
        "it" => &[
            "  models                  i modelli locali: del catalogo e scaricati",
            "  models pull <modello>   scarica un id del catalogo o un modello di Hugging Face",
            "                          (proprietario/nome:quantizzazione), riprende se interrotto",
            "  models rm <id>          cancella un modello scaricato",
            "  llm start|stop|status   il server dei modelli locali (chat e agenti)",
            "  diagnostics             crea lo zip da allegare a una segnalazione (segreti mascherati)",
        ],
        "en" => &[
            "  models                  the local models: the catalog's and the downloaded ones",
            "  models pull <model>     download a catalog id or a Hugging Face model",
            "                          (owner/name:quantization), resumed if interrupted",
            "  models rm <id>          delete a downloaded model",
            "  llm start|stop|status   the server of the local models (chat and agents)",
            "  diagnostics             make the zip to attach to a problem report (secrets masked)",
        ],
        "fr" => &[
            "  models                  les modèles locaux : ceux du catalogue et les téléchargés",
            "  models pull <modèle>    télécharge un id du catalogue ou un modèle de Hugging Face",
            "                          (propriétaire/nom:quantification), reprend s'il est interrompu",
            "  models rm <id>          supprime un modèle téléchargé",
            "  llm start|stop|status   le serveur des modèles locaux (chat et agents)",
            "  diagnostics             crée le zip à joindre à un signalement (secrets masqués)",
        ],
        _ => &[
            "  models                  die lokalen Modelle: die des Katalogs und die heruntergeladenen",
            "  models pull <Modell>    lädt eine Katalog-id oder ein Modell von Hugging Face herunter",
            "                          (Besitzer/Name:Quantisierung), setzt nach Abbruch fort",
            "  models rm <id>          löscht ein heruntergeladenes Modell",
            "  llm start|stop|status   der Server der lokalen Modelle (Chat und Agenten)",
            "  diagnostics             erstellt das Zip für eine Problemmeldung (Geheimnisse maskiert)",
        ],
    }
}

/// Windows keeps the help it has always printed, byte for byte, apart
/// from the new `--target` lines: `Atollo.exe` whatever the file is
/// called, and `agy.cmd` on the keyring line.
#[test]
fn help_on_windows_is_the_old_text_plus_target() {
    let exe = Path::new(r"D:\Kit\Atollo.exe");
    let os = kit::Os::Win32;
    assert_eq!(
        help_text("it", os, Some(exe)),
        splice_target_lines(
            OLD_HELP_IT,
            &[
                "      --target <id>       per un altro sistema: win32-x64, win32-arm64,",
                "                          darwin-arm64, darwin-x64, linux-x64, linux-arm64",
            ],
            "      --target <id>       prepara per un altro sistema (id come per components)",
        )
    );
    assert_eq!(
        help_text("en", os, None),
        splice_target_lines(
            OLD_HELP_EN,
            &[
                "      --target <id>       for another system: win32-x64, win32-arm64,",
                "                          darwin-arm64, darwin-x64, linux-x64, linux-arm64",
            ],
            "      --target <id>       prepare for another system (ids as for components)",
        )
    );
    assert_eq!(
        help_text("fr", os, Some(exe)),
        splice_target_lines(
            OLD_HELP_FR,
            &[
                "      --target <id>       pour un autre système : win32-x64, win32-arm64,",
                "                          darwin-arm64, darwin-x64, linux-x64, linux-arm64",
            ],
            "      --target <id>       prépare pour un autre système (id comme pour components)",
        )
    );
    assert_eq!(
        help_text("de", os, Some(exe)),
        splice_target_lines(
            OLD_HELP_DE,
            &[
                "      --target <id>       für ein anderes System: win32-x64, win32-arm64,",
                "                          darwin-arm64, darwin-x64, linux-x64, linux-arm64",
            ],
            "      --target <id>       bereitet für ein anderes System vor (ids wie bei components)",
        )
    );
}

/// Elsewhere the keyring line names the kit's `agy`, never `agy.cmd`.
#[test]
fn help_off_windows_names_agy() {
    for os in [kit::Os::Linux, kit::Os::Darwin] {
        for lang in ["it", "en", "fr", "de"] {
            let text = help_text(lang, os, None);
            assert!(
                text.contains(" agy ") || text.contains(" agy)"),
                "{os:?} {lang}"
            );
            assert!(!text.contains("agy.cmd"), "{os:?} {lang}");
            assert!(!text.contains("{1}"), "{os:?} {lang}");
        }
    }
}

/// From an AppImage the help names the AppImage the user started (what
/// `kit::app_exe` returns there), not the inner program of its mount.
#[test]
fn help_names_the_appimage_on_linux() {
    let exe = Path::new("/home/u/Kit/Atollo.AppImage");
    assert_eq!(program_name(kit::Os::Linux, Some(exe)), "Atollo.AppImage");
    assert!(help_text("en", kit::Os::Linux, Some(exe))
        .starts_with("Usage: Atollo.AppImage [command]\n"));
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
    // The mount of the kit's disk (D-105) is never sent on Windows.
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
/// on every system (permissions/pty.toml, D-148), never an editor window.
#[test]
fn terminal_sessions_only_for_main() {
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
    assert_eq!(r, Ok(serde_json::json!([])));
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

/// The local models (permissions/llm.toml, D-151): the main window only,
/// never an editor window nor a detached terminal.
#[test]
fn local_models_only_for_main() {
    let app = mock_builder()
        .manage(state())
        .invoke_handler(command_handler!())
        .build(context())
        .unwrap();
    let data = std::env::temp_dir().join("atollo-ipc-test-webview-llm");
    let main = WebviewWindowBuilder::new(&app, MAIN, WebviewUrl::App("index.html".into()))
        .data_directory(data.clone())
        .build()
        .unwrap();
    let r = invoke_cmd(&main, "llm_status", LOCAL);
    assert_eq!(
        r.as_ref().map(|v| v["state"].clone()),
        Ok(serde_json::json!("stopped")),
        "{r:?}"
    );
    let editor_url = "http://127.0.0.1:7000/";
    let editor = WebviewWindowBuilder::new(
        &app,
        "editor-1",
        WebviewUrl::External(editor_url.parse().unwrap()),
    )
    .data_directory(data.clone())
    .build()
    .unwrap();
    assert!(invoke_cmd(&editor, "llm_status", editor_url).is_err());
    let terminal =
        WebviewWindowBuilder::new(&app, "terminal-1", WebviewUrl::App("terminal.html".into()))
            .data_directory(data)
            .build()
            .unwrap();
    assert!(invoke_cmd(&terminal, "llm_key", LOCAL).is_err());
}

fn invoke_with(
    webview: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    body: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    get_ipc_response(
        webview,
        InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: LOCAL.parse().unwrap(),
            body: tauri::ipc::InvokeBody::Json(body),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    )
    .map(|b| b.deserialize::<serde_json::Value>().unwrap())
}

/// D-149: the window of a detached session (`terminal-<id>`) reaches the
/// terminal's commands for its own session only, and none of the app's.
#[test]
fn detached_terminal_reaches_only_its_session() {
    let app = mock_builder()
        .manage(state())
        .manage(editor::terminal::Consoles::default())
        .invoke_handler(command_handler!())
        .build(context())
        .unwrap();
    let data = std::env::temp_dir().join("atollo-ipc-test-webview-pty-window");
    let window =
        WebviewWindowBuilder::new(&app, "terminal-1", WebviewUrl::App("terminal.html".into()))
            .data_directory(data)
            .build()
            .unwrap();
    // Its own session (gone here): listed as none, asked for by id.
    assert_eq!(
        invoke_cmd(&window, "pty_sessions", LOCAL),
        Ok(serde_json::json!([]))
    );
    let own = invoke_with(&window, "pty_backlog", serde_json::json!({ "id": 1 }));
    assert!(
        own.is_err_and(|e| e.to_string().contains("not found")),
        "its own: allowed"
    );
    for (cmd, body) in [
        ("pty_backlog", serde_json::json!({ "id": 2 })),
        ("pty_write", serde_json::json!({ "id": 2, "data": "x" })),
        (
            "pty_resize",
            serde_json::json!({ "id": 2, "cols": 80, "rows": 24 }),
        ),
        ("pty_close", serde_json::json!({ "id": 2 })),
        ("pty_dock", serde_json::json!({ "id": 2 })),
        // Only the panel detaches.
        ("pty_detach", serde_json::json!({ "id": 1 })),
    ] {
        let r = invoke_with(&window, cmd, body);
        assert!(
            r.is_err_and(|e| e.to_string().contains("forbidden")),
            "{cmd}"
        );
    }
    // No other command of the app (capabilities/terminal.json).
    assert!(invoke(&window, LOCAL).is_err());
    assert!(invoke_with(&window, "terminal_launch", serde_json::json!({ "id": "x" })).is_err());
}
