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
/// `.venv\Scripts\python.exe`...). Both separators on every system: a
/// command written on Windows reaches macOS and Linux in a shared kit.
fn component_of(program: &str) -> Option<&'static str> {
    file_name(program.trim_matches('"')).and_then(|name| component_of_name(&name))
}

/// The last part of a path, with `/` or `\` as separators, as Windows'
/// `Path::file_name` reads it: trailing separators and `.` parts are
/// skipped, a drive (`C:node.exe`) is not part of the name, `..` has none.
pub(crate) fn file_name(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    let path = if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
        &path[2..]
    } else {
        path
    };
    let last = path
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty() && *part != ".")?;
    (last != "..").then(|| last.to_owned())
}

/// The component of a program's file name (`npm`, `python.exe`).
fn component_of_name(name: &str) -> Option<&'static str> {
    let name = name.to_ascii_lowercase();
    let stem = [".exe", ".cmd", ".bat", ".ps1"]
        .iter()
        .find_map(|ext| name.strip_suffix(ext))
        .unwrap_or(&name);
    Some(match stem {
        "node" | "npm" | "npx" => "node",
        "pnpm" | "pnpx" | "yarn" | "yarnpkg" | "corepack" => "pnpm",
        "uv" | "uvx" => "uv",
        "python" | "python3" | "py" | "pythonw" | "pip" | "pip3" => "python",
        "php" | "php-cgi" | "php-fpm" | "composer" => "php",
        "httpd" => "apache",
        "caddy" => "caddy",
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
        "mysql" | "mysqld" | "mysqldump" | "mysqladmin" => "mysql",
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
/// the project's own Apache/Caddy needs the web server and PHP).
pub fn of_action(action: &Action) -> BTreeSet<&'static str> {
    if action.static_dir.is_some() {
        return BTreeSet::new();
    }
    if action.apache_dir.is_some() {
        return BTreeSet::from(["apache", "php"]);
    }
    if action.caddy_dir.is_some() {
        return BTreeSet::from(["caddy", "php"]);
    }
    of_command(&action.command)
}

/// What the agent `id` needs to start in `project_dir`: the agent itself and
/// the programs of the kit's and the project's MCP servers (`npx`, `uvx`,
/// `python`...). The catalog's own dependencies (Claude Code needs Git) are
/// added by the installation.
// No agent starts on the PC since D-178 (the tests; the MCP servers of the
// agents in the VM are still to do, D-175).
#[allow(dead_code)]
pub fn of_agent(kit: &Kit, id: &str, project_dir: Option<&Path>) -> BTreeSet<&'static str> {
    let mut out = BTreeSet::new();
    if let Some(c) = Catalog::get().find(id) {
        out.insert(c.id.as_str());
    }
    let mut servers: Vec<Option<String>> =
        crate::agents::sync::mcp::load(&crate::agents::sync::Paths::of(kit))
            .unwrap_or_default()
            .values()
            .map(|s| s.get("command").and_then(|c| c.as_str()).map(str::to_owned))
            .collect();
    if let Some(dir) = project_dir {
        servers.extend(
            crate::projects::mcp::get(dir)
                .servers
                .into_iter()
                .map(|s| s.command),
        );
    }
    for program in servers.iter().flatten() {
        out.extend(component_of(program));
    }
    out
}

#[cfg(test)]
#[path = "../../../test/rust/setup/needs.rs"]
mod tests;
