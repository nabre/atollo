use super::*;
use crate::kit::{tests::TempDir, Arch, Os, Target};

/// R and PHP are available on Windows.
#[test]
fn r_and_php_are_available_on_windows() {
    let tmp = TempDir::new("cmd-comps");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "it");
        if target.is_windows() {
            for id in ["r", "php"] {
                let c = infos.iter().find(|c| c.id == id).unwrap();
                assert!(c.available, "{id}");
                assert_eq!(c.unavailable_reason, None, "{id}");
            }
        }
    }
}

/// Unavailable on another system: the page gets the system as the reason
/// (BUG-24c). Apache, the example before D-116, is now on macOS and Linux
/// too.
#[test]
fn unavailable_for_the_system_is_not_arm64() {
    let tmp = TempDir::new("cmd-comps-os");
    for target in [Target::DARWIN_ARM64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "en");
        // Apache is on every system since D-116.
        let apache = infos.iter().find(|c| c.id == "apache").unwrap();
        assert!(apache.available && apache.unavailable_reason.is_none());
        let python = infos.iter().find(|c| c.id == "python").unwrap();
        assert_eq!(python.unavailable_reason, None);
    }
}

/// The catalog's own reason, translated, in place of "not available for
/// <target>" (D-108): Android on Linux ARM64 (SQLite is there since
/// D-112). The Windows rows are as before.
#[test]
fn catalog_reasons_reach_the_page() {
    let tmp = TempDir::new("cmd-comps-reason");
    let kit = Kit::at_target(tmp.0.join("la"), Target::LINUX_ARM64);
    kit.prepare().unwrap();
    let infos = component_infos(&kit, "en");
    let row = |id: &str| infos.iter().find(|c| c.id == id).unwrap();
    let c = row("android");
    assert!(!c.available, "android");
    assert_eq!(
        c.unavailable_reason.as_deref(),
        Some("Google does not publish platform-tools and build-tools for Linux ARM64")
    );
    let sqlite = row("sqlite");
    assert!(sqlite.available && sqlite.unavailable_reason.is_none());
    assert_eq!(row("apache").unavailable_reason, None);
    assert_eq!(row("python").unavailable_reason, None);
    let it = component_infos(&kit, "it");
    let android = it.iter().find(|c| c.id == "android").unwrap();
    assert_eq!(
        android.unavailable_reason.as_deref(),
        Some("Google non pubblica platform-tools e build-tools per Linux ARM64")
    );

    // Elsewhere both are there, with no reason.
    for target in [Target::WIN32_X64, Target::DARWIN_ARM64, Target::LINUX_X64] {
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

/// The page gets what each component needs on the kit's target (C and C++
/// need Zig on macOS and Linux, Rust nothing anywhere), and
/// the Xcode notice only for C and C++ of a darwin kit on a Mac without the
/// Command Line Tools.
#[test]
fn needs_and_xcode_follow_the_target() {
    let tmp = TempDir::new("cmd-needs");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "it");
        let row = |id: &str| infos.iter().find(|c| c.id == id).unwrap();
        let zig: &[&str] = if target.is_unix() { &["zig"] } else { &[] };
        assert_eq!(row("cc").needs, zig, "{target}");
        assert!(row("rust").needs.is_empty(), "{target}");
        assert_eq!(row("kotlin").needs, ["java"], "{target}");
        // Only a Mac can tell; never for another component.
        assert!(infos
            .iter()
            .all(|c| !c.xcode_missing || (c.id == "cc" && cfg!(target_os = "macos"))));
    }
    for target in Target::ALL {
        for host in [Os::Win32, Os::Darwin, Os::Linux] {
            let mac = target.is_macos() && host == Os::Darwin;
            assert_eq!(
                xcode_missing(target, host, || None),
                mac,
                "{target} {host:?}"
            );
            assert_eq!(xcode_missing(target, host, || Some(" ".into())), mac);
            assert!(!xcode_missing(target, host, || Some(
                "/Library/Developer/CommandLineTools".into()
            )));
        }
    }
}

/// On Linux ARM64 (a Linux kit on every host) the Android SDK, which has
/// no build there, keeps its choice.
#[test]
fn unavailable_components_keep_their_choice() {
    let tmp = TempDir::new("cmd-choice");
    let catalog = Catalog::get();
    for (arch, kept) in [(Arch::X64, false), (Arch::Arm64, true)] {
        let kit = Kit::at_target(tmp.0.join(arch.as_str()), Target::new(Os::Linux, arch));
        kit.prepare().unwrap();
        let mut saved = Choice::defaults(catalog);
        saved.set(catalog, "android", true);
        saved.write(&kit, catalog).unwrap();
        // Java ticked: the Android SDK needs it on every target.
        let text = choice_text(&kit, &["git".into(), "php".into(), "java".into()]);
        let choice = Choice::parse(&text, catalog);
        assert_eq!(choice.is_chosen("android"), kept, "{arch:?}");
        assert!(choice.is_chosen("php") && !choice.is_chosen("go"));
    }
}

/// One `components.txt` for every system (review of D-108): a Windows kit
/// with C and C++ on and Zig off, shown and applied on the page of another
/// target, keeps C and C++ on and Zig off for Windows. The page shows Zig
/// ticked where C and C++ need it; ticking C and C++ there saves C and C++
/// only; Zig ticked alone is saved.
#[test]
fn needs_of_one_target_stay_out_of_the_choice() {
    let tmp = TempDir::new("cmd-needs-choice");
    let catalog = Catalog::get();
    let windows = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    windows.prepare().unwrap();
    let page = |kit: &Kit| -> Vec<String> {
        component_infos(kit, "it")
            .into_iter()
            .filter(|c| c.chosen)
            .map(|c| c.id)
            .collect()
    };
    for t in Target::ALL {
        let kit = windows.for_target(t);
        Choice::parse("cc=1\nzig=0\n", catalog)
            .write(&kit, catalog)
            .unwrap();
        let chosen = page(&kit);
        assert!(chosen.contains(&"cc".into()), "{t}");
        assert_eq!(chosen.contains(&"zig".into()), t.is_unix(), "{t}");
        let choice = Choice::parse(&choice_text(&kit, &chosen), catalog);
        choice.write(&kit, catalog).unwrap();
        let back = Choice::read(&windows, catalog).unwrap();
        assert!(back.is_chosen("cc") && !back.is_chosen("zig"), "{t}");

        // Ticked on this page: the toggle of the page ticks what it needs.
        Choice::parse("cc=0\nzig=0\n", catalog)
            .write(&kit, catalog)
            .unwrap();
        let mut chosen = page(&kit);
        chosen.extend(
            component_infos(&kit, "it")
                .into_iter()
                .find(|c| c.id == "cc")
                .unwrap()
                .needs,
        );
        chosen.push("cc".into());
        let choice = Choice::parse(&choice_text(&kit, &chosen), catalog);
        assert!(choice.is_chosen("cc") && !choice.is_chosen("zig"), "{t}");

        let mut chosen = page(&kit);
        chosen.push("zig".into());
        let choice = Choice::parse(&choice_text(&kit, &chosen), catalog);
        assert!(choice.is_chosen("zig") && !choice.is_chosen("cc"), "{t}");

        // Unticked on this page: the toggle removes C and C++ only, so Zig,
        // seeded as ticked from the saved choice, stays in the page's set
        // and must not be saved.
        Choice::parse("cc=1\nzig=0\n", catalog)
            .write(&kit, catalog)
            .unwrap();
        let chosen: Vec<String> = page(&kit).into_iter().filter(|id| id != "cc").collect();
        assert_eq!(chosen.contains(&"zig".into()), t.is_unix(), "{t}");
        let choice = Choice::parse(&choice_text(&kit, &chosen), catalog);
        assert!(!choice.is_chosen("cc") && !choice.is_chosen("zig"), "{t}");
        choice.write(&kit, catalog).unwrap();
        let back = Choice::read(&windows, catalog).unwrap();
        assert!(!back.is_chosen("cc") && !back.is_chosen("zig"), "{t}");

        // Zig chosen for every system stays chosen when C and C++ go off.
        Choice::parse("cc=1\nzig=1\n", catalog)
            .write(&kit, catalog)
            .unwrap();
        let chosen: Vec<String> = page(&kit).into_iter().filter(|id| id != "cc").collect();
        let choice = Choice::parse(&choice_text(&kit, &chosen), catalog);
        assert!(!choice.is_chosen("cc") && choice.is_chosen("zig"), "{t}");
    }
}

/// With the VM on (D-174): the grid names the VM's Linux, and on the PC's
/// column the tools the VM runs are not needed there; the system
/// components, the parts of the app and what only finishes on its own
/// system stay as they are.
#[test]
fn the_grid_marks_the_vm() {
    let tmp = TempDir::new("cmd-grid-vm");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    assert_eq!(target_grid(&kit, "en", None).vm, None);
    let grid = target_grid(&kit, "en", Some(Target::LINUX_X64));
    assert_eq!(grid.vm.as_deref(), Some("linux-x64"));
    let column = |t: Target| {
        grid.targets
            .iter()
            .find(|s| s.target == t.as_str())
            .unwrap()
    };
    let via = |id: &str| {
        column(Target::WIN32_X64)
            .tools
            .iter()
            .find(|x| x.id == id)
            .unwrap()
            .via_vm
    };
    assert!(via("node") && via("python") && via("go"));
    for kept in ["pwsh", "gh", "sandboxes", "vscodium-server"] {
        assert!(!via(kept), "{kept}");
    }
    // Rust finishes on its own system only: from a PC with Linux x64 (the
    // tests on Linux) the VM's Linux is that system.
    assert_eq!(via("rust"), Target::detect() == Target::LINUX_X64);
    assert!(column(Target::LINUX_X64).tools.iter().all(|x| !x.via_vm));
}

/// «Piattaforma» is remembered in settings.json (D-121): the grid gives it
/// back, the other settings stay, an unknown system is refused, `None`
/// forgets it; a file that cannot be read is not overwritten.
#[test]
fn tools_target_is_remembered() {
    let tmp = TempDir::new("cmd-tools-target");
    let kit = Kit::at_target(tmp.0.clone(), Target::detect());
    kit.prepare().unwrap();
    let grid = target_grid(&kit, "en", None);
    assert_eq!(grid.chosen, None);
    assert_eq!(grid.profiles.map(|p| p.id), Target::ALL.map(Target::as_str));
    assert_eq!(grid.targets.len(), 4);
    assert_eq!(grid.profiles.iter().filter(|p| p.is_host).count(), 1);

    let mut settings = Settings::load(&kit).settings;
    settings.lang = Some("de".into());
    settings.save(&kit).unwrap();
    save_tools_target(&kit, Some(" linux-arm64 ")).unwrap();
    let saved = Settings::load(&kit).settings;
    assert_eq!(saved.tools_target.as_deref(), Some("linux-arm64"));
    assert_eq!(saved.lang.as_deref(), Some("de"));
    assert!(saved.validate().is_ok());
    assert_eq!(
        target_grid(&kit, "en", None).chosen.as_deref(),
        Some("linux-arm64")
    );

    assert!(save_tools_target(&kit, Some("beos-x64")).is_err());
    assert_eq!(
        Settings::load(&kit).settings.tools_target.as_deref(),
        Some("linux-arm64")
    );
    save_tools_target(&kit, None).unwrap();
    assert_eq!(Settings::load(&kit).settings.tools_target, None);

    // A value written by hand that is not one of the four is not chosen
    // (D-176): it reads as this PC's, and the Settings page can still save.
    for unknown in ["beos-x64", "unsupported"] {
        let text = format!(r#"{{"toolsTarget":"{unknown}","lang":"fr"}}"#);
        std::fs::write(kit.settings_file(), text).unwrap();
        assert_eq!(target_grid(&kit, "en", None).chosen, None, "{unknown}");
        let loaded = Settings::load(&kit).settings;
        assert_eq!(loaded.tools_target, None, "{unknown}");
        assert_eq!(loaded.lang.as_deref(), Some("fr"));
        assert!(loaded.validate().is_ok(), "{unknown}");
    }

    std::fs::write(kit.settings_file(), "{ broken").unwrap();
    assert!(save_tools_target(&kit, Some("win32-x64")).is_err());
    assert_eq!(
        std::fs::read_to_string(kit.settings_file()).unwrap(),
        "{ broken"
    );
}
