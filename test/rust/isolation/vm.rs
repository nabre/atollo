use super::*;

/// The project's name readable, then a hash of its exact name: two
/// projects never share a sandbox, a project never an agent's.
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
    // An agent's sandbox is in its own namespace.
    assert_ne!(name("shop-claude"), agent_name("shop", "claude"));
    assert!(agent_name("shop", "claude").starts_with("atollo-a-claude-shop-"));
    // A name of symbols only is its hash; a long one is cut, its hash kept.
    assert_eq!(name("___").len(), "atollo-p-".len() + 24);
    assert!(name(&"a".repeat(100)).len() <= "atollo-p-".len() + 40 + 1 + 24);
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

/// The kit's terminals have no sandbox any more (D-178): the names an
/// earlier Atollo gave them are apart from every name it gives now.
#[test]
fn no_name_given_now_is_a_kit_terminals() {
    for name in [
        name("shop"),
        agent_name("shop", "claude"),
        agent_name("t-x", "codex"),
        name("t-default"),
    ] {
        assert!(!name.starts_with(TERMINAL_PREFIX), "{name}");
    }
}

/// Pi in the project's VM (D-178): `sbx exec` in the project's sandbox and
/// folder, the project's tools first on the `PATH` and its database's file,
/// then Pi's build for Linux from the kit, quoted for the VM's shell.
#[test]
fn pi_runs_in_the_projects_vm_with_its_tools() {
    let tools = crate::isolation::GuestTools {
        path: vec!["/d/kit/data/apps/linux-x64/node/24".into()],
        vars: vec![("JAVA_HOME".into(), "/d/kit/java".into())],
        env_file: Some(std::path::PathBuf::from("db.env")),
    };
    let vm = name("shop");
    let pi = "/d/kit/data/apps/linux-x64/pi/pi";
    let args = pi_session_args(&vm, "/d/kit/projects/shop", pi, &tools, &[]);
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
        pi,
        "Pi is the program, nothing after it"
    );
    // What the user typed after `pi` (a forwarder) follows, as it is.
    let typed = ["--continue".to_owned(), "it's \"mine\"".to_owned()];
    let with = pi_session_args(&vm, "/d/kit/projects/shop", pi, &tools, &typed);
    assert_eq!(with[..args.len()], args[..]);
    assert_eq!(with[args.len()..], typed);
    assert_eq!(sbx_agent("pi"), None, "Pi has no sandbox of `sbx`'s");
}

/// Antigravity in the project's VM (D-179, LEVELS.md §9): as Pi, its build
/// for Linux from the kit, its own updates off; no key, no keyring, no
/// variable of the kit's Antigravity data.
#[test]
fn agy_session_args() {
    let tools = crate::isolation::GuestTools {
        path: vec!["/d/kit/data/apps/linux-x64/node/24".into()],
        vars: vec![("JAVA_HOME".into(), "/d/kit/java".into())],
        env_file: Some(std::path::PathBuf::from("db.env")),
    };
    let vm = name("shop");
    let agy = "/d/kit/data/apps/linux-x64/antigravity/agy";
    let dir = "/d/kit/projects/shop/web";
    let args = super::agy_session_args(&vm, dir, agy, &tools, &[]);
    assert_eq!(args[..4], ["exec", "-it", "-w", dir]);
    let at = |s: &str| args.iter().position(|a| a == s).unwrap();
    assert!(vm.starts_with("atollo-p-"), "the project's own VM");
    assert!(at("--env-file") < at(&vm));
    assert!(args.contains(&"ATOLLO_PATH=/d/kit/data/apps/linux-x64/node/24".to_owned()));
    let update = at("AGY_CLI_DISABLE_AUTO_UPDATE=true");
    assert_eq!(args[update - 1], "-e");
    assert!(update < at(&vm), "an option of sbx exec");
    assert_eq!(args[at(&vm) + 1..at(&vm) + 3], ["sh", "-c"]);
    let script = &args[at(&vm) + 3];
    assert!(script.starts_with(TOOLS_FIRST), "{script}");
    assert!(script.ends_with(r#"exec "$0" "$@""#), "{script}");
    assert_eq!(args.last().unwrap(), agy, "agy is the program");
    for word in &args {
        for kit_only in ["GEMINI", "GOOGLE_API_KEY", "CP_ANTIGRAVITY", "keyring"] {
            assert!(!word.contains(kit_only), "{word}");
        }
    }
    // The words typed after `agy` (a forwarder) follow, as they are.
    let typed = ["--resume".to_owned(), "a b".to_owned()];
    let with = super::agy_session_args(&vm, dir, agy, &tools, &typed);
    assert_eq!(with[..args.len()], args[..]);
    assert_eq!(with[args.len()..], typed);
    // Pi's are the same but for agy's own variable.
    let pi = pi_session_args(&vm, dir, agy, &tools, &[]);
    let without: Vec<&String> = args
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != update && *i != update - 1)
        .map(|(_, a)| a)
        .collect();
    assert_eq!(without, pi.iter().collect::<Vec<_>>());
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

/// The kit's terminal in the VM: a login bash in its folder, the kit's
/// tools first on the PATH, their variables passed.
#[test]
fn a_kit_terminal_runs_bash_with_the_kits_tools_first() {
    let tools = crate::isolation::GuestTools {
        path: vec![
            "/d/kit/data/apps/linux-x64/node/24.22/bin".into(),
            "/d/kit/data/apps/linux-x64/python/3.15/bin".into(),
        ],
        vars: vec![("JAVA_HOME".into(), "/d/kit/data/apps/linux-x64/java".into())],
        env_file: Some("/kit/data/app/sandboxes/x/db.env".into()),
    };
    let args = kit_terminal_args("atollo-t-x", "/d/sites/shop", &tools, None);
    // With the session's level line, printed by the VM's shell first.
    let with = kit_terminal_args("atollo-t-x", "/d/sites/shop", &tools, Some("[Atollo] VM"));
    assert_eq!(with[..11], args[..11]);
    assert_eq!(
        with[11..],
        [
            "sh",
            "-c",
            r#"printf '%s\n' "$1" >&2; PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH" exec bash -l"#,
            "sh",
            "[Atollo] VM",
        ]
    );
    assert_eq!(
        args,
        [
            "exec",
            "-it",
            "-w",
            "/d/sites/shop",
            "--env-file",
            "/kit/data/app/sandboxes/x/db.env",
            "-e",
            "ATOLLO_PATH=/d/kit/data/apps/linux-x64/node/24.22/bin:/d/kit/data/apps/linux-x64/python/3.15/bin",
            "-e",
            "JAVA_HOME=/d/kit/data/apps/linux-x64/java",
            "atollo-t-x",
            "sh",
            "-c",
            r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH" exec bash -l"#,
        ]
    );
    assert_eq!(POLICY_ARGS, ["policy", "init", "balanced"]);
}

/// An existing sandbox starts again by its name only: `sbx` refuses any
/// folder after the first, even those it was made with (sbx 0.49.2).
#[test]
fn an_existing_sandbox_starts_again_by_its_name() {
    assert_eq!(
        restart_args("atollo-p-shop"),
        ["run", "-d", "--name", "atollo-p-shop"]
    );
}

#[test]
fn the_sandbox_starts_with_the_project_as_its_workspace() {
    let dir = Path::new("/kit/projects/shop");
    assert_eq!(
        start_args("atollo-shop", dir, &[], None, "shell"),
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
            &["/kit/data/apps/linux-x64".into()],
            None,
            "shell"
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
            Some(Path::new("/kit/data/app/sandboxes/atollo-shop")),
            "shell"
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
    assert_eq!(
        publish_args("atollo-shop", 7101, false),
        ["ports", "atollo-shop", "--publish", "127.0.0.1:7101:7101"]
    );
    // The local network when Settings allow it (tried with sbx 0.49.2).
    assert_eq!(
        publish_args("atollo-shop", 7101, true)[3],
        "0.0.0.0:7101:7101"
    );
    assert_eq!(
        unpublish_args("atollo-shop", 7101, false),
        ["ports", "atollo-shop", "--unpublish", "127.0.0.1:7101:7101"]
    );
    // Published already, on the address asked only (sbx 0.49.2's text).
    let local = "error: publish ports: publish ports: request failed: 409 Conflict: request[0]: port 127.0.0.1:7101/tcp4 is already published";
    assert!(published_already(local, 7101, false));
    assert!(!published_already(local, 7101, true), "not on the network");
    assert!(!published_already(local, 7102, false), "another port");
    assert!(!published_already("error: daemon not running", 7101, false));
}

/// Ending `sbx exec` on the PC leaves the action running in the VM (tried
/// with sbx 0.49.2): it notes its process group, and the stop ends that
/// group inside the VM, never with a note that is not a plain number.
#[test]
fn an_action_in_the_vm_is_stopped_inside_it() {
    let tag = run_tag("Web Server");
    assert!(tag.starts_with("web-server-"), "{tag}");
    assert_eq!(tag.len(), "web-server-".len() + 12);
    assert_ne!(run_tag("a"), run_tag("A"));
    assert_eq!(
        tracked(&tag, "npm start"),
        format!("mkdir -p /tmp/atollo-runs && echo $$ > /tmp/atollo-runs/{tag} && npm start")
    );
    let args = stop_args("atollo-p-shop", &tag, Some("atollo-web-site"));
    assert_eq!(args[..4], ["exec", "atollo-p-shop", "sh", "-c"]);
    let script = &args[4];
    assert!(
        script.starts_with(&format!("f='/tmp/atollo-runs/{tag}';")),
        "{script}"
    );
    assert!(script.contains("''|*[!0-9]*|0|1) ;;"), "{script}");
    assert!(script.contains("kill -TERM -$g"), "{script}");
    assert!(script.contains("kill -KILL -$g"), "{script}");
    assert!(
        script.ends_with("; docker rm -f 'atollo-web-site' >/dev/null 2>&1; true"),
        "{script}"
    );
    assert!(stop_args("atollo-p-shop", &tag, None)[4].ends_with("esac; true"));
}

/// Claude Code and Codex in `sbx`'s own agents, made like the project's
/// sandbox; the others have none.
#[test]
fn only_sbx_agents_have_a_sandbox() {
    assert_eq!(sbx_agent("claude"), Some("claude"));
    assert_eq!(sbx_agent("codex"), Some("codex"));
    assert_eq!(sbx_agent("antigravity"), None);
    assert_eq!(sbx_agent("pi"), None);
    let dir = Path::new("/p");
    let claude = agent_name("shop", "claude");
    assert_eq!(
        start_args(&claude, dir, &[], None, "claude")[4..],
        ["claude", "/p"]
    );
}

/// The agent's session (D-175): the project's database from its file, the
/// kit's tools first on the image's own `PATH` (as read in the sandbox),
/// the tools' variables after.
#[test]
fn an_agent_gets_the_projects_tools_and_database() {
    let tools = crate::isolation::GuestTools {
        path: vec!["/d/kit/data/apps/linux-x64/node/24.22/bin".into()],
        vars: vec![("JAVA_HOME".into(), "/d/j".into())],
        env_file: Some("/kit/db.env".into()),
    };
    let image = "/home/agent/.local/bin:/usr/local/bin:/usr/bin:/bin";
    assert_eq!(
        agent_session_args("atollo-a-claude-shop", image, &tools),
        [
            "run",
            "--name",
            "atollo-a-claude-shop",
            "--env-file",
            "/kit/db.env",
            "-e",
            "PATH=/d/kit/data/apps/linux-x64/node/24.22/bin:/home/agent/.local/bin:/usr/local/bin:/usr/bin:/bin",
            "-e",
            "JAVA_HOME=/d/j"
        ]
    );
    let none = crate::isolation::GuestTools::default();
    assert_eq!(
        agent_session_args("atollo-a-codex-shop", image, &none),
        ["run", "--name", "atollo-a-codex-shop"]
    );
}

/// The action's variables go to the VM with `-e`; its folder and line are
/// one word for the VM's shell.
#[test]
fn a_command_line_runs_in_the_vm() {
    let env = vec![("PORT".to_owned(), "7101".to_owned())];
    let none = crate::isolation::GuestTools::default();
    assert_eq!(
        exec_args("atollo-shop", None, "npm run dev", &env, &none),
        [
            "exec",
            "-e",
            "PORT=7101",
            "atollo-shop",
            "sh",
            "-lc",
            "npm run dev"
        ]
    );
    let args = exec_args("atollo-shop", Some("web app"), "npm start", &[], &none);
    assert_eq!(args.last().unwrap(), "cd 'web app' && npm start");
    let args = exec_args("atollo-shop", Some("a'b"), "x", &[], &none);
    assert_eq!(args.last().unwrap(), r"cd 'a'\''b' && x");
    // With the kit's tools (D-170): first on the PATH, set before the
    // `cd`; the database's login from a file, never on the line (D-171);
    // the tools' variables before the action's.
    let tools = crate::isolation::GuestTools {
        path: vec!["/d/kit/data/apps/linux-x64/node/24.22/bin".into()],
        vars: vec![("JAVA_HOME".into(), "/d/j".into())],
        env_file: Some("/kit/db.env".into()),
    };
    let args = exec_args("atollo-shop", Some("web"), "npm start", &env, &tools);
    assert_eq!(
        args[..9],
        [
            "exec",
            "--env-file",
            "/kit/db.env",
            "-e",
            "ATOLLO_PATH=/d/kit/data/apps/linux-x64/node/24.22/bin",
            "-e",
            "JAVA_HOME=/d/j",
            "-e",
            "PORT=7101"
        ]
    );
    assert_eq!(
        args.last().unwrap(),
        r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH"; export PATH; cd 'web' && npm start"#
    );
}

#[test]
fn a_php_site_is_apache_of_its_line_on_its_port() {
    let line = web_line(
        "site",
        &Web::Php { line: "8.3".into() },
        "public",
        7102,
        &[],
    );
    assert!(
        line.starts_with("docker rm -f 'atollo-web-site' >/dev/null 2>&1; "),
        "{line}"
    );
    assert!(line.contains("--network host"), "{line}");
    assert!(
        line.contains(r#"-v "$PWD"/'public':/var/www/html php:8.3-apache"#),
        "{line}"
    );
    assert!(line.contains("Listen 7102"), "{line}");
    assert!(line.contains("apache2-foreground"), "{line}");
    let line = web_line("docs", &Web::Static, ".", 7103, &[]);
    assert!(line.contains("httpd:2.4"), "{line}");
    assert!(line.contains("/usr/local/apache2/htdocs"), "{line}");
    assert!(line.contains("httpd-foreground"), "{line}");
    assert!(!line.contains("Alias"), "{line}");
}

/// A site's path aliases of the project itself: each folder mounted in the
/// container and an `Alias` of Apache for it, quoted for the VM's shell.
#[test]
fn a_sites_aliases_reach_its_container() {
    let aliases = vec![(
        "/media".to_owned(),
        "/d/kit/projects/my shop/storage/up".to_owned(),
    )];
    let line = web_line(
        "site",
        &Web::Php { line: "8.4".into() },
        "public",
        7104,
        &aliases,
    );
    assert!(
        line.contains("-v '/d/kit/projects/my shop/storage/up:/srv/alias/0' php:8.4-apache"),
        "{line}"
    );
    assert!(
        line.contains("/etc/apache2/conf-enabled/atollo-aliases.conf"),
        "{line}"
    );
    assert!(line.contains(r#"Alias "/media" "/srv/alias/0""#), "{line}");
    assert!(line.contains("Require all granted"), "{line}");
    let line = web_line("docs", &Web::Static, ".", 7105, &aliases);
    assert!(
        line.contains(">> /usr/local/apache2/conf/httpd.conf"),
        "{line}"
    );
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
