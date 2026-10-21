use super::*;

fn args(list: &[&str]) -> Vec<OsString> {
    list.iter().map(OsString::from).collect()
}

#[test]
fn commands() {
    assert_eq!(parse(&[]), Cli::Gui);
    assert_eq!(parse(&args(&["env"])), Cli::Env { project: None });
    assert_eq!(
        parse(&args(&["env", "--project", "app"])),
        Cli::Env {
            project: Some("app".into())
        }
    );
    assert_eq!(parse(&args(&["versions"])), Cli::Versions(None));
    assert_eq!(
        parse(&args(&["versions", "Node"])),
        Cli::Versions(Some("node".into()))
    );
    assert_eq!(parse(&args(&["--version"])), Cli::Version);
    assert_eq!(parse(&args(&["/?"])), Cli::Help);
    assert_eq!(parse(&args(&["components"])), Cli::Components(None));
    assert_eq!(parse(&args(&["x"])), Cli::Unknown("x".into()));
}

/// BUG-V6-33: `sync --project` without its value is a usage error, not a
/// sync of every project.
#[test]
fn sync_needs_the_project_value() {
    assert_eq!(
        parse(&args(&[
            "sync",
            "--project",
            r"D:\O’Brien\My App {x}",
            "--verbose"
        ])),
        Cli::Sync {
            project: Some(r"D:\O’Brien\My App {x}".into()),
            verbose: true
        }
    );
    assert_eq!(
        parse(&args(&["sync"])),
        Cli::Sync {
            project: None,
            verbose: false
        }
    );
    assert_eq!(
        parse(&args(&["sync", "--project"])),
        Cli::Unknown("sync --project".into())
    );
    assert_eq!(
        parse(&args(&["sync", "--verbose", "--project"])),
        Cli::Unknown("sync --project".into())
    );
}

#[test]
fn components_for_a_target() {
    use crate::kit::Target;
    assert_eq!(
        parse(&args(&["components", "--target", "Linux-ARM64"])),
        Cli::Components(Some(Target::LINUX_ARM64))
    );
    assert_eq!(
        parse(&args(&["components", "--target", "beos-x64"])),
        Cli::Unknown("components --target".into())
    );
    assert_eq!(
        parse(&args(&["components", "--target"])),
        Cli::Unknown("components --target".into())
    );
}

/// Zig is ticked next to C and C++ on macOS and Linux, as on the Tools
/// page, though the saved choice does not have it; not on Windows.
#[test]
fn components_mark_the_needs_of_one_target() {
    use crate::kit::{tests::TempDir, Target};
    let tmp = TempDir::new("cli-components-zig");
    let catalog = Catalog::get();
    for target in [Target::LINUX_X64, Target::DARWIN_ARM64, Target::WIN32_X64] {
        let kit = Kit::at_target(tmp.0.join(target.as_str()), target);
        kit.prepare().unwrap();
        let mut choice = Choice::defaults(catalog);
        choice.set(catalog, "cc", true);
        choice.set(catalog, "zig", false);
        choice.write(&kit, catalog).unwrap();
        assert!(!Choice::read(&kit, catalog).unwrap().is_chosen("zig"));
        let mut out = Vec::new();
        print_components(&kit, "en", &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        let mark = |id: &str| {
            text.lines()
                .find(|l| l.get(4..).and_then(|r| r.split_whitespace().next()) == Some(id))
                .and_then(|l| l.get(..3))
                .unwrap_or_else(|| panic!("no line for {id}: {text}"))
                .to_owned()
        };
        assert_eq!(mark("cc"), "[x]", "{target}");
        let zig = if target.is_unix() { "[x]" } else { "[ ]" };
        assert_eq!(mark("zig"), zig, "{target}");
    }
}

/// D-108: `components --target linux-arm64` names what is missing there,
/// with the catalog's reason.
#[test]
fn components_say_why_not_on_linux_arm64() {
    let tmp = std::env::temp_dir().join(format!("atollo-cli-components-{}", std::process::id()));
    let kit = Kit::at(tmp.clone(), crate::kit::Arch::detect())
        .for_target(crate::kit::Target::LINUX_ARM64);
    let mut out = Vec::new();
    print_components(&kit, "en", &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    let line = |id: &str| {
        text.lines()
            // After the mark `[x] ` / `[ ] `.
            .find(|l| l.get(4..).and_then(|r| r.split_whitespace().next()) == Some(id))
            .unwrap_or_else(|| panic!("no line for {id}: {text}"))
            .to_owned()
    };
    assert!(
        line("android").contains("not available"),
        "{}",
        line("android")
    );
    // SQLite comes from conda-forge there (D-112).
    for id in ["node", "sqlite"] {
        assert!(!line(id).contains("not available"), "{}", line(id));
    }
    let _ = std::fs::remove_dir_all(tmp);
}

#[test]
fn prepare_options() {
    let parsed = parse(&args(&[
        "prepare",
        "--only",
        "pwsh,Node",
        "--skip",
        "codex;rust",
        "--update-all",
        "--force",
    ]));
    assert_eq!(
        parsed,
        Cli::Prepare(PrepareArgs {
            target: None,
            only: vec!["pwsh".into(), "node".into()],
            skip: vec!["codex".into(), "rust".into()],
            update_all: true,
            force: true,
            lines: Vec::new(),
        })
    );
    let Cli::Prepare(p) = parse(&args(&[
        "prepare",
        "--target",
        "linux-x64",
        "--line",
        "Node=22.21",
        "--line",
        "go=1.24",
    ])) else {
        panic!("prepare")
    };
    assert_eq!(p.target, Some(crate::kit::Target::LINUX_X64));
    assert_eq!(
        p.lines,
        [
            ("node".into(), "22.21".into()),
            ("go".into(), "1.24".into())
        ]
    );
    assert!(matches!(
        parse(&args(&["prepare", "--line", "node"])),
        Cli::Unknown(_)
    ));
    assert_eq!(
        parse(&args(&["prepare", "--bogus"])),
        Cli::Unknown("prepare --bogus".into())
    );
}

#[test]
fn printer_uses_kit_markers() {
    let mut buf = Vec::new();
    {
        let mut p = Printer::new(&mut buf, "it");
        p.emit(Event::Step {
            index: 1,
            total: 2,
            id: "pwsh".into(),
            name: "PowerShell 7".into(),
        });
        p.emit(Event::Done {
            id: "pwsh".into(),
            outcome: Outcome::Installed {
                version: "7.6.7".into(),
            },
        });
        p.emit(Event::Done {
            id: "pwsh".into(),
            outcome: Outcome::Failed {
                message: "boom".into(),
                by_hand: None,
            },
        });
    }
    let text = String::from_utf8(buf).unwrap();
    assert!(text.contains("── 1. PowerShell 7 ──"));
    assert!(text.contains("  √ PowerShell 7: installato 7.6.7"));
    assert!(text.contains("  x PowerShell 7: non riuscito: boom"));
}

/// The hint of a file to download by hand names the folder where it is
/// looked for, without "del kit" (it is in the system's TEMP, D-90), in
/// every language (BUG-17r).
#[test]
fn failed_by_hand_shows_downloads_dir() {
    let failed = || Event::Done {
        id: "gh".into(),
        outcome: Outcome::Failed {
            message: "offline".into(),
            by_hand: Some(crate::setup::runner::ByHand {
                url: "https://example.org/gh.zip".into(),
                file_name: "gh.zip".into(),
            }),
        },
    };
    let dir = std::path::PathBuf::from(r"C:\Temp\Atollo\abc\downloads");
    for (lang, kit) in [
        ("it", "del kit"),
        ("en", "kit's"),
        ("fr", "du kit"),
        ("de", "des Kits"),
    ] {
        let mut buf = Vec::new();
        {
            let mut p = Printer::new(&mut buf, lang).with_downloads_dir(dir.clone());
            p.emit(failed());
        }
        let text = String::from_utf8(buf).unwrap();
        assert!(text.contains(&dir.display().to_string()), "{lang}: {text}");
        assert!(
            text.contains("https://example.org/gh.zip"),
            "{lang}: {text}"
        );
        assert!(
            !text.contains(kit) && !text.contains("{1}"),
            "{lang}: {text}"
        );
    }
    // Without the folder of the system: the one in the kit.
    let mut buf = Vec::new();
    {
        let mut p = Printer::new(&mut buf, "it");
        p.emit(failed());
    }
    let text = String::from_utf8(buf).unwrap();
    assert!(
        text.contains("mettilo nella cartella data\\downloads e riprova."),
        "{text}"
    );
}
