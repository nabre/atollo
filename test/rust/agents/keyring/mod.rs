use super::files::files_in;
use super::*;
use crate::kit::tests::TempDir;
use crate::test_support::prepared_kit;

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

/// The name of the sealing key comes from the kit, not from its folder: a
/// kit moved keeps it; without the saved id, the kit's id as before.
#[test]
fn sealing_key_name_survives_a_move() {
    assert_eq!(
        seal_id(Some("0a1b2c3d4e5f6071\n"), "ffff"),
        "0a1b2c3d4e5f6071"
    );
    assert_eq!(seal_id(None, "ffff"), "ffff");
    for bad in ["", "  ", "../x", "a b", &"a".repeat(65)] {
        assert_eq!(seal_id(Some(bad), "ffff"), "ffff", "{bad:?}");
    }
    let (_tmp, kit) = kit();
    let system = System::of(&kit);
    assert_eq!(system.seal_user, format!("antigravity-before-{}", kit.id()));
    let saved = kit.data().join("antigravity").join(SEAL_ID_FILE);
    fs::create_dir_all(saved.parent().unwrap()).unwrap();
    fs::write(&saved, "00112233aabbccdd").unwrap();
    assert_eq!(
        System::of(&kit).seal_user,
        "antigravity-before-00112233aabbccdd"
    );
}

pub(crate) const T: &str = "gemini:antigravity";

pub(crate) fn item(target: &str, secret: &str) -> Item {
    Item {
        target: target.into(),
        user: Some("antigravity".into()),
        blob: Some(secret.into()),
        written: None,
        persist: None,
        kit: None,
    }
}

pub(crate) fn kit() -> (TempDir, Kit) {
    prepared_kit("keyring-fake")
}
