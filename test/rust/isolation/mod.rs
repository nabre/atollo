use super::*;

/// «Lancia» runs only `sbx`'s own commands (D-178, zero admin): no fix of
/// the PC's, which asks for an administrator or a new sign-in
/// (hypervisor, `kvm`, AppArmor), can be asked of the app.
#[test]
fn no_admin_fix_can_be_launched() {
    for code in ["login", "ls"] {
        assert!(serde_json::from_value::<Fix>(serde_json::json!(code)).is_ok());
    }
    for code in [
        "hypervisor_off",
        "hypervisor",
        "kvm_denied",
        "kvm",
        "apparmor",
        "daemon_stop",
        "foreign_daemon",
    ] {
        assert!(
            serde_json::from_value::<Fix>(serde_json::json!(code)).is_err(),
            "{code}"
        );
    }
    for fix in [Fix::Login, Fix::Ls] {
        let args = fix.args();
        assert!(args
            .iter()
            .all(|a| !a.contains("sudo") && !a.contains("Enable-")));
    }
}

/// The network policy: the kit's `sbx` gets `balanced` (set already is
/// fine); texts of sbx 0.49.2.
#[test]
fn no_vm_starts_without_a_network_policy() {
    assert_eq!(
        policy_answer(
            true,
            "Initialized global network policy balanced
"
        ),
        Ok(())
    );
    assert_eq!(
        policy_answer(
            false,
            "error: global network policy is already initialized; reset it first to change it
"
        ),
        Ok(())
    );
    assert!(matches!(
        policy_answer(
            false,
            "error: daemon not running
"
        ),
        Err(Blocker::SbxFailed { .. })
    ));
}

/// The list of the VMs: Atollo's sandboxes only, with their owner from
/// their kits; the sandboxes a project or a terminal takes with it.
#[test]
fn atollos_sandboxes_are_listed_and_go_with_their_owner() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-vms");
    let shop = vm::name("shop");
    let owner = kit::Owner::Project {
        project: "shop".into(),
    };
    kit::write(&kit, &shop, &owner).unwrap();
    let json = format!(
        r#"[{{"name":"{shop}","status":"running","last_used_at":"2026-10-25T17:40:30Z"}},
            {{"name":"claude-mine","status":"stopped"}},
            {{"name":"atollo-t-default-0123","status":"stopped"}},
            {{"name":"atollo-a-claude-shop-0123","status":"stopped"}}]"#
    );
    let vms = vms_of(&kit, &json);
    assert_eq!(
        vms.len(),
        2,
        "the user's own sandbox is not Atollo's, a kit terminal's is never listed (D-178)"
    );
    assert_eq!(terminal_vms_in(&json), ["atollo-t-default-0123"]);
    assert!(terminal_vms_in("not json").is_empty());
    assert_eq!(vms[0].name, shop);
    assert_eq!(vms[0].owner, Some(owner));
    assert_eq!(vms[0].status, "running");
    assert_eq!(vms[0].last_used.as_deref(), Some("2026-10-25T17:40:30Z"));
    assert_eq!(
        vms[1].owner, None,
        "an agent's sandbox has no kit of Atollo"
    );
    assert!(vms_of(&kit, "not json").is_empty());

    assert!(is_atollos(&shop) && is_atollos("atollo-t-x-1"));
    for name in [
        "atollo-",
        "claude-mine",
        "atollo-X",
        "atollo-a b",
        "atollo-a;rm",
    ] {
        assert!(!is_atollos(name), "{name}");
    }

    let names = project_sandboxes("shop");
    assert_eq!(names[0], shop);
    assert!(names.contains(&vm::agent_name("shop", "claude")));
    assert!(names.contains(&vm::agent_name("shop", "codex")));

    let legacy = |dir: &str| kit::Owner::Terminal {
        terminal: "legacy".into(),
        dir: dir.into(),
    };
    kit::write(&kit, "atollo-t-legacy-a", &legacy("/a")).unwrap();
    kit::write(&kit, "atollo-t-legacy-b", &legacy("/b")).unwrap();
    assert_eq!(
        terminal_sandboxes(&kit, "legacy"),
        ["atollo-t-legacy-a", "atollo-t-legacy-b"]
    );
    assert!(terminal_sandboxes(&kit, "default").is_empty());
}

/// A project in its VM gets the kit's tools for Linux (D-170): only what
/// lies in `data/apps/linux-…`, never its sealed PC-side variables (the
/// project's MySQL login file, the kit's home); the tools are those the
/// kit chose, not those on the PC's disk; the folders mounted at a
/// sandbox's making are noted.
#[test]
fn a_project_gets_the_kits_tools_for_linux_in_its_vm() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-project-guest");
    let guest = kit.for_target(crate::kit::Target::LINUX_X64);
    let node = crate::setup::Catalog::get().find("node").unwrap();
    for key in node.key_paths(&guest) {
        std::fs::create_dir_all(key.parent().unwrap()).unwrap();
        std::fs::write(&key, b"").unwrap();
    }
    crate::db::auth::set_project_password(&kit, "shop", "mysql", "pw").unwrap();
    let base = KitEnv::build(&guest, &Default::default(), "it", &Host::default());
    let tools = project_guest_tools(&kit, &guest, "shop", base);
    let apps = vm::guest_path(kit.target().is_windows(), &guest.apps()).unwrap();
    assert!(!tools.path.is_empty());
    assert!(
        tools.path.iter().all(|p| p.starts_with(&apps)),
        "{:?}",
        tools.path
    );
    assert!(
        tools.vars.iter().all(|(_, v)| v.starts_with(&apps)),
        "{:?}",
        tools.vars
    );
    assert!(!tools.vars.iter().any(|(n, _)| n == "ATOLLO_MYSQL_CNF"));

    let mut choice =
        crate::setup::catalog::Choice::read_or_defaults(&kit, crate::setup::Catalog::get());
    choice.set(crate::setup::Catalog::get(), "node", true);
    choice.write(&kit, crate::setup::Catalog::get()).unwrap();
    let chosen = crate::editor::terminals::chosen_tools(&kit);
    assert_eq!(chosen.get("node"), Some(&None));
    assert!(
        !chosen.contains_key("gh"),
        "a component of the app is no tool"
    );

    assert_eq!(kit::mounts(&kit, "atollo-p-shop-1"), None);
    kit::write(
        &kit,
        "atollo-p-shop-1",
        &kit::Owner::Project {
            project: "shop".into(),
        },
    )
    .unwrap();
    kit::mark_mounts(&kit, "atollo-p-shop-1", "/p/shop\n/k/apps\n").unwrap();
    assert_eq!(
        kit::mounts(&kit, "atollo-p-shop-1").as_deref(),
        Some("/p/shop\n/k/apps\n")
    );
}

/// A sandbox made before its mounts were noted: its workspace is the one
/// `sbx` says (sbx 0.49.2's `ls --json`), never assumed.
#[test]
fn an_unnoted_sandbox_has_the_workspace_sbx_says() {
    let json = r#"[{"name":"atollo-p-a","workspaces":["D:\\kit\\projects\\a"]},
                   {"name":"atollo-p-b","workspaces":[]}]"#;
    assert_eq!(
        workspace_of(json, "atollo-p-a").as_deref(),
        Some(r"D:\kit\projects\a")
    );
    assert_eq!(workspace_of(json, "atollo-p-b"), None);
    assert_eq!(workspace_of(json, "atollo-p-c"), None);
    assert_eq!(workspace_of("not json", "atollo-p-a"), None);
}

/// A project's database as a program in its VM reaches it (D-171): the
/// same user, database and password, through the VM's way to the PC; the
/// `.env` keys the kit writes for it point there; the ports opened noted.
#[test]
fn a_projects_database_is_reached_from_its_vm_on_the_pc() {
    let db = crate::db::auth::project_db("mysql", "shop", "pw", 3307);
    let vm_db = db_in_vm(&db, 3308);
    assert_eq!(vm_db.host, "host.docker.internal");
    assert_eq!(vm_db.port, 3308);
    assert_eq!(vm_db.user, "shop");
    assert_eq!(vm_db.url, "mysql://shop:pw@host.docker.internal:3308/shop");
    let laravel = "APP_NAME=x\nDB_CONNECTION=mysql\nDB_HOST=127.0.0.1\n";
    let keys = crate::db::auth::env_wanted(laravel, &vm_db);
    assert!(keys.contains(&("DB_HOST".into(), "host.docker.internal".into())));
    assert!(keys.contains(&("DB_PORT".into(), "3308".into())));
    let plain = crate::db::auth::env_wanted("", &vm_db);
    assert_eq!(
        plain,
        [(
            "DATABASE_URL".to_owned(),
            "mysql://shop:pw@host.docker.internal:3308/shop".to_owned()
        )]
    );
    let mongo = db_in_vm(
        &crate::db::auth::project_db("mongodb", "shop", "pw", 27018),
        27018,
    );
    assert_eq!(
        mongo.url,
        "mongodb://shop:pw@host.docker.internal:27018/shop?authSource=shop"
    );

    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-ports");
    kit::write(
        &kit,
        "atollo-p-shop-1",
        &kit::Owner::Project {
            project: "shop".into(),
        },
    )
    .unwrap();
    assert!(kit::host_ports(&kit, "atollo-p-shop-1").is_empty());
    kit::mark_host_ports(&kit, "atollo-p-shop-1", &[3307, 27018]).unwrap();
    assert_eq!(kit::host_ports(&kit, "atollo-p-shop-1"), [3307, 27018]);
}

#[test]
fn no_blocker_is_the_vm() {
    assert_eq!(Status::of(Vec::new(), None).level, Level::Vm);
    assert_eq!(
        Status::of(vec![Blocker::SbxMissing { failed: None }], None).level,
        Level::Env
    );
}

/// «Lancia» runs only `sbx login` and `sbx ls`, with the environment
/// where that `sbx` keeps its sign-in: the kit's for the kit's, the PC's
/// for the PC's.
#[test]
fn a_fix_runs_with_the_environment_of_its_sbx() {
    assert_eq!(Fix::Login.args(), ["login"]);
    assert_eq!(Fix::Ls.args(), ["ls"]);
    assert_eq!(
        serde_json::from_str::<Fix>("\"login\"").unwrap(),
        Fix::Login
    );
    assert!(serde_json::from_str::<Fix>("\"rm -rf\"").is_err());

    let mut env = KitEnv::default();
    env.set("LOCALAPPDATA", "kit-home");
    env.set("TEMP", r"E:\a\deep\kit\folder\data\tmp\system");
    env.set("CP_KIT", "x");
    let host = Host::from_pairs([
        ("LOCALAPPDATA".into(), "pc-home".into()),
        ("TEMP".into(), r"C:\T".into()),
    ]);

    // The kit's keeps the kit's home, with the PC's short temp: the
    // daemon's junction goes there (sockets of at most 107 bytes).
    let kit_env = Sbx::at(PathBuf::from("sbx")).session_env(&env, &host);
    assert_eq!(kit_env.get("LOCALAPPDATA"), Some("kit-home"));
    assert_eq!(kit_env.get("CP_KIT"), Some("x"));
    for name in ["TEMP", "TMP", "TMPDIR"] {
        assert_eq!(kit_env.get(name), Some(r"C:\T"), "{name}");
    }
}

#[test]
fn sbx_signed_out_is_told_apart_from_other_errors() {
    assert_eq!(answer(true, ""), Ok(()));
    assert_eq!(
        answer(
            false,
            "Error: you are not logged in. Run `sbx login` first.\n"
        ),
        Err(Blocker::SbxSignedOut)
    );
    assert_eq!(
        answer(false, "starting\nError: daemon not running\n\n"),
        Err(Blocker::SbxFailed {
            message: "Error: daemon not running".into()
        })
    );
    assert_eq!(
        answer(false, ""),
        Err(Blocker::SbxFailed {
            message: "sbx".into()
        })
    );
}

#[test]
fn an_unsupported_system_is_the_only_reason_given() {
    assert!(Blocker::OldWindows { build: 19045 }.unsupported());
    assert!(!Blocker::HypervisorOff.unsupported());
    assert!(!Blocker::SbxMissing { failed: None }.unsupported());
}

/// The blockers reach the interface by `code`, with their details.
#[test]
fn blockers_serialize_with_their_code() {
    let json = serde_json::to_value(Status::of(
        vec![Blocker::OldWindows { build: 19045 }, Blocker::SbxSignedOut],
        Some("sbx".into()),
    ))
    .unwrap();
    assert_eq!(json["level"], "env");
    assert_eq!(json["blockers"][0]["code"], "old_windows");
    assert_eq!(json["blockers"][0]["build"], 19045);
    assert_eq!(json["blockers"][1]["code"], "sbx_signed_out");
    assert_eq!(json["sbx"], "sbx");
}

/// D-178: Pi in the project's VM on the kit's local models: the server's
/// port of the PC is opened for that sandbox only (once, beside the
/// database's), and Pi's address points through the VM's way to the PC,
/// never at the VM's own `127.0.0.1`.
#[test]
fn pi_in_the_vm_reaches_the_local_models_on_the_pc() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-pi-llm");
    let dir = kit.projects().join("shop");
    let vm = Vm::for_tests(&kit, "shop", &dir)
        .with_host_ports(vec![3307])
        .with_host_port(8085)
        .with_host_port(8085);
    assert_eq!(vm.host_ports, [3307, 8085]);
    assert_eq!(
        vm::host_port_allow(&vm.name, 8085),
        [
            "policy",
            "allow",
            "network",
            "--sandbox",
            vm.name.as_str(),
            "--protocol",
            "tcp",
            "localhost:8085"
        ]
    );
    let vars = pi_local_models_env(8085, "k3y");
    assert_eq!(
        vars,
        [
            (
                "LLAMA_BASE_URL".to_owned(),
                "http://host.docker.internal:8085".to_owned()
            ),
            ("LLAMA_API_KEY".to_owned(), "k3y".to_owned()),
        ]
    );
}

/// D-178: whether Pi runs is decided on the PC, never asked to the VM: the
/// port stays open while Pi is being started, or while a session of the
/// app or of the editor runs; never past twelve hours since the last start
/// from the PC.
#[test]
fn the_port_of_the_local_models_follows_the_pcs_sessions_of_pi() {
    let s = Duration::from_secs;
    assert!(pi_port_kept(s(0), 0, 0), "being started");
    assert!(pi_port_kept(PI_GRACE - s(1), 0, 0));
    assert!(!pi_port_kept(PI_GRACE, 0, 0), "no session: it closes");
    assert!(pi_port_kept(s(3600), 1, 0), "the app's session runs");
    assert!(pi_port_kept(s(3600), 0, 2), "the editor's sessions run");
    assert_eq!(PI_MAX, s(12 * 60 * 60));
    assert!(pi_port_kept(PI_MAX - s(1), 1, 1));
    assert!(!pi_port_kept(PI_MAX, 1, 1), "past the cap, whatever runs");
    assert!(!pi_port_kept(PI_MAX + s(60), 0, 3));
}

/// D-178: an editor's session of Pi runs only while its marker's pid runs
/// on the PC as the kit's PowerShell: a marker left behind, its pid now
/// another program's, or another PowerShell, is none.
#[test]
fn an_editors_pi_runs_only_as_the_kits_powershell() {
    let pwsh = std::env::temp_dir()
        .join("atollo-no-kit")
        .join("pwsh")
        .join("pwsh.exe");
    let keys = pwsh_keys(&pwsh);
    assert_eq!(keys, [path_key(&pwsh.display().to_string())]);
    let proc = |pid: u32, path: &Path| Proc {
        pid,
        parent: 1,
        name: "pwsh".into(),
        path: path.to_path_buf(),
    };
    let other = std::env::temp_dir().join("pwsh").join("pwsh.exe");
    let procs = [
        proc(40, &pwsh),
        proc(41, &other),
        proc(42, Path::new("/usr/bin/sleep")),
        proc(43, &pwsh),
    ];
    assert_eq!(
        editor_pis_alive(&[40, 41, 42, 43, 44], &procs, &keys),
        [40, 43]
    );
    assert!(editor_pis_alive(&[44], &procs, &keys).is_empty(), "stale");
    assert!(editor_pis_alive(&[], &procs, &keys).is_empty());
    // A real file: its resolved path names it too, once.
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-pwsh-keys");
    let real = kit.root().join("pwsh");
    std::fs::write(&real, "").unwrap();
    let keys = pwsh_keys(&real);
    assert!(!keys.is_empty() && keys.len() <= 2, "{keys:?}");
    assert!(keys.iter().all(|k| !k.starts_with(r"\\?\")), "{keys:?}");
}

/// D-178: once Pi ended, every port of the local models opened for it in
/// the sandbox closes, the one of before a change of the settings too,
/// once each; never a database's, nor one closed already. A session or a
/// start in a sandbox never watched changes nothing.
#[test]
fn every_port_opened_for_pi_closes_when_it_ends() {
    assert_eq!(
        pi_ports_to_close(&[8085, 8090], &[3307, 8085, 8090], &[3307]),
        [8085, 8090]
    );
    assert_eq!(
        pi_ports_to_close(&[8085, 8090, 8085], &[8085, 8090], &[]),
        [8085, 8090]
    );
    assert_eq!(pi_ports_to_close(&[8085, 8090], &[8090], &[]), [8090]);
    assert!(pi_ports_to_close(&[3307], &[3307], &[3307]).is_empty());
    let name = "atollo-p-never-watched-2";
    pi_port_touch(name, 8090);
    assert_eq!(pi_port_of(name), None);
    drop(pi_session(name));
    assert_eq!(pi_port_of(name), None);
}

/// D-178: the ports of the local models close only while open for the
/// sandbox and never when a database needs them; the noted ports are
/// rewritten without the closed one, the others kept in their order.
#[test]
fn the_port_of_the_local_models_closes_when_pi_ends() {
    assert!(pi_port_closes(8085, &[3307, 8085], &[3307]));
    assert!(!pi_port_closes(8085, &[3307], &[3307]), "closed already");
    assert!(!pi_port_closes(8085, &[8085], &[8085]), "a database's too");
    assert_eq!(ports_without(&[3307, 8085, 27018], 8085), [3307, 27018]);
    assert_eq!(ports_without(&[3307], 8085), [3307]);
    // The noted ports of the sandbox, rewritten without it.
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-pi-port");
    let name = vm::name("shop");
    kit::write(
        &kit,
        &name,
        &kit::Owner::Project {
            project: "shop".into(),
        },
    )
    .unwrap();
    kit::mark_host_ports(&kit, &name, &[3307, 8085]).unwrap();
    let noted = ports_without(&kit::host_ports(&kit, &name), 8085);
    kit::mark_host_ports(&kit, &name, &noted).unwrap();
    assert_eq!(kit::host_ports(&kit, &name), [3307]);
    // The databases' ports of a VM, before Pi's is added.
    let vm = Vm::for_tests(&kit, "shop", &kit.projects().join("shop")).with_host_ports(vec![3307]);
    assert_eq!(vm.host_ports(), [3307]);
    assert_eq!(vm.with_host_port(8085).host_ports(), [3307, 8085]);
}

/// D-178: while Pi's watcher keeps the port of the local models open, any
/// other start of the sandbox (the project's terminal, an action, the
/// editor) opens it too, once, so it never closes it under a running Pi;
/// with no watcher, its own ports only.
#[test]
fn another_start_keeps_the_port_of_a_running_pi() {
    assert_eq!(ports_with(&[3307], Some(8085)), [3307, 8085]);
    assert_eq!(ports_with(&[3307, 8085], Some(8085)), [3307, 8085]);
    assert_eq!(ports_with(&[], Some(8085)), [8085]);
    assert_eq!(ports_with(&[3307, 27018], None), [3307, 27018]);
    assert_eq!(pi_port_of("atollo-p-never-watched"), None);
}

/// D-178: every reason the VM is off has a line for the editor's terminals
/// and the agents refused, with the administrator's command named (never
/// run), in Italian and in the other languages; then that Atollo checks
/// again by itself.
#[test]
fn every_reason_is_told_outside_the_notice() {
    let all = vec![
        Blocker::OldWindows { build: 19045 },
        Blocker::OldMacos {
            version: "13.6".into(),
        },
        Blocker::Distribution {
            name: "Fedora".into(),
        },
        Blocker::HypervisorOff,
        Blocker::KvmMissing,
        Blocker::KvmDenied,
        Blocker::AppArmor {
            shim: "/k/shim".into(),
        },
        Blocker::SbxMissing { failed: None },
        Blocker::SbxSignedOut,
        Blocker::SbxFailed {
            message: "boom".into(),
        },
        Blocker::ForeignDaemon,
        Blocker::KitDisk { fs: "exFAT".into() },
    ];
    for b in &all {
        for lang in ["it", "en", "fr", "de"] {
            assert!(b.explain(lang).len() > 20, "{b:?} {lang}");
        }
    }
    assert!(Blocker::HypervisorOff
        .explain("it")
        .contains(HYPERVISOR_COMMAND));
    assert!(Blocker::KvmDenied.explain("en").contains(KVM_COMMAND));
    assert!(Blocker::SbxFailed {
        message: "boom".into()
    }
    .explain("it")
    .contains("boom"));
    let status = Status::of(vec![Blocker::HypervisorOff, Blocker::SbxSignedOut], None);
    let text = why_no_vm(&status, "it");
    assert_eq!(text.lines().filter(|l| l.starts_with("- ")).count(), 2);
    assert!(text.starts_with("Perché la VM non è attiva:"), "{text}");
    assert!(text.contains("ricontrolla da solo"), "{text}");
    let editor = session::no_vm_text(&status, "it");
    assert!(editor.contains("sul PC non si apre nulla"), "{editor}");
    assert!(editor.contains(HYPERVISOR_COMMAND), "{editor}");
    assert!(agents_notice("it").contains("solo nella VM"));
    assert_ne!(agents_notice("en"), agents_notice("it"));
}

/// A note written under the old name of a PC without `HOSTNAME` moves to
/// the PC's real name, so its registration can still be taken back.
#[test]
fn a_legacy_pc_note_moves_to_the_real_name() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-legacy-pc");
    let vm = vm::name("shop");
    secrets::note(&kit, "github", LEGACY_PC, &vm, Some("cmd")).unwrap();
    // Not listed by this PC's `sbx`: another PC's, left where it is.
    assert!(mine_github(&kit, "mac", &vm, || false).is_none());
    assert!(secrets::mine(&kit, "github", LEGACY_PC, &vm).is_some());
    assert_eq!(
        mine_github(&kit, "mac", &vm, || true).unwrap().command,
        "cmd"
    );
    assert!(secrets::mine(&kit, "github", LEGACY_PC, &vm).is_none());
    assert_eq!(
        secrets::mine(&kit, "github", "mac", &vm).unwrap().command,
        "cmd"
    );
    assert!(mine_github(&kit, LEGACY_PC, "other", || true).is_none());
}

/// A failed installation of Docker Sandboxes is said in the notice, until
/// one succeeds.
#[test]
fn a_failed_installation_of_sbx_is_told() {
    use crate::setup::runner::Outcome;
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-install-failed");
    assert_eq!(install_failure(&kit), None);
    note_install(
        &kit,
        &Outcome::Failed {
            message: "checksum mismatch".into(),
            by_hand: None,
        },
    );
    assert_eq!(install_failure(&kit).as_deref(), Some("checksum mismatch"));
    assert_eq!(
        serde_json::to_value(Blocker::SbxMissing {
            failed: install_failure(&kit)
        })
        .unwrap(),
        serde_json::json!({ "code": "sbx_missing", "failed": "checksum mismatch" })
    );
    note_install(&kit, &Outcome::Cancelled);
    assert!(install_failure(&kit).is_some(), "a cancel says nothing");
    note_install(
        &kit,
        &Outcome::Installed {
            version: "0.49.2".into(),
        },
    );
    assert_eq!(install_failure(&kit), None);
    assert_eq!(
        serde_json::to_value(Blocker::SbxMissing { failed: None }).unwrap(),
        serde_json::json!({ "code": "sbx_missing" })
    );
}
