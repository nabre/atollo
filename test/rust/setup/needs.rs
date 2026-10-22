use super::*;
use crate::kit::{tests::TempDir, Arch};

#[test]
fn commands() {
    let ids = |c: &str| of_command(c).into_iter().collect::<Vec<_>>();
    assert_eq!(ids("npm run dev -- --port 7100"), ["node"]);
    assert_eq!(ids("npx electron ."), ["node"]);
    assert_eq!(
        ids(r".venv\Scripts\python.exe -m uvicorn app:api"),
        ["python"]
    );
    assert_eq!(ids("uv run python manage.py runserver"), ["uv"]);
    assert_eq!(ids("composer install && php -S 127.0.0.1:7100"), ["php"]);
    assert_eq!(ids("cargo run | tee out.txt"), ["rust"]);
    assert_eq!(ids("pnpm dev"), ["pnpm"]);
    assert_eq!(ids("\"C:\\x y\\node.exe\" server.js"), ["node"]);
    assert!(ids("echo hi").is_empty());
    // Both separators on every system: a kit shared between Windows and Unix.
    assert_eq!(ids(".venv/bin/python -m pytest"), ["python"]);
    assert_eq!(ids(r"tools\bin/cargo.EXE build"), ["rust"]);
    assert!(ids(r"tools\ x").is_empty());
    for id in ids("npm i & go build & dotnet run & ruby x.rb & Rscript a.R") {
        assert!(Catalog::get().find(id).is_some(), "{id}");
    }
    assert_eq!(ids("mysql -e 'select 1'"), ["mysql"]);
    assert_eq!(ids("mysqldump app"), ["mysql"]);
    assert_eq!(ids("caddy run & php-fpm -F"), ["caddy", "php"]);
}

/// A project's own Caddy site needs Caddy and PHP before it starts.
#[test]
fn a_caddy_site_needs_caddy_and_php() {
    let mut a = crate::detect::Action::for_test("site", "", crate::detect::Kind::Web);
    a.caddy_dir = Some("public".into());
    assert_eq!(
        of_action(&a).into_iter().collect::<Vec<_>>(),
        ["caddy", "php"]
    );
}

/// Both separators on every system, and on Windows the same name as
/// `Path::file_name` ("Windows invariato").
#[test]
fn file_name_reads_windows_and_unix_paths() {
    let cases = [
        (r"C:\kit\node.exe", Some("node.exe")),
        ("/opt/kit/bin/node", Some("node")),
        (r"dir\", Some("dir")),
        ("dir//", Some("dir")),
        ("C:node.exe", Some("node.exe")),
        (r"a\.\b\.", Some("b")),
        (r"a\..", None),
        ("..", None),
        ("C:", None),
        ("", None),
        ("npm", Some("npm")),
    ];
    for (path, want) in cases {
        assert_eq!(file_name(path).as_deref(), want, "{path}");
        #[cfg(windows)]
        assert_eq!(
            std::path::Path::new(path)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .as_deref(),
            want,
            "Path::file_name of {path}"
        );
    }
    assert_eq!(component_of(r"dir\npm\"), Some("node"));
    assert_eq!(component_of("C:python.exe"), Some("python"));
}

#[test]
fn every_id_is_in_the_catalog() {
    for program in [
        "node", "pnpm", "uv", "python", "php", "httpd", "cargo", "go", "java", "kotlinc", "dotnet",
        "dart", "zig", "julia", "bun", "deno", "typst", "latexmk", "cmake", "gcc", "ruby",
        "rscript", "git", "gh",
    ] {
        let id = component_of(program).unwrap();
        assert!(Catalog::get().find(id).is_some(), "{program} -> {id}");
    }
}

#[test]
fn agents_need_their_mcp_programs() {
    let tmp = TempDir::new("needs-agent");
    let kit = Kit::at(tmp.0.clone(), Arch::X64);
    kit.prepare().unwrap();
    assert_eq!(
        of_agent(&kit, "codex", None)
            .into_iter()
            .collect::<Vec<_>>(),
        ["codex"]
    );
    // A project's own servers count too.
    let project = tmp.0.join("projects").join("p");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(
        project.join(".mcp.json"),
        r#"{"mcpServers": {"db": {"command": "npx", "args": ["-y", "@bytebase/dbhub"]}}}"#,
    )
    .unwrap();
    assert_eq!(
        of_agent(&kit, "codex", Some(&project))
            .into_iter()
            .collect::<Vec<_>>(),
        ["codex", "node"]
    );
    let file = crate::agents::sync::Paths::of(&kit).mcp_file();
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        r#"{"mcpServers": {"a": {"command": "npx", "args": ["x"]}, "b": {"command": "uvx"}, "c": {"url": "https://x"}}}"#,
    )
    .unwrap();
    assert_eq!(
        of_agent(&kit, "claude", None)
            .into_iter()
            .collect::<Vec<_>>(),
        ["claude", "node", "uv"]
    );
}
