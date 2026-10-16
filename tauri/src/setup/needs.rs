//! What a command needs from the kit: the components installed on demand
//! before it runs (DECISIONS.md D-59). Only PowerShell 7 and GitHub CLI are
//! always installed; Git, Node.js, uv and the languages come when an agent,
//! a clone or a project action needs them.

use std::collections::BTreeSet;
use std::path::Path;

use super::catalog::Catalog;
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
        "php" | "php-cgi" | "composer" => "php",
        "httpd" => "apache",
        "nginx" => "nginx",
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

/// The components a project action needs (`Atollo.exe serve` needs none,
/// the project's own Apache/Nginx needs the web server and PHP).
pub fn of_action(action: &Action) -> BTreeSet<&'static str> {
    if action.static_dir.is_some() {
        return BTreeSet::new();
    }
    if action.apache_dir.is_some() {
        return BTreeSet::from(["apache", "php"]);
    }
    if action.nginx_dir.is_some() {
        return BTreeSet::from(["nginx", "php"]);
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
        crate::agents::sync::mcp::load(&crate::agents::sync::Paths::of(kit)).unwrap_or_default();
    for server in servers.values() {
        if let Some(program) = server.get("command").and_then(|c| c.as_str()) {
            out.extend(component_of(program));
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../test/rust/setup/needs.rs"]
mod tests;
