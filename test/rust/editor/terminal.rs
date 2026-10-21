use super::*;
use crate::kit::{tests::TempDir, Arch, Target};

/// On Windows a batch agent (`agy.cmd`) runs through the kit's PowerShell,
/// in the app's terminal, never through cmd or a shell of the PC.
#[test]
fn batch_runs_through_the_kit_powershell() {
    let tmp = TempDir::new("term");
    let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64).for_target(Target::WIN32_X64);
    kit.prepare().unwrap();
    let script = kit.root().join("it's").join("agy.cmd");
    let dir = kit.projects().join("app");
    let l = pty::command(&kit, &script, &dir);
    let pwsh = crate::setup::catalog::kit_path(&kit, r"pwsh\pwsh.exe");
    assert_eq!(l.program, pwsh);
    assert_eq!(
        l.args[..5],
        [
            "-NoLogo",
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command"
        ]
    );
    let quoted = format!("& '{}'; exit $LASTEXITCODE", script.display()).replace("it's", "it''s");
    assert_eq!(l.args[5], quoted);
    assert_eq!(l.title, "app · agy");
    assert_eq!(
        l.conpty,
        Some(crate::setup::catalog::kit_path(&kit, "conpty"))
    );

    // An exe agent runs directly.
    let claude = kit.apps().join("claude").join("claude.exe");
    let l = pty::command(&kit, &claude, &dir);
    assert_eq!(l.program, claude);
    assert!(l.args.is_empty());

    // The shell is always the kit's PowerShell on Windows, even before it
    // is installed: the session then says it is missing, never bash or cmd.
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
