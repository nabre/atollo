use super::*;
use crate::setup::Catalog;

/// A component limited by `targets` to Windows: none in the catalog is
/// since Apache came to macOS and Linux (D-116).
fn windows_only() -> crate::setup::Component {
    serde_json::from_value(serde_json::json!({
        "id": "w", "name": "W", "group": "languages", "def": false, "mb": 1,
        "desc": "w", "home": "https://w", "key": "w/w.exe",
        "targets": ["win32-x64"]
    }))
    .unwrap()
}

/// Rosetta 2 missing says so, before "not available for <target>".
#[test]
fn rosetta_reason() {
    let catalog = Catalog::get();
    let python = catalog.find("python").unwrap();
    let windows = windows_only();
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, true, "en").as_deref(),
        Some("needs Rosetta 2: install it with softwareupdate --install-rosetta (an administrator is needed)")
    );
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, false, "en"),
        None
    );
    assert_eq!(
        unavailable_reason(&windows, Target::DARWIN_ARM64, false, "en").as_deref(),
        Some("not available for darwin-arm64")
    );
}

/// The acceptance of D-108: on `linux-arm64` Android says why, in the
/// sentence of an action and in every language of the catalog. SQLite is
/// there since D-112 (conda-forge).
#[test]
fn linux_arm64_messages_give_the_catalog_reason() {
    let catalog = Catalog::get();
    let android = catalog.find("android").unwrap();
    let sqlite = catalog.find("sqlite").unwrap();
    assert_eq!(
        unavailable_message(android, Target::LINUX_ARM64, false, "en").as_deref(),
        Some("Android SDK is not available: Google does not publish platform-tools and build-tools for Linux ARM64.")
    );
    assert_eq!(
        unavailable_message(android, Target::LINUX_ARM64, false, "it").as_deref(),
        Some("Android SDK non è disponibile: Google non pubblica platform-tools e build-tools per Linux ARM64.")
    );
    for lang in ["it", "en", "fr", "de"] {
        let c = android;
        let reason = unavailable_reason(c, Target::LINUX_ARM64, false, lang).unwrap();
        let msg = unavailable_message(c, Target::LINUX_ARM64, false, lang).unwrap();
        assert!(msg.contains(&reason), "{lang} {}: {msg}", c.id);
        assert!(msg.contains(&c.name), "{lang} {}: {msg}", c.id);
    }
    // SQLite is available everywhere.
    for target in Target::ALL {
        assert_eq!(unavailable_message(sqlite, target, false, "en"), None);
    }
}

/// Without a reason of its own: the generic sentence naming the system;
/// Rosetta 2 missing gives its reason.
#[test]
fn messages_without_a_catalog_reason() {
    let catalog = Catalog::get();
    let python = catalog.find("python").unwrap();
    assert_eq!(
        unavailable_message(&windows_only(), Target::LINUX_X64, false, "en").as_deref(),
        Some("W is not available for linux-x64.")
    );
    assert_eq!(
        unavailable_message(python, Target::DARWIN_ARM64, true, "en").as_deref(),
        Some("Python is not available: needs Rosetta 2: install it with softwareupdate --install-rosetta (an administrator is needed).")
    );
    assert_eq!(
        unavailable_message(python, Target::WIN32_X64, false, "en"),
        None
    );
}

/// Nothing is available for a system Atollo does not support (D-176), not
/// even a component without a `targets` list.
#[test]
fn nothing_is_available_for_a_removed_system() {
    use crate::kit::{Arch, Os};
    let catalog = Catalog::get();
    for removed in [
        Target::new(Os::Win32, Arch::Arm64),
        Target::new(Os::Darwin, Arch::X64),
    ] {
        for c in catalog.all() {
            assert!(!c.is_available_on(removed), "{} on {removed}", c.id);
            assert!(
                unavailable_message(c, removed, false, "en").is_some(),
                "{} on {removed}",
                c.id
            );
        }
    }
}

/// On a host that is not a Mac with Apple Silicon Rosetta is never missing:
/// the `_on_host` variant is the pure one with `no_rosetta` false.
#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
#[test]
fn on_host_matches_the_pure_rule_off_a_mac() {
    let catalog = Catalog::get();
    for c in catalog.all() {
        for target in Target::ALL {
            assert_eq!(
                unavailable_message_on_host(c, target, "en"),
                unavailable_message(c, target, false, "en"),
                "{} {target}",
                c.id
            );
        }
    }
}
