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
        of_agent(&kit, "codex").into_iter().collect::<Vec<_>>(),
        ["codex"]
    );
    let file = crate::agents::sync::Paths::of(&kit).mcp_file();
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(
        &file,
        r#"{"mcpServers": {"a": {"command": "npx", "args": ["x"]}, "b": {"command": "uvx"}, "c": {"url": "https://x"}}}"#,
    )
    .unwrap();
    assert_eq!(
        of_agent(&kit, "claude").into_iter().collect::<Vec<_>>(),
        ["claude", "node", "uv"]
    );
}
