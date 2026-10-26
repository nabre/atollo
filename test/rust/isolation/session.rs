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
    set_level(&kit, false, "no hypervisor").unwrap();
    assert_eq!(
        std::fs::read_to_string(base.join(".reason")).unwrap(),
        "no hypervisor\n"
    );
    set_level(&kit, true, "").unwrap();
    assert_eq!(
        std::fs::read_to_string(base.join(".level")).unwrap(),
        "vm\n"
    );
    assert!(!base.join(".reason").exists(), "no reason with the VM");
    clear(&kit, "shop").unwrap();
    assert!(!base.join("shop").exists());
    assert!(
        base.join(".level").is_file(),
        "the level is the kit's, not a project's"
    );
    set_level(&kit, false, "why").unwrap();
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

    // An editor's profile asks for an agent's session: taken once, only for
    // an agent that starts in the VM (Pi too, D-178; never Antigravity) and
    // a project with a session.
    for request in [
        "shop.claude.request",
        "shop.pi.request",
        "shop.antigravity.request",
        "blog.codex.request",
    ] {
        std::fs::write(root(&kit).join(request), "1").unwrap();
    }
    let mut taken = take_requests(&kit);
    taken.sort();
    assert_eq!(
        taken,
        [
            ("shop".to_owned(), "claude".to_owned()),
            ("shop".to_owned(), "pi".to_owned())
        ]
    );
    assert!(take_requests(&kit).is_empty());
    write_agent(&kit, "shop", "claude", &["run".into(), "--name".into()]).unwrap();
    assert_eq!(read("claude"), "run\n--name\n");
    agent_failed(&kit, "shop", "codex", "no VM").unwrap();
    assert_eq!(read("codex.error"), "no VM\n");
}

/// D-178: the editor's Pi sessions of a project, from `vm.ps1`'s markers
/// (`<project>.pi.<pid>`, the pid digits only): never another project's, a
/// request, or a folder. They are not requests, and the next start of the
/// editor (which rewrites `<project>/`) keeps them.
#[test]
fn the_editors_pi_sessions_are_its_markers() {
    let names = [
        "shop.pi.4120",
        "shop.pi.77",
        "shop.pi.77",
        "shop.pi.",
        "shop.pi.12a",
        "shop.pi.+5",
        "shop.pi.5.pi.9",
        "shop.pi.request",
        "shop.pi.request.request",
        "blog.pi.300",
        "shop",
        "xshop.pi.8",
    ];
    assert_eq!(pi_marker_pids(names, "shop"), [77, 4120]);
    assert_eq!(pi_marker_pids(names, "shop.pi.5"), [9]);
    assert_eq!(pi_marker_pids(names, "blog"), [300]);
    assert!(pi_marker_pids(names, "news").is_empty());

    let (_tmp, kit) = crate::test_support::prepared_kit("isolation-session-pi");
    let dir = kit.projects().join("shop");
    std::fs::create_dir_all(&dir).unwrap();
    let vm = Vm::for_tests(&kit, "shop", &dir);
    write(&kit, "shop", &dir, &vm, &KitEnv::default()).unwrap();
    std::fs::write(root(&kit).join("shop.pi.4120"), "4120\n").unwrap();
    std::fs::create_dir_all(root(&kit).join("shop.pi.99")).unwrap();
    assert!(take_requests(&kit).is_empty(), "not a request");
    pending(&kit, "shop").unwrap();
    write(&kit, "shop", &dir, &vm, &KitEnv::default()).unwrap();
    assert_eq!(pi_markers(&kit, "shop"), [4120], "kept, files only");
    assert!(pi_markers(&kit, "blog").is_empty());

    // `vm.ps1` writes the marker with its own pid around Pi's `sbx exec`,
    // and removes it when `sbx exec` returns, whatever the way.
    let script = include_str!("../../../runtime/vm.ps1").replace("\r\n", "\n");
    let marker = format!("$marker = Join-Path $sessions \"$project{PI_MARKER}$PID\"");
    assert!(script.contains(&marker), "{marker}");
    let write_at = script.find("Set-Content -LiteralPath $marker -Value $PID");
    let try_at = script.find("try {\n            & $sbx @argv");
    let remove_at = script.find("} finally {\n            Remove-Item -LiteralPath $marker");
    assert!(
        matches!((write_at, try_at, remove_at), (Some(w), Some(t), Some(r)) if w < t && t < r),
        "{write_at:?} {try_at:?} {remove_at:?}"
    );
    assert!(script.contains("if ($Agent -eq 'pi') {"));
}
