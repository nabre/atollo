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
    assert_eq!(parse(&args(&["components"])), Cli::Components);
    assert_eq!(parse(&args(&["x"])), Cli::Unknown("x".into()));
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
            only: vec!["pwsh".into(), "node".into()],
            skip: vec!["codex".into(), "rust".into()],
            update_all: true,
            force: true,
            lines: Vec::new(),
        })
    );
    let Cli::Prepare(p) = parse(&args(&[
        "prepare",
        "--line",
        "Node=22.21",
        "--line",
        "go=1.24",
    ])) else {
        panic!("prepare")
    };
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
