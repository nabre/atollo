use super::*;
use crate::editor::pty::{Launch, Sessions};
use crate::kit::tests::TempDir;
use crate::test_support::wait_for;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[test]
fn arguments_are_quoted_as_windows_reads_them() {
    assert_eq!(quote_arg("plain"), "plain");
    assert_eq!(quote_arg(""), "\"\"");
    assert_eq!(quote_arg(r"C:\Program Files\x"), r#""C:\Program Files\x""#);
    // Backslashes before a quote or the closing quote are doubled.
    assert_eq!(quote_arg(r"a b\"), r#""a b\\""#);
    assert_eq!(quote_arg(r#"say "hi""#), r#""say \"hi\"""#);
    assert_eq!(quote_arg(r#"x\"y"#), r#""x\\\"y""#);
    assert_eq!(
        command_line(
            Path::new(r"C:\k k\pwsh.exe"),
            &["-Command".into(), "& 'a'; exit".into()]
        ),
        r#""C:\k k\pwsh.exe" -Command "& 'a'; exit""#
    );
}

#[test]
fn the_environment_block_is_sorted_and_double_terminated() {
    let mut env = KitEnv::default();
    env.set("b", "2");
    env.set("A", "1");
    let block = String::from_utf16(&env_block(&env)).unwrap();
    assert_eq!(block, "A=1\0b=2\0\0");
    assert_eq!(env_block(&KitEnv::default()), [0, 0]);
}

#[test]
fn the_kit_conpty_needs_both_files() {
    let tmp = TempDir::new("conpty-files");
    assert!(!installed_in(&tmp.0));
    std::fs::write(tmp.0.join("conpty.dll"), b"").unwrap();
    assert!(!installed_in(&tmp.0), "OpenConsole.exe missing");
    std::fs::write(tmp.0.join("OpenConsole.exe"), b"").unwrap();
    assert!(installed_in(&tmp.0));
}

fn env() -> KitEnv {
    let mut env = KitEnv::default();
    env.set("ATOLLO_PROBE", "kit-env");
    // Windows on ARM64 starts no process without SystemRoot.
    if let Some(root) = std::env::var_os("SystemRoot") {
        env.set("SystemRoot", root.to_string_lossy());
    }
    env
}

fn cmd(args: &[&str]) -> Launch {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Launch {
        program: PathBuf::from(root).join(r"System32\cmd.exe"),
        args: args.iter().map(|a| (*a).to_owned()).collect(),
        shell: PathBuf::new(),
        title: "test".into(),
        // Windows' own ConPTY: no kit here.
        conpty: None,
    }
}

/// A real pseudo console: the kit environment only, the folder (with
/// spaces), the exit code, then the reader ends.
#[test]
fn a_real_session_runs_with_the_kit_env_and_ends() {
    let tmp = TempDir::new("conpty-real");
    let dir = tmp.0.join("a b");
    std::fs::create_dir_all(&dir).unwrap();
    let sessions = Sessions::default();
    let (tx, rx) = std::sync::mpsc::channel();
    let info = sessions
        .open(
            &cmd(&[
                "/d",
                "/c",
                "echo probe-%ATOLLO_PROBE%-%USERNAME%-%CD%& exit /b 3",
            ]),
            &env(),
            &dir,
            "it",
            |_| {},
            |_, _| {},
            move |info| tx.send(info).unwrap(),
        )
        .unwrap();
    let ended = rx.recv_timeout(Duration::from_secs(20)).unwrap();
    assert_eq!(ended.code, Some(3));
    let out = wait_for(&sessions, info.id, "probe-");
    // USERNAME is not in the kit environment given here: cmd leaves it.
    assert!(
        out.contains(&format!("probe-kit-env-%USERNAME%-{}", dir.display())),
        "{out:?}"
    );
    // Windows' own ConPTY: the session says so first.
    assert!(out.contains("ConPTY"), "{out:?}");
    assert_eq!(sessions.running(), 0);
    sessions.close(info.id, Duration::ZERO).unwrap();
}

/// What is typed reaches the program; closing the session ends the program
/// and what it started.
#[test]
fn typed_input_reaches_the_program_and_close_ends_its_tree() {
    let tmp = TempDir::new("conpty-input");
    let sessions = Sessions::default();
    let info = sessions
        .open(
            &cmd(&["/d", "/q", "/k", "prompt $G"]),
            &env(),
            &tmp.0,
            "it",
            |_| {},
            |_, _| {},
            |_| {},
        )
        .unwrap();
    sessions.resize(info.id, 100, 40).unwrap();
    sessions
        .write(info.id, "echo got-%ATOLLO_PROBE%\r")
        .unwrap();
    let out = wait_for(&sessions, info.id, "got-kit-env");
    assert!(out.contains("got-kit-env"), "{out:?}");
    // A child of the shell: in the session's job.
    // No PATH in this environment: by its full path.
    sessions
        .write(
            info.id,
            "%SystemRoot%\\System32\\ping.exe -n 60 127.0.0.1\r",
        )
        .unwrap();
    let until = Instant::now() + Duration::from_secs(20);
    while sessions.pids().len() < 2 && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(50));
    }
    let pids = sessions.pids();
    assert!(
        pids.len() >= 2,
        "{pids:?} {:?}",
        sessions.backlog(info.id).unwrap().data
    );
    sessions.close(info.id, Duration::from_millis(500)).unwrap();
    let all = crate::platform::processes::all();
    for pid in pids {
        assert!(!all.iter().any(|p| p.pid == pid), "{pid} still runs");
    }
}
