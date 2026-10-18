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
        let infos = component_infos_from(&kit, "it", Target::WIN32_ARM64);
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
    let infos = component_infos_from(&kit, "en", Target::WIN32_ARM64);
    let r = infos.iter().find(|c| c.id == "r").unwrap();
    assert!(r.no_arm64 && r.unavailable_reason.is_none());
    // "Not available on ARM64 PCs" only on such a PC: from another one the
    // system is named, as for macOS and Linux.
    for host in [Target::WIN32_X64, Target::DARWIN_ARM64, Target::LINUX_ARM64] {
        let infos = component_infos_from(&kit, "en", host);
        let r = infos.iter().find(|c| c.id == "r").unwrap();
        assert!(!r.available && !r.no_arm64, "{host}");
        assert_eq!(
            r.unavailable_reason.as_deref(),
            Some("not available for win32-arm64")
        );
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

/// The page gets what each component needs on the kit's target (C and C++
/// need Zig on macOS and Linux, Rust needs C and C++ on Windows ARM64), and
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
        let cc: &[&str] = if target == Target::WIN32_ARM64 {
            &["cc"]
        } else {
            &[]
        };
        assert_eq!(row("rust").needs, cc, "{target}");
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

    // Windows ARM64, where Rust needs C and C++: unticking C and C++ (the
    // page unticks Rust with it) saves both off, though the saved choice
    // has them on and cc is a need of this target only.
    let arm = windows.for_target(Target::WIN32_ARM64);
    Choice::parse("rust=1\ncc=1\n", catalog)
        .write(&arm, catalog)
        .unwrap();
    let chosen: Vec<String> = page(&arm)
        .into_iter()
        .filter(|id| id != "cc" && id != "rust")
        .collect();
    let choice = Choice::parse(&choice_text(&arm, &chosen), catalog);
    assert!(!choice.is_chosen("cc") && !choice.is_chosen("rust"));
}

/// «Piattaforma» is remembered in settings.json (D-121): the grid gives it
/// back, the other settings stay, an unknown system is refused, `None`
/// forgets it; a file that cannot be read is not overwritten.
#[test]
fn tools_target_is_remembered() {
    let tmp = TempDir::new("cmd-tools-target");
    let kit = Kit::at_target(tmp.0.clone(), Target::detect());
    kit.prepare().unwrap();
    let grid = target_grid(&kit, "en");
    assert_eq!(grid.chosen, None);
    assert_eq!(grid.profiles.map(|p| p.id), Target::ALL.map(Target::as_str));
    assert_eq!(grid.targets.len(), 6);
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
        target_grid(&kit, "en").chosen.as_deref(),
        Some("linux-arm64")
    );

    assert!(save_tools_target(&kit, Some("beos-x64")).is_err());
    assert_eq!(
        Settings::load(&kit).settings.tools_target.as_deref(),
        Some("linux-arm64")
    );
    save_tools_target(&kit, None).unwrap();
    assert_eq!(Settings::load(&kit).settings.tools_target, None);

    // A value written by hand that the app does not know is not chosen.
    std::fs::write(kit.settings_file(), r#"{"toolsTarget":"beos-x64"}"#).unwrap();
    assert_eq!(target_grid(&kit, "en").chosen, None);
    assert!(Settings::load(&kit).settings.validate().is_err());

    std::fs::write(kit.settings_file(), "{ broken").unwrap();
    assert!(save_tools_target(&kit, Some("win32-x64")).is_err());
    assert_eq!(
        std::fs::read_to_string(kit.settings_file()).unwrap(),
        "{ broken"
    );
}
