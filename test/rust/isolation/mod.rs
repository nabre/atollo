use super::*;

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

/// The kit's terminal in the VM gets the kit's tools for Linux: only what
/// lies in `data/apps/linux-…`, as the VM sees it; never the kit's home,
/// temp or agents.
#[test]
fn a_kit_terminal_gets_the_kits_tools_for_linux() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-guest");
    let guest = kit.for_target(crate::kit::Target::LINUX_X64);
    let node = crate::setup::Catalog::get().find("node").unwrap();
    for key in node.key_paths(&guest) {
        std::fs::create_dir_all(key.parent().unwrap()).unwrap();
        std::fs::write(&key, b"").unwrap();
    }
    let terminal = crate::editor::terminals::find(&kit, "default", "it").unwrap();
    let base = KitEnv::build(&guest, &Default::default(), "it", &Host::default());
    let GuestTools { path, vars, .. } = guest_tools(&kit, &guest, &terminal, base);
    let windows = kit.target().is_windows();
    let apps = vm::guest_path(windows, &guest.apps()).unwrap();
    assert!(!path.is_empty(), "node's folder on the PATH");
    for entry in &path {
        assert!(entry.starts_with(&apps), "{entry}");
        assert!(!entry.contains('\\'), "{entry}");
    }
    for (name, value) in &vars {
        assert!(
            name == "ATOLLO_TERMINAL" || value.starts_with(&apps),
            "{name}={value}"
        );
        assert!(!["HOME", "TEMP", "TMPDIR", "CLAUDE_CONFIG_DIR"].contains(&name.as_str()));
    }
    assert!(vars.iter().any(|(n, _)| n == "ATOLLO_TERMINAL"));
}

/// A VM never writes where the PC runs files: not a drive's root, not a
/// folder holding the kit or the profile, not inside the kit's data or the
/// profile's settings; a project's folder and the projects' folder are fine.
#[test]
fn a_vm_never_gets_the_kit_or_the_profile_writable() {
    // The kit is its whole temporary folder; the PC's other folders are in
    // another one. Both by their real path: on macOS the temporary folder
    // is under /var, a link to /private/var, and a folder reached through
    // a link is never a workspace.
    let real = |p: &Path| {
        if cfg!(target_os = "macos") {
            std::fs::canonicalize(p).unwrap()
        } else {
            p.to_path_buf()
        }
    };
    let tmp = crate::kit::tests::TempDir::new("isolation-workspace");
    let kit = Kit::at(real(&tmp.0), crate::test_support::kit_arch());
    kit.prepare().unwrap();
    let pc = crate::kit::tests::TempDir::new("isolation-workspace-pc");
    let pc_root = real(&pc.0);
    let profile = pc_root.join("user");
    let in_profile = profile.join("Documents").join("shop");
    let sites = pc_root.join("sites").join("shop");
    let system = pc_root.join("Windows");
    for d in [
        &in_profile,
        &sites,
        &system,
        &kit.projects().join("shop"),
        &kit.data(),
    ] {
        std::fs::create_dir_all(d).unwrap();
    }
    let host = Host::from_pairs([
        ("USERPROFILE".into(), profile.clone().into_os_string()),
        ("SystemRoot".into(), system.clone().into_os_string()),
        ("HOME".into(), profile.clone().into_os_string()),
    ]);
    let allowed = |d: &Path| vm_workspace(&kit, &host, d);
    // The kit's projects, and a folder elsewhere, as their canonical path.
    let shop = allowed(&kit.projects().join("shop")).expect("a project of the kit");
    assert_eq!(
        shop,
        plain(&std::fs::canonicalize(kit.projects().join("shop")).unwrap())
    );
    assert!(!shop.to_string_lossy().starts_with(r"\\?\"), "{shop:?}");
    assert_eq!(
        allowed(&kit.projects()),
        None,
        "every project, writable, in one VM"
    );
    assert!(allowed(&sites).is_some(), "a folder elsewhere");
    // Never the kit, the profile, the system, nor what holds them.
    assert_eq!(allowed(kit.root()), None, "the kit's root");
    assert_eq!(allowed(&kit.data()), None, "the kit's data");
    assert_eq!(
        allowed(pc_root.parent().unwrap()),
        None,
        "holding the kit and the profile"
    );
    assert_eq!(allowed(&profile), None, "the profile");
    assert_eq!(allowed(&in_profile), None, "anything in the profile");
    if kit.target().is_windows() {
        assert_eq!(allowed(&system), None, "the system's folder");
    }
    assert_eq!(allowed(&pc_root.join("missing")), None, "unreadable");
    assert_eq!(allowed(Path::new("relative")), None);
    let root = kit.root().ancestors().last().unwrap();
    assert_eq!(allowed(root), None, "a drive's root");
    // A link on the way: the folder checked would not be the one mounted.
    let linked = pc_root.join("linked");
    crate::platform::safe_path::tests::junction(&linked, &sites);
    assert_eq!(allowed(&linked), None, "a junction");
    assert_eq!(allowed(&linked.join("x")), None, "under a junction");
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
            {{"name":"atollo-a-claude-shop-0123","status":"stopped"}}]"#
    );
    let vms = vms_of(&kit, &json);
    assert_eq!(vms.len(), 2, "the user's own sandbox is not Atollo's");
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

/// A project's programs lose the way to the user's profile, SSH keys,
/// session bus and the root's MySQL login; they log in as the project's
/// own user. A kit terminal is sealed the same way, with no login.
#[test]
fn a_project_gets_a_sealed_environment() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-seal");
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
    assert_eq!(env.get("PATH"), Some("kit"));

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
