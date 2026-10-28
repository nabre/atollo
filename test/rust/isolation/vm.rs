use super::*;

/// The project's name readable, then a hash of its exact name: two
/// projects never share a sandbox.
#[test]
fn the_sandbox_is_named_after_the_project() {
    let shop = name("shop");
    assert!(shop.starts_with("atollo-p-shop-"), "{shop}");
    assert_eq!(shop.len(), "atollo-p-shop-".len() + 24);
    assert!(name("My_Shop 2").starts_with("atollo-p-my-shop-2-"));
    assert!(name("v1.2").starts_with("atollo-p-v1.2-"));
    assert!(name("_x_").starts_with("atollo-p-x-"));
    // Names that look the same are told apart.
    assert_ne!(name("My Shop"), name("my-shop"));
    assert_ne!(name("Shop"), name("shop"));
    // A name of symbols only is its hash; a long one is cut, its hash kept.
    assert_eq!(name("___").len(), "atollo-p-".len() + 24);
    assert_eq!(name(&"a".repeat(100)).len(), NAME_MAX);
    assert_ne!(name(&"a".repeat(100)), name(&"a".repeat(101)));
    // `sbx kit add`'s swap container (`<name>-swap-<8 hex>`) stays within
    // Docker's 63 characters.
    assert_eq!(NAME_MAX + "-swap-".len() + 8, 63);
    // The same project, the same sandbox at every start.
    assert_eq!(name("shop"), shop);
}

/// A folder added read-only to an existing sandbox: the full form (the
/// short `<host>:ro` names `ro` as the target), the drive's `:` kept.
#[test]
fn a_folder_is_mounted_read_only_in_an_existing_sandbox() {
    assert_eq!(
        mount_args(
            "atollo-p-shop",
            Path::new(r"D:\kit\data\apps\linux-x64"),
            "/d/kit/data/apps/linux-x64"
        ),
        [
            "mount",
            "atollo-p-shop",
            r"D:\kit\data\apps\linux-x64:/d/kit/data/apps/linux-x64:ro"
        ]
    );
}

/// An agent's account (D-185) mounted read-write: the same form without
/// `:ro`.
#[test]
fn an_account_is_mounted_read_write() {
    assert_eq!(
        mount_rw_args(
            "atollo-p-shop",
            Path::new(r"D:\kit\data\accounts\claude\default"),
            "/d/kit/data/accounts/claude/default"
        ),
        [
            "mount",
            "atollo-p-shop",
            r"D:\kit\data\accounts\claude\default:/d/kit/data/accounts/claude/default"
        ]
    );
}

/// The kit's terminals (D-178) and Claude Code and Codex (D-184) have no
/// sandbox of their own any more: the names an earlier Atollo gave them
/// are apart from every name it gives now.
#[test]
fn no_name_given_now_is_a_legacy_one() {
    for name in [name("shop"), name("t-default"), name("a-claude-shop")] {
        assert!(!name.starts_with(TERMINAL_PREFIX), "{name}");
        assert!(!name.starts_with(AGENT_PREFIX), "{name}");
    }
}

/// Every agent in the project's VM (D-184): `sbx exec` in the project's
/// sandbox and folder, the project's tools first on the `PATH` and its
/// database's file, then the agent's build for Linux from the kit, quoted
/// for the VM's shell.
#[test]
fn every_agent_runs_in_the_projects_vm_with_its_tools() {
    let tools = crate::isolation::GuestTools {
        path: vec!["/d/kit/data/apps/linux-x64/node/24".into()],
        vars: vec![("JAVA_HOME".into(), "/d/kit/java".into())],
        env_file: Some(std::path::PathBuf::from("db.env")),
    };
    let vm = name("shop");
    for agent in ["claude", "codex", "pi", "antigravity"] {
        let program = format!("/d/kit/data/apps/linux-x64/{agent}/{agent}");
        let args = agent_session_args(&vm, "/d/kit/projects/shop", agent, &program, &tools, &[]);
        assert_eq!(args[..4], ["exec", "-it", "-w", "/d/kit/projects/shop"]);
        let at = |s: &str| args.iter().position(|a| a == s).unwrap();
        assert!(at("--env-file") < at(&vm));
        assert!(args.contains(&"ATOLLO_PATH=/d/kit/data/apps/linux-x64/node/24".to_owned()));
        assert!(args.contains(&"JAVA_HOME=/d/kit/java".to_owned()));
        assert!(vm.starts_with("atollo-p-"), "the project's own VM");
        assert_eq!(args[at(&vm) + 1..at(&vm) + 3], ["sh", "-c"]);
        let script = &args[at(&vm) + 3];
        assert!(script.starts_with(TOOLS_FIRST), "{script}");
        assert!(script.ends_with(r#"exec "$0" "$@""#), "{script}");
        assert_eq!(
            args.last().unwrap(),
            &program,
            "the agent is the program, nothing after it"
        );
        // Its own updates off, as options of sbx exec.
        for (k, v) in agent_vars(agent) {
            let var = at(&format!("{k}={v}"));
            assert_eq!(args[var - 1], "-e");
            assert!(var < at(&vm), "an option of sbx exec");
        }
        for word in &args {
            for kit_only in ["GEMINI", "GOOGLE_API_KEY", "ANTHROPIC", "OPENAI", "keyring"] {
                assert!(!word.contains(kit_only), "{agent}: {word}");
            }
        }
        // What the user typed after the agent's command (a forwarder)
        // follows, as it is.
        let typed = ["--continue".to_owned(), "it's \"mine\"".to_owned()];
        let with = agent_session_args(&vm, "/d/kit/projects/shop", agent, &program, &tools, &typed);
        assert_eq!(with[..args.len()], args[..]);
        assert_eq!(with[args.len()..], typed);
    }
}

/// The kit's builds are read-only in the VM and the kit updates them:
/// Antigravity's CLI and Claude Code are told not to update themselves;
/// Codex and Pi have no switch.
#[test]
fn the_agents_do_not_update_themselves() {
    assert_eq!(
        agent_vars("antigravity"),
        [("AGY_CLI_DISABLE_AUTO_UPDATE", "true")]
    );
    assert_eq!(agent_vars("claude"), [("DISABLE_AUTOUPDATER", "1")]);
    assert!(agent_vars("codex").is_empty());
    assert!(agent_vars("pi").is_empty());
}

/// Where the PC's folders are in the VM: `sbx` mounts `D:\a` at `/d/a`
/// (seen on Windows 11 with sbx 0.49.2), the same path elsewhere.
#[test]
fn the_pcs_folders_keep_their_path_in_the_vm() {
    assert_eq!(
        guest_path(true, Path::new(r"D:\Projects\My Kit\data\apps\linux-x64")).as_deref(),
        Some("/d/Projects/My Kit/data/apps/linux-x64")
    );
    assert_eq!(guest_path(true, Path::new(r"C:\")).as_deref(), Some("/c"));
    assert_eq!(
        guest_path(true, Path::new(r"\\?\E:\kit")).as_deref(),
        Some("/e/kit")
    );
    assert_eq!(guest_path(true, Path::new(r"\\server\share\kit")), None);
    assert_eq!(
        guest_path(false, Path::new("/media/u/KIT")).as_deref(),
        Some("/media/u/KIT")
    );
}

#[test]
fn the_vm_is_linux_of_the_pcs_processor() {
    use crate::kit::Target;
    assert_eq!(guest_target(Target::WIN32_X64), Some(Target::LINUX_X64));
    assert_eq!(
        guest_target(Target::DARWIN_ARM64),
        Some(Target::LINUX_ARM64)
    );
    assert_eq!(guest_target(Target::LINUX_ARM64), Some(Target::LINUX_ARM64));
    use crate::kit::{Arch, Os};
    assert_eq!(guest_target(Target::new(Os::Win32, Arch::Arm64)), None);
    assert_eq!(guest_target(Target::new(Os::Darwin, Arch::X64)), None);
}

/// An existing sandbox starts again by its name only: `sbx` refuses any
/// folder after the first, even those it was made with (sbx 0.49.2).
#[test]
fn an_existing_sandbox_starts_again_by_its_name() {
    assert_eq!(
        restart_args("atollo-p-shop"),
        ["run", "-d", "--name", "atollo-p-shop"]
    );
    assert_eq!(POLICY_ARGS, ["policy", "init", "balanced"]);
}

#[test]
fn the_sandbox_starts_with_the_project_as_its_workspace() {
    let dir = Path::new("/kit/projects/shop");
    assert_eq!(
        start_args("atollo-shop", dir, &[], &[], None),
        [
            "run",
            "-d",
            "--name",
            "atollo-shop",
            "shell",
            "/kit/projects/shop"
        ]
    );
    assert_eq!(
        start_args(
            "atollo-t",
            dir,
            &[],
            &["/kit/data/apps/linux-x64".into()],
            None
        )
        .last()
        .map(String::as_str),
        Some("/kit/data/apps/linux-x64:ro"),
        "the kit's tools are mounted where the VM cannot change them"
    );
    // Made with its own kit, before the agent (`sbx run [flags] AGENT`).
    assert_eq!(
        start_args(
            "atollo-shop",
            dir,
            &[],
            &[],
            Some(Path::new("/kit/data/app/sandboxes/atollo-shop"))
        ),
        [
            "run",
            "-d",
            "--name",
            "atollo-shop",
            "--kit",
            "/kit/data/app/sandboxes/atollo-shop",
            "shell",
            "/kit/projects/shop"
        ]
    );
}

/// The kit's tools first, then what the user installs in the VM's home
/// (D-181): the kit's folders are read-only there.
#[test]
fn the_users_own_installations_go_to_the_vms_home() {
    assert!(TOOLS_FIRST.starts_with(r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$HOME/.cargo/bin:"#));
    for (name, dir) in [
        ("GEM_HOME", "$HOME/.gem"),
        ("NPM_CONFIG_PREFIX", "$HOME/.npm-global"),
        ("R_LIBS_USER", "$HOME/R/library"),
    ] {
        assert!(
            TOOLS_FIRST.contains(&format!("{name}=\"${{{name}:-{dir}}}\"")),
            "{name}"
        );
        assert!(VM_HOMES.contains(&name), "{name}");
    }
    assert!(VM_HOMES.contains(&"CARGO_HOME"));
    // Only variables: «Verifica strumenti» reads only; a session makes R's
    // library.
    assert!(!TOOLS_FIRST.contains("mkdir") && !TOOLS_FIRST.contains('>'));
    assert_eq!(session_start(), format!("{TOOLS_FIRST}; {MAKE_HOMES}"));
}

/// The project's database stays on the PC (D-171): its sandbox, and it
/// only, reaches that one port over TCP; the rule goes again without a
/// question on a terminal (sbx 0.49.2 asks one without `--force`).
#[test]
fn a_sandbox_reaches_only_its_databases_port_on_the_pc() {
    assert_eq!(
        host_port_allow("atollo-p-shop", 3307),
        [
            "policy",
            "allow",
            "network",
            "--sandbox",
            "atollo-p-shop",
            "--protocol",
            "tcp",
            "localhost:3307"
        ]
    );
    assert_eq!(
        host_port_close("atollo-p-shop", 3307),
        [
            "policy",
            "rm",
            "network",
            "--sandbox",
            "atollo-p-shop",
            "--resource",
            "localhost:3307",
            "--force"
        ]
    );
    assert_eq!(HOST, "host.docker.internal");
}

/// The tools sandbox (D-180): its other writable folders after the
/// workspace, before any read-only one; its hosts a rule of its own.
#[test]
fn the_tools_sandbox_mounts_writable_folders_and_reaches_its_hosts() {
    let args = start_args(
        "atollo-k-tools",
        Path::new("/kit/data/apps/linux-x64"),
        &["/kit/data/npm-global".into()],
        &["/kit/ro".into()],
        None,
    );
    assert_eq!(
        args[4..],
        [
            "shell",
            "/kit/data/apps/linux-x64",
            "/kit/data/npm-global",
            "/kit/ro:ro"
        ]
    );
    assert_eq!(
        hosts_allow("atollo-k-tools", &["conda.anaconda.org", "dl.google.com"]),
        [
            "policy",
            "allow",
            "network",
            "--sandbox",
            "atollo-k-tools",
            "conda.anaconda.org,dl.google.com"
        ]
    );
}
