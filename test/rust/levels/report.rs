//! The VM report, «Verifica strumenti» and the actions on a sandbox
//! (LEVELS.md §8.1, §8.2, §8.5, §12.3): the report from a fake `sbx ls`
//! and the sandbox's kit files; the check's commands, script and verdicts;
//! the actions through `decide`, with a fake [`VmEffects`], under the
//! sandbox's lock.

use std::sync::Mutex;

use super::*;
use crate::isolation::kit as sandbox_kit;
use crate::levels::prepare::tests::{facts_of, kit_with_shop, shop_db, Fake};
use crate::levels::Origin;
#[cfg(unix)]
use crate::test_support::TempDir;

/// What a fake `sbx ls --json` says: the project's sandbox running, Claude
/// Code's stopped, Codex's not there.
fn ls(project: &str) -> String {
    serde_json::json!([
        {
            "name": vm::name(project),
            "status": "running",
            "last_used_at": "2026-10-26T10:00:00Z",
            "workspaces": ["/d/kit/projects/shop"]
        },
        { "name": vm::agent_name(project, "claude"), "status": "stopped" },
        { "name": "atollo-p-other-0123", "status": "running" }
    ])
    .to_string()
}

/// §8.1: every sandbox of the project, the project's first, each with its
/// state from `sbx ls`, its mounts (the workspace rw, the rest ro), the
/// PC's ports it reaches with what listens there, its `info.json`, and the
/// app's sessions in it.
#[test]
fn the_report_reads_every_sandbox() {
    let (_tmp, kit) = kit_with_shop("report");
    let name = vm::name("shop");
    let fx = Fake {
        dbs: vec![shop_db()],
        ..Fake::default()
    };
    let req = sandbox_request(&kit, &kit.projects().join("shop"), None);
    let placement = decide(&req, &facts_of(&kit, &req, true));
    let base = crate::env::KitEnv::build(&kit, &Default::default(), "it", &Default::default());
    let env =
        super::super::env::environment(&kit, &base, &placement, None, "it", &Default::default())
            .unwrap();
    super::super::prepare::prepare(&fx, &kit, &Default::default(), placement, env, &[], "it")
        .unwrap();
    let project_dir = kit.projects().join("shop");
    let tools = kit
        .for_target(vm::guest_target(kit.target()).unwrap())
        .apps();
    let guest_dir = sandbox_kit::guest_dir(&kit, &name);
    sandbox_kit::mark_mounts(
        &kit,
        &name,
        &format!(
            "{}\n{}\n{}\n",
            project_dir.display(),
            tools.display(),
            guest_dir.display()
        ),
    )
    .unwrap();
    sandbox_kit::mark_host_ports(&kit, &name, &[3307, 8085]).unwrap();
    let seen = Seen {
        on: true,
        reason: None,
        ls: Some(ls("shop")),
        published: vec![PublishedPort {
            port: 5173,
            address: "127.0.0.1".into(),
            action: "dev".into(),
        }],
        github: true,
    };
    let report = report(&kit, "shop", seen, &|sandbox| {
        if sandbox == name {
            vec!["shop · bash".to_owned()]
        } else {
            Vec::new()
        }
    });
    assert_eq!(report.vm, VmOnOff::On);
    let names: Vec<(&str, Option<Agent>, SandboxRun)> = report
        .sandboxes
        .iter()
        .map(|s| (s.name.as_str(), s.agent, s.state))
        .collect();
    let claude = vm::agent_name("shop", "claude");
    let codex = vm::agent_name("shop", "codex");
    assert_eq!(
        names,
        [
            (name.as_str(), None, SandboxRun::Running),
            (claude.as_str(), Some(Agent::Claude), SandboxRun::Stopped),
            (codex.as_str(), Some(Agent::Codex), SandboxRun::Missing),
        ]
    );
    let own = &report.sandboxes[0];
    assert_eq!(own.last_used.as_deref(), Some("2026-10-26T10:00:00Z"));
    let windows = kit.target().is_windows();
    let modes: Vec<(Mode, String)> = own
        .mounts
        .iter()
        .map(|m| (m.mode, m.guest.clone()))
        .collect();
    assert_eq!(
        modes,
        [
            (Mode::Rw, vm::guest_path(windows, &project_dir).unwrap()),
            (Mode::Ro, vm::guest_path(windows, &tools).unwrap()),
            (Mode::Ro, vm::guest_path(windows, &guest_dir).unwrap()),
        ]
    );
    assert_eq!(
        own.host_ports,
        [
            HostPortInfo {
                port: 3307,
                what: "mysql shop".into()
            },
            HostPortInfo {
                port: 8085,
                what: "llama-server".into()
            },
        ]
    );
    assert_eq!(
        own.info.as_ref().map(|i| i.sandbox.as_str()),
        Some(name.as_str())
    );
    assert_eq!(own.sessions, ["shop · bash"]);
    assert!(own.tools.is_none());
    assert!(report.sandboxes[2].info.is_none());
    assert!(report.sandboxes[2].mounts.is_empty());
    assert!(report.github);
    assert_eq!(report.published[0].port, 5173);
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["vm"], "on");
    assert_eq!(json["sandboxes"][0]["state"], "running");
    assert_eq!(json["sandboxes"][0]["mounts"][0]["mode"], "rw");
    assert_eq!(json["sandboxes"][1]["agent"], "claude");
    assert_eq!(json["sandboxes"][2]["state"], "missing");
}

/// §8.4: what the report shows of a sandbox is what its `atollo` prints.
#[test]
fn the_vm_info_follows_the_report() {
    let (_tmp, kit) = kit_with_shop("report-info");
    let name = vm::name("shop");
    let req = sandbox_request(&kit, &kit.projects().join("shop"), None);
    let placement = decide(&req, &facts_of(&kit, &req, true));
    let base = crate::env::KitEnv::build(&kit, &Default::default(), "it", &Default::default());
    let env =
        super::super::env::environment(&kit, &base, &placement, None, "it", &Default::default())
            .unwrap();
    let fx = Fake {
        dbs: vec![shop_db()],
        ..Fake::default()
    };
    super::super::prepare::prepare(&fx, &kit, &Default::default(), placement, env, &[], "it")
        .unwrap();
    let report = report(&kit, "shop", Seen::default(), &|_| Vec::new());
    let shown = report.sandboxes[0].info.clone().unwrap();
    let file: SandboxInfo =
        serde_json::from_str(&std::fs::read_to_string(info_file(&kit, &name)).unwrap()).unwrap();
    assert_eq!(shown, file);
    // Without `sbx`'s answer (the VM off): made once is stopped.
    assert_eq!(report.vm, VmOnOff::Off);
    assert_eq!(report.sandboxes[1].state, SandboxRun::Missing);
    sandbox_kit::mark_applied(&kit, &name, "x").unwrap();
    assert_eq!(sandbox_state(&kit, None, &name).0, SandboxRun::Stopped);
}

/// §8.2: the command each tool of the catalog runs in the VM, its program
/// by its name on `PATH`; those that never run there are `notInVm`; the
/// kit's programs on the PC are never checked.
#[test]
fn the_check_command_of_every_catalog_tool() {
    let (_tmp, kit) = kit_with_shop("check-commands");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    let commands = [
        ("git", "git --version"),
        ("python", "python3 --version"),
        ("node", "node --version"),
        ("uv", "uv --version"),
        ("php", "php --version"),
        ("go", "go version"),
        ("java", "java -version"),
        ("kotlin", "kotlinc -version"),
        ("dotnet", "dotnet --version"),
        ("dart", "dart --version"),
        ("zig", "zig version"),
        ("julia", "julia --version"),
        ("bun", "bun --version"),
        ("deno", "deno --version"),
        ("typst", "typst --version"),
        ("tinytex", "pdftex --version"),
        ("cmake", "cmake --version"),
        ("cc", "clang --version"),
        ("ruby", "ruby --version"),
        ("mysql", "mysql --version"),
        ("mongodb", "mongod --version"),
        ("sqlite", "sqlite3 --version"),
        ("pi", "pi --version"),
        ("antigravity", "agy --version"),
        ("claude", "claude --version"),
        ("codex", "codex --version"),
    ];
    for (id, command) in commands {
        let spec = check_spec(&kit, &guest, id, None).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(spec.command(), command, "{id}");
    }
    for id in NOT_IN_VM {
        let spec = check_spec(&kit, &guest, id, None).unwrap_or_else(|| panic!("{id}"));
        assert_eq!(spec.program, None, "{id}");
        assert_eq!(spec.command(), "");
    }
    for id in NOT_CHECKED {
        assert_eq!(check_spec(&kit, &guest, id, None), None, "{id}");
    }
    // Pi's and Antigravity's own folder on `PATH` for their check only.
    let pi = check_spec(&kit, &guest, "pi", None).unwrap();
    assert_eq!(
        pi.extra_path,
        vm::guest_path(kit.target().is_windows(), &guest.apps().join("pi"))
    );
    assert_eq!(
        check_spec(&kit, &guest, "node", None).unwrap().extra_path,
        None
    );
}

fn spec(id: &str, chosen: Option<&str>) -> CheckSpec {
    CheckSpec {
        id: id.into(),
        name: id.into(),
        chosen: chosen.map(str::to_owned),
        program: Some(id.into()),
        args: vec!["--version".into()],
        extra_path: None,
        from_image: false,
    }
}

const MOUNT: &str = "/d/kit/data/apps/linux-x64";

/// §8.2: a line matches its version (`22` → `22.11.0`), never another
/// (`2` is not `22.x`); an unversioned tool is fine once found.
#[test]
fn versions_are_compared_by_line() {
    let path = format!("{MOUNT}/node/22/bin/node");
    let at = |chosen, found| verdict(&spec("node", chosen), Some(&path), found, MOUNT);
    assert_eq!(at(Some("22"), Some("22.11.0")), Verdict::Ok);
    assert_eq!(at(Some("22.11"), Some("22.11.0")), Verdict::Ok);
    assert_eq!(at(Some("22.11.0"), Some("22.11.0")), Verdict::Ok);
    assert_eq!(at(Some("2"), Some("22.11.0")), Verdict::OtherLine);
    assert_eq!(at(Some("20"), Some("22.11.0")), Verdict::OtherLine);
    assert_eq!(at(Some("22"), None), Verdict::OtherLine);
    assert_eq!(at(None, None), Verdict::Ok);
    assert_eq!(
        verdict(&spec("node", Some("22")), None, None, MOUNT),
        Verdict::Missing
    );
    assert_eq!(found_version("v22.11.0"), Some("22.11.0".into()));
    assert_eq!(
        found_version("go version go1.23.2 linux/amd64"),
        Some("1.23.2".into())
    );
    assert_eq!(found_version("Python 3.13.1"), Some("3.13.1".into()));
    assert_eq!(found_version("no number"), None);
}

/// §8.2: a program found outside the kit's tools mounted in the VM (the
/// image's own) is `notFromKit`; an agent of `sbx` comes from its image.
#[test]
fn a_tool_outside_the_mount_is_not_from_the_kit() {
    assert_eq!(
        verdict(
            &spec("node", Some("22")),
            Some("/usr/bin/node"),
            Some("22.1.0"),
            MOUNT
        ),
        Verdict::NotFromKit
    );
    // A folder that only starts with the same text is not the mount.
    let beside = format!("{MOUNT}-old/node");
    assert_eq!(
        verdict(&spec("node", None), Some(&beside), None, MOUNT),
        Verdict::NotFromKit
    );
    let claude = CheckSpec {
        from_image: true,
        ..spec("claude", None)
    };
    assert_eq!(
        verdict(&claude, Some("/usr/local/bin/claude"), Some("2.0.1"), MOUNT),
        Verdict::Ok
    );
    let apache = CheckSpec {
        program: None,
        ..spec("apache", None)
    };
    assert_eq!(verdict(&apache, None, None, MOUNT), Verdict::NotInVm);
}

/// `java -version` writes on stderr: the script takes it with `2>&1`, and
/// its version is read from the quotes.
#[test]
fn java_version_on_stderr_is_read() {
    let java = CheckSpec {
        args: vec!["-version".into()],
        ..spec("java", Some("21"))
    };
    let script = check_script(std::slice::from_ref(&java));
    assert!(
        script.contains("'java' '-version' 2>&1 | head -n 3"),
        "{script}"
    );
    let out = format!(
        "@@java\n{MOUNT}/java/21/bin/java\nopenjdk version \"21.0.5\" 2024-10-15\nOpenJDK Runtime Environment\nOpenJDK 64-Bit Server VM\n"
    );
    let checks = parse_check(&out, &[java], MOUNT);
    assert_eq!(checks[0].found.as_deref(), Some("21.0.5"));
    assert_eq!(checks[0].verdict, Verdict::Ok);
    assert_eq!(checks[0].output.lines().count(), 3);
}

/// The output of a check read back: a match, another line, a tool
/// missing, one outside the kit, one not meant for the VM.
#[test]
fn the_check_reads_match_mismatch_and_missing() {
    let specs = [
        spec("node", Some("22")),
        spec("python", Some("3.13")),
        spec("go", Some("1.23")),
        spec("php", None),
        CheckSpec {
            program: None,
            ..spec("apache", None)
        },
    ];
    let out = format!(
        "@@node\n{MOUNT}/node/22/bin/node\nv22.11.0\n\
         @@python\n{MOUNT}/python/3.12/bin/python3\nPython 3.12.7\n\
         @@go\n\n\
         @@php\n/usr/bin/php\nPHP 8.2.1 (cli)\n"
    );
    let checks = parse_check(&out, &specs, MOUNT);
    let verdicts: Vec<(&str, Verdict)> =
        checks.iter().map(|c| (c.id.as_str(), c.verdict)).collect();
    assert_eq!(
        verdicts,
        [
            ("node", Verdict::Ok),
            ("python", Verdict::OtherLine),
            ("go", Verdict::Missing),
            ("php", Verdict::NotFromKit),
            ("apache", Verdict::NotInVm),
        ]
    );
    assert_eq!(checks[1].chosen.as_deref(), Some("3.13"));
    assert_eq!(checks[1].found.as_deref(), Some("3.12.7"));
    assert_eq!(checks[2].path, None);
    assert_eq!(checks[2].found, None);
    assert_eq!(checks[0].command, "node --version");
    // A sandbox not running: never started, every tool missing.
    let off = not_running(&specs);
    assert!(off[..4].iter().all(|c| c.verdict == Verdict::Missing));
    assert_eq!(off[4].verdict, Verdict::NotInVm);
}

/// The check's script only reads: the kit's tools first on `PATH`, each
/// word quoted, no redirection but stderr into the pipe.
#[test]
fn the_check_script_only_reads() {
    let pi = CheckSpec {
        extra_path: Some("/d/kit/data/apps/linux-x64/pi".into()),
        ..spec("pi", None)
    };
    let script = check_script(&[spec("node", Some("22")), pi]);
    assert!(script.starts_with(vm::TOOLS_FIRST), "{script}");
    assert!(script.contains("printf '@@%s\\n' 'node'"), "{script}");
    assert!(script.contains("command -v 'node'"), "{script}");
    assert!(
        script.contains("(PATH=\"$PATH\":'/d/kit/data/apps/linux-x64/pi'; "),
        "{script}"
    );
    for line in script.lines() {
        assert!(
            line.replace("2>&1", "").find('>').is_none(),
            "a write: {line}"
        );
    }
    for word in ["rm ", "curl", "wget", "mkdir", "tee"] {
        assert!(!script.contains(word), "{word}: {script}");
    }
}

/// The check for real with a POSIX shell, on fake tools: one of the
/// chosen line in the kit's folder, one of another line, one missing.
#[cfg(unix)]
#[test]
fn the_check_runs_in_a_shell() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = TempDir::new("check-run");
    let mount = tmp.0.join("apps");
    let bin = mount.join("node").join("22").join("bin");
    std::fs::create_dir_all(&bin).unwrap();
    let fake = |name: &str, text: &str| {
        let file = bin.join(name);
        std::fs::write(&file, format!("#!/bin/sh\necho '{text}'\n")).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    fake("node", "v22.11.0");
    fake("python3", "Python 3.12.7");
    let specs = [
        spec("node", Some("22")),
        CheckSpec {
            program: Some("python3".into()),
            ..spec("python", Some("3.13"))
        },
        CheckSpec {
            program: Some("atollo-no-such-tool".into()),
            ..spec("go", Some("1.23"))
        },
    ];
    let out = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(check_script(&specs))
        .env("ATOLLO_PATH", &bin)
        .output()
        .unwrap();
    assert!(out.status.success());
    let checks = parse_check(
        &String::from_utf8_lossy(&out.stdout),
        &specs,
        &mount.display().to_string(),
    );
    let verdicts: Vec<Verdict> = checks.iter().map(|c| c.verdict).collect();
    assert_eq!(
        verdicts,
        [Verdict::Ok, Verdict::OtherLine, Verdict::Missing],
        "{checks:?}"
    );
}

/// What an action asked of the app, in order.
#[derive(Default)]
struct Calls {
    sessions: Vec<String>,
    calls: Mutex<Vec<String>>,
}

impl Calls {
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
    fn note(&self, call: String) {
        self.calls.lock().unwrap().push(call);
    }
}

impl VmEffects for Calls {
    fn sessions(&self, sandbox: &str) -> Vec<String> {
        self.note(format!("sessions {sandbox}"));
        self.sessions.clone()
    }
    fn end_sessions(&self, sandbox: &str) {
        self.note(format!("end_sessions {sandbox}"));
    }
    fn stop_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        self.note(format!("stop {sandbox}"));
        Ok(())
    }
    fn remove_sandbox(&self, sandbox: &str) -> anyhow::Result<()> {
        self.note(format!("remove {sandbox}"));
        Ok(())
    }
    fn prepare(&self, req: &Request, _facts: &Facts) -> anyhow::Result<()> {
        self.note(format!("prepare {:?}", req.origin));
        Ok(())
    }
}

/// §8.5: «Riavvia» and «Ricrea» list the sandbox's sessions, stop them,
/// then stop or remove it and prepare it again; «Aggiorna strumenti» only
/// prepares, its sessions left running.
#[test]
fn the_actions_stop_the_sessions_then_prepare() {
    let (_tmp, kit) = kit_with_shop("actions");
    let name = vm::name("shop");
    let req = sandbox_request(&kit, &kit.projects().join("shop"), None);
    assert_eq!(req.origin, Origin::ProjectTerminal);
    let facts = facts_of(&kit, &req, true);
    let fx = Calls {
        sessions: vec!["shop · bash".into(), "dev".into()],
        ..Calls::default()
    };
    let stopped = act(&fx, &req, &facts, VmAction::Restart).unwrap();
    assert_eq!(stopped, ["shop · bash", "dev"]);
    assert_eq!(
        fx.calls(),
        [
            format!("sessions {name}"),
            format!("end_sessions {name}"),
            format!("stop {name}"),
            "prepare ProjectTerminal".to_owned(),
        ]
    );
    let fx = Calls {
        sessions: vec!["dev".into()],
        ..Calls::default()
    };
    act(&fx, &req, &facts, VmAction::Recreate).unwrap();
    assert_eq!(
        fx.calls(),
        [
            format!("sessions {name}"),
            format!("end_sessions {name}"),
            format!("remove {name}"),
            "prepare ProjectTerminal".to_owned(),
        ]
    );
    let fx = Calls {
        sessions: vec!["dev".into()],
        ..Calls::default()
    };
    act(&fx, &req, &facts, VmAction::Refresh).unwrap();
    assert_eq!(
        fx.calls(),
        [
            format!("sessions {name}"),
            "prepare ProjectTerminal".to_owned()
        ]
    );
}

/// §8.5: `--agent claude` and the agent's row of the panel act on the
/// agent's own sandbox, decided as the agent's button.
#[test]
fn the_actions_take_the_agents_sandbox() {
    let (_tmp, kit) = kit_with_shop("actions-agent");
    for agent in [Agent::Claude, Agent::Codex] {
        let req = sandbox_request(&kit, &kit.projects().join("shop"), Some(agent));
        let facts = facts_of(&kit, &req, true);
        let fx = Calls::default();
        act(&fx, &req, &facts, VmAction::Restart).unwrap();
        let name = vm::agent_name("shop", agent.id());
        assert_eq!(
            fx.calls(),
            [
                format!("sessions {name}"),
                format!("stop {name}"),
                format!("prepare AgentButton {{ agent: {agent:?} }}"),
            ]
        );
    }
}

/// Every action goes through `decide`: the VM off, nothing is touched; an
/// agent's sandbox is refused with its reason and fix (R1).
#[test]
fn an_action_goes_through_decide() {
    let (_tmp, kit) = kit_with_shop("actions-decide");
    let dir = kit.projects().join("shop");
    let req = sandbox_request(&kit, &dir, None);
    let fx = Calls::default();
    let e = act(&fx, &req, &facts_of(&kit, &req, false), VmAction::Recreate).unwrap_err();
    assert!(matches!(e, ActError::NoVm), "{e:?}");
    assert!(act_error_text(&e, "it").contains("La VM non è attiva"));
    let req = sandbox_request(&kit, &dir, Some(Agent::Claude));
    let e = act(&fx, &req, &facts_of(&kit, &req, false), VmAction::Restart).unwrap_err();
    match e {
        ActError::Refused { reason, fix } => {
            assert!(matches!(reason, Refusal::AgentsNeedVm { .. }));
            assert_eq!(fix, Fix::OpenVmNotice);
        }
        other => panic!("{other:?}"),
    }
    let mut facts = facts_of(&kit, &req, true);
    facts.restart_pending = true;
    let e = act(&fx, &req, &facts, VmAction::Refresh).unwrap_err();
    assert!(matches!(
        e,
        ActError::Refused {
            reason: Refusal::DaemonRestartPending,
            ..
        }
    ));
    assert!(fx.calls().is_empty());
    // A check is decided too: off, nothing runs.
    let mut s = report(&kit, "shop", Seen::default(), &|_| Vec::new()).sandboxes[0].clone();
    let req = sandbox_request(&kit, &dir, None);
    let e = check(
        &kit,
        &Default::default(),
        &crate::env::KitEnv::build(&kit, &Default::default(), "it", &Default::default()),
        &req,
        &facts_of(&kit, &req, false),
        "shop",
        &mut s,
        "it",
    )
    .unwrap_err();
    assert!(matches!(e, ActError::NoVm));
    assert!(s.tools.is_none());
}

/// §8.2: a sandbox not running is never started by the check: its tools
/// are `missing`, with what to do.
#[test]
fn a_check_never_starts_a_sandbox() {
    let (_tmp, kit) = kit_with_shop("check-off");
    let dir = kit.projects().join("shop");
    let mut s = report(&kit, "shop", Seen::default(), &|_| Vec::new()).sandboxes[0].clone();
    assert_eq!(s.state, SandboxRun::Missing);
    let req = sandbox_request(&kit, &dir, None);
    check(
        &kit,
        &Default::default(),
        &crate::env::KitEnv::build(&kit, &Default::default(), "it", &Default::default()),
        &req,
        &facts_of(&kit, &req, true),
        "shop",
        &mut s,
        "it",
    )
    .unwrap();
    let tools = s.tools.unwrap();
    assert!(!tools.is_empty());
    assert!(tools
        .iter()
        .all(|c| matches!(c.verdict, Verdict::Missing | Verdict::NotInVm)));
    assert!(s.note.unwrap().contains("avvia il terminale"));
}

/// Two actions on one sandbox never run together: the second waits for
/// the sandbox's action lock.
#[test]
fn an_action_waits_for_the_sandboxs_lock() {
    let (_tmp, kit) = kit_with_shop("actions-lock");
    let req = sandbox_request(&kit, &kit.projects().join("shop"), Some(Agent::Codex));
    let facts = facts_of(&kit, &req, true);
    let lock = action_lock(&vm::agent_name("shop", "codex"));
    let held = lock.lock().unwrap();
    let fx = std::sync::Arc::new(Calls::default());
    let worker = {
        let (fx, req, facts) = (std::sync::Arc::clone(&fx), req.clone(), facts.clone());
        std::thread::spawn(move || act(&*fx, &req, &facts, VmAction::Refresh).map(drop))
    };
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert!(fx.calls().is_empty(), "it waited for the lock");
    drop(held);
    worker.join().unwrap().unwrap();
    assert_eq!(fx.calls().len(), 2);
}

/// What «Ricrea» says it removes and what it keeps.
#[test]
fn a_recreation_says_what_is_lost_and_what_stays() {
    let (lost, kept) = recreate_warning("it");
    assert!(lost.contains("home della VM"), "{lost}");
    assert!(lost.contains("immagini"), "{lost}");
    for word in ["progetto", "database", "strumenti", "accessi"] {
        assert!(kept.contains(word), "{word}: {kept}");
    }
    assert_eq!(VmAction::from_word("recreate"), Some(VmAction::Recreate));
    assert_eq!(VmAction::from_word("x"), None);
    assert!(!VmAction::Refresh.stops_sessions());
}

/// The sandboxes of a project: its own, then Claude Code's and Codex's;
/// Pi and Antigravity run in the project's.
#[test]
fn a_project_has_its_sandbox_and_one_per_agent_of_sbx() {
    let all = sandboxes_of("shop");
    assert_eq!(
        all,
        [
            (vm::name("shop"), None),
            (vm::agent_name("shop", "claude"), Some(Agent::Claude)),
            (vm::agent_name("shop", "codex"), Some(Agent::Codex)),
        ]
    );
}
