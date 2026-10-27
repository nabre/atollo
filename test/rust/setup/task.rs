use super::*;
use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;
use std::fs;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn missing_required_lists_only_installable_ones() {
    let (_tmp, kit) = prepared_kit("task-missing");
    let missing = missing_required(&kit);
    // ConPTY is the console of Windows only.
    let mut want = vec!["gh", "pwsh", "vscodium-server", "conpty", "sandboxes"];
    if !kit.target().is_windows() {
        want.retain(|id| *id != "conpty");
    }
    assert_eq!(missing, want, "git only when needed (D-81)");
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

/// D-177: ConPTY not needed on macOS and Linux is never missing there,
/// on every target whatever the host; on Windows it is required.
#[test]
fn conpty_is_missing_on_windows_only() {
    use crate::kit::Target;
    let tmp = TempDir::new("task-conpty");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let missing = missing_required(&kit);
        assert_eq!(
            missing.contains(&"conpty".to_owned()),
            target.is_windows(),
            "{target}: {missing:?}"
        );
        assert!(missing.contains(&"gh".to_owned()), "{target}");
    }
}

/// Docker Sandboxes is required, but the editor and the consoles open
/// without it (the notice says the VM is off), and a system without its
/// build never tries to install it (D-167).
#[test]
fn sandboxes_never_block_and_skip_systems_without_a_build() {
    let (_tmp, kit) = prepared_kit("task-sandboxes");
    assert!(missing_required(&kit).contains(&"sandboxes".to_owned()));
    assert!(!missing_to_open(&kit).contains(&"sandboxes".to_owned()));

    use crate::kit::{Arch, Os, Target};
    for target in [
        Target::new(Os::Win32, Arch::Arm64),
        Target::new(Os::Darwin, Arch::X64),
    ] {
        let other = kit.for_target(target);
        assert!(
            !missing_required(&other).contains(&"sandboxes".to_owned()),
            "{target}"
        );
    }
}

/// The editor at another version than this release's is installed again
/// at start, but it does not block the editor and the consoles.
#[test]
fn editor_of_another_release_is_updated_at_start() {
    let (_tmp, kit) = prepared_kit("task-pinned");
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
    let (_tmp, kit) = prepared_kit("task-panic");
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
    let (_tmp, kit) = prepared_kit("task-run");
    let catalog = Catalog::get();
    // Those of this system only: ConPTY is a part of the app on Windows.
    let required: Vec<_> = catalog
        .all()
        .iter()
        .filter(|c| catalog.is_locked(&c.id) && c.is_available_on(kit.target()))
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

/// A kit whose required components (of its system) are all installed,
/// and the options of a run that has nothing to download.
fn idle_kit(name: &str) -> (TempDir, Kit, Options) {
    let (tmp, kit) = prepared_kit(name);
    let catalog = Catalog::get();
    for c in catalog
        .all()
        .iter()
        .filter(|c| catalog.is_locked(&c.id) && c.is_available_on(kit.target()))
    {
        for key in c.key_paths(&kit) {
            fs::create_dir_all(key.parent().unwrap()).unwrap();
            fs::write(&key, b"").unwrap();
        }
        c.save_version(&kit, c.version.as_deref().unwrap_or("1.0"))
            .unwrap();
    }
    let opts = Options {
        skip: catalog.all().iter().map(|c| c.id.clone()).collect(),
        ..Options::default()
    };
    (tmp, kit, opts)
}

fn quiet() -> Notify {
    Box::new(|_| ())
}

/// LEVELS.md §8.6: a run asked while another one runs waits in the queue
/// with its job (not refused); one cancelled there never runs; when the
/// run going on ends, the next of the queue starts in the same thread and
/// the snapshot stays `running` until the queue is empty.
#[test]
fn runs_asked_meanwhile_wait_in_the_queue() {
    let (_tmp, kit, opts) = idle_kit("task-queue");
    let task = Arc::new(SetupTask::new(
        kit,
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    // A run going on.
    task.lock().running = true;
    let node = Options {
        only: vec!["node".into()],
        ..opts.clone()
    };
    let a = task.start(node, false, quiet()).unwrap();
    let b = task.start(opts.clone(), false, quiet()).unwrap();
    let jobs = Arc::clone(task.jobs());
    assert_eq!(jobs.get(&a).unwrap().state, JobState::Queued);
    assert_eq!(jobs.get(&a).unwrap().items, ["node"]);
    let pending: Vec<String> = task.snapshot().pending.into_iter().map(|p| p.job).collect();
    assert_eq!(pending, [a.clone(), b.clone()]);

    assert!(task.cancel_job(&a, &quiet()));
    assert_eq!(jobs.get(&a).unwrap().state, JobState::Cancelled);
    assert_eq!(task.snapshot().pending.len(), 1);
    assert!(!task.cancel_job(&a, &quiet()), "already cancelled");

    // The run going on ends: `b` runs, then the queue is empty.
    let first = jobs.add(
        JobKind::Install,
        "first".into(),
        None,
        Vec::new(),
        JobState::Running,
        false,
    );
    task.chain(Pending {
        job: first.clone(),
        opts,
        automatic: false,
        notify: quiet(),
    });
    let last = task.snapshot();
    assert!(!last.running);
    assert!(last.pending.is_empty());
    assert_eq!(jobs.get(&first).unwrap().state, JobState::Done);
    assert_eq!(jobs.get(&b).unwrap().state, JobState::Done);
    assert_eq!(jobs.get(&a).unwrap().state, JobState::Cancelled);
    assert!(task.is_over(&b));
}

/// The end of a run is told (in the app: the kit's environment built
/// again, `ipc::setup::setup_notify`) without the snapshot's lock: the
/// snapshot answers meanwhile and says `running` until the telling
/// returns, so who waits for the end reads the new environment; a run
/// asked meanwhile is queued and runs after.
#[test]
fn the_end_is_told_without_holding_the_snapshot() {
    let (_tmp, kit, opts) = idle_kit("task-end");
    let task = Arc::new(SetupTask::new(
        kit,
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    let (ending_tx, ending_rx) = mpsc::channel();
    let (go_tx, go_rx) = mpsc::channel::<()>();
    let notify: Notify = Box::new(move |s| {
        if !s.running {
            let _ = ending_tx.send(());
            let _ = go_rx.recv_timeout(Duration::from_secs(10));
        }
    });
    task.start(opts.clone(), false, notify).unwrap();
    ending_rx.recv_timeout(Duration::from_secs(30)).unwrap();

    let (tx, rx) = mpsc::channel();
    let reader = Arc::clone(&task);
    std::thread::spawn(move || {
        let _ = tx.send(reader.snapshot().running);
    });
    let running = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("the snapshot is not held while the end is told");
    assert!(running, "running until the end is told");
    let next = task.start(opts, false, quiet()).unwrap();
    assert_eq!(task.jobs().get(&next).unwrap().state, JobState::Queued);

    go_tx.send(()).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(30);
    while task.snapshot().running {
        assert!(std::time::Instant::now() < deadline, "still running");
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(task.jobs().get(&next).unwrap().state, JobState::Done);
}

/// Cancelling the job that runs stops that run only (the flag the
/// downloads read), never a job of another task.
#[test]
fn cancelling_the_running_job_stops_its_run() {
    let (_tmp, kit, _) = idle_kit("task-cancel");
    let task = Arc::new(SetupTask::new(
        kit,
        KitEnv::default(),
        Settings::default(),
        None,
    ));
    {
        let mut state = task.lock();
        state.running = true;
        state.job = Some("job-running".into());
    }
    assert!(!task.cancel_job("job-other", &quiet()));
    assert!(!task.cancel.load(Ordering::Relaxed));
    assert!(task.cancel_job("job-running", &quiet()));
    assert!(task.cancel.load(Ordering::Relaxed));
    assert!(task.snapshot().cancelling);
}

/// The installer's events become the job's: the step `index` of `total`
/// with its name, the bytes of the download, a failure as the step's
/// error and the run's.
#[test]
fn the_runs_events_are_its_jobs_progress() {
    let (_tmp, kit, _) = idle_kit("task-progress");
    let task = SetupTask::new(kit, KitEnv::default(), Settings::default(), None);
    let job = task.jobs().add(
        JobKind::Update,
        "Aggiorno Node.js".into(),
        None,
        vec!["node".into()],
        JobState::Running,
        false,
    );
    let notify = quiet();
    let mut sink = TaskSink {
        task: &task,
        notify: &notify,
        job: &job,
        lang: "it",
    };
    sink.emit(Event::Step {
        index: 1,
        total: 2,
        id: "node".into(),
        name: "Node.js".into(),
    });
    sink.emit(Event::Bytes {
        id: "node".into(),
        done: 5 << 20,
        total: Some(20 << 20),
    });
    let now = task.jobs().get(&job).unwrap();
    let step = now.step.unwrap();
    assert_eq!(
        (step.index, step.total, step.label.as_str()),
        (1, 2, "Node.js")
    );
    assert_eq!(
        now.bytes,
        Some(crate::common::jobs::JobBytes {
            done: 5 << 20,
            total: Some(20 << 20)
        })
    );
    sink.emit(Event::Done {
        id: "node".into(),
        outcome: Outcome::Failed {
            message: "checksum".into(),
            by_hand: None,
        },
    });
    let now = task.jobs().get(&job).unwrap();
    assert_eq!(now.steps[0].state, JobState::Error);
    assert_eq!(now.steps[0].error.as_deref(), Some("checksum"));
    assert_eq!(task.snapshot().steps[0].done, Some(5 << 20));
}

/// The title of a run says what it does, in the window's language.
#[test]
fn a_runs_title_says_what_it_does() {
    let (_tmp, kit, _) = idle_kit("task-title");
    let of = |opts: Options| describe(&kit, &opts, false, "it");
    let (kind, title, items) = of(Options {
        only: vec!["node".into()],
        update_all: true,
        ..Options::default()
    });
    assert_eq!(
        (kind, title.as_str()),
        (JobKind::Update, "Aggiorno Node.js")
    );
    assert_eq!(items, ["node"]);
    let (kind, title, _) = of(Options {
        lines: vec![("node".into(), "22.11".into())],
        ..Options::default()
    });
    assert_eq!(
        (kind, title.as_str()),
        (JobKind::Install, "Scarico Node.js 22.11")
    );
    let (_, title, items) = of(Options::default());
    assert_eq!(title, "Applico la scelta degli strumenti");
    assert!(items.is_empty());
    if let Some(guest) = crate::isolation::vm::guest_target(kit.target()) {
        if guest != kit.target() {
            let (_, title, _) = of(Options {
                target: Some(guest),
                only: vec!["node".into()],
                ..Options::default()
            });
            assert_eq!(title, "Installo Node.js · VM");
        }
    }
}
