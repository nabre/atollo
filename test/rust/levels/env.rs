//! The one environment builder (LEVELS.md §3.5, §12.3), on the four
//! systems' layouts (Windows' `.exe` and `;`, macOS and Linux's folders and
//! `:`), whatever system runs the tests.

use super::*;
use crate::common::fsx::join_rel;
use crate::editor::terminals::{self, Terminal};
use crate::env::{Host, KitEnv};
use crate::kit::{Kit, Target};
use crate::levels::{decide, facts, Agent, Facts, Origin, Place, Request, VmState};
use crate::setup::catalog::translate_rel;
use crate::setup::{Catalog, Component};
use crate::test_support::TempDir;
use crate::versions::{install_dir, saved_dir, set_project_choice, write_version};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const TARGETS: [Target; 4] = [
    Target::WIN32_X64,
    Target::DARWIN_ARM64,
    Target::LINUX_X64,
    Target::LINUX_ARM64,
];

/// What the kit holds for the user or itself (LEVELS.md §11): never in a
/// VM's environment.
const SECRETS: [(&str, &str); 6] = [
    ("CLAUDE_CODE_OAUTH_TOKEN", "KIT-SECRET-claude"),
    ("GEMINI_API_KEY", "KIT-SECRET-gemini"),
    ("GH_TOKEN", "KIT-SECRET-gh"),
    ("LLAMA_API_KEY", "KIT-SECRET-llama"),
    ("ATOLLO_MYSQL_CNF", "KIT-SECRET-root-mysql.cnf"),
    ("ATOLLO_EDITOR_TOKEN", "KIT-SECRET-editor"),
];

fn component(id: &str) -> &'static Component {
    Catalog::get().find(id).unwrap()
}

/// The key file of `c` in `dir` (a folder of one of its lines), in the
/// layout of `kit`'s system.
fn fake_in(kit: &Kit, c: &Component, dir: &Path) {
    let key = translate_rel(kit.target(), &c.key);
    let rel = key.strip_prefix(c.folder.as_deref().unwrap()).unwrap();
    let key = join_rel(dir, rel);
    std::fs::create_dir_all(key.parent().unwrap()).unwrap();
    std::fs::write(key, b"").unwrap();
}

/// `id` installed in its default folder of `kit`.
fn install(kit: &Kit, id: &str) {
    for key in component(id).key_paths(kit) {
        std::fs::create_dir_all(key.parent().unwrap()).unwrap();
        std::fs::write(&key, b"").unwrap();
    }
}

/// A kit of `target` with Node.js and the four agents installed, for the
/// PC and for the VM's Linux.
fn kit_of(target: Target, name: &str) -> (TempDir, Kit) {
    let tmp = TempDir::new(&format!("levels-env-{name}-{}", target.as_str()));
    let kit = Kit::at_target(tmp.0.clone(), target);
    kit.prepare().unwrap();
    let guest = kit.for_target(crate::isolation::vm::guest_target(target).unwrap());
    for k in [&kit, &guest] {
        for id in ["node", "claude", "codex", "antigravity", "pi"] {
            install(k, id);
        }
    }
    (tmp, kit)
}

/// The kit's environment, with what the sealing takes away and the kit's
/// secrets in it.
fn base_of(kit: &Kit) -> KitEnv {
    let mut base = KitEnv::build(kit, &Default::default(), "it", &Host::default());
    for name in SEALED {
        base.set(name, "x");
    }
    for (name, value) in SECRETS {
        base.set(name, value);
    }
    base
}

/// A user terminal that chose the agents too (they never reach it).
fn mine() -> Terminal {
    Terminal {
        id: "mine".into(),
        name: "Mine".into(),
        dir: None,
        tools: [("claude", None), ("pi", None), ("node", None)]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
    }
}

/// Every origin of LEVELS.md §3.2.
fn origins() -> Vec<Origin> {
    vec![
        Origin::KitTerminal {
            terminal: "default".into(),
        },
        Origin::KitTerminal {
            terminal: "mine".into(),
        },
        Origin::EditorTerminal,
        Origin::ProjectTerminal,
        Origin::EditorTask {
            command: "npm test".into(),
        },
        Origin::Action {
            action: "dev".into(),
            web: true,
        },
        Origin::Site {
            action: "web".into(),
        },
        Origin::Pool { pool: "p".into() },
        Origin::Scaffold { base: "b".into() },
        Origin::AgentButton {
            agent: Agent::Claude,
        },
        Origin::EditorAgentProfile {
            agent: Agent::Codex,
        },
        Origin::Forwarded {
            agent: Agent::Pi,
            from: Level::Manual,
        },
        Origin::EditorExtension {
            extension: "x".into(),
            agent: Agent::Antigravity,
        },
    ]
}

/// One placement of every shape `decide` gives for `kit`: every origin in
/// a project, a project's subfolder and another folder, with the VM on and
/// off; with the terminal each one gets.
fn shapes(kit: &Kit) -> Vec<(Placement, Option<Terminal>)> {
    let shop = kit.projects().join("shop");
    std::fs::create_dir_all(&shop).unwrap();
    let places = [
        Place::Project {
            name: "shop".into(),
            dir: shop.clone(),
            below: None,
        },
        Place::Project {
            name: "shop".into(),
            dir: shop,
            below: Some("web".into()),
        },
        Place::Folder { dir: kit.data() },
    ];
    let mut out = Vec::new();
    for origin in origins() {
        let terminal = match &origin {
            Origin::KitTerminal { terminal } if terminal == "mine" => Some(mine()),
            Origin::KitTerminal { .. } => terminals::find(kit, "default", "it"),
            _ => None,
        };
        for place in &places {
            for vm in [VmState::On, VmState::Off { blockers: vec![] }] {
                let req = Request {
                    origin: origin.clone(),
                    place: place.clone(),
                };
                let facts = Facts {
                    vm,
                    guest_dir: facts::guest_dir(kit, place),
                    restart_pending: false,
                    linux_build: true,
                    guest_tools: facts::guest_tools(kit),
                };
                out.push((decide(&req, &facts), terminal.clone()));
            }
        }
    }
    out
}

fn build(kit: &Kit, base: &KitEnv, p: &Placement, t: Option<&Terminal>) -> Option<Environment> {
    environment(kit, base, p, t, "it", &Host::default())
}

/// `env`'s variables, for comparing two environments.
fn vars(env: &KitEnv) -> BTreeMap<String, String> {
    env.iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect()
}

/// `p` compared as the PC's file system of `kit` does (`\` and `/` alike).
fn key(p: &str) -> String {
    crate::platform::processes::path_key(&p.replace('/', "\\"))
}

/// `entry` is one of `dirs` or inside one.
fn inside(entry: &str, dirs: &[PathBuf]) -> bool {
    let e = key(entry);
    dirs.iter().any(|d| {
        let d = key(&d.display().to_string());
        e == d || e.starts_with(&format!("{d}\\"))
    })
}

/// I4: no `PATH` entry of `env` is an agent's folder of `kit` or inside
/// one, nor a variable pointing there; Node.js is still there.
fn assert_no_agents(kit: &Kit, env: &KitEnv, what: &str) {
    let dirs = agent_dirs(kit);
    for entry in env.path_entries() {
        assert!(!inside(entry, &dirs), "{what}: {entry} is an agent's");
    }
    for (name, value) in env.iter() {
        assert!(
            name.eq_ignore_ascii_case("PATH") || !inside(value, &dirs),
            "{what}: {name} points to an agent"
        );
    }
    assert!(
        env.path_entries().iter().any(|e| key(e).contains("node")),
        "{what}: the other tools stay"
    );
    assert_eq!(env.get("CP_ANTIGRAVITY"), None, "{what}");
}

fn pc_env(env: Option<Environment>, what: &str) -> KitEnv {
    match env {
        Some(Environment::Pc(env)) => env,
        other => panic!("{what}: {other:?}"),
    }
}

/// I4: on the four systems, every placement on the PC has no agent's
/// folder on its `PATH`, although the kit's own environment has them.
#[test]
fn a_pc_environment_has_no_agent_folder() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "no-agents");
        let base = base_of(&kit);
        let apps = |rel: &str| kit.apps().join(rel).display().to_string();
        assert!(base.path_entries().contains(&apps("claude").as_str()));
        assert!(base.get("CP_ANTIGRAVITY").is_some());
        let mut seen = 0;
        for (p, t) in shapes(&kit) {
            if let Placement::Pc { .. } = p {
                let what = format!("{} {p:?}", target.as_str());
                let env = pc_env(build(&kit, &base, &p, t.as_ref()), &what);
                assert_no_agents(&kit, &env, &what);
                seen += 1;
            }
        }
        assert!(seen > 0, "{}", target.as_str());
        // No agent among the tools a kit terminal offers.
        assert!(terminals::tools(&kit)
            .iter()
            .all(|t| Agent::from_id(&t.id).is_none()));
    }
}

/// The folder of the agents' commands (the stand-ins until step 5 of
/// §12.2, then the forwarders) first on every PC `PATH`, once.
#[test]
fn the_forwarders_come_first() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "forward");
        let base = base_of(&kit);
        let first = forward_dir(&kit).display().to_string();
        for (p, t) in shapes(&kit) {
            if let Placement::Pc { .. } = p {
                let what = format!("{} {p:?}", target.as_str());
                let env = pc_env(build(&kit, &base, &p, t.as_ref()), &what);
                let entries = env.path_entries();
                assert_eq!(entries.first().copied(), Some(first.as_str()), "{what}");
                assert_eq!(entries.iter().filter(|e| **e == first).count(), 1, "{what}");
            }
        }
    }
}

/// A kit of `target` with Node.js on 22.11 (the default), 20.18 and 18.20,
/// Python on 3.13 (the default) and 3.12; the kit's environment on its
/// lines, with a `PATH` of their folders.
struct Lines {
    _tmp: TempDir,
    kit: Kit,
    base: KitEnv,
    node: [PathBuf; 3],
    python: [PathBuf; 2],
}

fn lines_of(target: Target, name: &str) -> Lines {
    let tmp = TempDir::new(&format!("levels-env-{name}-{}", target.as_str()));
    let kit = Kit::at_target(tmp.0.clone(), target);
    kit.prepare().unwrap();
    let (node, python) = (component("node"), component("python"));
    let node_main = install_dir(&kit, node, "v22.11.0");
    fake_in(&kit, node, &node_main);
    node.save_version(&kit, "v22.11.0").unwrap();
    let mut node_dirs = vec![node_main];
    for (line, version) in [("20.18", "v20.18.1"), ("18.20", "v18.20.5")] {
        let dir = saved_dir(&kit, node, line);
        fake_in(&kit, node, &dir);
        write_version(&dir, version).unwrap();
        node_dirs.push(dir);
    }
    let py_main = install_dir(&kit, python, "3.13.1");
    fake_in(&kit, python, &py_main);
    python.save_version(&kit, "3.13.1").unwrap();
    let py_old = saved_dir(&kit, python, "3.12");
    fake_in(&kit, python, &py_old);
    write_version(&py_old, "3.12.8").unwrap();
    assert!(crate::versions::kit_choice(&kit).is_empty());

    let mut base = KitEnv::build(&kit, &Default::default(), "it", &Host::default());
    let path = [&node_dirs[0], &py_main]
        .map(|d| d.display().to_string())
        .join(&base.path_sep().to_string());
    base.set("PATH", path);
    Lines {
        _tmp: tmp,
        kit,
        base,
        node: node_dirs.try_into().unwrap(),
        python: [py_main, py_old],
    }
}

impl Lines {
    /// A user terminal on Node.js 20.18 and Python 3.12.
    fn terminal() -> Terminal {
        Terminal {
            id: "old".into(),
            name: "Old".into(),
            dir: None,
            tools: BTreeMap::from([
                ("node".to_owned(), Some("20.18".to_owned())),
                ("python".to_owned(), Some("3.12".to_owned())),
            ]),
        }
    }

    /// The kit terminal `t` opened in the project `shop` (on Node.js 18.20).
    fn in_project(&self, t: &Terminal) -> KitEnv {
        set_project_choice(&self.kit, "shop", "node", Some("18.20")).unwrap();
        let p = Placement::Pc {
            level: Level::Manual,
            project: Some("shop".into()),
            dir: self.kit.projects().join("shop"),
        };
        pc_env(build(&self.kit, &self.base, &p, Some(t)), "in a project")
    }

    /// The `PATH` entries of `env` past the first (the agents' commands).
    fn path(env: &KitEnv) -> Vec<String> {
        env.path_entries()
            .into_iter()
            .skip(1)
            .map(str::to_owned)
            .collect()
    }
}

/// Q1 (decided 26/10/2026): in a project's folder a kit terminal has the
/// tools the project chose at the project's lines, the default terminal
/// and a user terminal alike (§13 point 1: today's terminals ignored them).
#[test]
fn the_project_versions_win_in_a_project_folder() {
    for target in TARGETS {
        let l = lines_of(target, "project-lines");
        let s = |p: &PathBuf| p.display().to_string();
        let what = target.as_str();
        let default = terminals::find(&l.kit, "default", "it").unwrap();
        let env = l.in_project(&default);
        assert_eq!(
            Lines::path(&env),
            [s(&l.node[2]), s(&l.python[0])],
            "{what}"
        );
        let env = l.in_project(&Lines::terminal());
        assert_eq!(Lines::path(&env)[0], s(&l.node[2]), "{what}");
        assert_eq!(env.get(PROJECT_VAR), Some("shop"), "{what}");
        assert_eq!(env.get(LEVEL_VAR), Some("manual"), "{what}");
        assert_eq!(env.get(TERMINAL_VAR), Some("Old"), "{what}");
        // A project's program (level 2) on the same lines.
        let p = Placement::Pc {
            level: Level::Project,
            project: Some("shop".into()),
            dir: l.kit.projects().join("shop"),
        };
        let env = pc_env(build(&l.kit, &l.base, &p, None), what);
        assert_eq!(
            Lines::path(&env),
            [s(&l.node[2]), s(&l.python[0])],
            "{what}"
        );
        assert_eq!(env.get(LEVEL_VAR), Some("project"), "{what}");
        assert_eq!(env.get(TERMINAL_VAR), None, "{what}");
    }
}

/// Q1: the tools the project did not choose keep the terminal's lines; out
/// of a project, the terminal's lines for all.
#[test]
fn a_user_terminal_keeps_its_other_lines() {
    for target in TARGETS {
        let l = lines_of(target, "terminal-lines");
        let s = |p: &PathBuf| p.display().to_string();
        let what = target.as_str();
        let env = l.in_project(&Lines::terminal());
        assert_eq!(
            Lines::path(&env),
            [s(&l.node[2]), s(&l.python[1])],
            "{what}"
        );
        let p = Placement::Pc {
            level: Level::Manual,
            project: None,
            dir: l.kit.data(),
        };
        let env = pc_env(build(&l.kit, &l.base, &p, Some(&Lines::terminal())), what);
        assert_eq!(
            Lines::path(&env),
            [s(&l.node[1]), s(&l.python[1])],
            "{what}"
        );
        assert_eq!(env.get(PROJECT_VAR), None, "{what}");
    }
}

/// Levels 1 and 2 on the PC never get the user's profile, SSH keys,
/// session bus or the kit's MySQL root (D-167); in a project, the
/// project's own login instead.
#[test]
fn sealed_variables_never_reach_levels_1_and_2() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "sealed");
        crate::db::auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
        let base = base_of(&kit);
        for (p, t) in shapes(&kit) {
            let Placement::Pc { project, .. } = &p else {
                continue;
            };
            let what = format!("{} {p:?}", target.as_str());
            let env = pc_env(build(&kit, &base, &p, t.as_ref()), &what);
            for name in SEALED {
                let value = env.get(name);
                if name == "ATOLLO_MYSQL_CNF" && project.is_some() {
                    let cnf = std::fs::read_to_string(value.expect("the project's login")).unwrap();
                    assert!(cnf.contains("user=shop\n"), "{what}");
                } else {
                    assert_eq!(value, None, "{what}: {name}");
                }
            }
        }
    }
}

/// `seal` alone: the sealed variables out, the project's login in; it
/// touches nothing else.
#[test]
fn a_project_gets_a_sealed_environment() {
    let (_tmp, kit) = crate::test_support::prepared_kit("levels-seal");
    let full = || {
        let mut env = KitEnv::default();
        for name in SEALED {
            env.set(name, "x");
        }
        env.set("PATH", "kit");
        env
    };
    let mut env = full();
    seal(&kit, None, &mut env);
    for name in SEALED {
        assert_eq!(env.get(name), None, "{name}");
    }
    assert_eq!(env.path_entries(), ["kit"]);

    crate::db::auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
    let mut env = full();
    seal(&kit, Some("shop"), &mut env);
    let cnf = env.get("ATOLLO_MYSQL_CNF").expect("the project's login");
    assert!(std::fs::read_to_string(cnf)
        .unwrap()
        .contains("user=shop\n"));
    assert_eq!(env.get("CP_USER_PROFILE"), None);
    assert_eq!(env.get("SSH_AUTH_SOCK"), None);
}

fn vm_env(env: Option<Environment>, what: &str) -> (KitEnv, crate::isolation::GuestTools) {
    match env {
        Some(Environment::Vm { sbx_env, guest }) => (sbx_env, guest),
        other => panic!("{what}: {other:?}"),
    }
}

/// The kit's Linux tools as the VM sees them, and whether the PC's folder
/// of the kit has a path in the VM (not a Windows kit's folder on another
/// system that runs the tests).
fn guest_apps(kit: &Kit) -> (Kit, Option<String>) {
    let guest = kit.for_target(crate::isolation::vm::guest_target(kit.target()).unwrap());
    let apps = crate::isolation::vm::guest_path(kit.target().is_windows(), &guest.apps());
    (guest, apps)
}

/// A VM placement gets only the kit's Linux tools (`PATH` entries and
/// variables in `data/apps/linux-…`, as the VM sees them) and the level;
/// none of the PC's, no agent's but its own; `sbx` runs with the daemon's
/// variables.
#[test]
fn a_vm_environment_has_only_the_kits_linux_tools() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "vm");
        let base = base_of(&kit);
        let (guest_kit, apps) = guest_apps(&kit);
        let agents: Vec<String> = agent_dirs(&guest_kit)
            .iter()
            .filter_map(|d| crate::isolation::vm::guest_path(kit.target().is_windows(), d))
            .collect();
        let mut seen = 0;
        for (p, t) in shapes(&kit) {
            let Placement::Vm { level, program, .. } = &p else {
                continue;
            };
            seen += 1;
            let what = format!("{} {p:?}", target.as_str());
            let (sbx_env, guest) = vm_env(build(&kit, &base, &p, t.as_ref()), &what);
            let apps = apps
                .as_deref()
                .expect("a VM only where the kit has a path in it");
            assert!(
                guest.path.iter().all(|e| Path::new(e).starts_with(apps)),
                "{what}: {:?}",
                guest.path
            );
            assert!(
                guest.path.iter().any(|e| e.contains("node")),
                "{what}: {:?}",
                guest.path
            );
            // No agent's folder, but the agent's own for its own session.
            let own = match program {
                VmProgram::AgentBinary { agent } => crate::isolation::vm::guest_path(
                    kit.target().is_windows(),
                    &guest_kit.apps().join(agent.id()),
                ),
                _ => None,
            };
            for entry in &guest.path {
                let agent = agents.iter().find(|a| {
                    entry == *a
                        || entry.starts_with(&format!("{a}/"))
                        || entry.starts_with(&format!("{a}\\"))
                });
                if let Some(agent) = agent {
                    assert_eq!(Some(agent), own.as_ref(), "{what}: {entry}");
                }
            }
            let (levels, others): (Vec<_>, Vec<_>) =
                guest.vars.iter().partition(|(name, _)| name == LEVEL_VAR);
            assert_eq!(
                levels,
                [&(LEVEL_VAR.to_owned(), level_value(*level).to_owned())],
                "{what}"
            );
            assert!(
                others.iter().all(|(_, v)| Path::new(v).starts_with(apps)),
                "{what}: {others:?}"
            );
            assert_eq!(guest.env_file, None, "{what}: set by prepare");
            for (name, value) in crate::isolation::daemon::ENV {
                assert_eq!(sbx_env.get(name), Some(value), "{what}");
            }
            // `sbx` runs on the PC with the kit's environment.
            assert_eq!(sbx_env.get("PATH"), base.get("PATH"), "{what}");
        }
        // Where the kit's folder has no path in the VM (a Windows kit on
        // another system) `decide` gives no VM placement at all.
        assert_eq!(seen > 0, apps.is_some(), "{}", target.as_str());
    }
}

/// I5: nothing of the kit's secrets reaches the VM's environment, whatever
/// the PC's environment holds.
#[test]
fn no_kit_secret_reaches_a_vm_environment() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "vm-keys");
        crate::db::auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
        let base = base_of(&kit);
        for (p, t) in shapes(&kit) {
            if let Placement::Vm { .. } = p {
                let what = format!("{} {p:?}", target.as_str());
                let (_, guest) = vm_env(build(&kit, &base, &p, t.as_ref()), &what);
                for (name, value) in &guest.vars {
                    assert!(
                        !SECRETS.iter().any(|(n, _)| n == name) && !value.contains("KIT-SECRET"),
                        "{what}: {name}"
                    );
                    assert!(!SEALED.contains(&name.as_str()), "{what}: {name}");
                }
                assert!(
                    guest.path.iter().all(|e| !e.contains("KIT-SECRET")),
                    "{what}"
                );
            }
        }
    }
}

/// Pi and Antigravity run their own Linux build: its folder comes back on
/// the `PATH` of their session only, never on another program's.
#[test]
fn pi_and_agy_get_their_own_folder_back_only_for_themselves() {
    for target in [Target::LINUX_X64, Target::LINUX_ARM64, Target::DARWIN_ARM64] {
        let (_tmp, kit) = kit_of(target, "agent-back");
        let base = base_of(&kit);
        let (guest_kit, apps) = guest_apps(&kit);
        let apps = apps.unwrap();
        let folder = |agent: Agent| {
            crate::isolation::vm::guest_path(false, &guest_kit.apps().join(agent.id())).unwrap()
        };
        let shop = kit.projects().join("shop");
        let placement = |program: VmProgram, level: Level| Placement::Vm {
            level,
            project: "shop".into(),
            sandbox: crate::levels::Sandbox::Project {
                name: crate::isolation::vm::name("shop"),
            },
            workdir: shop.display().to_string(),
            mounts: crate::levels::Mounts {
                workspace_rw: shop.clone(),
                tools_ro: vec![guest_kit.apps()],
            },
            host: crate::levels::HostAccess {
                databases: true,
                local_models: false,
            },
            publish: false,
            program,
        };
        let path = |program: VmProgram, level: Level| {
            let what = format!("{} {program:?}", target.as_str());
            vm_env(build(&kit, &base, &placement(program, level), None), &what)
                .1
                .path
        };
        let shell = path(VmProgram::Shell, Level::Project);
        let pi = path(VmProgram::AgentBinary { agent: Agent::Pi }, Level::Agent);
        let agy = path(
            VmProgram::AgentBinary {
                agent: Agent::Antigravity,
            },
            Level::Agent,
        );
        let claude = path(
            VmProgram::SbxAgent {
                agent: Agent::Claude,
            },
            Level::Agent,
        );
        let (pi_dir, agy_dir) = (folder(Agent::Pi), folder(Agent::Antigravity));
        assert!(pi_dir.starts_with(&apps));
        for (what, entries, has_pi, has_agy) in [
            ("shell", &shell, false, false),
            ("pi", &pi, true, false),
            ("agy", &agy, false, true),
            ("claude", &claude, false, false),
        ] {
            assert_eq!(entries.contains(&pi_dir), has_pi, "{what}: {entries:?}");
            assert_eq!(entries.contains(&agy_dir), has_agy, "{what}: {entries:?}");
        }
        // The rest is the project's terminal's, the agent's folder last.
        assert_eq!(&pi[..pi.len() - 1], &shell[..]);
        assert_eq!(pi.last(), Some(&pi_dir));
    }
}

/// A refusal starts nothing: no environment.
#[test]
fn refused_has_no_environment() {
    for target in TARGETS {
        let (_tmp, kit) = kit_of(target, "refused");
        let base = base_of(&kit);
        let mut seen = 0;
        for (p, t) in shapes(&kit) {
            let env = build(&kit, &base, &p, t.as_ref());
            assert_eq!(
                env.is_none(),
                matches!(p, Placement::Refused { .. }),
                "{p:?}"
            );
            seen += usize::from(env.is_none());
        }
        assert!(seen > 0);
    }
}

/// Built again from its own result, a PC environment is the same: no
/// second swap, no second folder of the agents' commands.
#[test]
fn the_environment_is_idempotent() {
    for target in TARGETS {
        let l = lines_of(target, "idempotent");
        set_project_choice(&l.kit, "shop", "node", Some("18.20")).unwrap();
        let default = terminals::find(&l.kit, "default", "it").unwrap();
        let t = Lines::terminal();
        for (level, project, terminal) in [
            (Level::Manual, None, Some(&default)),
            (Level::Manual, Some("shop"), Some(&default)),
            (Level::Manual, None, Some(&t)),
            (Level::Manual, Some("shop"), Some(&t)),
            (Level::Project, Some("shop"), None),
            (Level::Project, None, None),
        ] {
            let what = format!("{} {level:?} {project:?}", target.as_str());
            let once = pc_environment(&l.kit, l.base.clone(), level, project, terminal);
            let twice = pc_environment(&l.kit, once.clone(), level, project, terminal);
            assert_eq!(vars(&twice), vars(&once), "{what}");
        }
        // The agents stripped twice: the same.
        let (_tmp, kit) = kit_of(target, "idempotent-agents");
        let mut env = base_of(&kit);
        strip_agents(&kit, &mut env);
        forward_first(&kit, &mut env);
        let again = {
            let mut e = env.clone();
            strip_agents(&kit, &mut e);
            forward_first(&kit, &mut e);
            e
        };
        assert_eq!(vars(&again), vars(&env), "{}", target.as_str());
    }
}

/// Today's callers get today's environment (golden values), plus
/// `ATOLLO_LEVEL`/`ATOLLO_PROJECT` (§3.5 step 7); the default kit terminal
/// is now sealed like the others (§3.5 step 4, D-167).
#[test]
fn the_old_callers_get_the_same_environment() {
    for target in TARGETS {
        let l = lines_of(target, "golden");
        let kit = &l.kit;
        let s = |p: &Path| p.display().to_string();
        let system = if target.is_windows() {
            r"C:\Windows\System32"
        } else {
            "/usr/bin"
        };
        // A small base: the kit's commands, the agents' launchers, Claude
        // Code, Node.js and Python on the kit's lines, the system.
        let mut base = l.base.clone();
        let names: Vec<String> = base.iter().map(|(k, _)| k.to_owned()).collect();
        for name in names {
            base.remove(&name);
        }
        let sep = base.path_sep().to_string();
        let bin = kit.kit_files().join("bin");
        base.set(
            "PATH",
            [
                s(&bin),
                s(&kit.kit_files().join(crate::setup::kit_files::AGENTS_DIR)),
                s(&kit.apps().join("claude")),
                s(&l.node[0]),
                s(&l.python[0]),
                system.to_owned(),
            ]
            .join(&sep),
        );
        base.set(
            "CP_ANTIGRAVITY",
            s(&kit.apps().join("antigravity").join("agy")),
        );
        base.set("SSH_AUTH_SOCK", "/run/agent");
        base.set("FOO", "bar");
        let first = s(&forward_dir(kit));
        let expect = |path: &[&str], extra: &[(&str, &str)]| -> BTreeMap<String, String> {
            let mut out: BTreeMap<String, String> = [("FOO", "bar")]
                .iter()
                .chain(extra)
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect();
            let mut entries = vec![first.as_str()];
            entries.extend_from_slice(path);
            out.insert("PATH".into(), entries.join(&sep));
            out
        };
        let (bin, node_20, py) = (s(&bin), s(&l.node[1]), s(&l.python[0]));
        let what = target.as_str();

        // `versions::project_env`: the project's lines, sealed, no agent.
        set_project_choice(kit, "app", "node", Some("20.18")).unwrap();
        let env = crate::versions::project_env(kit, "app", base.clone());
        assert_eq!(
            vars(&env),
            expect(
                &[&bin, &node_20, &py, system],
                &[("ATOLLO_LEVEL", "project"), ("ATOLLO_PROJECT", "app")]
            ),
            "{what}"
        );

        // A user terminal (yesterday's `terminals::env`): its lines, the
        // tools it left out gone, sealed, no agent, its name.
        let only_node = Terminal {
            id: "n".into(),
            name: "Node 20".into(),
            dir: None,
            tools: BTreeMap::from([("node".to_owned(), Some("20.18".to_owned()))]),
        };
        let p = Placement::Pc {
            level: Level::Manual,
            project: None,
            dir: kit.data(),
        };
        let env = pc_env(build(kit, &base, &p, Some(&only_node)), what);
        assert_eq!(
            vars(&env),
            expect(
                &[&bin, &node_20, system],
                &[("ATOLLO_LEVEL", "manual"), ("ATOLLO_TERMINAL", "Node 20")]
            ),
            "{what}"
        );

        // The default terminal: the kit's lines, no agent, its name; now
        // sealed too (it kept `SSH_AUTH_SOCK` before D-179).
        let default = terminals::find(kit, "default", "it").unwrap();
        let env = pc_env(build(kit, &base, &p, Some(&default)), what);
        let node_main = s(&l.node[0]);
        assert_eq!(
            vars(&env),
            expect(
                &[&bin, &node_main, &py, system],
                &[
                    ("ATOLLO_LEVEL", "manual"),
                    ("ATOLLO_TERMINAL", default.name.as_str())
                ]
            ),
            "{what}"
        );
    }
}
