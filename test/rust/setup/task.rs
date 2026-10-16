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
        ["gh", "pwsh", "vscodium-server"],
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
