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

/// The environment of a `Pc` answer, or the test fails.
fn on_the_pc(answer: CmdResult<Placed>) -> (Level, KitEnv) {
    match answer {
        Ok(Placed::Pc { placement, env, .. }) => (placement.level(), env),
        other => panic!("not on the PC: {other:?}"),
    }
}

/// The refusal of an answer, as the UI receives it.
fn refusal_of(answer: CmdResult<Placed>) -> (Refusal, Fix, String) {
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
        let Ok(Placed::Pc { dir: at, .. }) = &answer else {
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

    let on = facts_of(&kit, &req, true);
    match place(&kit, &base, &req, &on) {
        Ok(Placed::Vm { workdir }) => assert_eq!(Some(workdir), on.guest_dir),
        other => panic!("in the VM: {other:?}"),
    }

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
        crate::versions::project_env(&kit, "shop", base).get("PATH"),
        "the same as the old wrapper"
    );
}
