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

/// A kit's terminal has a sandbox per folder, in its own namespace.
#[test]
fn a_kit_terminal_has_a_sandbox_per_folder() {
    let a = terminal_name("default", Path::new("/kit/projects"));
    assert!(a.starts_with("atollo-t-default-"), "{a}");
    assert_eq!(a.len(), "atollo-t-default-".len() + 24);
    assert_ne!(a, terminal_name("default", Path::new("/kit/projects/shop")));
    assert_ne!(a, terminal_name("legacy", Path::new("/kit/projects")));
    assert_eq!(a, terminal_name("default", Path::new("/kit/projects")));
    assert!(!a.starts_with("atollo-p-"));
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
    assert_eq!(guest_target(Target::WIN32_ARM64), None);
    assert_eq!(guest_target(Target::DARWIN_X64), None);
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
    let args = kit_terminal_args("atollo-t-x", "/d/sites/shop", &tools);
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

#[test]
fn the_sandbox_starts_with_the_project_as_its_workspace() {
    let dir = Path::new("/kit/projects/shop");
    assert_eq!(
        start_args("atollo-shop", dir, &[], None),
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
        start_args("atollo-t", dir, &["/kit/data/apps/linux-x64".into()], None)
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
    assert_eq!(
        publish_args("atollo-shop", 7101),
        ["ports", "atollo-shop", "--publish", "127.0.0.1:7101:7101"]
    );
}

/// Claude Code and Codex in `sbx`'s own agents; the others have none.
#[test]
fn only_sbx_agents_have_a_sandbox() {
    let dir = Path::new("/p");
    let claude = agent_name("shop", "claude");
    assert_eq!(
        agent_args("shop", "claude", dir).unwrap(),
        ["run", "--name", claude.as_str(), "claude", "/p"]
    );
    assert_eq!(
        agent_args("shop", "codex", dir).unwrap()[2],
        agent_name("shop", "codex")
    );
    assert_eq!(agent_args("atollo-shop", "antigravity", dir), None);
    assert_eq!(agent_args("atollo-shop", "pi", dir), None);
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
    let line = web_line("site", &Web::Php { line: "8.3".into() }, "public", 7102);
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
    let line = web_line("docs", &Web::Static, ".", 7103);
    assert!(line.contains("httpd:2.4"), "{line}");
    assert!(line.contains("/usr/local/apache2/htdocs"), "{line}");
    assert!(line.contains("httpd-foreground"), "{line}");
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
