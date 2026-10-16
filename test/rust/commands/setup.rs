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
        let term = infos.iter().find(|c| c.id == "terminal").unwrap();
        assert!(!term.available && !term.no_arm64, "{target:?}");
        assert_eq!(
            term.unavailable_reason.as_deref(),
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
