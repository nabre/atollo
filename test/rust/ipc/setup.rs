use super::*;
use crate::kit::{tests::TempDir, Arch, Os, Target};

/// R and PHP are available on Windows.
#[test]
fn r_and_php_are_available_on_windows() {
    let tmp = TempDir::new("cmd-comps");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let infos = component_infos(&kit, "it", &crate::levels::VmState::On);
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
        let infos = component_infos(&kit, "en", &crate::levels::VmState::On);
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
    let infos = component_infos(&kit, "en", &crate::levels::VmState::On);
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
    let it = component_infos(&kit, "it", &crate::levels::VmState::On);
    let android = it.iter().find(|c| c.id == "android").unwrap();
    assert_eq!(
        android.unavailable_reason.as_deref(),
        Some("Google non pubblica platform-tools e build-tools per Linux ARM64")
    );

    // Elsewhere both are there, with no reason.
    for target in [Target::WIN32_X64, Target::DARWIN_ARM64, Target::LINUX_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        for c in component_infos(&kit, "en", &crate::levels::VmState::On)
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

/// D-177: on macOS and Linux the page says ConPTY is not needed, with the
/// catalog's reason translated, never "not available"; on Windows it is
/// there as before.
#[test]
fn conpty_is_not_needed_on_macos_and_linux() {
    let tmp = TempDir::new("cmd-conpty");
    for target in Target::ALL {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        for (lang, reason) in [
            ("en", "the app's terminal uses the system's PTY"),
            ("it", "il terminale dell'app usa la pty del sistema"),
        ] {
            let infos = component_infos(&kit, lang, &crate::levels::VmState::On);
            let c = infos.iter().find(|c| c.id == "conpty").unwrap();
            assert_eq!(c.unavailable_reason, None, "{target}");
            if target.is_windows() {
                assert!(c.available, "{target}");
                assert_eq!(c.not_needed, None, "{target}");
            } else {
                assert!(!c.available, "{target}");
                assert_eq!(c.not_needed.as_deref(), Some(reason), "{target}");
            }
        }
    }
    // Serialized in camelCase for the page.
    let kit = Kit::at_target(tmp.0.join("json"), Target::LINUX_X64);
    kit.prepare().unwrap();
    let infos = component_infos(&kit, "en", &crate::levels::VmState::On);
    let json = serde_json::to_value(infos.iter().find(|c| c.id == "conpty").unwrap()).unwrap();
    assert_eq!(
        json["notNeeded"],
        "the app's terminal uses the system's PTY"
    );
    assert_eq!(json["available"], false);
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
        let infos = component_infos(&kit, "it", &crate::levels::VmState::On);
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
        component_infos(kit, "it", &crate::levels::VmState::On)
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
            component_infos(&kit, "it", &crate::levels::VmState::On)
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

/// With the VM on (D-174): the grid names the VM's Linux; the PC's column
/// keeps every tool, the project's code runs on the PC (D-183).
#[test]
fn the_grid_marks_the_vm() {
    let tmp = TempDir::new("cmd-grid-vm");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    assert_eq!(target_grid(&kit, "en", None).vm, None);
    let grid = target_grid(&kit, "en", Some(Target::LINUX_X64));
    assert_eq!(grid.vm.as_deref(), Some("linux-x64"));
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

/// LEVELS.md §7, Q11: the agents' rows of the Tools page. Claude Code and
/// Codex «fornito dalla VM» on every system; Pi and Antigravity «serve la
/// VM» with the reason without it, their state read on the VM's Linux; an
/// install of an earlier Atollo for the PC is a leftover to remove.
#[test]
fn the_agents_rows_of_the_tools_page() {
    use crate::levels::agents::AgentRow;
    use crate::levels::VmState;
    let tmp = TempDir::new("cmd-agents");
    let kit = Kit::at_target(tmp.0.clone(), Target::WIN32_X64);
    kit.prepare().unwrap();
    let off = VmState::Off {
        blockers: vec![crate::isolation::Blocker::SbxSignedOut],
    };
    let row = |vm: &VmState, id: &str| {
        component_infos(&kit, "it", vm)
            .into_iter()
            .find(|c| c.id == id)
            .unwrap()
    };
    // Claude Code and Codex as Pi (D-184): a row of the VM without it.
    for id in ["claude", "codex"] {
        let c = row(&off, id);
        assert!(matches!(c.agent, Some(AgentRow::NeedsVm { .. })), "{id}");
        assert!(!c.available && !c.leftover, "{id}");
        assert_eq!(row(&VmState::On, id).agent, None, "{id}");
    }
    let pi = row(&off, "pi");
    let Some(AgentRow::NeedsVm { reason }) = &pi.agent else {
        panic!("{:?}", pi.agent)
    };
    assert!(reason.contains("sbx login"), "{reason}");
    assert!(!pi.available);
    let pi = row(&VmState::On, "pi");
    assert_eq!(pi.agent, None);
    assert!(pi.available && !pi.installed);
    // Its build for the VM's Linux is its state; one for the PC a leftover.
    let guest = kit.for_target(Target::LINUX_X64);
    let c = Catalog::get().find("pi").unwrap();
    for k in [&guest, &kit] {
        for p in c.key_paths(k) {
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, b"").unwrap();
        }
    }
    c.save_version(&guest, "0.70.2").unwrap();
    let pi = row(&VmState::On, "pi");
    assert!(pi.installed && pi.leftover);
    assert_eq!(pi.version.as_deref(), Some("0.70.2"));
    let json = serde_json::to_value(row(&off, "claude")).unwrap();
    assert_eq!(json["agent"]["kind"], "needsVm");
    assert_eq!(json["leftover"], false);
}
