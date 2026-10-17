use super::*;
use crate::kit::{tests::TempDir, Arch, Target};

#[test]
fn r_is_not_available_on_arm64() {
    let tmp = TempDir::new("cmd-comps");
    for (arch, arm64) in [(Arch::X64, false), (Arch::Arm64, true)] {
        let kit = Kit::at(tmp.0.join(arch.as_str()), arch);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "it");
        let r = infos.iter().find(|c| c.id == "r").unwrap();
        assert_eq!(
            (r.available, r.no_arm64, r.x64_only),
            (!arm64, arm64, false)
        );
        assert_eq!(r.unavailable_reason, None);
        let php = infos.iter().find(|c| c.id == "php").unwrap();
        assert!(php.available && !php.no_arm64 && php.x64_only, "emulated");
    }
}

/// Unavailable on macOS or Linux is not "no ARM64 build": the page gets the
/// system as the reason (BUG-24c).
#[test]
fn unavailable_for_the_system_is_not_arm64() {
    let tmp = TempDir::new("cmd-comps-os");
    for target in [Target::DARWIN_ARM64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "en");
        let php = infos.iter().find(|c| c.id == "php").unwrap();
        assert!(!php.available && !php.no_arm64, "{target:?}");
        assert_eq!(
            php.unavailable_reason.as_deref(),
            Some(format!("not available for {}", target.as_str()).as_str())
        );
        let python = infos.iter().find(|c| c.id == "python").unwrap();
        assert_eq!(python.unavailable_reason, None);
    }
    let kit = Kit::at_target(tmp.0.join("arm"), Target::WIN32_ARM64);
    kit.prepare().unwrap();
    let infos = component_infos(&kit, "en");
    let r = infos.iter().find(|c| c.id == "r").unwrap();
    assert!(r.no_arm64 && r.unavailable_reason.is_none());
}

/// The catalog's own reason, translated, in place of "not available for
/// <target>" (D-108): Android and SQLite on Linux ARM64. The Windows rows
/// are as before.
#[test]
fn catalog_reasons_reach_the_page() {
    let tmp = TempDir::new("cmd-comps-reason");
    let kit = Kit::at_target(tmp.0.join("la"), Target::LINUX_ARM64);
    kit.prepare().unwrap();
    let infos = component_infos(&kit, "en");
    let row = |id: &str| infos.iter().find(|c| c.id == id).unwrap();
    for (id, reason) in [
        (
            "android",
            "Google does not publish platform-tools and build-tools for Linux ARM64",
        ),
        (
            "sqlite",
            "sqlite.org does not publish the command-line tools for Linux ARM64",
        ),
    ] {
        let c = row(id);
        assert!(!c.available && !c.no_arm64, "{id}");
        assert_eq!(c.unavailable_reason.as_deref(), Some(reason));
    }
    assert_eq!(
        row("php").unavailable_reason.as_deref(),
        Some("not available for linux-arm64")
    );
    assert_eq!(row("python").unavailable_reason, None);
    let it = component_infos(&kit, "it");
    let android = it.iter().find(|c| c.id == "android").unwrap();
    assert_eq!(
        android.unavailable_reason.as_deref(),
        Some("Google non pubblica platform-tools e build-tools per Linux ARM64")
    );

    // Elsewhere both are there, with no reason.
    for target in [Target::WIN32_X64, Target::WIN32_ARM64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        for c in component_infos(&kit, "en")
            .iter()
            .filter(|c| c.id == "android" || c.id == "sqlite")
        {
            assert!(
                c.available && c.unavailable_reason.is_none(),
                "{} {target}",
                c.id
            );
        }
    }
}

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

#[test]
fn unavailable_components_keep_their_choice() {
    let tmp = TempDir::new("cmd-choice");
    let catalog = Catalog::get();
    for (arch, kept) in [(Arch::X64, false), (Arch::Arm64, true)] {
        let kit = Kit::at(tmp.0.join(arch.as_str()), arch);
        kit.prepare().unwrap();
        let mut saved = Choice::defaults(catalog);
        saved.set(catalog, "r", true);
        saved.write(&kit, catalog).unwrap();
        let text = choice_text(&kit, &["git".into(), "php".into()]);
        let choice = Choice::parse(&text, catalog);
        assert_eq!(choice.is_chosen("r"), kept, "{arch:?}");
        assert!(choice.is_chosen("php") && !choice.is_chosen("go"));
    }
}
