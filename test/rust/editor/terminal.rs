use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn batch_runs_through_the_kit_powershell_in_conhost() {
    let tmp = TempDir::new("term");
    let kit = Kit::at(tmp.0.join("k & (1)"), Arch::X64);
    kit.prepare().unwrap();
    let script = kit.root().join(r"it's\agy.cmd");
    let env = KitEnv::default();
    let batch = Program::Command(&script);
    assert!(line(&kit, &env, &batch).is_err(), "no PC shell");
    assert!(line(&kit, &env, &Program::Shell).is_err());

    let pwsh = kit.apps().join(r"pwsh\pwsh.exe");
    std::fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
    std::fs::write(&pwsh, b"").unwrap();
    let l = line(&kit, &env, &batch).unwrap();
    assert!(l
        .conhost
        .to_string_lossy()
        .to_lowercase()
        .ends_with(r"\system32\conhost.exe"));
    assert_eq!(l.args[0], pwsh.display().to_string());
    assert!(
        l.args.last().unwrap().contains(r"it''s\agy.cmd'"),
        "{:?}",
        l.args
    );

    let shell = line(&kit, &env, &Program::Shell).unwrap();
    assert_eq!(shell.args[0], pwsh.display().to_string());
    assert!(shell.args.iter().any(|a| a.ends_with("profile.ps1")));
}

/// A real conhost runs the program (in a folder with spaces) with the
/// kit environment, and the console is counted until it closes.
#[test]
fn real_console_runs_and_ends() {
    let tmp = TempDir::new("term-real");
    let dir = tmp.0.join("a b");
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("probe.cmd");
    std::fs::write(&script, "@echo %ATOLLO_PROBE%> ran.txt\r\n").unwrap();
    let mut env = KitEnv::default();
    env.set("ATOLLO_PROBE", "kit-env");
    // The app's environment always has it: Windows on ARM64 starts no
    // process without SystemRoot.
    if let Some(root) = std::env::var_os("SystemRoot") {
        env.set("SystemRoot", root.to_string_lossy());
    }
    // Through cmd here: the test kit has no PowerShell.
    let l = Line {
        conhost: conhost(&env),
        args: vec![
            r"C:\Windows\System32\cmd.exe".into(),
            "/d".into(),
            "/c".into(),
            script.display().to_string(),
        ],
    };
    let consoles = Consoles::default();
    let (tx, rx) = std::sync::mpsc::channel();
    start(
        &consoles,
        &env,
        &dir,
        &l,
        Box::new(move || tx.send(()).unwrap()),
    )
    .unwrap();
    assert_eq!(consoles.count(), 1);
    rx.recv_timeout(std::time::Duration::from_secs(20)).unwrap();
    assert_eq!(consoles.count(), 0);
    let ran = std::fs::read_to_string(dir.join("ran.txt")).unwrap();
    assert_eq!(ran.trim(), "kit-env", "the kit environment only");
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
/// PowerShell, as the console starts it.
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
