use super::*;
use crate::kit::tests::TempDir;

/// Taken without waiting: one holder at a time, free again once dropped.
#[test]
fn try_take_has_one_holder() {
    let tmp = TempDir::new("platform-lock-try");
    let file = tmp.0.join("app").join("try.lock");
    let name = format!("Atollo-test-try-{}", std::process::id());
    let first = try_take(&name, &file).unwrap();
    assert!(first.is_some());
    assert!(try_take(&name, &file).unwrap().is_none());
    drop(first);
    assert!(try_take(&name, &file).unwrap().is_some());
}

/// Waited for: another thread waits for the holder, and gives up after its
/// timeout while the holder keeps it.
#[test]
fn wait_queues_the_other_threads() {
    let tmp = TempDir::new("platform-lock-wait");
    let file = tmp.0.join("app").join("wait.lock");
    let name = format!("Atollo-test-wait-{}", std::process::id());
    let first = wait(&name, &file, Duration::from_secs(5)).unwrap();
    assert!(first.is_some());
    let busy = std::thread::scope(|s| {
        s.spawn(|| wait(&name, &file, Duration::from_millis(200)).unwrap())
            .join()
            .unwrap()
            .is_none()
    });
    assert!(busy, "the other thread waited and gave up");
    drop(first);
    let free = std::thread::scope(|s| {
        s.spawn(|| wait(&name, &file, Duration::from_secs(5)).unwrap())
            .join()
            .unwrap()
            .is_some()
    });
    assert!(free);
}
