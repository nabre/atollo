//! The PC's entry points through the levels (LEVELS.md §5, step 3 of
//! §12.2): each command builds its request from a real folder, `decide`
//! answers with the facts given, a refusal comes back as the command's
//! error with its reason and fix, and a placement on the PC gets the
//! environment of `levels::env::environment`. On a kit of the system that
//! runs the tests ([`kit_arch`]), whatever it is.

use super::*;
use crate::editor::terminals::Terminal;
use crate::env::{Host, KitEnv};
use crate::isolation::Blocker;
use crate::levels::env::{forward_dir, LEVEL_VAR, PROJECT_VAR, SEALED, TERMINAL_VAR};
use crate::levels::prepare::tests::{install, shop_db, Fake};
use crate::levels::{Fix, Level, Origin, Place, Refusal, VmState};
use crate::test_support::{prepared_kit, TempDir};

/// A kit with the project `shop` (and its `web` folder).
fn kit_with_shop(name: &str) -> (TempDir, Kit) {
    let (tmp, kit) = prepared_kit(&format!("ipc-levels-{name}"));
    std::fs::create_dir_all(kit.projects().join("shop").join("web")).unwrap();
    (tmp, kit)
}

/// The kit's environment, with what the sealing takes away in it.
fn base_of(kit: &Kit) -> KitEnv {
    let mut base = KitEnv::build(kit, &Default::default(), "it", &Host::default());
    for name in SEALED {
        base.set(name, "x");
    }
    base
}

/// The facts of `req` with the VM on (or off, for a reason), the rest as
/// the kit has them.
fn facts_of(kit: &Kit, req: &Request, on: bool) -> Facts {
    Facts {
        vm: if on {
            VmState::On
        } else {
            VmState::Off {
                blockers: vec![Blocker::HypervisorOff],
            }
        },
        guest_dir: crate::levels::facts::guest_dir(kit, &req.place),
        restart_pending: false,
        linux_build: false,
        guest_tools: crate::levels::facts::guest_tools(kit),
    }
}

fn place(kit: &Kit, base: &KitEnv, req: &Request, facts: &Facts) -> CmdResult<Placed> {
    placed(kit, base, req, facts, None, "it", &Host::default())
}

/// [`ready`] with the fake effects of `levels::prepare`'s tests.
fn ready_with(
    fx: &Fake,
    kit: &Kit,
    base: &KitEnv,
    req: &Request,
    facts: &Facts,
    needs: &[String],
) -> CmdResult<Ready> {
    ready(fx, kit, base, req, facts, needs, "it", &Host::default())
}

/// The environment of a `Pc` answer, or the test fails.
fn on_the_pc(answer: CmdResult<Placed>) -> (Level, KitEnv) {
    match answer {
        Ok(Placed { placement, env }) => (placement.level(), env),
        other => panic!("not on the PC: {other:?}"),
    }
}

/// The refusal of an answer, as the UI receives it.
fn refusal_of<T: std::fmt::Debug>(answer: CmdResult<T>) -> (Refusal, Fix, String) {
    let e = answer.expect_err("refused");
    let (reason, fix) = e.refusal().expect("a refusal of the levels");
    (reason.clone(), fix, e.to_string())
}

fn first_on_path(kit: &Kit, env: &KitEnv) -> bool {
    env.get("PATH")
        .unwrap_or_default()
        .starts_with(&forward_dir(kit).display().to_string())
}

/// E1: a kit terminal in a project's subfolder is level 1 on the PC with
/// or without the VM (R3), at the project's lines, with the user's SSH
/// agent and nothing else of what level 2 loses.
#[test]
fn e1_a_kit_terminal_is_on_the_pc_with_or_without_the_vm() {
    let (_tmp, kit) = kit_with_shop("e1");
    let base = base_of(&kit);
    let terminal = Terminal {
        id: "default".into(),
        name: "Atollo".into(),
        dir: None,
        tools: Default::default(),
    };
    let dir = kit.projects().join("shop").join("web");
    let req = crate::ipc::terminals::kit_terminal_request(&kit, &terminal, &dir);
    assert_eq!(
        req.origin,
        Origin::KitTerminal {
            terminal: "default".into()
        }
    );
    assert!(matches!(&req.place, Place::Project { name, below, .. }
        if name == "shop" && below.as_deref() == Some("web")));
    for on in [true, false] {
        let facts = facts_of(&kit, &req, on);
        let answer = placed(
            &kit,
            &base,
            &req,
            &facts,
            Some(&terminal),
            "it",
            &Host::default(),
        );
        let Ok(Placed {
            placement: crate::levels::Placement::Pc { dir: at, .. },
            ..
        }) = &answer
        else {
            panic!("on the PC: {answer:?}");
        };
        assert_eq!(at, &dir);
        let (level, env) = on_the_pc(answer);
        assert_eq!(level, Level::Manual);
        assert_eq!(env.get(LEVEL_VAR), Some("manual"));
        assert_eq!(env.get(PROJECT_VAR), Some("shop"));
        assert_eq!(env.get(TERMINAL_VAR), Some("Atollo"));
        assert_eq!(env.get("SSH_AUTH_SOCK"), Some("x"), "level 1 keeps it");
        assert_eq!(env.get("ATOLLO_MYSQL_CNF"), None);
        assert!(first_on_path(&kit, &env));
    }
}

/// E3: the project terminal is refused while the kit's daemon must
/// restart (R9), in the VM when it is on (R14), on the PC sealed without
/// it (R5).
#[test]
fn e3_the_project_terminal_asks_decide() {
    let (_tmp, kit) = kit_with_shop("e3");
    let base = base_of(&kit);
    let req = crate::ipc::editing::project_terminal_request(&kit, &kit.projects().join("shop"));
    assert_eq!(req.origin, Origin::ProjectTerminal);

    let mut pending = facts_of(&kit, &req, true);
    pending.restart_pending = true;
    let (reason, fix, text) = refusal_of(place(&kit, &base, &req, &pending));
    assert_eq!(reason, Refusal::DaemonRestartPending);
    assert_eq!(fix, Fix::RestartDaemon);
    assert_eq!(
        text,
        crate::levels::labels::refusal_text(&Refusal::DaemonRestartPending, "it")
    );

    // With the VM on: never placed on the PC, made ready by
    // `levels::prepare` in the project's sandbox, its session the kit's
    // terminal in the project's folder in the VM (R14).
    let on = facts_of(&kit, &req, true);
    assert!(place(&kit, &base, &req, &on).is_err());
    install(&kit, crate::isolation::COMPONENT);
    let fx = Fake::default();
    let ready = ready_with(&fx, &kit, &base, &req, &on, &[]).unwrap();
    let vm = ready.vm.as_ref().expect("in the VM");
    assert_eq!(vm.name, crate::isolation::vm::name("shop"));
    assert!(fx.calls().iter().any(|c| c.starts_with("start_sandbox")));
    let args = crate::levels::start::vm_session_args(&kit, &ready, &[]).unwrap();
    // A shell takes no words after it: only an agent's command does.
    assert!(crate::levels::start::vm_session_args(&kit, &ready, &["x".into()]).is_err());
    assert_eq!(
        args,
        crate::isolation::vm::kit_terminal_args(
            &vm.name,
            on.guest_dir.as_deref().unwrap(),
            vm.tools()
        )
    );

    let (level, env) = on_the_pc(place(&kit, &base, &req, &facts_of(&kit, &req, false)));
    assert_eq!(level, Level::Project);
    assert_eq!(env.get(LEVEL_VAR), Some("project"));
    assert_eq!(env.get(PROJECT_VAR), Some("shop"));
    for name in SEALED {
        assert_eq!(env.get(name), None, "{name} sealed at level 2");
    }
    assert!(first_on_path(&kit, &env));
}

fn action(id: &str, kind: crate::detect::Kind, served: bool) -> crate::detect::Action {
    crate::detect::Action {
        id: id.into(),
        label: id.into(),
        command: if served {
            String::new()
        } else {
            "npm run dev".into()
        },
        kind,
        folder: None,
        url: None,
        env: Default::default(),
        static_dir: served.then(|| "public".into()),
        apache_dir: None,
        caddy_dir: None,
        server: None,
        domains: Vec::new(),
        aliases: Default::default(),
        custom: false,
    }
}

/// E5/E6: an action of the project is an `Action` (web when it listens), a
/// folder served by the kit a `Site`; on a folder the VM cannot reach it
/// is refused (R10), without the VM it gets the project's environment on
/// the PC (R5).
#[test]
fn e5_an_action_asks_decide() {
    let (_tmp, kit) = kit_with_shop("e5");
    let base = base_of(&kit);
    let dir = kit.projects().join("shop");
    let dev = action("dev", crate::detect::Kind::Web, false);
    let req = crate::ipc::runs::action_request(&kit, &dir, &dev);
    assert_eq!(
        req.origin,
        Origin::Action {
            action: "dev".into(),
            web: true
        }
    );
    let site = action("site", crate::detect::Kind::Web, true);
    assert_eq!(
        crate::ipc::runs::action_request(&kit, &dir, &site).origin,
        Origin::Site {
            action: "site".into()
        }
    );
    let tests = action("test", crate::detect::Kind::Command, false);
    assert_eq!(
        crate::ipc::runs::action_request(&kit, &dir, &tests).origin,
        Origin::Action {
            action: "test".into(),
            web: false
        }
    );

    let mut share = facts_of(&kit, &req, true);
    share.guest_dir = None;
    let (reason, fix, _) = refusal_of(place(&kit, &base, &req, &share));
    assert!(matches!(reason, Refusal::NoPathInVm { .. }), "{reason:?}");
    assert_eq!(fix, Fix::UseLocalDisk);

    let off = facts_of(&kit, &req, false);
    let placement = admitted(&req, &off, "it").unwrap();
    let env = pc_env(&kit, &base, &placement, None, "it", &Host::default()).unwrap();
    assert_eq!(env.get(LEVEL_VAR), Some("project"));
    assert_eq!(env.get(PROJECT_VAR), Some("shop"));
    assert_eq!(env.get("SSH_AUTH_SOCK"), None);
}

/// E6–E8: a pool (from the page, with the app, for a project's site) is
/// refused with the VM on (R2) and sealed for no project without it (R6).
#[test]
fn e7_a_pool_is_refused_with_the_vm_and_sealed_without() {
    let (_tmp, kit) = kit_with_shop("e7");
    let base = base_of(&kit);
    let req = crate::ipc::servers::pool_request(&kit, "caddy-2.10");
    assert_eq!(
        req.origin,
        Origin::Pool {
            pool: "caddy-2.10".into()
        }
    );
    assert!(matches!(req.place, Place::Folder { .. }), "{:?}", req.place);

    let (reason, fix, text) = refusal_of(place(&kit, &base, &req, &facts_of(&kit, &req, true)));
    assert_eq!(reason, Refusal::PoolsOnlyWithoutVm);
    assert_eq!(fix, Fix::StartFromProject);
    assert!(text.contains("server condivisi"), "{text}");

    let (level, env) = on_the_pc(place(&kit, &base, &req, &facts_of(&kit, &req, false)));
    assert_eq!(level, Level::Project);
    assert_eq!(env.get(LEVEL_VAR), Some("project"));
    assert_eq!(env.get(PROJECT_VAR), None);
    for name in SEALED {
        assert_eq!(env.get(name), None, "{name} sealed for a pool");
    }
    assert!(first_on_path(&kit, &env));
}

/// E9: a scaffold is decided in the new project's folder, made empty
/// first: refused while the daemon must restart (R9), on the PC with the
/// new project's environment without the VM (R5).
#[test]
fn e9_a_scaffold_is_decided_in_the_new_projects_folder() {
    let (_tmp, kit) = kit_with_shop("e9");
    let base = base_of(&kit);
    let dir = kit.projects().join("blog");
    std::fs::create_dir(&dir).unwrap();
    let req = crate::ipc::frameworks::scaffold_request(&kit, "laravel", &dir);
    assert_eq!(
        req.origin,
        Origin::Scaffold {
            base: "laravel".into()
        }
    );
    assert!(matches!(&req.place, Place::Project { name, .. } if name == "blog"));

    let mut pending = facts_of(&kit, &req, true);
    pending.restart_pending = true;
    let (reason, _, _) = refusal_of(place(&kit, &base, &req, &pending));
    assert_eq!(reason, Refusal::DaemonRestartPending);

    let (level, env) = on_the_pc(place(&kit, &base, &req, &facts_of(&kit, &req, false)));
    assert_eq!(level, Level::Project);
    assert_eq!(env.get(PROJECT_VAR), Some("blog"));
    assert_eq!(env.get("ATOLLO_MYSQL_CNF"), None);
}

/// E20: `atollo env --project` prints the project's environment on the PC
/// (`environment(Pc{Project})`), with its level.
#[test]
fn e20_the_cli_prints_the_projects_pc_environment() {
    let (_tmp, kit) = kit_with_shop("e20");
    let base = base_of(&kit);
    let env = crate::app::cli::project_env(&kit, &base, "shop", "it", &Host::default());
    assert_eq!(env.get(LEVEL_VAR), Some("project"));
    assert_eq!(env.get(PROJECT_VAR), Some("shop"));
    for name in SEALED {
        assert_eq!(env.get(name), None, "{name} sealed");
    }
    assert_eq!(
        env.get("PATH"),
        crate::levels::env::pc_environment(&kit, base, Level::Project, Some("shop"), None)
            .get("PATH"),
        "the project's environment on the PC"
    );
}

/// E4: with the VM on an action or a site is made ready by
/// `levels::prepare` in the project's sandbox (R16, R17): its tools, its
/// databases and their ports first, then the sandbox; a web action and a
/// site publish a port, a command does not.
#[test]
fn e4_an_action_in_the_vm_is_prepared() {
    let (_tmp, kit) = kit_with_shop("e4");
    install(&kit, crate::isolation::COMPONENT);
    let base = base_of(&kit);
    let dir = kit.projects().join("shop");
    let name = crate::isolation::vm::name("shop");
    for (a, publish) in [
        (action("dev", crate::detect::Kind::Web, false), true),
        (action("site", crate::detect::Kind::Web, true), true),
        (action("test", crate::detect::Kind::Command, false), false),
    ] {
        let req = crate::ipc::runs::action_request(&kit, &dir, &a);
        let fx = Fake {
            dbs: vec![shop_db()],
            ..Fake::default()
        };
        let ready = ready_with(&fx, &kit, &base, &req, &facts_of(&kit, &req, true), &[]).unwrap();
        assert_eq!(
            fx.calls(),
            [
                "ensure_guest_tools".to_owned(),
                "start_databases shop".to_owned(),
                format!("start_sandbox {name}"),
            ],
            "{}",
            a.id
        );
        match &ready.placement {
            crate::levels::Placement::Vm {
                publish: p,
                level: Level::Project,
                ..
            } => assert_eq!(*p, publish, "{}", a.id),
            other => panic!("{}: {other:?}", a.id),
        }
        assert_eq!(ready.vm.as_ref().unwrap().host_ports(), [3307]);
    }
}

/// E9 (new with step 4): with the VM on a creation runs in the new
/// project's sandbox (R15): made ready with the plan's tools and the kit's
/// PowerShell for the VM's Linux, each line an `sbx exec` of that
/// PowerShell in the project's folder in the VM, its variables in a file
/// (never on the command line), gone at the end.
#[test]
fn e9_a_scaffold_runs_in_the_new_projects_sandbox() {
    let (_tmp, kit) = kit_with_shop("e9-vm");
    install(&kit, crate::isolation::COMPONENT);
    let base = base_of(&kit);
    let dir = kit.projects().join("blog");
    std::fs::create_dir(&dir).unwrap();
    let req = crate::ipc::frameworks::scaffold_request(&kit, "laravel", &dir);
    let fx = Fake::default();
    let facts = facts_of(&kit, &req, true);
    let ready = ready_with(&fx, &kit, &base, &req, &facts, &["composer".to_owned()]).unwrap();
    let vm = ready.vm.as_ref().expect("in the VM");
    assert_eq!(vm.name, crate::isolation::vm::name("blog"));
    assert_eq!(vm.dir, dir, "the new project's folder, its workspace");
    {
        let wanted = fx.wanted.lock().unwrap();
        assert!(wanted.contains_key("pwsh") && wanted.contains_key("composer"));
    }

    let at = dir.join(crate::ipc::frameworks::VM_STAGING);
    std::fs::create_dir(&at).unwrap();
    let line = "composer create-project laravel/laravel:^13.0 'blog' --prefer-dist";
    let vars = [("ADMIN_PASSWORD".to_owned(), "s3cret-pw".to_owned())];
    let cmd = crate::levels::start::scaffold_command(&kit, &ready, line, &at, &vars).unwrap();
    assert_eq!(
        Some(cmd.get_program()),
        crate::isolation::kit_sbx(&kit)
            .as_deref()
            .map(|p| p.as_os_str())
    );
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let windows = kit.target().is_windows();
    let guest = kit.for_target(crate::isolation::vm::guest_target(kit.target()).unwrap());
    let pwsh = crate::isolation::vm::guest_path(
        windows,
        &crate::setup::kit_path(&guest, r"pwsh\pwsh.exe"),
    )
    .unwrap();
    let at_guest = crate::isolation::vm::guest_path(windows, &at).unwrap();
    assert_eq!(args[..3], ["exec", "-w", at_guest.as_str()]);
    assert!(args.contains(&vm.name));
    assert!(args.contains(&pwsh), "{args:?}");
    let encoded = crate::projects::frameworks::encoded_script(line);
    assert_eq!(args.last(), Some(&encoded));
    assert!(args.iter().all(|a| !a.contains("s3cret-pw")), "{args:?}");
    let file = crate::isolation::kit::dir(&kit, &vm.name).join(crate::levels::start::SCAFFOLD_ENV);
    let i = args.iter().position(|a| a == "--env-file").unwrap();
    assert_eq!(args[i + 1], file.display().to_string());
    assert!(std::fs::read_to_string(&file)
        .unwrap()
        .contains("ADMIN_PASSWORD=s3cret-pw\n"));
    crate::levels::start::scaffold_end(&kit, &ready);
    assert!(!file.exists());

    // Without the VM: the kit's PowerShell on the PC, in the folder given.
    let off = facts_of(&kit, &req, false);
    let ready = ready_with(&Fake::default(), &kit, &base, &req, &off, &[]).unwrap();
    let cmd = crate::levels::start::scaffold_command(&kit, &ready, line, &dir, &vars).unwrap();
    assert_eq!(
        cmd.get_program(),
        crate::setup::kit_path(&kit, r"pwsh\pwsh.exe").as_os_str()
    );
    assert_eq!(cmd.get_current_dir(), Some(dir.as_path()));
}

/// E10: an agent from the project's page is refused without the VM (R1),
/// with why and what to do; with it Claude Code and Codex are made ready in
/// a sandbox of their own (R12), Pi in the project's (R13).
#[test]
fn e10_an_agent_is_prepared_in_its_sandbox() {
    let (_tmp, kit) = kit_with_shop("e10");
    install(&kit, crate::isolation::COMPONENT);
    install(
        &kit.for_target(crate::isolation::vm::guest_target(kit.target()).unwrap()),
        "pi",
    );
    let base = base_of(&kit);
    let dir = kit.projects().join("shop");
    assert!(crate::ipc::agents::agent_request(&kit, "gemini", &dir).is_err());
    for (id, sandbox) in [
        ("claude", crate::isolation::vm::agent_name("shop", "claude")),
        ("codex", crate::isolation::vm::agent_name("shop", "codex")),
        ("pi", crate::isolation::vm::name("shop")),
    ] {
        let req = crate::ipc::agents::agent_request(&kit, id, &dir).unwrap();
        assert_eq!(req.origin.level(), Level::Agent);
        let fx = Fake::default();
        let off = facts_of(&kit, &req, false);
        let (reason, fix, _) = refusal_of(ready_with(&fx, &kit, &base, &req, &off, &[]));
        assert!(matches!(reason, Refusal::AgentsNeedVm { .. }), "{reason:?}");
        assert_eq!(fix, Fix::OpenVmNotice);
        assert!(fx.calls().is_empty(), "nothing prepared without the VM");

        let mut on = facts_of(&kit, &req, true);
        on.linux_build = crate::levels::facts::linux_build(&kit, &req.origin);
        let ready = ready_with(&fx, &kit, &base, &req, &on, &[]).unwrap();
        assert_eq!(ready.vm.as_ref().unwrap().name, sandbox, "{id}");
        assert_eq!(ready.placement.level(), Level::Agent);
        assert!(
            fx.calls().contains(&format!("start_sandbox {sandbox}")),
            "{id}"
        );
    }
}

/// LEVELS.md §8.6: the app makes a VM's preparation a job: titled with the
/// project (and the agent), its steps, the image's download said in the
/// step that starts the sandbox, a tool's download followed, its end.
#[test]
fn a_vms_preparation_is_a_job() {
    use crate::common::jobs::{JobKind, JobState, Jobs};
    use crate::levels::prepare::PrepareProgress;
    let jobs = Jobs::new();
    let vm = VmJob::default();
    vm.tell(
        &jobs,
        PrepareProgress::Begin {
            project: "shop".into(),
            sandbox: "atollo-p-shop".into(),
            agent: None,
            total: 3,
        },
        "it",
    );
    let job = jobs.list().pop().unwrap();
    assert_eq!(job.kind, JobKind::VmPrepare);
    assert_eq!(job.title, "Preparazione VM · shop");
    assert_eq!(job.project.as_deref(), Some("shop"));
    vm.tell(
        &jobs,
        PrepareProgress::Step {
            index: 1,
            total: 3,
            label: "Strumenti per il Linux della VM".into(),
        },
        "it",
    );
    let mut snapshot = crate::setup::task::Snapshot::default();
    snapshot.steps.push(crate::setup::task::Step {
        id: "node".into(),
        name: "Node.js".into(),
        done: Some(7),
        size: Some(10),
        ..Default::default()
    });
    vm.follow(&jobs, &snapshot);
    let now = jobs.get(&job.id).unwrap();
    assert_eq!(now.note.as_deref(), Some("Node.js"));
    assert_eq!(now.bytes.map(|b| (b.done, b.total)), Some((7, Some(10))));
    vm.tell(
        &jobs,
        PrepareProgress::Step {
            index: 3,
            total: 3,
            label: "Avvio la VM".into(),
        },
        "it",
    );
    vm.relabel(&jobs, "Scarico l'immagine della VM…".into());
    let now = jobs.get(&job.id).unwrap();
    let step = now.step.unwrap();
    assert_eq!(
        (step.index, step.label.as_str()),
        (3, "Scarico l'immagine della VM…")
    );
    assert_eq!(now.steps.len(), 2, "the same step, said again");
    vm.tell(&jobs, PrepareProgress::Done, "it");
    assert_eq!(jobs.get(&job.id).unwrap().state, JobState::Done);

    vm.tell(
        &jobs,
        PrepareProgress::Begin {
            project: "shop".into(),
            sandbox: "atollo-a-claude-shop".into(),
            agent: Some(crate::levels::Agent::Claude),
            total: 3,
        },
        "it",
    );
    let agent = jobs.list().pop().unwrap();
    assert_eq!(agent.title, "Preparazione VM · shop · claude");
    vm.tell(
        &jobs,
        PrepareProgress::Failed {
            error: "sbx: no".into(),
        },
        "it",
    );
    let agent = jobs.get(&agent.id).unwrap();
    assert_eq!(agent.state, JobState::Error);
    assert_eq!(agent.error.as_deref(), Some("sbx: no"));
}
