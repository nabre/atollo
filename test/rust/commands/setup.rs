use super::*;
use crate::kit::{tests::TempDir, Arch, Os, Target};

/// Windows x64 and ARM64 kits, whatever system runs the tests: `no_arm64`
/// and the x64 build in emulation are Windows rows.
#[test]
fn r_is_not_available_on_arm64() {
    let tmp = TempDir::new("cmd-comps");
    for (arch, arm64) in [(Arch::X64, false), (Arch::Arm64, true)] {
        let kit = Kit::at_target(tmp.0.join(arch.as_str()), Target::new(Os::Win32, arch));
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
        let apache = infos.iter().find(|c| c.id == "apache").unwrap();
        assert!(!apache.available && !apache.no_arm64, "{target:?}");
        assert_eq!(
            apache.unavailable_reason.as_deref(),
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
/// <target>" (D-108): Android on Linux ARM64 (SQLite is there since
/// WP-2.5). The Windows rows are as before.
#[test]
fn catalog_reasons_reach_the_page() {
    let tmp = TempDir::new("cmd-comps-reason");
    let kit = Kit::at_target(tmp.0.join("la"), Target::LINUX_ARM64);
    kit.prepare().unwrap();
    let infos = component_infos(&kit, "en");
    let row = |id: &str| infos.iter().find(|c| c.id == id).unwrap();
    let c = row("android");
    assert!(!c.available && !c.no_arm64, "android");
    assert_eq!(
        c.unavailable_reason.as_deref(),
        Some("Google does not publish platform-tools and build-tools for Linux ARM64")
    );
    let sqlite = row("sqlite");
    assert!(sqlite.available && sqlite.unavailable_reason.is_none());
    assert_eq!(
        row("apache").unavailable_reason.as_deref(),
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

/// On Windows ARM64 (a Windows kit on every host) R keeps its choice.
#[test]
fn unavailable_components_keep_their_choice() {
    let tmp = TempDir::new("cmd-choice");
    let catalog = Catalog::get();
    for (arch, kept) in [(Arch::X64, false), (Arch::Arm64, true)] {
        let kit = Kit::at_target(tmp.0.join(arch.as_str()), Target::new(Os::Win32, arch));
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
