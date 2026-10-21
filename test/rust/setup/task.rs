use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::fs;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn missing_required_lists_only_installable_ones() {
    let tmp = TempDir::new("task-missing");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let missing = missing_required(&kit);
    assert_eq!(
        missing,
        ["gh", "pwsh", "vscodium-server", "conpty"],
        "git only when needed (D-81)"
    );
    for id in ["node", "uv", "python"] {
        assert!(
            !missing.contains(&id.to_owned()),
            "{id}: installed when needed"
        );
    }

    let pwsh = Catalog::get().find("pwsh").unwrap().key_path(&kit);
    fs::create_dir_all(pwsh.parent().unwrap()).unwrap();
    fs::write(&pwsh, b"").unwrap();
    assert!(!missing_required(&kit).contains(&"pwsh".to_owned()));
}

/// The editor at another version than this release's is installed again
/// at start, but it does not block the editor and the consoles.
#[test]
fn editor_of_another_release_is_updated_at_start() {
    let tmp = TempDir::new("task-pinned");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let editor = Catalog::get().find("vscodium-server").unwrap();
    for key in editor.key_paths(&kit) {
        fs::create_dir_all(key.parent().unwrap()).unwrap();
        fs::write(&key, b"").unwrap();
    }
    editor.save_version(&kit, "1.0.0").unwrap();
    assert!(editor.is_installed(&kit) && editor.outdated(&kit));
    assert_eq!(outdated_app(&kit), [editor.id.as_str()]);
    assert!(
        !missing_required(&kit).contains(&editor.id),
        "the installed one keeps working"
    );
    editor
        .save_version(&kit, editor.version.as_deref().unwrap())
        .unwrap();
    assert!(outdated_app(&kit).is_empty());
}

/// D-92: Git marked when the kit moved is installed again at start, and
/// only while it is on disk.
#[test]
fn component_marked_after_a_move_is_reinstalled_at_start() {
    let tmp = TempDir::new("task-reinstall");
    let kit = Kit::at_target(tmp.0.clone(), crate::kit::Target::LINUX_X64);
    kit.prepare().unwrap();
    let git = Catalog::get().find("git").unwrap();
    git.mark_reinstall(&kit, "/old/kit").unwrap();
    assert!(git.needs_reinstall(&kit));
    assert!(
        outdated_app(&kit).is_empty(),
        "not installed: nothing to redo"
    );
    for key in git.key_paths(&kit) {
        fs::create_dir_all(key.parent().unwrap()).unwrap();
        fs::write(&key, b"").unwrap();
    }
    git.save_version(&kit, "2.51.0").unwrap();
    assert!(git.is_installed(&kit));
    assert_eq!(outdated_app(&kit), ["git"]);
    git.clear_reinstall(&kit);
    assert!(outdated_app(&kit).is_empty());
    // Removing it forgets the mark too.
    git.mark_reinstall(&kit, "/old/kit").unwrap();
    git.forget_version(&kit);
    assert!(!git.needs_reinstall(&kit));
}

/// A panic in the run's thread (here in its first event) ends the run with
/// an error: the next one is not refused as "already running".
#[test]
fn a_panic_in_the_run_does_not_leave_it_running() {
    let tmp = TempDir::new("task-panic");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    for c in catalog.all().iter().filter(|c| catalog.is_locked(&c.id)) {
        for key in c.key_paths(&kit) {
            fs::create_dir_all(key.parent().unwrap()).unwrap();
            fs::write(&key, b"").unwrap();
        }
        c.save_version(&kit, c.version.as_deref().unwrap_or("1.0"))
            .unwrap();
    }
    let task = Arc::new(SetupTask::new(
        kit,
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    let calls = std::sync::atomic::AtomicUsize::new(0);
    // The first call is the start's, in this thread; the second panics.
    let notify: Notify = Box::new(move |_| {
        if calls.fetch_add(1, Ordering::SeqCst) == 1 {
            panic!("installer bug");
        }
    });
    let opts = Options {
        skip: catalog.all().iter().map(|c| c.id.clone()).collect(),
        ..Options::default()
    };
    task.start(opts.clone(), false, notify).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while task.snapshot().running {
        assert!(std::time::Instant::now() < deadline, "still running");
        std::thread::sleep(Duration::from_millis(20));
    }
    let error = task.snapshot().error.unwrap_or_default();
    assert!(error.contains("installer bug"), "{error}");
    let (tx, rx) = mpsc::channel();
    let notify: Notify = Box::new(move |s| {
        let _ = tx.send(s.running);
    });
    assert!(task.start(opts, false, notify).is_ok());
    while rx.recv_timeout(Duration::from_secs(30)).unwrap() {}
}

/// No network: every required component is already installed.
#[test]
fn run_reports_steps_and_ends() {
    let tmp = TempDir::new("task-run");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let catalog = Catalog::get();
    let required: Vec<_> = catalog
        .all()
        .iter()
        .filter(|c| catalog.is_locked(&c.id))
        .collect();
    for c in &required {
        for key in c.key_paths(&kit) {
            fs::create_dir_all(key.parent().unwrap()).unwrap();
            fs::write(&key, b"").unwrap();
        }
        c.save_version(&kit, c.version.as_deref().unwrap_or("1.0"))
            .unwrap();
    }

    let task = Arc::new(SetupTask::new(
        kit,
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    let (tx, rx) = mpsc::channel();
    let notify: Notify = Box::new(move |s| {
        let _ = tx.send(s.clone());
    });
    let opts = Options {
        skip: catalog.all().iter().map(|c| c.id.clone()).collect(),
        ..Options::default()
    };
    task.start(opts, false, notify).unwrap();
    let last = loop {
        let s = rx.recv_timeout(Duration::from_secs(30)).unwrap();
        if !s.running {
            break s;
        }
    };
    assert_eq!(last.error, None);
    assert_eq!(last.steps.len(), required.len());
    assert!(last.steps.iter().all(|s| s.outcome
        == Some(Outcome::UpToDate {
            version: Some(
                catalog
                    .find(&s.id)
                    .and_then(|c| c.version.clone())
                    .unwrap_or_else(|| "1.0".into())
            )
        })));
}
