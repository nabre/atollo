//! `levels::prepare` with a fake [`Effects`] (LEVELS.md §3.6, §12.3): the
//! order of the effects, `db.env`, the PC's ports of a sandbox, Pi's local
//! models, an agent's sandbox, `info.json` without a secret; nothing for
//! the PC. On a kit of the system that runs the tests (`kit_arch`),
//! whatever it is: every supported one has a VM's Linux.

use std::sync::Mutex;

use super::*;
use crate::isolation::{kit as sandbox_kit, Blocker};
use crate::levels::env::environment;
use crate::levels::{decide, facts, Facts, Origin, Request, VmState};
use crate::setup::Catalog;
use crate::test_support::{prepared_kit, TempDir};

/// What `prepare` asked of the app, in order, and with what.
#[derive(Default)]
pub(crate) struct Fake {
    /// The project's databases and their ports.
    pub dbs: Vec<(ProjectDb, u16)>,
    /// Pi's local models, when Pi uses one.
    pub models: Option<LocalModels>,
    /// The effects asked, in order.
    pub calls: Mutex<Vec<String>>,
    /// The tools asked for the VM's Linux.
    pub wanted: Mutex<BTreeMap<String, Option<String>>>,
    /// The PC's ports of the sandbox when it was started.
    pub started_with: Mutex<Vec<u16>>,
}

impl Fake {
    pub fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    fn note(&self, call: String) {
        self.calls.lock().unwrap().push(call);
    }
}

impl Effects for Fake {
    fn ensure_guest_tools(&self, guest: &Kit, wanted: &BTreeMap<String, Option<String>>) {
        assert_eq!(Some(guest.target()), vm::guest_target(guest.target()));
        *self.wanted.lock().unwrap() = wanted.clone();
        self.note("ensure_guest_tools".into());
    }

    fn start_databases(&self, project: &str) -> Result<Vec<(ProjectDb, u16)>> {
        self.note(format!("start_databases {project}"));
        Ok(self.dbs.clone())
    }

    fn start_local_models(&self, _lang: &str) -> Option<LocalModels> {
        self.note("start_local_models".into());
        self.models.clone()
    }

    fn start_sandbox(&self, vm: &Vm, _env: &KitEnv, _lang: &str) -> Result<()> {
        *self.started_with.lock().unwrap() = vm.host_ports().to_vec();
        self.note(format!("start_sandbox {}", vm.name));
        Ok(())
    }

    fn watch_local_models(
        &self,
        vm: &Vm,
        _env: &KitEnv,
        project: &str,
        port: u16,
        db_ports: &[u16],
    ) {
        self.note(format!(
            "watch_local_models {} {project} {port} {db_ports:?}",
            vm.name
        ));
    }
}

/// `id` installed in `kit` (its key paths on disk), as far as
/// `Component::is_installed` sees.
pub(crate) fn install(kit: &Kit, id: &str) {
    for path in Catalog::get().find(id).unwrap().key_paths(kit) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"").unwrap();
    }
}

/// A kit with the project `shop` and the kit's `sbx`.
pub(crate) fn kit_with_shop(name: &str) -> (TempDir, Kit) {
    let (tmp, kit) = prepared_kit(&format!("prepare-{name}"));
    std::fs::create_dir_all(kit.projects().join("shop")).unwrap();
    install(&kit, crate::isolation::COMPONENT);
    (tmp, kit)
}

/// The project's MySQL database on the kit's server at 3307, its password
/// one the tests look for.
pub(crate) fn shop_db() -> (ProjectDb, u16) {
    (
        ProjectDb {
            engine: "mysql".into(),
            database: "shop".into(),
            user: "shop".into(),
            password: "s3cret-pw".into(),
            host: "127.0.0.1".into(),
            port: 3307,
            url: "mysql://shop:s3cret-pw@127.0.0.1:3307/shop".into(),
        },
        3307,
    )
}

/// The facts of `req` with the VM on (or off).
pub(crate) fn facts_of(kit: &Kit, req: &Request, on: bool) -> Facts {
    Facts {
        vm: if on {
            VmState::On
        } else {
            VmState::Off {
                blockers: vec![Blocker::HypervisorOff],
            }
        },
        guest_dir: facts::guest_dir(kit, &req.place),
        restart_pending: false,
        linux_build: true,
        guest_tools: facts::guest_tools(kit),
    }
}

fn request(kit: &Kit, origin: Origin) -> Request {
    Request {
        origin,
        place: crate::levels::place::place_of(kit, &kit.projects().join("shop")),
    }
}

/// `origin` in `shop` decided with the VM `on`, its environment, prepared
/// with `fx` (`needs` besides the project's tools).
fn prepared(kit: &Kit, fx: &Fake, origin: Origin, on: bool, needs: &[String]) -> Result<Ready> {
    let req = request(kit, origin);
    let placement = decide(&req, &facts_of(kit, &req, on));
    let base = KitEnv::build(kit, &Default::default(), "it", &Host::default());
    let env = environment(kit, &base, &placement, None, "it", &Host::default()).unwrap();
    prepare(fx, kit, &Host::default(), placement, env, needs, "it")
}

fn vm_of(ready: &Ready) -> &Vm {
    ready.vm.as_ref().expect("a sandbox")
}

/// The project's terminal in its VM: the kit's tools for the VM's Linux at
/// the project's lines asked first, then the databases, then the sandbox,
/// started with their ports; `db.env` beside the project's sandbox's kit,
/// with the VM's way to the PC, given to every session.
#[test]
fn a_project_vm_gets_tools_databases_and_ports() {
    let (_tmp, kit) = kit_with_shop("project");
    crate::versions::set_project_choice(&kit, "shop", "node", Some("20.18")).unwrap();
    let fx = Fake {
        dbs: vec![shop_db()],
        ..Fake::default()
    };
    let ready = prepared(&kit, &fx, Origin::ProjectTerminal, true, &[]).unwrap();
    let name = vm::name("shop");
    assert_eq!(
        fx.calls(),
        [
            "ensure_guest_tools".to_owned(),
            "start_databases shop".to_owned(),
            format!("start_sandbox {name}"),
        ]
    );
    assert_eq!(
        fx.wanted.lock().unwrap().get("node"),
        Some(&Some("20.18".to_owned())),
        "the project's line"
    );
    let vm = vm_of(&ready);
    assert_eq!(vm.name, name);
    assert_eq!(vm.dir, kit.projects().join("shop"));
    assert_eq!(
        vm.read_only,
        facts::guest_tools(&kit),
        "the kit's Linux tools, ro"
    );
    assert_eq!(
        *fx.started_with.lock().unwrap(),
        [3307],
        "started with its ports"
    );
    assert_eq!(vm.host_ports(), [3307]);
    let file = sandbox_kit::dir(&kit, &name).join(DB_ENV);
    assert_eq!(vm.tools().env_file.as_deref(), Some(file.as_path()));
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains(&format!("={}", vm::HOST)), "{text}");
    assert!(text.contains("MYSQL_TCP_PORT=3307\n"), "{text}");
    assert!(!text.contains("127.0.0.1"), "{text}");
    match &ready.env {
        Environment::Vm { guest, .. } => assert_eq!(guest, vm.tools()),
        other => panic!("{other:?}"),
    }

    // No database any more: its password leaves the VM's file.
    let fx = Fake::default();
    let ready = prepared(&kit, &fx, Origin::ProjectTerminal, true, &[]).unwrap();
    assert!(!file.exists());
    assert_eq!(vm_of(&ready).tools().env_file, None);
    assert!(vm_of(&ready).host_ports().is_empty());
}

/// §8.4: `info.json` beside the sandbox's kit (never in the project) says
/// where the sandbox is and what it has, the databases with their address
/// in the VM and their user, never a password or a key.
#[test]
fn the_vm_info_holds_no_secret() {
    let (_tmp, kit) = kit_with_shop("info");
    install(
        &kit.for_target(vm::guest_target(kit.target()).unwrap()),
        "pi",
    );
    let fx = Fake {
        dbs: vec![shop_db()],
        models: Some(LocalModels {
            port: 8085,
            key: "llama-k3y".into(),
        }),
        ..Fake::default()
    };
    let ready = prepared(
        &kit,
        &fx,
        Origin::AgentButton { agent: Agent::Pi },
        true,
        &[],
    )
    .unwrap();
    let vm = vm_of(&ready);
    let file = info_file(&kit, &vm.name);
    assert!(
        file.starts_with(sandbox_kit::root(&kit)),
        "{}",
        file.display()
    );
    let text = std::fs::read_to_string(&file).unwrap();
    for secret in ["s3cret-pw", "llama-k3y"] {
        assert!(!text.contains(secret), "{secret}: {text}");
    }
    let lower = text.to_lowercase();
    for word in ["password", "secret", "token", "\"key\"", "apikey"] {
        assert!(!lower.contains(word), "{word}: {text}");
    }
    let info: SandboxInfo = serde_json::from_str(&text).unwrap();
    assert_eq!(info.level, Level::Agent);
    assert_eq!(info.sandbox, vm.name);
    assert_eq!(info.project, "shop");
    assert_eq!(info.agent, Some(Agent::Pi));
    assert_eq!(
        info.databases,
        [DbInfo {
            engine: "mysql".into(),
            database: "shop".into(),
            user: "shop".into(),
            address: format!("{}:3307", vm::HOST),
        }]
    );
    assert_eq!(info.host_ports, [3307, 8085]);
    assert!(info.tools.contains_key("pi"));
}

/// D-178: Pi in the project's VM gets the local models' port for that
/// sandbox only, their address and key in `pi.env` with the databases'
/// variables, and a watcher that closes the port once Pi ends, never a
/// database's.
#[test]
fn pi_gets_the_local_models_port() {
    let (_tmp, kit) = kit_with_shop("pi");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    let fx = Fake {
        dbs: vec![shop_db()],
        models: Some(LocalModels {
            port: 8085,
            key: "llama-k3y".into(),
        }),
        ..Fake::default()
    };
    // Its build for the VM's Linux missing, and not installed by the
    // effects: no sandbox starts.
    let why = prepared(
        &kit,
        &fx,
        Origin::AgentButton { agent: Agent::Pi },
        true,
        &[],
    )
    .unwrap_err()
    .to_string();
    assert!(why.starts_with("Pi per il Linux della VM"), "{why}");
    assert!(!fx.calls().iter().any(|c| c.starts_with("start_sandbox")));

    install(&guest, "pi");
    let fx = Fake {
        dbs: vec![shop_db()],
        models: fx.models.clone(),
        ..Fake::default()
    };
    let ready = prepared(
        &kit,
        &fx,
        Origin::AgentButton { agent: Agent::Pi },
        true,
        &[],
    )
    .unwrap();
    let name = vm::name("shop");
    assert_eq!(
        fx.calls(),
        [
            "ensure_guest_tools".to_owned(),
            "start_databases shop".to_owned(),
            "start_local_models".to_owned(),
            format!("start_sandbox {name}"),
            format!("watch_local_models {name} shop 8085 [3307]"),
        ]
    );
    assert!(
        fx.wanted.lock().unwrap().contains_key("pi"),
        "its own build"
    );
    let vm = vm_of(&ready);
    assert_eq!(vm.name, name, "the project's sandbox");
    assert_eq!(*fx.started_with.lock().unwrap(), [3307, 8085]);
    let file = sandbox_kit::dir(&kit, &name).join(PI_ENV);
    assert_eq!(vm.tools().env_file.as_deref(), Some(file.as_path()));
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("LLAMA_API_KEY=llama-k3y\n"), "{text}");
    assert!(text.contains("MYSQL_TCP_PORT=3307\n"), "{text}");
    // Pi's folder back on its own session's PATH, last.
    let own = vm::guest_path(kit.target().is_windows(), &guest.apps().join("pi")).unwrap();
    assert_eq!(vm.tools().path.last(), Some(&own));

    // The models on a database's port: opened for both, never watched.
    let fx = Fake {
        dbs: vec![shop_db()],
        models: Some(LocalModels {
            port: 3307,
            key: "k".into(),
        }),
        ..Fake::default()
    };
    let ready = prepared(
        &kit,
        &fx,
        Origin::AgentButton { agent: Agent::Pi },
        true,
        &[],
    )
    .unwrap();
    assert!(!fx
        .calls()
        .iter()
        .any(|c| c.starts_with("watch_local_models")));
    assert_eq!(vm_of(&ready).host_ports(), [3307]);

    // Without a local model: no port, no `pi.env`.
    let fx = Fake::default();
    let ready = prepared(
        &kit,
        &fx,
        Origin::AgentButton { agent: Agent::Pi },
        true,
        &[],
    )
    .unwrap();
    assert!(vm_of(&ready).host_ports().is_empty());
    assert_eq!(vm_of(&ready).tools().env_file, None);
}

/// D-179 step 7 (LEVELS.md §9): Antigravity in the project's VM gets its
/// build for the VM's Linux (asked of the effects, a sandbox only once it
/// is there), its own folder last on its `PATH`, never the local models;
/// its sandbox gets the kit's Gemini key through the proxy of `sbx` at its
/// start, never in a file of the VM. Pi's and the project's terminal's
/// sandboxes are not given it.
#[test]
fn antigravity_gets_its_build_and_the_gemini_key_by_the_proxy() {
    let (_tmp, kit) = kit_with_shop("agy");
    let guest = kit.for_target(vm::guest_target(kit.target()).unwrap());
    let agy = Origin::AgentButton {
        agent: Agent::Antigravity,
    };
    let fx = Fake {
        dbs: vec![shop_db()],
        models: Some(LocalModels {
            port: 8085,
            key: "llama-k3y".into(),
        }),
        ..Fake::default()
    };
    let why = prepared(&kit, &fx, agy.clone(), true, &[])
        .unwrap_err()
        .to_string();
    assert!(
        why.starts_with("Antigravity per il Linux della VM"),
        "{why}"
    );
    assert!(fx.wanted.lock().unwrap().contains_key("antigravity"));
    assert!(!fx.calls().iter().any(|c| c.starts_with("start_sandbox")));

    install(&guest, "antigravity");
    let fx = Fake {
        dbs: vec![shop_db()],
        models: fx.models.clone(),
        ..Fake::default()
    };
    let ready = prepared(&kit, &fx, agy, true, &[]).unwrap();
    let name = vm::name("shop");
    assert_eq!(
        fx.calls(),
        [
            "ensure_guest_tools".to_owned(),
            "start_databases shop".to_owned(),
            format!("start_sandbox {name}"),
        ],
        "no local models"
    );
    let vm = vm_of(&ready);
    assert_eq!(vm.name, name, "the project's sandbox");
    assert!(vm.gemini(), "the Gemini key through the proxy");
    assert_eq!(vm.host_ports(), [3307]);
    let own = vm::guest_path(kit.target().is_windows(), &guest.apps().join("antigravity")).unwrap();
    assert_eq!(vm.tools().path.last(), Some(&own));
    let db_env = sandbox_kit::dir(&kit, &name).join(DB_ENV);
    assert_eq!(vm.tools().env_file.as_deref(), Some(db_env.as_path()));
    for (k, _) in &vm.tools().vars {
        assert!(!k.contains("GEMINI") && !k.contains("GOOGLE"), "{k}");
    }
    let info: SandboxInfo =
        serde_json::from_str(&std::fs::read_to_string(info_file(&kit, &name)).unwrap()).unwrap();
    assert_eq!(info.agent, Some(Agent::Antigravity));
    assert!(info.tools.contains_key("antigravity"));

    // The session runs its build with `vm::agy_session_args`.
    let args = crate::levels::start::vm_session_args(&kit, &ready, &["--help".into()]).unwrap();
    let program = isolation::agent_in_vm(&kit, &guest, "antigravity").unwrap();
    assert!(program.ends_with("/antigravity/agy"), "{program}");
    assert_eq!(args[args.len() - 2..], [program, "--help".to_owned()]);
    assert!(args.contains(&"AGY_CLI_DISABLE_AUTO_UPDATE=true".to_owned()));

    // Pi and the project's terminal: no Gemini key for their sandbox.
    install(&guest, "pi");
    for origin in [
        Origin::AgentButton { agent: Agent::Pi },
        Origin::ProjectTerminal,
    ] {
        let ready = prepared(&kit, &Fake::default(), origin, true, &[]).unwrap();
        assert!(!vm_of(&ready).gemini());
    }
}

/// D-175: Claude Code and Codex get a sandbox of their own with the same
/// workspace, Linux tools, databases and ports as the project's; never the
/// local models.
#[test]
fn an_agent_sandbox_mirrors_the_projects() {
    let (_tmp, kit) = kit_with_shop("agent");
    for agent in [Agent::Claude, Agent::Codex] {
        let fx = Fake {
            dbs: vec![shop_db()],
            models: Some(LocalModels {
                port: 8085,
                key: "k".into(),
            }),
            ..Fake::default()
        };
        let ready = prepared(&kit, &fx, Origin::AgentButton { agent }, true, &[]).unwrap();
        let name = vm::agent_name("shop", agent.id());
        assert_eq!(
            fx.calls(),
            [
                "ensure_guest_tools".to_owned(),
                "start_databases shop".to_owned(),
                format!("start_sandbox {name}"),
            ],
            "{agent:?}"
        );
        let vm = vm_of(&ready);
        assert_eq!(vm.name, name);
        assert_eq!(vm.dir, kit.projects().join("shop"));
        assert_eq!(vm.read_only, facts::guest_tools(&kit));
        assert_eq!(vm.host_ports(), [3307]);
        let db_env = sandbox_kit::dir(&kit, &vm::name("shop")).join(DB_ENV);
        assert_eq!(vm.tools().env_file.as_deref(), Some(db_env.as_path()));
        let info: SandboxInfo =
            serde_json::from_str(&std::fs::read_to_string(info_file(&kit, &name)).unwrap())
                .unwrap();
        assert_eq!(info.agent, Some(agent));
        assert_eq!(info.level, Level::Agent);
    }
}

/// A creation in the new project's VM gets the kit's PowerShell for the
/// VM's Linux (its lines are PowerShell's) and the plan's tools.
#[test]
fn a_scaffold_gets_the_kits_powershell_and_the_plans_tools() {
    let (_tmp, kit) = kit_with_shop("scaffold");
    let fx = Fake::default();
    let origin = Origin::Scaffold {
        base: "laravel".into(),
    };
    prepared(&kit, &fx, origin, true, &["composer".to_owned()]).unwrap();
    let wanted = fx.wanted.lock().unwrap();
    assert!(wanted.contains_key("pwsh"), "{wanted:?}");
    assert!(wanted.contains_key("composer"), "{wanted:?}");
}

/// A `Vm` placement without the kit's `sbx` is an error, never a start on
/// the PC (I7).
#[test]
fn no_sandbox_without_the_kits_sbx() {
    let (_tmp, kit) = prepared_kit("prepare-no-sbx");
    std::fs::create_dir_all(kit.projects().join("shop")).unwrap();
    let fx = Fake::default();
    let why = prepared(&kit, &fx, Origin::ProjectTerminal, true, &[])
        .unwrap_err()
        .to_string();
    assert_eq!(why, "Docker Sandboxes (sbx) non è nel kit.");
    assert!(!fx.calls().iter().any(|c| c.starts_with("start_sandbox")));
}

/// `Pc`: nothing to prepare (the caller's `ensure` ran before its
/// environment), the environment as given; a refusal is never prepared.
#[test]
fn nothing_is_prepared_for_the_pc() {
    let (_tmp, kit) = kit_with_shop("pc");
    let fx = Fake::default();
    let ready = prepared(&kit, &fx, Origin::ProjectTerminal, false, &[]).unwrap();
    assert!(fx.calls().is_empty(), "{:?}", fx.calls());
    assert!(ready.vm.is_none());
    assert!(matches!(ready.placement, Placement::Pc { .. }));
    assert!(matches!(ready.env, Environment::Pc(_)));

    let req = request(
        &kit,
        Origin::AgentButton {
            agent: Agent::Claude,
        },
    );
    let refused = decide(&req, &facts_of(&kit, &req, false));
    assert!(matches!(refused, Placement::Refused { .. }));
    let base = KitEnv::build(&kit, &Default::default(), "it", &Host::default());
    assert!(prepare(
        &fx,
        &kit,
        &Host::default(),
        refused,
        Environment::Pc(base),
        &[],
        "it"
    )
    .is_err());
    assert!(fx.calls().is_empty());
}
