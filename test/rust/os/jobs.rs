use super::*;
use std::time::{Duration, Instant};

fn system32(exe: &str) -> String {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    format!(r"{root}\System32\{exe}")
}

/// The test runner itself must not join a kill-on-close job, so the test
/// assigns a child to a private job and closes it.
#[test]
fn spawn_in_runs_and_drop_kills() {
    let job = Job::new().unwrap();
    let mut child = spawn_in(
        Command::new(system32("ping.exe")).args(["-n", "60", "127.0.0.1"]),
        &job,
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    assert!(child.try_wait().unwrap().is_none(), "running after resume");
    drop(job);
    let start = Instant::now();
    while child.try_wait().unwrap().is_none() {
        assert!(start.elapsed() < Duration::from_secs(5), "still alive");
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn batch_files_in_odd_folders() {
    let root = std::env::temp_dir().join(format!("devkit-bat-{}", std::process::id()));
    for name in ["a b (x) & y", "50%", "a!b", "%SystemRoot%", "%OS% & !OS!"] {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let bat = dir.join("hello.cmd");
        std::fs::write(&bat, "@echo hi %1\r\n").unwrap();
        let env_vars = [
            ("SystemRoot", std::env::var("SystemRoot").unwrap()),
            ("OS", "Windows_NT".to_owned()),
            ("PATH", String::new()),
        ];
        let out = batch_command(&bat, &["there"], &KitEnv::default())
            .envs(env_vars)
            .output()
            .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&out.stdout).trim(),
            "hi there",
            "{name}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn closing_the_job_kills_its_processes() {
    let job = Job::new().unwrap();
    let mut child = Command::new(system32("ping.exe"))
        .args(["-n", "60", "127.0.0.1"])
        .creation_flags(CREATE_NO_WINDOW.0)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    job.assign(&child).unwrap();
    assert!(child.try_wait().unwrap().is_none(), "still running");
    drop(job);
    let start = Instant::now();
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "child survived the job"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[test]
fn kit_command_does_not_inherit_the_environment() {
    let env = KitEnv::default();
    let cmd = kit_command(system32("cmd.exe"), &env);
    let inherited: Vec<_> = cmd.get_envs().collect();
    assert!(inherited.is_empty());
    let out = kit_command(system32("cmd.exe"), &env)
        .args(["/d", "/c", "set"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_ascii_uppercase();
    assert!(!text.contains("USERPROFILE="), "{text}");
}
