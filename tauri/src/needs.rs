//! What a command needs from the kit: the components installed on demand
//! before it runs (DECISIONS.md D-59). Only PowerShell 7 and GitHub CLI are
//! always installed; Git, Node.js, uv and the languages come when an agent,
//! a clone or a project action needs them.

use std::collections::BTreeSet;
use std::path::Path;

use crate::catalog::Catalog;
use crate::detect::Action;
use crate::kit::Kit;

/// The component of a program, by its name (`npm`, `python.exe`,
/// `.venv\Scripts\python.exe`...).
fn component_of(program: &str) -> Option<&'static str> {
    let name = Path::new(program.trim_matches('"'))
        .file_name()?
        .to_string_lossy()
        .to_ascii_lowercase();
    let stem = [".exe", ".cmd", ".bat", ".ps1"]
        .iter()
        .find_map(|ext| name.strip_suffix(ext))
        .unwrap_or(&name);
    Some(match stem {
        "node" | "npm" | "npx" => "node",
        "pnpm" | "pnpx" | "yarn" | "yarnpkg" | "corepack" => "pnpm",
        "uv" | "uvx" => "uv",
        "python" | "python3" | "py" | "pythonw" | "pip" | "pip3" => "python",
        "php" | "composer" => "php",
        "httpd" => "apache",
        "cargo" | "rustc" | "rustup" | "trunk" => "rust",
        "go" | "gofmt" => "go",
        "java" | "javac" | "jar" | "mvn" | "gradle" => "java",
        "kotlin" | "kotlinc" => "kotlin",
        "sdkmanager" | "adb" | "avdmanager" => "android",
        "dotnet" => "dotnet",
        "dart" => "dart",
        "zig" => "zig",
        "julia" => "julia",
        "bun" | "bunx" => "bun",
        "deno" => "deno",
        "typst" => "typst",
        "pdflatex" | "xelatex" | "lualatex" | "latexmk" | "tlmgr" => "tinytex",
        "cmake" | "ninja" => "cmake",
        "gcc" | "g++" | "cc" | "c++" | "clang" | "clang++" | "make" | "mingw32-make"
        | "windres" => "cc",
        "ruby" | "gem" | "bundle" | "bundler" | "rails" | "rake" => "ruby",
        "r" | "rscript" => "r",
        "mariadb" | "mariadbd" | "mysql" | "mysqld" | "mariadb-dump" | "mysqldump" => "mariadb",
        "mongod" | "mongosh" => "mongodb",
        "sqlite3" => "sqlite",
        "git" => "git",
        "gh" => "gh",
        _ => return None,
    })
}

/// The components a `cmd.exe` command line needs: the first program of each
/// command (`a && b`, `a | b`...).
pub fn of_command(command: &str) -> BTreeSet<&'static str> {
    command
        .split(['&', '|'])
        .filter_map(first_program)
        .filter_map(component_of)
        .collect()
}

/// The first word of a command, or the quoted path it starts with.
fn first_program(command: &str) -> Option<&str> {
    let command = command.trim_start();
    match command.strip_prefix('"') {
        Some(rest) => rest.split('"').next(),
        None => command.split_whitespace().next(),
    }
}

/// The components a project action needs (`Dev-Kit.exe serve` needs none,
/// the project's own Apache needs Apache and PHP).
pub fn of_action(action: &Action) -> BTreeSet<&'static str> {
    if action.static_dir.is_some() {
        return BTreeSet::new();
    }
    if action.apache_dir.is_some() {
        return BTreeSet::from(["apache", "php"]);
    }
    of_command(&action.command)
}

/// What the agent `id` needs to start: the agent itself and the programs
/// of the kit's MCP servers (`npx`, `uvx`, `python`...). The catalog's own
/// dependencies (Claude Code needs Git) are added by the installation.
pub fn of_agent(kit: &Kit, id: &str) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    if let Some(c) = Catalog::get().find(id) {
        out.insert(c.id.as_str());
    }
    let servers =
        crate::agents_sync::mcp::load(&crate::agents_sync::Paths::of(kit)).unwrap_or_default();
    for server in servers.values() {
        if let Some(program) = server.get("command").and_then(|c| c.as_str()) {
            out.extend(component_of(program));
        }
    }
    out
}

#[cfg(test)]
mod tests {
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
        for id in ids("npm i & go build & dotnet run & ruby x.rb & Rscript a.R") {
            assert!(Catalog::get().find(id).is_some(), "{id}");
        }
    }

    #[test]
    fn every_id_is_in_the_catalog() {
        for program in [
            "node", "pnpm", "uv", "python", "php", "httpd", "cargo", "go", "java", "kotlinc",
            "dotnet", "dart", "zig", "julia", "bun", "deno", "typst", "latexmk", "cmake", "gcc",
            "ruby", "rscript", "git", "gh",
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
        let file = crate::agents_sync::Paths::of(&kit).mcp_file();
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
}
