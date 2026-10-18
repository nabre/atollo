use super::*;
use crate::kit::{tests::TempDir, Arch};
use std::cell::{Cell, RefCell};

#[cfg(windows)]
#[test]
fn dpapi_roundtrip() {
    let blob = base64::engine::general_purpose::STANDARD.encode(b"secret of the PC");
    let sealed = protect(&blob).unwrap();
    assert!(sealed.starts_with(DPAPI) && !sealed.contains(&blob));
    assert_eq!(unprotect(&sealed).unwrap(), blob);
    assert_eq!(unprotect(&blob).unwrap(), blob);
}

#[test]
fn names() {
    assert!(is_antigravity("gemini:antigravity"));
    assert!(is_antigravity("jetski-auth:user"));
    assert!(is_antigravity("Antigravity:x"));
    assert!(!is_antigravity("git:https://github.com"));
    assert!(!is_antigravity("myantigravity:x"));
    assert!(!is_antigravity("antigravity"), "no colon: not go-keyring");
}

/// The pairs looked up on macOS and Linux are the ones agy writes on
/// Windows, so keyring.json is the same file on every system.
#[test]
fn known_names_match_windows_targets() {
    assert!(KNOWN.contains(&("gemini", "antigravity")));
    for (service, user) in KNOWN {
        let target = format!("{service}:{user}");
        assert!(is_antigravity(&target));
        assert_eq!(split_target(&target), Some((*service, *user)));
    }
    assert_eq!(split_target("gemini"), None);
    assert_eq!(split_target(":antigravity"), None);
    assert_eq!(split_target("gemini:"), None);
}

/// go-keyring on macOS stores `go-keyring-base64:<base64>` and reads also
/// the hex form and a value without a prefix.
#[test]
fn keychain_values_as_go_keyring() {
    let raw = br#"{"token":"ya29.a0","refresh":"1//x"}"#;
    let stored = mac_encode(raw);
    assert!(stored.starts_with("go-keyring-base64:"));
    assert_eq!(mac_decode(&stored).unwrap(), raw);
    assert_eq!(
        mac_decode(&format!("{stored}\n")).unwrap(),
        raw,
        "security -w ends with a newline"
    );
    assert_eq!(mac_decode("go-keyring-encoded:6869").unwrap(), b"hi");
    assert!(mac_decode("go-keyring-encoded:686").is_err());
    assert_eq!(mac_decode("plain").unwrap(), b"plain");
    assert!(mac_decode("go-keyring-base64:@@").is_err());
}

/// The secret goes through stdin, never among the arguments; names that
/// `security -i` would split or interpret are refused.
#[test]
fn keychain_add_command() {
    let line = mac_add_command("gemini", "antigravity", b"s3cret").unwrap();
    assert_eq!(
        line,
        format!(
            "add-generic-password -U -s gemini -a antigravity -w {}\n",
            mac_encode(b"s3cret")
        )
    );
    assert!(mac_add_command("gem ini", "antigravity", b"x").is_err());
    assert!(mac_add_command("gemini", "a;rm", b"x").is_err());
    assert!(mac_add_command("gemini", "a\"b", b"x").is_err());
    assert!(mac_add_command("", "antigravity", b"x").is_err());
    // Any secret is fine: base64 has only plain characters.
    assert!(mac_add_command("gemini", "antigravity", "è \n'\"$".as_bytes()).is_ok());
}

#[test]
fn unix_items_and_fingerprints() {
    let a = unix_item("gemini", "antigravity", b"one");
    assert_eq!(a.target, "gemini:antigravity");
    assert_eq!(a.user.as_deref(), Some("antigravity"));
    assert_eq!(
        a.blob.as_deref(),
        Some(
            base64::engine::general_purpose::STANDARD
                .encode(b"one")
                .as_str()
        )
    );
    let b = unix_item("gemini", "antigravity", b"two");
    assert_ne!(a.written, b.written, "a changed secret looks rewritten");
    assert_eq!(
        a.written,
        unix_item("gemini", "antigravity", b"one").written
    );
    assert!(
        !format!("{a:?}").contains(a.blob.as_deref().unwrap()),
        "no secret in Debug"
    );
}

#[test]
fn sealing_round_trip() {
    let key = [7u8; 32];
    let nonce = [3u8; NONCE];
    let secret = b"secret of this computer, longer than one block of 32 bytes";
    let blob = base64::engine::general_purpose::STANDARD.encode(secret);
    let sealed = seal(&key, &nonce, &blob).unwrap();
    assert!(sealed.starts_with(SEALED));
    assert!(!sealed.contains(&blob));
    assert_eq!(open_sealed(&key, &sealed).unwrap(), blob);
    // Another key, an altered byte, a cut value: refused.
    assert!(open_sealed(&[8u8; 32], &sealed).is_err());
    let mut raw = base64::engine::general_purpose::STANDARD
        .decode(&sealed[SEALED.len()..])
        .unwrap();
    raw[NONCE] ^= 1;
    let altered = format!(
        "{SEALED}{}",
        base64::engine::general_purpose::STANDARD.encode(&raw)
    );
    assert!(open_sealed(&key, &altered).is_err());
    assert!(open_sealed(&key, "kc1:AAAA").is_err());
    assert!(open_sealed(&key, &blob).is_err(), "not sealed");
    // Another nonce, another value for the same secret.
    assert_ne!(seal(&key, &[4u8; NONCE], &blob).unwrap(), sealed);
    // Empty secret.
    let empty = seal(&key, &nonce, "").unwrap();
    assert_eq!(open_sealed(&key, &empty).unwrap(), "");
}

#[test]
fn files_of_a_run_per_computer() {
    let dir = Path::new("kit").join("data").join("antigravity");
    let win = files_in(&dir, None);
    assert_eq!(win.store, dir.join("keyring.json"));
    assert_eq!(win.seen, dir.join("keyring-before.json"));
    assert_eq!(win.starting, dir.join("keyring-starting"));
    let mac = files_in(&dir, Some("MacBook-Pro"));
    assert_eq!(mac.store, dir.join("keyring.json"), "the login is shared");
    assert_eq!(mac.seen, dir.join("keyring-before-MacBook-Pro.json"));
    assert_eq!(mac.starting, dir.join("keyring-starting-MacBook-Pro"));
    assert_eq!(host_tag("my mac.local"), "my-mac-local");
    assert_eq!(host_tag("../x"), "---x");
    assert_eq!(host_tag("  "), "host");
    assert_eq!(host_tag(&"a".repeat(300)).len(), 64);
}

/// A store in memory: `written` is a counter, as a last-write time.
#[derive(Default)]
struct Fake {
    items: RefCell<Vec<Item>>,
    clock: Cell<i64>,
    down: bool,
    forgot: Cell<u32>,
}

impl Fake {
    fn with(items: &[(&str, &str)]) -> Fake {
        let fake = Fake::default();
        for (target, secret) in items {
            fake.agy_writes(target, secret);
        }
        fake
    }

    /// What agy does: writes its login.
    fn agy_writes(&self, target: &str, secret: &str) {
        self.write(&item(target, secret), PERSIST_SESSION).unwrap();
    }

    fn secret_of(&self, target: &str) -> Option<String> {
        self.items
            .borrow()
            .iter()
            .find(|i| i.target == target)
            .and_then(|i| i.blob.clone())
    }
}

impl Store for Fake {
    fn list(&self) -> Result<Vec<Item>> {
        if self.down {
            anyhow::bail!("no Secret Service");
        }
        Ok(self.items.borrow().clone())
    }
    fn write(&self, item: &Item, persist: u32) -> Result<()> {
        if self.down {
            anyhow::bail!("no Secret Service");
        }
        self.clock.set(self.clock.get() + 1);
        let mut items = self.items.borrow_mut();
        items.retain(|i| i.target != item.target);
        items.push(Item {
            written: Some(self.clock.get()),
            persist: Some(persist),
            kit: None,
            ..item.clone()
        });
        Ok(())
    }
    fn delete(&self, target: &str) {
        self.items.borrow_mut().retain(|i| i.target != target);
    }
    fn protect(&self, blob: &str) -> Result<String> {
        Ok(format!("sealed:{blob}"))
    }
    fn unprotect(&self, blob: &str) -> Result<String> {
        Ok(blob.strip_prefix("sealed:").unwrap_or(blob).to_owned())
    }
    fn forget_key(&self) {
        self.forgot.set(self.forgot.get() + 1);
    }
}

const T: &str = "gemini:antigravity";

fn item(target: &str, secret: &str) -> Item {
    Item {
        target: target.into(),
        user: Some("antigravity".into()),
        blob: Some(secret.into()),
        written: None,
        persist: None,
        kit: None,
    }
}

fn kit() -> (TempDir, Kit) {
    let tmp = TempDir::new("keyring-fake");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    (tmp, kit)
}

/// agy ended: its "starting" mark is gone (as `save_after_run`).
fn end_run(kit: &Kit, store: &Fake) {
    save_after_run_with(kit, store).unwrap();
}

#[test]
fn kit_login_in_and_out_of_the_store() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit-1")]).unwrap();
    let store = Fake::default();

    restore_with(&kit, &store).unwrap();
    assert_eq!(
        store.secret_of(T).as_deref(),
        Some("kit-1"),
        "written before agy"
    );
    assert!(pending(&kit) && f.starting.exists());

    store.agy_writes(T, "kit-2");
    end_run(&kit, &store);
    assert_eq!(store.secret_of(T), None, "deleted after agy");
    assert!(!pending(&kit));
    assert_eq!(store.forgot.get(), 1, "the sealing key goes with the file");
    let kept = read(&f.store);
    assert_eq!(kept.len(), 1);
    assert_eq!(
        kept[0].blob.as_deref(),
        Some("kit-2"),
        "refreshed token kept"
    );
    assert!(has_login(&kit));
}

#[test]
fn computer_login_left_as_it_was() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake::with(&[(T, "own")]);

    restore_with(&kit, &store).unwrap();
    assert_eq!(
        store.secret_of(T).as_deref(),
        Some("own"),
        "not overwritten"
    );
    let before = read(&f.seen);
    assert_eq!(before[0].kit, Some(false));
    assert_eq!(
        before[0].blob.as_deref(),
        Some("sealed:own"),
        "sealed in the file"
    );

    end_run(&kit, &store);
    assert_eq!(store.secret_of(T).as_deref(), Some("own"));
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit"));
}

#[test]
fn computer_login_put_back_after_agy_replaced_it() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    let store = Fake::with(&[(T, "own")]);
    restore_with(&kit, &store).unwrap();
    // A new sign-in inside the kit's agy replaces the computer's.
    store.agy_writes(T, "new");
    end_run(&kit, &store);
    assert_eq!(
        store.secret_of(T).as_deref(),
        Some("own"),
        "put back, unsealed"
    );
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("new"));
}

#[test]
fn signed_out_inside_agy() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake::default();
    restore_with(&kit, &store).unwrap();
    store.delete(T);
    end_run(&kit, &store);
    assert!(read(&f.store).is_empty());
    assert!(!has_login(&kit));
}

/// A crash (agy or the app killed): the next start of the app saves and
/// deletes what was left in the store, once the "starting" mark is old.
#[test]
fn leftover_deleted_at_the_next_start() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake::default();
    restore_with(&kit, &store).unwrap();
    // Within 20 s of a start nothing is touched (agy may still be starting).
    save_with(&kit, &store).unwrap();
    assert!(pending(&kit));
    assert_eq!(store.secret_of(T).as_deref(), Some("kit"));
    // The mark of the crashed run is old.
    let old = std::time::SystemTime::now() - Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&f.starting)
        .unwrap()
        .set_modified(old)
        .unwrap();
    save_with(&kit, &store).unwrap();
    assert!(!pending(&kit) && !f.starting.exists());
    assert_eq!(store.secret_of(T), None);
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit"));
}

/// Linux without a Secret Service: nothing moved, agy starts anyway.
#[test]
fn no_store_moves_nothing() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake {
        down: true,
        ..Fake::default()
    };
    restore_with(&kit, &store).unwrap();
    assert!(!pending(&kit));
    save_with(&kit, &store).unwrap();
    end_run(&kit, &store);
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit"));
}

/// The kit's copy is owner-only on macOS and Linux.
#[cfg(unix)]
#[test]
fn kit_copy_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let mode = fs::metadata(&f.store).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
}

/// The real Credential Manager, with a test-only name: restore puts the
/// kit's login in Windows (session only), save takes it back.
#[cfg(windows)]
#[test]
fn restore_and_save_round_trip() {
    let tmp = TempDir::new("keyring");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    let target = format!("antigravity-devkit-test-{}:user", std::process::id());
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
