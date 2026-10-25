use super::*;

/// The network policy: the kit's `sbx` gets `balanced` (set already is
/// fine), the PC's is only told; texts of sbx 0.49.2.
#[test]
fn no_vm_starts_without_a_network_policy() {
    let kit = Sbx::Kit(PathBuf::from("sbx"));
    let pc = Sbx::Pc(PathBuf::from("sbx"));
    assert_eq!(
        policy_answer(&kit, true, "Initialized global network policy balanced\n"),
        Ok(())
    );
    assert_eq!(
        policy_answer(
            &kit,
            false,
            "error: global network policy is already initialized; reset it first to change it\n"
        ),
        Ok(())
    );
    let unset = "error: global network policy has not been initialized\n  try: sbx policy init <allow-all|balanced|deny-all>\n";
    assert_eq!(policy_answer(&pc, false, unset), Err(Blocker::PolicyUnset));
    assert_eq!(
        policy_answer(
            &pc,
            true,
            "POLICY         SOURCE   APPLIES TO   SUMMARY\nlocal-policy   local    all          network: 195 allow\n"
        ),
        Ok(())
    );
    assert!(matches!(
        policy_answer(&kit, false, "error: daemon not running\n"),
        Err(Blocker::SbxFailed { .. })
    ));
    assert_eq!(Fix::Policy.args(), ["policy", "init", "balanced"]);
    assert_eq!(
        serde_json::to_value(Blocker::PolicyUnset).unwrap(),
        serde_json::json!({ "code": "policy_unset" })
    );
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
    let (path, vars) = guest_tools(&kit, &guest, &terminal, base);
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
    // another one.
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-workspace");
    let pc = crate::kit::tests::TempDir::new("isolation-workspace-pc");
    let profile = pc.0.join("user");
    let in_profile = profile.join("Documents").join("shop");
    let sites = pc.0.join("sites").join("shop");
    let system = pc.0.join("Windows");
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
    assert!(allowed(&kit.projects()).is_some());
    assert!(allowed(&sites).is_some(), "a folder elsewhere");
    // Never the kit, the profile, the system, nor what holds them.
    assert_eq!(allowed(kit.root()), None, "the kit's root");
    assert_eq!(allowed(&kit.data()), None, "the kit's data");
    assert_eq!(
        allowed(pc.0.parent().unwrap()),
        None,
        "holding the kit and the profile"
    );
    assert_eq!(allowed(&profile), None, "the profile");
    assert_eq!(allowed(&in_profile), None, "anything in the profile");
    if kit.target().is_windows() {
        assert_eq!(allowed(&system), None, "the system's folder");
    }
    assert_eq!(allowed(&pc.0.join("missing")), None, "unreadable");
    assert_eq!(allowed(Path::new("relative")), None);
    let root = kit.root().ancestors().last().unwrap();
    assert_eq!(allowed(root), None, "a drive's root");
    // A link on the way: the folder checked would not be the one mounted.
    let linked = pc.0.join("linked");
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

#[test]
fn no_blocker_is_the_vm() {
    assert_eq!(Status::of(Vec::new(), None).level, Level::Vm);
    assert_eq!(
        Status::of(vec![Blocker::SbxMissing], None).level,
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
    let kit_env = Sbx::Kit(PathBuf::from("sbx")).session_env(&env, &host);
    assert_eq!(kit_env.get("LOCALAPPDATA"), Some("kit-home"));
    assert_eq!(kit_env.get("CP_KIT"), Some("x"));
    for name in ["TEMP", "TMP", "TMPDIR"] {
        assert_eq!(kit_env.get(name), Some(r"C:\T"), "{name}");
    }

    let pc_env = Sbx::Pc(PathBuf::from("sbx")).session_env(&env, &host);
    assert_eq!(pc_env.get("LOCALAPPDATA"), Some("pc-home"));
    assert_eq!(pc_env.get("CP_KIT"), None);
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
fn sbx_is_looked_for_in_the_path_then_where_installers_put_it() {
    let host = Host::from_pairs([
        ("Path".into(), r#"C:\Tools;"C:\Program Files\X";;"#.into()),
        ("LOCALAPPDATA".into(), r"C:\Users\u\AppData\Local".into()),
    ]);
    let dirs = sbx_dirs(Os::Win32, &host);
    assert_eq!(dirs[0], PathBuf::from(r"C:\Tools"));
    assert_eq!(dirs[1], PathBuf::from(r"C:\Program Files\X"));
    let winget = PathBuf::from(r"C:\Users\u\AppData\Local")
        .join("Microsoft")
        .join("WinGet")
        .join("Links");
    assert!(dirs.contains(&winget));
    let unix = Host::from_pairs([("PATH".into(), "/usr/bin:/home/u/bin".into())]);
    let dirs = sbx_dirs(Os::Linux, &unix);
    assert_eq!(
        dirs[..2],
        [PathBuf::from("/usr/bin"), PathBuf::from("/home/u/bin")]
    );
    assert_eq!(
        dirs.iter().filter(|d| **d == Path::new("/usr/bin")).count(),
        1
    );
    assert!(dirs.contains(&PathBuf::from("/opt/homebrew/bin")));
}

#[test]
fn an_unsupported_system_is_the_only_reason_given() {
    assert!(Blocker::Processor.unsupported());
    assert!(!Blocker::HypervisorOff.unsupported());
    assert!(!Blocker::SbxMissing.unsupported());
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
