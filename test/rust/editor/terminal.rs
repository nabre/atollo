use super::*;
use crate::kit::{tests::TempDir, Kit, Target};

/// The shell is always the kit's PowerShell on Windows, even before it is
/// installed: the session then says it is missing, never bash or cmd.
#[test]
fn the_shell_is_the_kit_powershell_on_windows() {
    let tmp = TempDir::new("term");
    let kit = Kit::at(tmp.0.join("k & (1)"), crate::test_support::kit_arch())
        .for_target(Target::WIN32_X64);
    kit.prepare().unwrap();
    let dir = kit.projects().join("app");
    let pwsh = crate::setup::kit_path(&kit, r"pwsh\pwsh.exe");
    let shell = pty::shell(&kit, &dir);
    assert_eq!(shell.program, pwsh);
    assert!(shell.args.iter().any(|a| a.ends_with("profile.ps1")));
    let e = pty::ready(&shell, &dir, "it").unwrap_err();
    assert!(e.to_string().contains("non"), "{e:#}");
}

/// The close question and "Rimuovi l'unità" count the sessions that run.
#[test]
fn consoles_count_running_sessions() {
    let consoles = Consoles::default();
    assert_eq!(consoles.count(), 0);
    assert!(consoles.pids().is_empty());
    consoles.close_all();
}

/// BUG-V6-31: PowerShell closes a `'...'` string on ’ and its siblings
/// too: each is doubled like `'`.
#[test]
fn powershell_quotes_typographic_apostrophes() {
    assert_eq!(ps_single_quoted("D:/O'Brien"), "D:/O''Brien");
    assert_eq!(
        ps_single_quoted("D:/O’Brien ‘a’ ‚b‛ {x}"),
        "D:/O’’Brien ‘‘a’’ ‚‚b‛‛ {x}"
    );
    assert_eq!(ps_single_quoted("plain"), "plain");
}

/// The quoted path of a batch in `D:\…\O’Brien {x}` runs in a real
/// PowerShell, as the terminal starts it.
#[cfg(windows)]
#[test]
fn powershell_runs_a_batch_under_a_typographic_apostrophe() {
    let tmp = TempDir::new("term-quote");
    let dir = tmp.0.join("O’Brien's {x}");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("go.cmd");
    std::fs::write(&script, "@exit /b 7\r\n").unwrap();
    let quoted = ps_single_quoted(&script.display().to_string());
    let status = std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
        ])
        .arg(format!("& '{quoted}'; exit $LASTEXITCODE"))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(7));
}

/// D-149: only `terminal-<id>` names the window of a detached session.
#[test]
fn window_labels_name_their_session() {
    assert_eq!(label(7), "terminal-7");
    assert_eq!(session_of("terminal-7"), Some(7));
    assert_eq!(
        session_of(&label(18_446_744_073_709_551_615)),
        Some(u64::MAX)
    );
    for other in [
        "terminal-",
        "terminal-01",
        "terminal-+1",
        "terminal-1a",
        "terminal-99999999999999999999",
        "main",
        "editor-1",
        "Terminal-1",
    ] {
        assert_eq!(session_of(other), None, "{other}");
    }
}

/// A session's output and exit reach a listener of its views once: one
/// emit for the panel and the session's own window, not one each.
#[test]
fn output_and_exit_reach_a_view_once() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use tauri::Listener;

    let app = tauri::test::mock_app();
    let handle = app.handle();
    let (outputs, exits) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let (o, e) = (Arc::clone(&outputs), Arc::clone(&exits));
    handle.listen_any(crate::ipc::events::pty_output(1), move |_| {
        o.fetch_add(1, Ordering::SeqCst);
    });
    handle.listen_any(crate::ipc::events::PTY_EXIT, move |_| {
        e.fetch_add(1, Ordering::SeqCst);
    });
    let chunk = pty::Chunk {
        seq: 1,
        data: "ok".into(),
    };
    emit_output(handle, 1, &chunk);
    let sessions = pty::Sessions::default();
    let s = sessions.insert("t".into(), Path::new("/p"), Box::new(NoIo));
    emit_exit(handle, &s.info());
    assert_eq!(outputs.load(Ordering::SeqCst), 1);
    assert_eq!(exits.load(Ordering::SeqCst), 1);
}

/// The views' filter: the main window and the session's own window only.
#[test]
fn only_the_views_of_the_session_match() {
    let l = |label: &str| EventTarget::AnyLabel {
        label: label.into(),
    };
    assert!(for_views(&EventTarget::Any, 3));
    assert!(for_views(&l(crate::ipc::MAIN), 3));
    assert!(for_views(&l("terminal-3"), 3));
    assert!(for_views(
        &EventTarget::WebviewWindow {
            label: "terminal-3".into()
        },
        3
    ));
    assert!(!for_views(&l("terminal-4"), 3));
    assert!(!for_views(&l("editor-1"), 3));
    assert!(!for_views(&EventTarget::App, 3));
}

/// A PTY that does nothing.
struct NoIo;

impl crate::platform::pty::Io for NoIo {
    fn write(&self, _data: &[u8]) -> anyhow::Result<()> {
        Ok(())
    }
    fn resize(&self, _cols: u16, _rows: u16) -> anyhow::Result<()> {
        Ok(())
    }
    fn pid(&self) -> Option<u32> {
        None
    }
    fn hang_up(&self) {}
    fn left(&self, _leader_running: bool) -> Vec<u32> {
        Vec::new()
    }
    fn kill(&self, _pids: &[u32]) {}
    fn release(&self) {}
}
