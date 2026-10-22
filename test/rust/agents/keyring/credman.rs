use super::*;
use crate::agents::keyring::files::{files, read, save_file};
use crate::agents::keyring::{restore, save};
use crate::test_support::prepared_kit;
use std::fs;
use std::time::Duration;
use windows::Win32::Security::Credentials::CRED_PERSIST_SESSION;

#[cfg(windows)]
#[test]
fn dpapi_roundtrip() {
    let blob = base64::engine::general_purpose::STANDARD.encode(b"secret of the PC");
    let sealed = protect(&blob).unwrap();
    assert!(sealed.starts_with(DPAPI) && !sealed.contains(&blob));
    assert_eq!(unprotect(&sealed).unwrap(), blob);
    assert_eq!(unprotect(&blob).unwrap(), blob);
}

/// The real Credential Manager, with a test-only name: restore puts the
/// kit's login in Windows (session only), save takes it back.
#[cfg(windows)]
#[test]
fn restore_and_save_round_trip() {
    let (_tmp, kit) = prepared_kit("keyring");
    let target = format!("antigravity-atollo-test-{}:user", std::process::id());
    let f = files(&kit);
    let blob = base64::engine::general_purpose::STANDARD.encode(b"token-1");
    save_file(
        &f.store,
        &[Item {
            target: target.clone(),
            user: Some("u".into()),
            blob: Some(blob),
            written: None,
            persist: None,
            kit: None,
        }],
    )
    .unwrap();

    restore(&kit).unwrap();
    let in_windows = list()
        .into_iter()
        .find(|c| c.target == target)
        .expect("restored");
    assert_eq!(
        in_windows.persist,
        Some(CRED_PERSIST_SESSION.0),
        "never on disk"
    );
    assert!(f.seen.exists());

    // agy refreshes the token meanwhile.
    std::thread::sleep(Duration::from_millis(50));
    let new_blob = base64::engine::general_purpose::STANDARD.encode(b"token-2");
    write(
        &Item {
            target: target.clone(),
            user: Some("u".into()),
            blob: Some(new_blob.clone()),
            written: None,
            persist: None,
            kit: None,
        },
        CRED_PERSIST_SESSION.0,
    )
    .unwrap();
    let _ = fs::remove_file(&f.starting);
    save(&kit).unwrap();
    assert!(
        list().iter().all(|c| c.target != target),
        "gone from Windows"
    );
    assert!(!f.seen.exists());
    let stored = read(&f.store);
    assert_eq!(stored.len(), 1);
    assert_eq!(
        stored[0].blob.as_deref(),
        Some(new_blob.as_str()),
        "new token kept in the kit"
    );
    delete(&target);
}
