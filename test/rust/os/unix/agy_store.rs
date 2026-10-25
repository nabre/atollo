// The real Secret Service: Linux only.
#![cfg(not(target_os = "macos"))]

use super::*;
use crate::agents::keyring::tests::{item, kit, T};
use crate::agents::keyring::PERSIST_SESSION;
use crate::agents::keyring::{login_collection, NO_LOGIN_COLLECTION};

/// The real Secret Service of Linux (`--ignored`, TESTING.md). With a
/// `login` collection agy's sign-in is written there, as go-keyring reads
/// it, listed back and deleted; without one nothing is written anywhere.
/// Refuses to run where agy is signed in already.
#[test]
#[ignore = "needs a Secret Service (TESTING.md)"]
fn real_secret_service_login_collection() {
    use dbus_secret_service::{EncryptionType, SecretService};
    let (_tmp, kit) = kit();
    let store = System::of(&kit);
    assert!(store.list().unwrap().is_empty(), "agy signed in here");
    let blob = base64::engine::general_purpose::STANDARD.encode(b"kit-secret");
    let item = Item {
        blob: Some(blob.clone()),
        ..item(T, "")
    };
    let ss = SecretService::connect(EncryptionType::Plain).unwrap();
    let all = ss.get_all_collections().unwrap();
    let paths: Vec<&str> = all.iter().map(|c| &*c.path).collect();
    eprintln!("collections: {paths:?}");
    let Ok(at) = login_collection(paths.iter().copied()) else {
        let e = store.write(&item, PERSIST_SESSION).unwrap_err();
        assert_eq!(format!("{e:#}"), NO_LOGIN_COLLECTION);
        assert!(store.list().unwrap().is_empty(), "nothing written");
        return;
    };
    store.write(&item, PERSIST_SESSION).unwrap();
    let found = all[at]
        .search_items(std::collections::HashMap::from([
            ("service", "gemini"),
            ("username", "antigravity"),
        ]))
        .unwrap();
    assert_eq!(found.len(), 1, "one item, in login");
    assert_eq!(found[0].get_secret().unwrap(), b"kit-secret");
    let listed = store.list().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].blob.as_deref(), Some(blob.as_str()));
    store.delete(T);
    assert!(store.list().unwrap().is_empty());
}
