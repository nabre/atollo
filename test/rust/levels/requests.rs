//! The request files (LEVELS.md §4.3, §12.3): a request taken once and
//! answered, a malformed one removed and never run, the refusal with its
//! reason, fix and text, the PC's reply with only what changes, the VM's
//! with the user's words after the agent, Pi's marker, and the reply for
//! sh. With the fake effects of `levels::prepare`'s tests, on the kit of
//! the system that runs them.

use super::*;
use crate::isolation::{vm, Blocker};
use crate::levels::env::{pc_environment, LEVEL_VAR, PROJECT_VAR};
use crate::levels::prepare::tests::{facts_of, install, kit_with_shop, Fake};
use crate::levels::{Agent, Level, Origin};

const ID: &str = "0123456789abcdef0123456789abcdef";

/// Writes a request as a client does: `<id>.tmp`, renamed.
fn ask(kit: &Kit, id: &str, body: &str) {
    let root = dir(kit);
    std::fs::create_dir_all(&root).unwrap();
    let tmp = root.join(format!("{id}.tmp"));
    std::fs::write(&tmp, body).unwrap();
    std::fs::rename(&tmp, root.join(format!("{id}.json"))).unwrap();
}

fn body(origin: &Origin, cwd: &std::path::Path, args: &[&str]) -> String {
    serde_json::to_string(&serde_json::json!({
        "v": 1,
        "origin": origin,
        "cwd": cwd,
        "args": args,
    }))
    .unwrap()
}

fn base(kit: &Kit) -> KitEnv {
    KitEnv::build(kit, &Default::default(), "it", &Host::default())
}

/// The editor server's environment (`Editor::start_on`).
fn editor(kit: &Kit, base: &KitEnv) -> KitEnv {
    pc_environment(kit, base.clone(), Level::Manual, None, None)
}

fn request(kit: &Kit, origin: Origin, dir: &std::path::Path) -> Request {
    Request {
        origin,
        place: crate::levels::place::place_of(kit, dir),
    }
}

/// A request written by a client comes back as asked, once; its reply is
/// written for both clients, the JSON one as §4.3 shows it.
#[test]
fn a_request_round_trips() {
    let (_tmp, kit) = kit_with_shop("requests-round");
    let cwd = kit.projects().join("shop");
    let origin = Origin::Forwarded {
        agent: Agent::Claude,
        from: Level::Manual,
    };
    ask(&kit, ID, &body(&origin, &cwd, &["--resume", "it's"]));
    let taken = take(&kit);
    assert_eq!(
        taken,
        [Taken::Asked(Asked {
            id: ID.into(),
            origin: origin.clone(),
            cwd: cwd.clone(),
            args: vec!["--resume".into(), "it's".into()],
        })]
    );
    let Taken::Asked(asked) = &taken[0] else {
        unreachable!()
    };
    assert_eq!(request_of(&kit, asked).place.dir(), cwd);
    // The JSON of the PowerShell client, with its BOM if a PowerShell 5 wrote it.
    let ps = "\u{feff}{\"v\":1,\"origin\":{\"kind\":\"editorTask\",\"command\":\"npm test\"},\"cwd\":\"D:\\\\kit\\\\projects\\\\shop\\\\web\",\"args\":[]}";
    assert!(matches!(
        parse(ID, ps.as_bytes()),
        Taken::Asked(Asked { origin: Origin::EditorTask { ref command }, .. }) if command == "npm test"
    ));

    let reply = Reply::error("no");
    answer(&kit, ID, &reply).unwrap();
    let json = dir(&kit).join(format!("{ID}{REPLY_JSON}"));
    let sh = dir(&kit).join(format!("{ID}{REPLY_SH}"));
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(value["v"], 1);
    assert_eq!(value["placement"], "error");
    assert_eq!(value["text"], "no");
    assert!(std::fs::read_to_string(&sh)
        .unwrap()
        .contains("placement='error'\n"));
    // Replies are no requests; nothing half-written is left beside them.
    assert!(take(&kit).is_empty());
    let left: Vec<String> = std::fs::read_dir(dir(&kit))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(left.len(), 2, "{left:?}");
}

/// A request is taken once: a second look finds nothing, and one its
/// client took back before the app came is never answered.
#[test]
fn a_request_is_taken_once() {
    let (_tmp, kit) = kit_with_shop("requests-once");
    let cwd = kit.projects().join("shop");
    let origin = Origin::EditorAgentProfile { agent: Agent::Pi };
    ask(&kit, ID, &body(&origin, &cwd, &[]));
    assert_eq!(take(&kit).len(), 1);
    assert!(take(&kit).is_empty());
    let other = "fedcba9876543210fedcba9876543210";
    ask(&kit, other, &body(&origin, &cwd, &[]));
    std::fs::remove_file(dir(&kit).join(format!("{other}.json"))).unwrap();
    assert!(take(&kit).is_empty());
    // A client still writing it (`<id>.tmp`) is not asking yet.
    std::fs::write(dir(&kit).join(format!("{other}.tmp")), "{").unwrap();
    assert!(take(&kit).is_empty());
    assert!(dir(&kit).join(format!("{other}.tmp")).exists());
}

/// A file named like no request goes, unanswered; a request that cannot be
/// read, of another version, or of an origin that never asks (level 1, the
/// app's own buttons) goes too and is answered with an error: nothing is
/// decided or started for it.
#[test]
fn a_malformed_request_is_ignored_and_removed() {
    let (_tmp, kit) = kit_with_shop("requests-malformed");
    let cwd = kit.projects().join("shop");
    let root = dir(&kit);
    std::fs::create_dir_all(&root).unwrap();
    for name in [
        "shop.json",
        "0123456789ABCDEF0123456789ABCDEF.json",
        "x.json",
    ] {
        std::fs::write(root.join(name), "{}").unwrap();
    }
    assert!(take(&kit).is_empty());
    for name in ["shop.json", "x.json"] {
        assert!(!root.join(name).exists(), "{name}");
    }
    let bad = [
        "not json".to_owned(),
        body(
            &Origin::EditorTask {
                command: "x".into(),
            },
            &cwd,
            &[],
        )
        .replace("\"v\":1", "\"v\":2"),
        body(
            &Origin::KitTerminal {
                terminal: "default".into(),
            },
            &cwd,
            &[],
        ),
        body(&Origin::EditorTerminal, &cwd, &[]),
        body(
            &Origin::AgentButton {
                agent: Agent::Claude,
            },
            &cwd,
            &[],
        ),
        body(&Origin::ProjectTerminal, &cwd, &[]),
        r#"{"v":1,"origin":{"kind":"forwarded","agent":"gemini","from":"manual"},"cwd":"x"}"#
            .to_owned(),
    ];
    for text in &bad {
        ask(&kit, ID, text);
        match take(&kit).as_slice() {
            [Taken::Malformed { id, why }] => assert!(id == ID && !why.is_empty(), "{text}"),
            other => panic!("{text}: {other:?}"),
        }
        assert!(!root.join(format!("{ID}.json")).exists());
    }
    assert!(asks_by_request(&Origin::EditorExtension {
        extension: "anthropic.claude-code".into(),
        agent: Agent::Claude
    }));
}

/// A refusal says its reason, its fix and the text in the user's language
/// (I7): an agent without the VM (R1), an agent outside a project (R4).
#[test]
fn a_refused_reply_carries_reason_fix_and_text() {
    let (_tmp, kit) = kit_with_shop("requests-refused");
    let base = base(&kit);
    let fx = Fake::default();
    let shop = kit.projects().join("shop");
    let claude = Origin::Forwarded {
        agent: Agent::Claude,
        from: Level::Manual,
    };
    let reply_of = |origin: Origin, dir: &std::path::Path, on: bool| {
        let req = request(&kit, origin, dir);
        let facts = facts_of(&kit, &req, on);
        reply_to(
            &fx,
            &kit,
            &base,
            &editor(&kit, &base),
            &Host::default(),
            &req,
            &[],
            &facts,
            "it",
        )
    };

    let off = reply_of(claude.clone(), &shop, false);
    assert_eq!(off.placement, Where::Refused);
    assert_eq!(off.level, Some(Level::Agent));
    assert_eq!(
        off.refusal,
        Some(Refusal::AgentsNeedVm {
            blockers: vec![Blocker::HypervisorOff]
        })
    );
    assert_eq!(off.fix, Some(Fix::OpenVmNotice));
    assert!(
        off.text.as_deref().unwrap().contains("solo nella VM"),
        "{off:?}"
    );
    assert!(off.program.is_none() && off.argv.is_empty());
    let value = serde_json::to_value(&off).unwrap();
    assert_eq!(value["placement"], "refused");
    assert_eq!(value["level"], "agent");
    assert_eq!(value["refusal"]["code"], "agents_need_vm");
    assert_eq!(value["fix"], "open_vm_notice");

    let outside = reply_of(claude, &kit.root().join("elsewhere"), true);
    assert_eq!(outside.refusal, Some(Refusal::AgentOutsideProject));
    assert_eq!(outside.fix, Some(Fix::OpenFromProject));
    assert!(outside
        .text
        .as_deref()
        .unwrap()
        .contains("cartella di un progetto"));

    assert!(fx.calls().is_empty(), "nothing prepared for a refusal");
    let sh = reply_sh(&outside, false);
    assert!(
        sh.contains("placement='refused'\n") && sh.contains("set --\n"),
        "{sh}"
    );
}

/// An editor's task without the VM runs on the PC (R5) with the kit's
/// shell, and the reply says only what changes from the editor server's
/// environment: the project's level and name, and the user's SSH agent out
/// (level 2 keeps none of the sealed variables).
#[test]
fn a_pc_reply_carries_only_the_environment_difference() {
    let (_tmp, kit) = kit_with_shop("requests-pc");
    install(&kit, "pwsh");
    let mut base = base(&kit);
    base.set("SSH_AUTH_SOCK", "/tmp/agent.sock");
    let editor = editor(&kit, &base);
    assert_eq!(editor.get("SSH_AUTH_SOCK"), Some("/tmp/agent.sock"));
    let fx = Fake::default();
    let web = kit.projects().join("shop").join("web");
    std::fs::create_dir_all(&web).unwrap();
    let req = request(
        &kit,
        Origin::EditorTask {
            command: "npm test".into(),
        },
        &web,
    );
    let facts = facts_of(&kit, &req, false);
    let reply = reply_to(
        &fx,
        &kit,
        &base,
        &editor,
        &Host::default(),
        &req,
        &[],
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Pc, "{reply:?}");
    assert_eq!(reply.level, Some(Level::Project));
    let (program, argv) = task_on_pc(&kit, "npm test").unwrap();
    assert_eq!(reply.program.as_deref(), Some(program.as_str()));
    assert_eq!(reply.argv, argv);
    assert_eq!(reply.argv.last().map(String::as_str), Some("npm test"));
    assert_eq!(
        reply.env.set.get(LEVEL_VAR).map(String::as_str),
        Some("project")
    );
    assert_eq!(
        reply.env.set.get(PROJECT_VAR).map(String::as_str),
        Some("shop")
    );
    assert!(
        reply.env.unset.contains(&"SSH_AUTH_SOCK".to_owned()),
        "{:?}",
        reply.env
    );
    for (k, v) in &reply.env.set {
        assert_ne!(editor.get(k), Some(v.as_str()), "{k} is no change");
    }
    assert!(reply.banner.as_deref().unwrap().contains("PC"), "{reply:?}");
    assert!(fx.calls().is_empty(), "nothing prepared on the PC");

    // The kit's PowerShell on Windows, the system's bash elsewhere; a
    // Windows kit with neither it nor Git Bash runs no task (E18).
    let program = task_on_pc(&kit, "x").unwrap().0.replace('\\', "/");
    if kit.target().is_windows() {
        assert!(program.ends_with("pwsh/pwsh.exe"), "{program}");
        let (_t, bare) = crate::test_support::prepared_kit("requests-pc-bare");
        assert_eq!(task_on_pc(&bare, "x"), None);
    } else {
        assert_eq!(program, "/bin/bash");
    }
}

/// The words typed after the agent's command reach the agent, after its
/// own (Pi in the project's VM, R9): its marker named for the project.
#[test]
fn args_are_appended_after_the_agent() {
    let (_tmp, kit) = kit_with_shop("requests-args");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    install(&guest, "pi");
    let base = base(&kit);
    let editor = editor(&kit, &base);
    let web = kit.projects().join("shop").join("web");
    std::fs::create_dir_all(&web).unwrap();
    let fx = Fake::default();
    let req = request(
        &kit,
        Origin::Forwarded {
            agent: Agent::Pi,
            from: Level::Manual,
        },
        &web,
    );
    let facts = facts_of(&kit, &req, true);
    let typed = ["--continue".to_owned(), "it's \"mine\"".to_owned()];
    let reply = reply_to(
        &fx,
        &kit,
        &base,
        &editor,
        &Host::default(),
        &req,
        &typed,
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Vm, "{reply:?}");
    assert_eq!(reply.level, Some(Level::Agent));
    let n = reply.argv.len();
    assert_eq!(reply.argv[n - 2..], typed, "{:?}", reply.argv);
    assert!(reply.argv[n - 3].ends_with("/pi/pi"), "{:?}", reply.argv);
    assert!(reply.argv.contains(&vm::name("shop")));
    let w = reply.argv.iter().position(|a| a == "-w").unwrap();
    assert!(reply.argv[w + 1].ends_with("/shop/web"), "{:?}", reply.argv);
    assert!(reply
        .program
        .as_deref()
        .unwrap()
        .replace('\\', "/")
        .contains("sandboxes"));
    assert_eq!(reply.marker.as_deref(), Some("shop.pi"));
    assert_eq!(
        reply.env.set.get("SBX_NO_TELEMETRY").map(String::as_str),
        Some("1")
    );
    assert!(
        reply.banner.as_deref().unwrap().contains("pi in atollo-p-"),
        "{reply:?}"
    );
}

/// D-179 step 7, §6.8: `agy` (or `antigravity`) typed in a terminal of the
/// PC in a project folder is forwarded to Antigravity's build in the
/// project's VM, the words after it kept, no marker (Pi's only), no kit
/// key in the reply; outside a project it is refused with why and what to
/// do, never run on the PC.
#[test]
fn agy_is_forwarded_to_the_projects_vm() {
    let (_tmp, kit) = kit_with_shop("requests-agy");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    install(&guest, "antigravity");
    let base = base(&kit);
    let editor = editor(&kit, &base);
    let web = kit.projects().join("shop").join("web");
    std::fs::create_dir_all(&web).unwrap();
    let agent = Agent::from_command("agy").unwrap();
    assert_eq!(Agent::from_command("antigravity"), Some(agent));
    let origin = Origin::Forwarded {
        agent,
        from: Level::Manual,
    };
    let fx = Fake::default();
    let req = request(&kit, origin.clone(), &web);
    let facts = facts_of(&kit, &req, true);
    let typed = ["--resume".to_owned()];
    let reply = reply_to(
        &fx,
        &kit,
        &base,
        &editor,
        &Host::default(),
        &req,
        &typed,
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Vm, "{reply:?}");
    assert_eq!(reply.level, Some(Level::Agent));
    let n = reply.argv.len();
    assert_eq!(reply.argv[n - 1..], typed, "{:?}", reply.argv);
    assert!(
        reply.argv[n - 2].ends_with("/antigravity/agy"),
        "{:?}",
        reply.argv
    );
    assert!(reply.argv.contains(&vm::name("shop")), "the project's VM");
    assert!(reply
        .argv
        .contains(&"AGY_CLI_DISABLE_AUTO_UPDATE=true".to_owned()));
    let w = reply.argv.iter().position(|a| a == "-w").unwrap();
    assert!(reply.argv[w + 1].ends_with("/shop/web"), "{:?}", reply.argv);
    assert!(reply.marker.is_none(), "a marker is Pi's only");
    for k in reply.env.set.keys() {
        assert!(!k.contains("GEMINI") && !k.contains("GOOGLE"), "{k}");
    }
    assert!(
        reply
            .banner
            .as_deref()
            .unwrap()
            .contains("antigravity in atollo-p-"),
        "{reply:?}"
    );

    // Outside a project: refused (R8), never on the PC.
    let elsewhere = kit.root().join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let req = request(&kit, origin, &elsewhere);
    let facts = facts_of(&kit, &req, true);
    let reply = reply_to(
        &fx,
        &kit,
        &base,
        &editor,
        &Host::default(),
        &req,
        &typed,
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Refused, "{reply:?}");
    assert!(reply.program.is_none() && reply.argv.is_empty());
}

/// The reply for `launch.sh`: every field a quoted word, the variables in
/// `atollo_env` (names sh cannot set left out), the arguments with
/// `set --`; on Windows the program with `/` and Git Bash told not to
/// rewrite the VM's paths.
#[test]
fn the_reply_for_sh_sets_its_words() {
    let mut reply = Reply::error("x");
    reply.placement = Where::Vm;
    reply.program = Some(r"D:\kit\sbx.exe".into());
    reply.argv = vec!["exec".into(), "it's".into(), "$HOME".into()];
    reply.banner = Some("[Atollo] Agente · VM: pi in atollo-p-shop".into());
    reply.env.set.insert("SBX_NO_TELEMETRY".into(), "1".into());
    reply
        .env
        .set
        .insert("ProgramFiles(x86)".into(), "C:".into());
    reply.env.unset.push("SSH_AUTH_SOCK".into());
    let sh = reply_sh(&reply, true);
    assert!(sh.contains("program='D:/kit/sbx.exe'\n"), "{sh}");
    assert!(sh.contains("set -- 'exec' 'it'\\''s' '$HOME'\n"), "{sh}");
    assert!(sh.contains("    export 'SBX_NO_TELEMETRY=1'\n"), "{sh}");
    assert!(sh.contains("    unset 'SSH_AUTH_SOCK'\n"), "{sh}");
    assert!(sh.contains("MSYS_NO_PATHCONV=1"), "{sh}");
    assert!(!sh.contains("ProgramFiles"), "{sh}");
    let unix = reply_sh(&reply, false);
    assert!(unix.contains(r"program='D:\kit\sbx.exe'"), "{unix}");
    assert!(!unix.contains("MSYS"), "{unix}");
}

/// D-178: the clients' Pi sessions of a project, from their markers
/// (`<project>.pi.<pid>`, the pid digits only): never another project's, a
/// request, a reply or a folder; the app's start clears them with the rest.
#[test]
fn pi_sessions_are_the_clients_markers() {
    let names = [
        "shop.pi.4120",
        "shop.pi.77",
        "shop.pi.77",
        "shop.pi.",
        "shop.pi.12a",
        "shop.pi.+5",
        "shop.pi.5.pi.9",
        "shop.pi.request",
        "blog.pi.300",
        "shop",
        "xshop.pi.8",
    ];
    assert_eq!(pi_marker_pids(names, "shop"), [77, 4120]);
    assert_eq!(pi_marker_pids(names, "shop.pi.5"), [9]);
    assert_eq!(pi_marker_pids(names, "blog"), [300]);
    assert!(pi_marker_pids(names, "news").is_empty());

    let (_tmp, kit) = kit_with_shop("requests-pi");
    let root = dir(&kit);
    std::fs::create_dir_all(root.join("shop.pi.99")).unwrap();
    std::fs::write(root.join("shop.pi.4120"), "4120\n").unwrap();
    assert!(take(&kit).is_empty(), "not a request");
    assert_eq!(pi_markers(&kit, "shop"), [4120], "kept, files only");
    assert!(pi_markers(&kit, "blog").is_empty());
    clear_all(&kit).unwrap();
    assert!(!root.exists());
    assert!(pi_markers(&kit, "shop").is_empty());
    clear_all(&kit).unwrap();
}

/// LEVELS.md §8.6: a request from the editor or a forwarder that prepares
/// a VM reports its progress through the effects like the app's entry
/// points (the app makes it a job): it begins, its steps, its end.
#[test]
fn a_request_in_the_vm_reports_its_progress() {
    use crate::levels::prepare::PrepareProgress;
    let (_tmp, kit) = kit_with_shop("requests-progress");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    install(&guest, "pi");
    let base = base(&kit);
    let editor = editor(&kit, &base);
    let fx = Fake::default();
    let req = request(
        &kit,
        Origin::Forwarded {
            agent: Agent::Pi,
            from: Level::Manual,
        },
        &kit.projects().join("shop"),
    );
    let facts = facts_of(&kit, &req, true);
    let reply = reply_to(
        &fx,
        &kit,
        &base,
        &editor,
        &Host::default(),
        &req,
        &[],
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Vm, "{reply:?}");
    let events = fx.progress();
    assert!(
        matches!(
            events.first(),
            Some(PrepareProgress::Begin {
                agent: Some(Agent::Pi),
                ..
            })
        ),
        "{events:?}"
    );
    assert!(events
        .iter()
        .any(|e| matches!(e, PrepareProgress::Step { index: 1, .. })));
    assert_eq!(events.last(), Some(&PrepareProgress::Done));
}

/// LEVELS.md §6.9, Q2 (the spike, as far as it goes without a VM): the
/// Claude Code extension runs its wrapper as `<wrapper> <bundled binary>
/// <flags…>`; the bundled binary of the PC is dropped and the agent is the
/// sandbox's own `claude` (R12), the flags kept in their order. Other
/// origins keep every word; a first word that is a flag is kept too (the
/// path is passed only "when present").
#[test]
fn the_extensions_bundled_binary_is_dropped() {
    let ext = Origin::EditorExtension {
        extension: "anthropic.claude-code".into(),
        agent: Agent::Claude,
    };
    let words = |w: &[&str]| w.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let bundled = words(&[
        r"C:\Users\me\.vscode-oss\extensions\anthropic.claude-code-2.1.310-win32-x64\resources\native-binary\claude.exe",
        "--output-format",
        "stream-json",
        "--input-format",
        "stream-json",
        "--verbose",
    ]);
    assert_eq!(agent_args(&ext, &bundled), bundled[1..]);
    let unix = words(&["/home/me/.ext/claude", "-p", "it's \"mine\""]);
    assert_eq!(agent_args(&ext, &unix), ["-p", "it's \"mine\""]);
    let flags = words(&["--resume", "abc"]);
    assert_eq!(agent_args(&ext, &flags), flags, "no path: nothing dropped");
    assert!(agent_args(&ext, &[]).is_empty());
    let typed = Origin::Forwarded {
        agent: Agent::Claude,
        from: Level::Manual,
    };
    assert_eq!(
        agent_args(&typed, &unix),
        unix,
        "a forwarder keeps every word"
    );

    // The whole reply drops it too: the flags after the agent, no PC path.
    // With Pi's build in the project's VM, which the fake effects run to
    // the end (Claude's own sandbox asks the real `sbx` for its session).
    let (_tmp, kit) = kit_with_shop("requests-extension");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    install(&guest, "pi");
    let base = base(&kit);
    let ext = Origin::EditorExtension {
        extension: "anthropic.claude-code".into(),
        agent: Agent::Pi,
    };
    let req = request(&kit, ext, &kit.projects().join("shop"));
    let facts = facts_of(&kit, &req, true);
    let reply = reply_to(
        &Fake::default(),
        &kit,
        &base,
        &editor(&kit, &base),
        &Host::default(),
        &req,
        &bundled,
        &facts,
        "it",
    );
    assert_eq!(reply.placement, Where::Vm, "{reply:?}");
    assert_eq!(reply.level, Some(Level::Agent));
    let n = reply.argv.len();
    assert_eq!(reply.argv[n - 5..], bundled[1..], "{:?}", reply.argv);
    assert!(
        !reply.argv.iter().any(|a| a.contains("native-binary")),
        "{:?}",
        reply.argv
    );
    assert!(reply.argv[n - 6].ends_with("/pi/pi"), "{:?}", reply.argv);
}
