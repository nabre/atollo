use super::*;

fn proc(pid: u32, path: &str) -> Proc {
    let path = PathBuf::from(path);
    Proc {
        pid,
        parent: 1,
        name: path.file_name().unwrap().to_string_lossy().into_owned(),
        path,
    }
}

/// The kit's daemon is the one whose pid the kit's state holds, running
/// the kit's `sbx`; any other `sbx`, or the pipe without it, is another's.
#[test]
fn the_daemon_is_the_kits_only_when_its_pid_runs_the_kits_sbx() {
    let kit_sbx = Path::new("/kit/data/apps/x/sandboxes/bin/sbx.exe");
    let mine = proc(10, "/kit/data/apps/x/sandboxes/bin/sbx.exe");
    let pcs = proc(20, "/Users/me/AppData/Local/DockerSandboxes/bin/sbx.exe");
    assert_eq!(
        which(Some(10), std::slice::from_ref(&mine), kit_sbx, true),
        Daemon::Kit
    );
    // The kit's pid, gone: the pipe is another's daemon.
    assert_eq!(which(Some(10), &[], kit_sbx, true), Daemon::Other);
    // The kit's pid, now another program.
    assert_eq!(
        which(Some(20), std::slice::from_ref(&pcs), kit_sbx, false),
        Daemon::Other
    );
    // Nothing runs: the kit's `sbx` starts one.
    assert_eq!(which(None, &[], kit_sbx, false), Daemon::None);
    // Only the kit's own commands run (a session in the terminal).
    assert_eq!(which(None, &[mine], kit_sbx, false), Daemon::None);
    // The PC's `sbx` runs, no pid of the kit (macOS, Linux: no pipe).
    assert_eq!(which(None, &[pcs], kit_sbx, false), Daemon::Other);
}

#[test]
fn a_running_sandbox_is_seen_in_the_list() {
    let stopped = r#"[{"name":"atollo-p-a","status":"stopped"}]"#;
    let running =
        r#"[{"name":"atollo-p-a","status":"stopped"},{"name":"atollo-t-b","status":"running"}]"#;
    assert!(!crate::isolation::sandboxes_running("[]"));
    assert!(!crate::isolation::sandboxes_running(stopped));
    assert!(crate::isolation::sandboxes_running(running));
    // An answer that cannot be read: as if one ran (no restart).
    assert!(crate::isolation::sandboxes_running("not json"));
}

/// The daemon's state is found in the kit's home, and its pid read.
#[test]
fn the_kits_daemon_state_is_in_its_home() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-daemon");
    assert_eq!(state_dir(&kit), None);
    let dir = kit
        .home()
        .join("AppData/Local/DockerSandboxes/sandboxes/state/sandboxd");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("sandboxd.pid"), "9412\n").unwrap();
    assert_eq!(state_dir(&kit), Some(dir));
    assert_eq!(kit_pid(&kit), Some(9412));
    assert_eq!(ENV[0], ("LLMMAN_RUNTIME", "path"));
    assert_eq!(SETTINGS[0], ("ssh.agentForwardingEnabled", "false"));
}
