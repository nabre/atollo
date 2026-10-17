use super::*;
use crate::setup::catalog::Catalog;

/// Rosetta 2 missing says so, before "not available for <target>".
#[test]
fn rosetta_reason() {
    let catalog = Catalog::get();
    let python = catalog.find("python").unwrap();
    let php = catalog.find("php").unwrap();
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, true, "en").as_deref(),
        Some("needs Rosetta 2: install it with softwareupdate --install-rosetta (an administrator is needed)")
    );
    assert_eq!(
        unavailable_reason(python, Target::DARWIN_ARM64, false, "en"),
        None
    );
    assert_eq!(
        unavailable_reason(php, Target::DARWIN_ARM64, false, "en").as_deref(),
        Some("not available for darwin-arm64")
    );
    // Windows ARM64 without a build: the page's own "no ARM64" text.
    let r = catalog.find("r").unwrap();
    assert_eq!(
        unavailable_reason(r, Target::WIN32_ARM64, false, "en"),
        None
    );
}

/// The acceptance of WP-0.4: on `linux-arm64` Android says why, in the
/// sentence of an action and in every language of the catalog. SQLite is
/// there since WP-2.5 (conda-forge).
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

/// Without a reason of its own: the old generic sentences, unchanged on
/// Windows ARM64; Rosetta 2 missing gives its reason.
#[test]
fn messages_without_a_catalog_reason() {
    let catalog = Catalog::get();
    let r = catalog.find("r").unwrap();
    let php = catalog.find("php").unwrap();
    let python = catalog.find("python").unwrap();
    assert_eq!(
        unavailable_message(r, Target::WIN32_ARM64, false, "en").as_deref(),
        Some("R is not available on ARM64 PCs.")
    );
    assert_eq!(
        unavailable_message(php, Target::LINUX_X64, false, "en").as_deref(),
        Some("PHP is not available for linux-x64.")
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
