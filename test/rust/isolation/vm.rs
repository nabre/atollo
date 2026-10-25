use super::*;

#[test]
fn the_sandbox_is_named_after_the_project() {
    assert_eq!(name("shop"), "atollo-shop");
    assert_eq!(name("My_Shop 2"), "atollo-my-shop-2");
    assert_eq!(name("v1.2"), "atollo-v1.2");
    assert_eq!(name("_x_"), "atollo-x");
}

#[test]
fn the_sandbox_starts_with_the_project_as_its_workspace() {
    let dir = Path::new("/kit/projects/shop");
    assert_eq!(
        start_args("atollo-shop", dir),
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
        terminal_args("atollo-shop"),
        ["exec", "-it", "atollo-shop", "bash", "-l"]
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
    assert_eq!(
        agent_args("atollo-shop", "claude", dir).unwrap(),
        ["run", "--name", "atollo-shop-claude", "claude", "/p"]
    );
    assert_eq!(
        agent_args("atollo-shop", "codex", dir).unwrap()[2],
        "atollo-shop-codex"
    );
    assert_eq!(agent_args("atollo-shop", "antigravity", dir), None);
    assert_eq!(agent_args("atollo-shop", "pi", dir), None);
}

/// The action's variables go to the VM with `-e`; its folder and line are
/// one word for the VM's shell.
#[test]
fn a_command_line_runs_in_the_vm() {
    let env = vec![("PORT".to_owned(), "7101".to_owned())];
    assert_eq!(
        exec_args("atollo-shop", None, "npm run dev", &env),
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
    let args = exec_args("atollo-shop", Some("web app"), "npm start", &[]);
    assert_eq!(args.last().unwrap(), "cd 'web app' && npm start");
    let args = exec_args("atollo-shop", Some("a'b"), "x", &[]);
    assert_eq!(args.last().unwrap(), r"cd 'a'\''b' && x");
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

/// The project's MySQL keeps its data in a volume of the VM, its user and
/// database as on the PC, on the port of its `DB_PORT`.
#[test]
fn the_projects_mysql_starts_once() {
    let line = mysql_line("8.4", "shop", "pw'1", 3307);
    assert!(
        line.starts_with("docker start 'atollo-mysql' >/dev/null 2>&1 || docker run -d"),
        "{line}"
    );
    assert!(
        line.contains("-v 'atollo-mysql-data':/var/lib/mysql"),
        "{line}"
    );
    assert!(line.contains("-e MYSQL_USER='shop'"), "{line}");
    assert!(line.contains(r"-e MYSQL_PASSWORD='pw'\''1'"), "{line}");
    assert!(line.contains("mysql:'8.4' --port=3307"), "{line}");
    // A root password nobody knows: the project has its user only.
    assert!(line.contains("MYSQL_RANDOM_ROOT_PASSWORD=yes"), "{line}");
    assert!(!line.contains("MYSQL_ROOT_PASSWORD="), "{line}");
}
