use super::*;

#[test]
fn records_round_trip() {
    for (pgid, start, add) in [
        (4242, 77, true),
        (4242, 0, false),
        (i32::MAX, u64::MAX, true),
    ] {
        assert_eq!(decode(&encode(pgid, start, add)), (pgid, start, add));
    }
    assert_eq!(encode(1, 2, true).len(), 16);
}

/// The table forgets emptied groups at every tick, so that a stale number
/// never reaches a group that is not the kit's (review of D-105).
#[test]
fn table_forgets_the_groups_that_emptied() {
    let mut groups = [(0i32, 0u64); 3];
    assert!(insert(&mut groups, 10, 1));
    assert!(insert(&mut groups, 20, 2));
    assert!(insert(&mut groups, 30, 3));
    // Full: a fourth group is not followed, and the caller knows.
    assert!(!insert(&mut groups, 40, 4));
    // A group already there only renews its start time.
    assert!(insert(&mut groups, 20, 22));
    assert!(groups.contains(&(20, 22)));

    prune(&mut groups, |pgid| pgid != 10);
    assert!(!groups.iter().any(|s| s.0 == 10));
    assert!(groups.contains(&(30, 3)));
    assert!(insert(&mut groups, 40, 4));

    forget(&mut groups, 30);
    assert!(!groups.iter().any(|s| s.0 == 30));
    // Empty slots are never asked about.
    prune(&mut groups, |pgid| {
        assert_ne!(pgid, 0);
        true
    });
    assert_eq!(groups.iter().filter(|s| s.0 != 0).count(), 2);
}

/// `pkill`/`killall` by name reach the guard too (it has the app's name):
/// it must survive `SIGTERM` and still end when its pipe closes.
#[test]
fn guard_survives_sigterm() {
    use std::time::Duration;
    let (reader, writer) = std::io::pipe().unwrap();
    let guard = fork_guard(reader).unwrap();
    std::thread::sleep(Duration::from_millis(200));
    let mut code = 0;
    // SAFETY: signals and waits for the guard, a child of this process.
    unsafe {
        assert_eq!(libc::kill(guard, libc::SIGTERM), 0);
        assert_eq!(libc::kill(guard, libc::SIGQUIT), 0);
        std::thread::sleep(Duration::from_millis(300));
        assert_eq!(
            libc::waitpid(guard, &mut code, libc::WNOHANG),
            0,
            "guard died"
        );
    }
    drop(writer);
    // SAFETY: as above.
    assert_eq!(unsafe { libc::waitpid(guard, &mut code, 0) }, guard);
    assert!(libc::WIFEXITED(code));
}

/// The guard ends the groups it watches when its pipe closes (the app is
/// gone), and leaves alone the ones it was told to forget.
#[test]
fn guard_ends_the_groups_when_the_app_is_gone() {
    use std::io::Write;
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::Command;
    use std::time::{Duration, Instant};

    let sleeper = || {
        Command::new("/bin/sleep")
            .arg("30")
            .process_group(0)
            .spawn()
            .unwrap()
    };
    let (mut watched, mut forgotten) = (sleeper(), sleeper());
    let id = |c: &std::process::Child| i32::try_from(c.id()).unwrap();
    let (reader, mut writer) = std::io::pipe().unwrap();
    let guard = fork_guard(reader).unwrap();
    let w = id(&watched);
    let f = id(&forgotten);
    writer
        .write_all(&encode(w, start_time(w).unwrap_or(0), true))
        .unwrap();
    writer
        .write_all(&encode(f, start_time(f).unwrap_or(0), true))
        .unwrap();
    writer.write_all(&encode(f, 0, false)).unwrap();
    // A group whose leader is another process now (another start time).
    writer
        .write_all(&encode(f, start_time(f).unwrap_or(0).wrapping_add(1), true))
        .unwrap();
    std::thread::sleep(Duration::from_millis(200));
    drop(writer);

    let begin = Instant::now();
    let status = loop {
        if let Some(status) = watched.try_wait().unwrap() {
            break status;
        }
        assert!(begin.elapsed() < Duration::from_secs(6), "still running");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert_eq!(status.signal(), Some(libc::SIGTERM));
    // SAFETY: waits for the guard, a child of this process.
    let mut code = 0;
    assert_eq!(unsafe { libc::waitpid(guard, &mut code, 0) }, guard);
    assert!(
        forgotten.try_wait().unwrap().is_none(),
        "forgotten group ended"
    );
    forgotten.kill().unwrap();
    forgotten.wait().unwrap();
}
