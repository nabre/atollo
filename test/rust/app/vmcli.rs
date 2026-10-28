//! `atollo vm` (LEVELS.md §4.4, §8.5): its words, the question of
//! «Ricrea», the exit code and the report as text.

use super::*;
use crate::app::cli::{parse, Cli};
use crate::levels::report::{Mode, MountInfo, ToolCheck};
use std::ffi::OsString;

fn words(line: &str) -> Vec<OsString> {
    line.split_whitespace().map(OsString::from).collect()
}

/// The words of `vm`: the project, one action at most, `--yes` only with
/// `--recreate`; anything else is a usage error (exit 2). No `--agent`:
/// every agent is in the project's sandbox (D-184).
#[test]
fn vm_parses_its_words() {
    assert_eq!(
        parse(&words("vm shop")),
        Cli::Vm(VmArgs {
            project: "shop".into(),
            ..VmArgs::default()
        })
    );
    assert_eq!(
        parse(&words("vm shop --check --json")),
        Cli::Vm(VmArgs {
            project: "shop".into(),
            check: true,
            json: true,
            ..VmArgs::default()
        })
    );
    assert_eq!(
        parse(&words("vm --recreate shop --yes")),
        Cli::Vm(VmArgs {
            project: "shop".into(),
            action: Some(VmAction::Recreate),
            yes: true,
            ..VmArgs::default()
        })
    );
    for (line, action) in [
        ("vm shop --restart", VmAction::Restart),
        ("vm shop --refresh --check", VmAction::Refresh),
    ] {
        match parse(&words(line)) {
            Cli::Vm(v) => assert_eq!(v.action, Some(action), "{line}"),
            other => panic!("{line}: {other:?}"),
        }
    }
    for bad in [
        "vm",
        "vm --check",
        "vm shop other",
        "vm shop --restart --recreate",
        "vm shop --yes",
        "vm shop --restart --yes",
        "vm shop --agent",
        "vm shop --agent claude",
        "vm shop --force",
    ] {
        assert!(
            matches!(parse(&words(bad)), Cli::Unknown(_)),
            "{bad}: {:?}",
            parse(&words(bad))
        );
    }
}

/// «Ricrea» says what is lost and what stays, then asks; `--yes` does not
/// ask; nothing read is no.
#[test]
fn recreate_asks_unless_yes() {
    let ask = |answer: &str, yes: bool| {
        let mut out = Vec::new();
        let mut input = std::io::Cursor::new(answer.as_bytes().to_vec());
        let ok = confirm_recreate("atollo-p-shop-x", yes, "it", &mut out, &mut input);
        (ok, String::from_utf8(out).unwrap())
    };
    let (ok, text) = ask("s\n", false);
    assert!(ok);
    assert!(text.contains("atollo-p-shop-x"), "{text}");
    assert!(text.contains("Si perde: la home della VM"), "{text}");
    assert!(
        text.contains("Resta: il progetto, i suoi database"),
        "{text}"
    );
    assert!(text.contains("Procedo? [s/N]"), "{text}");
    assert!(ask("y\n", false).0);
    assert!(ask("Sì\n", false).0);
    assert!(!ask("n\n", false).0);
    assert!(!ask("\n", false).0);
    assert!(!ask("", false).0, "no console: no");
    let (ok, text) = ask("", true);
    assert!(ok);
    assert!(text.contains("Si perde"), "said with --yes too: {text}");
    assert!(!text.contains("Procedo?"), "{text}");
}

/// Without a yes nothing is touched, and the command says so (exit 1); a
/// project that is not one is an error before anything.
#[test]
fn recreate_without_a_yes_changes_nothing() {
    let (_tmp, kit) = crate::test_support::prepared_kit("vmcli-no");
    std::fs::create_dir_all(kit.projects().join("shop")).unwrap();
    let env = KitEnv::build(&kit, &Settings::default(), "it", &Host::default());
    let args = VmArgs {
        project: "shop".into(),
        action: Some(VmAction::Recreate),
        ..VmArgs::default()
    };
    let mut out = Vec::new();
    let mut input = std::io::Cursor::new(b"no\n".to_vec());
    let code = run_with(
        &kit,
        &env,
        &Settings::default(),
        &Host::default(),
        "it",
        &args,
        &mut out,
        &mut input,
    );
    let text = String::from_utf8(out).unwrap();
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("Nulla è cambiato."), "{text}");
    assert!(!crate::isolation::kit::dir(&kit, &vm::name("shop")).exists());

    let args = VmArgs {
        project: "nope".into(),
        ..VmArgs::default()
    };
    let mut out = Vec::new();
    let code = run_with(
        &kit,
        &env,
        &Settings::default(),
        &Host::default(),
        "it",
        &args,
        &mut out,
        &mut std::io::Cursor::new(Vec::new()),
    );
    assert_eq!(code, 1);
    assert!(String::from_utf8(out)
        .unwrap()
        .contains("Progetto non trovato: nope"));
}

fn sandbox(state: SandboxRun, tools: Option<Vec<ToolCheck>>) -> SandboxReport {
    SandboxReport {
        name: vm::name("shop"),
        state,
        last_used: None,
        mounts: vec![
            MountInfo {
                host: "D:\\kit\\projects\\shop".into(),
                guest: "/d/kit/projects/shop".into(),
                mode: Mode::Rw,
            },
            MountInfo {
                host: "D:\\kit\\data\\apps\\linux-x64".into(),
                guest: "/d/kit/data/apps/linux-x64".into(),
                mode: Mode::Ro,
            },
        ],
        host_ports: vec![report::HostPortInfo {
            port: 3307,
            what: "mysql shop".into(),
        }],
        info: None,
        sessions: Vec::new(),
        tools,
        note: None,
    }
}

fn tool(id: &str, verdict: Verdict, chosen: Option<&str>, found: Option<&str>) -> ToolCheck {
    ToolCheck {
        id: id.into(),
        name: id.into(),
        chosen: chosen.map(str::to_owned),
        command: format!("{id} --version"),
        path: None,
        found: found.map(str::to_owned),
        verdict,
        output: String::new(),
    }
}

fn vm_report(vm: VmOnOff, sandboxes: Vec<SandboxReport>) -> VmReport {
    VmReport {
        project: "shop".into(),
        vm,
        reason: (vm == VmOnOff::Off).then(|| "Hyper-V è spento.".to_owned()),
        sandboxes,
        github: false,
    }
}

/// Exit 0 only with the VM on, the project's sandbox there, and every tool
/// checked `ok` (or not meant for the VM).
#[test]
fn the_exit_code_says_whether_all_is_good() {
    let good = vm_report(
        VmOnOff::On,
        vec![sandbox(
            SandboxRun::Running,
            Some(vec![
                tool("node", Verdict::Ok, Some("22"), Some("22.1.0")),
                tool("apache", Verdict::NotInVm, None, None),
            ]),
        )],
    );
    assert!(all_good(&good));
    let mut bad = good.clone();
    bad.sandboxes[0].tools = Some(vec![tool(
        "node",
        Verdict::OtherLine,
        Some("22"),
        Some("20.1.0"),
    )]);
    assert!(!all_good(&bad));
    let off = vm_report(VmOnOff::Off, vec![sandbox(SandboxRun::Stopped, None)]);
    assert!(!all_good(&off));
    let missing = vm_report(VmOnOff::On, vec![sandbox(SandboxRun::Missing, None)]);
    assert!(!all_good(&missing));
}

/// The report as text: the reason the VM is off, the project's sandbox
/// with its state, its mounts rw/ro, its ports and its tools with expected
/// and found.
#[test]
fn the_report_prints_states_mounts_and_tools() {
    let report = vm_report(
        VmOnOff::Off,
        vec![sandbox(
            SandboxRun::Running,
            Some(vec![
                tool("node", Verdict::Ok, Some("22"), Some("22.1.0")),
                tool("python", Verdict::OtherLine, Some("3.13"), Some("3.12.7")),
                tool("go", Verdict::Missing, Some("1.23"), None),
            ]),
        )],
    );
    let mut out = Vec::new();
    print_report(&report, "it", &mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    for line in [
        "── VM del progetto shop ──",
        "  x La VM non è attiva.",
        "    Hyper-V è spento.",
        &format!("  Progetto: {} (accesa)", vm::name("shop")),
        "    rw  D:\\kit\\projects\\shop → /d/kit/projects/shop",
        "    ro  D:\\kit\\data\\apps\\linux-x64 → /d/kit/data/apps/linux-x64",
        "    Porta del PC raggiunta: 3307 (mysql shop)",
        "    √ node 22.1.0",
        "    x python atteso 3.13, trovato 3.12.7",
        "    x go manca",
        "  · GitHub nella VM: no",
    ] {
        assert!(text.contains(line), "{line:?} in:\n{text}");
    }
}
