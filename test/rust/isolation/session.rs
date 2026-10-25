use super::*;

/// What `vm.ps1` reads: pending while the app makes the VM ready, the
/// session once ready, why when it failed, nothing once cleared; the VM's
/// mark apart.
#[test]
fn the_editors_session_goes_from_pending_to_ready_or_failed() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-session");
    let base = root(&kit);
    pending(&kit, "shop").unwrap();
    assert!(base.join("shop.pending").is_file());
    failed(&kit, "shop", "sbx run: boom").unwrap();
    assert!(!base.join("shop.pending").exists());
    assert_eq!(
        std::fs::read_to_string(base.join("shop").join("error")).unwrap(),
        "sbx run: boom\n"
    );
    set_level(&kit, true).unwrap();
    assert_eq!(
        std::fs::read_to_string(base.join(".level")).unwrap(),
        "vm\n"
    );
    clear(&kit, "shop").unwrap();
    assert!(!base.join("shop").exists());
    assert!(
        base.join(".level").is_file(),
        "the level is the kit's, not a project's"
    );
    set_level(&kit, false).unwrap();
    assert_eq!(
        std::fs::read_to_string(base.join(".level")).unwrap(),
        "env\n"
    );
    pending(&kit, "blog").unwrap();
    clear_all(&kit).unwrap();
    assert!(!base.exists());
    clear_all(&kit).unwrap();
}

/// The session's files, one argument per line: the interactive session in
/// the project's folder as the VM sees it, a task's line in place of
/// [`LINE`], the agents' sandboxes, and only the variables `sbx` needs
/// beside the editor's.
#[test]
fn a_session_holds_the_vms_commands() {
    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-session-write");
    let dir = kit.projects().join("shop");
    std::fs::create_dir_all(&dir).unwrap();
    let vm = Vm::for_tests(&kit, "shop", &dir);
    let mut env = KitEnv::default();
    env.set("PATH", "/kit/bin");
    write(&kit, "shop", &dir, &vm, &env).unwrap();
    let read = |name: &str| std::fs::read_to_string(root(&kit).join("shop").join(name)).unwrap();
    assert_eq!(read("sbx").trim(), "sbx");
    let vars = read("env");
    assert!(vars.contains("SBX_NO_TELEMETRY=1\n"), "{vars}");
    assert!(!vars.contains("PATH="), "{vars}");
    let shell: Vec<String> = read("shell").lines().map(str::to_owned).collect();
    let guest = vm::guest_path(kit.target().is_windows(), &dir).unwrap();
    assert_eq!(shell[..4], ["exec", "-it", "-w", guest.as_str()]);
    assert!(shell.contains(&vm.name));
    assert!(read("run").trim_end().ends_with(LINE));
    assert!(
        !root(&kit).join("shop/claude").exists(),
        "made on request only"
    );
    assert!(!root(&kit).join("shop.pending").exists());

    // An editor's profile asks for an agent's sandbox: taken once, only for
    // an agent of `sbx` and a project with a session.
    for request in [
        "shop.claude.request",
        "shop.pi.request",
        "blog.codex.request",
    ] {
        std::fs::write(root(&kit).join(request), "1").unwrap();
    }
    assert_eq!(
        take_requests(&kit),
        [("shop".to_owned(), "claude".to_owned())]
    );
    assert!(take_requests(&kit).is_empty());
    write_agent(&kit, "shop", "claude", &["run".into(), "--name".into()]).unwrap();
    assert_eq!(read("claude"), "run\n--name\n");
    agent_failed(&kit, "shop", "codex", "no VM").unwrap();
    assert_eq!(read("codex.error"), "no VM\n");
}
