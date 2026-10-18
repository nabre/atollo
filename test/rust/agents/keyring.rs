use super::*;
use crate::kit::{tests::TempDir, Arch, Os};
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
    down: Down,
    /// No sealing key (store locked, access denied).
    no_key: bool,
    forgot: Cell<u32>,
    /// Locked meanwhile (the Keychain locked after N minutes, an unlock
    /// prompt cancelled): `list` fails from now on.
    locked_now: Cell<bool>,
    /// A target that cannot be written.
    refuse: Option<&'static str>,
    /// The only targets `list` returns, as the stores of macOS and Linux
    /// ([`KNOWN`]); `None`: every one, as Windows.
    only: Option<&'static [&'static str]>,
    /// The sealing key is not this one (a "before" file of another
    /// computer): sealed secrets cannot be opened.
    other_key: bool,
}

#[derive(Default, Clone, Copy, PartialEq)]
enum Down {
    #[default]
    Up,
    /// No Secret Service at all.
    Missing,
    /// A store that cannot be used now.
    Locked,
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
        match self.down {
            Down::Up => {}
            Down::Missing => return Err(anyhow::Error::new(NoStore("not provided".into()))),
            Down::Locked => anyhow::bail!("collection is locked"),
        }
        if self.locked_now.get() {
            anyhow::bail!("Keychain: security exited with 36");
        }
        Ok(self
            .items
            .borrow()
            .iter()
            .filter(|i| self.listable(&i.target))
            .cloned()
            .collect())
    }
    fn write(&self, item: &Item, persist: u32) -> Result<()> {
        if self.down != Down::Up {
            anyhow::bail!("no Secret Service");
        }
        if self.refuse == Some(item.target.as_str()) {
            anyhow::bail!("{}: not written", item.target);
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
        if self.no_key {
            anyhow::bail!("sealing key: access denied");
        }
        Ok(format!("sealed:{blob}"))
    }
    fn unprotect(&self, blob: &str) -> Result<String> {
        match blob.strip_prefix("sealed:") {
            Some(_) if self.other_key => anyhow::bail!("sealed secret: wrong key or altered"),
            Some(rest) => Ok(rest.to_owned()),
            None => Ok(blob.to_owned()),
        }
    }
    fn forget_key(&self) {
        self.forgot.set(self.forgot.get() + 1);
    }
    fn listable(&self, target: &str) -> bool {
        self.only.is_none_or(|only| only.contains(&target))
    }
    fn undo_partial(&self) -> bool {
        true
    }
}

/// The store of macOS and Linux: only the pairs in [`KNOWN`] are listed.
const UNIX_ONLY: &[&str] = &["gemini:antigravity"];

/// Every Windows-only name in keyring.json (`jetski…`) stays there and is
/// never written to a store that cannot list it back.
#[test]
fn names_the_store_cannot_list_stay_in_the_kit() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit"), item("jetski:other", "win")]).unwrap();
    let store = Fake {
        only: Some(UNIX_ONLY),
        ..Fake::default()
    };
    restore_with(&kit, &store).unwrap();
    assert_eq!(store.secret_of(T).as_deref(), Some("kit"));
    assert_eq!(store.secret_of("jetski:other"), None, "never moved in");
    store.agy_writes(T, "kit-2");
    end_run(&kit, &store);
    assert!(store.items.borrow().is_empty(), "nothing left in the store");
    let kept = read(&f.store);
    assert_eq!(kept.len(), 2, "both kept: {kept:?}");
    let blob = |t: &str| {
        kept.iter()
            .find(|i| i.target == t)
            .and_then(|i| i.blob.clone())
    };
    assert_eq!(blob(T).as_deref(), Some("kit-2"));
    assert_eq!(blob("jetski:other").as_deref(), Some("win"));
    // Signed out inside agy: only the name agy uses here is forgotten.
    restore_with(&kit, &store).unwrap();
    store.delete(T);
    end_run(&kit, &store);
    let kept = read(&f.store);
    assert_eq!(kept.len(), 1);
    assert_eq!(kept[0].target, "jetski:other");
}

/// The real stores of macOS and Linux list and write only [`KNOWN`].
#[test]
fn known_targets_only() {
    assert!(known_target("gemini:antigravity"));
    assert!(!known_target("jetski:other"));
    assert!(!known_target("antigravity:x"));
    assert!(!known_target("gemini"));
    assert!(!known_target("gemini:antigravity:x"));
}

/// A restore that fails after writing some of the kit's credentials
/// deletes them: no "before" file would tell they are the kit's.
#[test]
fn failed_restore_takes_back_what_it_wrote() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit"), item("jetski:b", "kit-b")]).unwrap();
    let store = Fake {
        refuse: Some("jetski:b"),
        ..Fake::default()
    };
    assert!(restore_with(&kit, &store).is_err());
    assert!(!f.seen.exists() && !f.starting.exists());
    assert!(store.items.borrow().is_empty(), "the first one taken back");
    assert_eq!(read(&f.store).len(), 2, "the kit's copy untouched");
}

/// A "before" file whose sealed secrets cannot be opened here (another
/// computer's, or its key gone): the save changes nothing, and never copies
/// this computer's login into the kit.
#[test]
fn before_file_sealed_elsewhere_changes_nothing() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let mut other = item(T, "sealed:theirs");
    other.kit = Some(false);
    other.written = Some(-1);
    save_file(&f.seen, &[other]).unwrap();
    let kit_file = fs::read(&f.store).unwrap();
    let store = Fake {
        other_key: true,
        ..Fake::with(&[(T, "mine")])
    };
    let e = save_with(&kit, &store).unwrap_err();
    assert!(format!("{e:#}").contains("cannot be opened"), "{e:#}");
    assert_eq!(
        fs::read(&f.store).unwrap(),
        kit_file,
        "kit's copy unchanged"
    );
    assert_eq!(
        store.secret_of(T).as_deref(),
        Some("mine"),
        "store untouched"
    );
    assert!(f.seen.exists(), "kept, not dropped");
}

/// The tag of a run's files: the machine and the account, never the host
/// name (which changes with the network and repeats between Macs).
#[test]
fn run_files_tagged_by_machine_and_account() {
    let mac = "4C4C4544-0042-3510-8051-B4C04F4D4C31";
    let a = machine_tag(mac, 501);
    assert_eq!(a, machine_tag(&mac.to_ascii_lowercase(), 501), "stable");
    assert_eq!(a, machine_tag(&format!("{mac}\n"), 501));
    assert_ne!(a, machine_tag(mac, 502), "another account");
    assert_ne!(a, machine_tag("4C4C4544-0042-3510-8051-B4C04F4D4C32", 501));
    assert_eq!(a.len(), 17);
    assert!(a.starts_with('m') && a[1..].bytes().all(|b| b.is_ascii_hexdigit()));
    assert!(
        !a.to_ascii_uppercase().contains("4C4C4544"),
        "the id is not written"
    );
    assert_eq!(host_tag(&a), a, "usable in a file name");
}

/// Two Macs with the same name (or the name changed between a crash and
/// the next start): the other computer's "before" file is never read.
#[cfg(unix)]
#[test]
fn another_computers_before_file_is_ignored() {
    let (_tmp, kit) = kit();
    let mine = files(&kit);
    let dir = mine.store.parent().unwrap().to_path_buf();
    let theirs = files_in(&dir, Some(&machine_tag("another-mac", 501)));
    assert_ne!(theirs.seen, mine.seen);
    let mut kit_item = item(T, "x");
    kit_item.kit = Some(true);
    save_file(&theirs.seen, &[kit_item]).unwrap();
    let store = Fake::with(&[(T, "mine")]);
    assert!(!pending(&kit));
    save_with(&kit, &store).unwrap();
    assert_eq!(store.secret_of(T).as_deref(), Some("mine"));
    assert!(
        !mine.store.exists(),
        "this Mac's login never copied into the kit"
    );
    assert!(theirs.seen.exists(), "left for its own computer");
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

/// A second start of agy within 20 s of the first, before the first shows
/// as running: the kit's login already in the store is not taken for this
/// computer's own, so it does not stay in the store after the run.
#[test]
fn second_start_right_after_the_first() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit-1")]).unwrap();
    let store = Fake::default();

    restore_with(&kit, &store).unwrap();
    restore_with(&kit, &store).unwrap();
    assert_eq!(store.secret_of(T).as_deref(), Some("kit-1"));
    assert!(
        read(&f.seen).iter().all(|i| i.kit == Some(true)),
        "nothing of this computer's: {:?}",
        read(&f.seen)
    );

    let old = std::time::SystemTime::now() - Duration::from_secs(60);
    fs::File::options()
        .write(true)
        .open(&f.starting)
        .unwrap()
        .set_modified(old)
        .unwrap();
    end_run(&kit, &store);
    assert_eq!(store.secret_of(T), None, "the kit's login left the store");
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit-1"));
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

/// Linux without a Secret Service, on a disk that keeps 600: nothing
/// moved, agy starts anyway (the plan's unencrypted fallback).
#[test]
fn no_store_moves_nothing() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake {
        down: Down::Missing,
        ..Fake::default()
    };
    restore_checked(&kit, &store, Os::Linux, |_| Some(0o100600)).unwrap();
    assert!(!pending(&kit));
    save_with(&kit, &store).unwrap();
    end_run(&kit, &store);
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit"));
}

/// The unencrypted fallback only on Linux and only where 600 sticks.
#[test]
fn no_store_refused_without_permissions_or_on_macos() {
    assert!(plaintext_allowed(Os::Linux, Some(0o100600)));
    assert!(
        !plaintext_allowed(Os::Linux, Some(0o100644)),
        "exFAT, FAT: the mode does not stick"
    );
    assert!(!plaintext_allowed(Os::Linux, Some(0o100777)));
    assert!(!plaintext_allowed(Os::Linux, None));
    assert!(
        !plaintext_allowed(Os::Darwin, Some(0o100600)),
        "the Keychain always exists"
    );
    assert!(!plaintext_allowed(Os::Win32, Some(0o100600)));
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake {
        down: Down::Missing,
        ..Fake::default()
    };
    for (os, mode) in [(Os::Linux, Some(0o100755)), (Os::Darwin, Some(0o100600))] {
        let e = restore_checked(&kit, &store, os, |_| mode).unwrap_err();
        assert!(format!("{e:#}").contains("does not keep owner-only permissions"));
        assert!(!pending(&kit));
    }
    assert!(
        NO_STORE.contains("UNENCRYPTED") && NO_STORE.contains("whoever copies the kit gets it")
    );
}

/// A store that exists but cannot be used (locked, denied): an error,
/// nothing moved, nothing written.
#[test]
fn locked_store_fails_closed() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake {
        down: Down::Locked,
        ..Fake::default()
    };
    let e = restore_checked(&kit, &store, Os::Linux, |_| Some(0o100600)).unwrap_err();
    assert!(format!("{e:#}").contains("not moved"));
    assert!(!pending(&kit) && !f.starting.exists());
}

/// No sealing key for this computer's own sign-in: an error, and neither
/// its secret nor the kit's sign-in is written anywhere (fail closed, as
/// DPAPI).
#[test]
fn no_sealing_key_fails_closed() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item("jetski:other", "kit")]).unwrap();
    let store = Fake {
        no_key: true,
        ..Fake::with(&[(T, "own")])
    };
    let e = restore_with(&kit, &store).unwrap_err();
    assert!(format!("{e:#}").contains("cannot be sealed"));
    assert!(!f.seen.exists(), "no secret of the computer in the kit");
    assert!(!f.starting.exists());
    assert_eq!(
        store.secret_of("jetski:other"),
        None,
        "the kit's sign-in not moved"
    );
    assert_eq!(
        store.secret_of(T).as_deref(),
        Some("own"),
        "the computer's untouched"
    );
    // Without secrets of the computer nothing needs sealing: it works.
    let empty = Fake {
        no_key: true,
        ..Fake::default()
    };
    restore_with(&kit, &empty).unwrap();
    assert_eq!(empty.secret_of("jetski:other").as_deref(), Some("kit"));
}

/// `security find-generic-password`: only 44 is "no such item"; 36
/// (interaction not allowed), 128 (unlock cancelled) or a kill are errors,
/// never an empty Keychain.
#[test]
fn keychain_read_failures_are_errors() {
    assert!(mac_found(Some(0)).unwrap());
    assert!(!mac_found(Some(44)).unwrap());
    for code in [Some(36), Some(128), Some(1), Some(51), None] {
        assert!(mac_found(code).is_err(), "{code:?}");
    }
    assert!(format!("{:#}", mac_found(Some(36)).unwrap_err()).contains("36"));
}

/// The store becomes unreadable between the start and the end of agy: the
/// save stops before touching anything (the kit's login kept, the
/// "before" file kept to retry), and the next save cleans up.
#[test]
fn unreadable_store_at_save_changes_nothing() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item(T, "kit")]).unwrap();
    let store = Fake::default();
    restore_with(&kit, &store).unwrap();
    let kit_file = fs::read(&f.store).unwrap();
    let seen_file = fs::read(&f.seen).unwrap();

    store.locked_now.set(true);
    let _ = fs::remove_file(&f.starting);
    assert!(save_with(&kit, &store).is_err());
    assert_eq!(
        fs::read(&f.store).unwrap(),
        kit_file,
        "the kit's login not dropped"
    );
    assert_eq!(fs::read(&f.seen).unwrap(), seen_file, "kept to retry");
    assert_eq!(store.forgot.get(), 0, "the sealing key kept");
    assert_eq!(store.secret_of(T).as_deref(), Some("kit"));
    // A restore refuses too, without overwriting anything.
    assert!(restore_with(&kit, &store).is_err());
    assert_eq!(fs::read(&f.seen).unwrap(), seen_file);

    store.locked_now.set(false);
    save_with(&kit, &store).unwrap();
    assert!(!pending(&kit));
    assert_eq!(store.secret_of(T), None, "cleaned once readable");
    assert_eq!(read(&f.store)[0].blob.as_deref(), Some("kit"));
}

/// A restore that fails after the computer's sign-in was sealed: no
/// "before" file, so the sealing key made for it goes too.
#[test]
fn failed_restore_forgets_the_sealing_key() {
    let (_tmp, kit) = kit();
    let f = files(&kit);
    save_file(&f.store, &[item("jetski:other", "kit")]).unwrap();
    let store = Fake {
        refuse: Some("jetski:other"),
        ..Fake::with(&[(T, "own")])
    };
    assert!(restore_with(&kit, &store).is_err());
    assert!(!f.seen.exists() && !f.starting.exists());
    assert_eq!(store.forgot.get(), 1);
    assert_eq!(store.secret_of(T).as_deref(), Some("own"));
}

#[test]
fn missing_secret_service_recognized() {
    let real = "Secret Service: Platform secure storage failure: DBus error: The name org.freedesktop.secrets was not provided by any .service files";
    assert!(no_secret_service(real));
    assert!(no_secret_service(
        "org.freedesktop.DBus.Error.ServiceUnknown: x"
    ));
    assert!(no_secret_service(
        "Unable to autolaunch a dbus-daemon without a $DISPLAY"
    ));
    assert!(!no_secret_service(
        "Secret Service: Couldn't access platform secure storage: collection is locked"
    ));
    assert!(!no_secret_service("prompt dismissed"));
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
