use super::*;
use crate::setup::Catalog;

/// The tests below look from a Windows ARM64 PC: the only host that says
/// "ARM64 PCs" (see `seen_from_another_pc_windows_arm64_is_named`).
const HOST: Target = Target::WIN32_ARM64;

/// A component limited by `targets` to Windows: none in the catalog is
/// since Apache came to macOS and Linux (D-116).
fn windows_only() -> crate::setup::Component {
    serde_json::from_value(serde_json::json!({
        "id": "w", "name": "W", "group": "languages", "def": false, "mb": 1,
        "desc": "w", "home": "https://w", "key": "w/w.exe",
        "targets": ["win32-x64", "win32-arm64"]
    }))
    .unwrap()
}

/// A `noArm64` component: none in the catalog is since R runs emulated on
/// Windows ARM64 (`x64Only`).
fn no_arm64() -> crate::setup::Component {
    serde_json::from_value(serde_json::json!({
        "id": "n", "name": "N", "group": "languages", "def": false, "mb": 1,
        "desc": "n", "home": "https://n", "key": "n/n.exe", "noArm64": true
    }))
    .unwrap()
}

/// From a PC that is not Windows ARM64 the component without an ARM64
/// build is "not available for win32-arm64", like any other system; the
/// other reasons do not depend on the host.
#[test]
fn seen_from_another_pc_windows_arm64_is_named() {
    let catalog = Catalog::get();
    let r = &no_arm64();
    for host in Target::ALL.into_iter().filter(|h| *h != HOST) {
        assert_eq!(
            unavailable_reason(r, Target::WIN32_ARM64, host, false, "en").as_deref(),
            Some("not available for win32-arm64"),
            "{host}"
        );
        assert_eq!(
            unavailable_message(r, Target::WIN32_ARM64, host, false, "en").as_deref(),
            Some("N is not available for win32-arm64."),
            "{host}"
        );
        assert!(!on_arm64_pc(Target::WIN32_ARM64, host));
        for c in catalog.all() {
            for target in Target::ALL.into_iter().filter(|t| *t != HOST) {
                assert_eq!(
                    unavailable_message(c, target, host, false, "en"),
                    unavailable_message(c, target, HOST, false, "en"),
                    "{} {target} from {host}",
                    c.id
                );
            }
        }
    }
    assert!(on_arm64_pc(HOST, HOST));
}

/// Rosetta 2 missing says so, before "not available for <target>".
#[test]
fn rosetta_reason() {
    let catalog = Catalog::get();
    let python = catalog.find("python").unwrap();
    let windows = windows_only();
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, HOST, true, "en").as_deref(),
        Some("needs Rosetta 2: install it with softwareupdate --install-rosetta (an administrator is needed)")
    );
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, HOST, false, "en"),
        None
    );
    assert_eq!(
        unavailable_reason(&windows, Target::DARWIN_ARM64, HOST, false, "en").as_deref(),
        Some("not available for darwin-arm64")
    );
    // Windows ARM64 without a build: the page's own "no ARM64" text.
    let r = &no_arm64();
    assert_eq!(
        unavailable_reason(r, Target::WIN32_ARM64, HOST, false, "en"),
        None
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
        unavailable_message(android, Target::LINUX_ARM64, HOST, false, "en").as_deref(),
        Some("Android SDK is not available: Google does not publish platform-tools and build-tools for Linux ARM64.")
    );
    assert_eq!(
        unavailable_message(android, Target::LINUX_ARM64, HOST, false, "it").as_deref(),
        Some("Android SDK non è disponibile: Google non pubblica platform-tools e build-tools per Linux ARM64.")
    );
    for lang in ["it", "en", "fr", "de"] {
        let c = android;
        let reason = unavailable_reason(c, Target::LINUX_ARM64, HOST, false, lang).unwrap();
        let msg = unavailable_message(c, Target::LINUX_ARM64, HOST, false, lang).unwrap();
        assert!(msg.contains(&reason), "{lang} {}: {msg}", c.id);
        assert!(msg.contains(&c.name), "{lang} {}: {msg}", c.id);
    }
    // SQLite is available everywhere.
    for target in Target::ALL {
        assert_eq!(unavailable_message(sqlite, target, HOST, false, "en"), None);
    }
}

/// Without a reason of its own: the old generic sentences, unchanged on
/// Windows ARM64; Rosetta 2 missing gives its reason.
#[test]
fn messages_without_a_catalog_reason() {
    let catalog = Catalog::get();
    let r = &no_arm64();
    let python = catalog.find("python").unwrap();
    assert_eq!(
        unavailable_message(r, Target::WIN32_ARM64, HOST, false, "en").as_deref(),
        Some("N is not available on ARM64 PCs.")
    );
    assert_eq!(
        unavailable_message(&windows_only(), Target::LINUX_X64, HOST, false, "en").as_deref(),
        Some("W is not available for linux-x64.")
    );
    assert_eq!(
        unavailable_message(python, Target::DARWIN_ARM64, HOST, true, "en").as_deref(),
        Some("Python is not available: needs Rosetta 2: install it with softwareupdate --install-rosetta (an administrator is needed).")
    );
    assert_eq!(
        unavailable_message(python, Target::WIN32_X64, HOST, false, "en"),
        None
    );
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
                unavailable_message(c, target, Target::detect(), false, "en"),
                "{} {target}",
                c.id
            );
        }
    }
}
