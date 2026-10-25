//! A project in its microVM (D-167): one Docker Sandboxes sandbox per
//! project, `atollo-<project>`, with the project's folder as its workspace
//! (the folder the commands start in). What runs there:
//!
//! - the terminal: `sbx exec -it <vm> bash -l`;
//! - an action's command line: `sbx exec -e … <vm> sh -lc '<line>'`, its
//!   port published on the PC's `127.0.0.1` (`sbx ports`);
//! - the web servers and MySQL: containers of the VM's own Docker on the
//!   versions the project chose (`php:<line>-apache`, `httpd:2.4`,
//!   `mysql:<line>`), on the VM's network, so `127.0.0.1:<DB_PORT>` of the
//!   project's `.env` reaches its database there as it does on the PC;
//! - Claude Code and Codex: `sbx`'s own agents, a sandbox each
//!   (`atollo-<project>-claude`), whose keys the proxy of `sbx` adds to
//!   their requests. The other agents have no sandbox: not started.
//!
//! The words given to `sh -lc` are quoted for the VM's shell (POSIX):
//! never text of the PC's shell.

use std::path::Path;

use crate::platform::sh_quote as shell_quote_posix;

/// The prefix of the projects' sandboxes.
pub const PREFIX: &str = "atollo-";

/// The sandbox of `project`: `atollo-` and the project's name with what
/// `sbx` does not take (`[^A-Za-z0-9.-]`) turned into `-`, lower case.
pub fn name(project: &str) -> String {
    let body: String = project
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    format!("{PREFIX}{}", body.trim_matches(['-', '.']))
}

/// Creates the sandbox the first time, starts it again when stopped:
/// `sbx run -d --name <vm> shell <dir>`.
pub fn start_args(vm: &str, dir: &Path) -> Vec<String> {
    vec![
        "run".into(),
        "-d".into(),
        "--name".into(),
        vm.into(),
        "shell".into(),
        dir.display().to_string(),
    ]
}

/// The terminal of the project: a login bash in its workspace.
pub fn terminal_args(vm: &str) -> Vec<String> {
    ["exec", "-it", vm, "bash", "-l"].map(String::from).into()
}

/// The agents `sbx` runs in a sandbox of their own: Atollo's id → `sbx`'s.
pub fn sbx_agent(agent: &str) -> Option<&'static str> {
    match agent {
        "claude" => Some("claude"),
        "codex" => Some("codex"),
        _ => None,
    }
}

/// `agent` in its sandbox for the project: `sbx run --name
/// <vm>-<agent> <agent> <dir>` (created the first time, attached after).
pub fn agent_args(vm: &str, agent: &str, dir: &Path) -> Option<Vec<String>> {
    let id = sbx_agent(agent)?;
    Some(vec![
        "run".into(),
        "--name".into(),
        format!("{vm}-{id}"),
        id.into(),
        dir.display().to_string(),
    ])
}

/// Publishes `port` of the VM on the PC's `127.0.0.1:<port>`.
pub fn publish_args(vm: &str, port: u16) -> Vec<String> {
    vec![
        "ports".into(),
        vm.into(),
        "--publish".into(),
        format!("127.0.0.1:{port}:{port}"),
    ]
}

/// A command line run in the VM: `sbx exec -e K=V … <vm> sh -lc '<line>'`,
/// in `folder` of the workspace when given (relative, `/`), with the
/// action's variables. A server in the VM listens on every card of the VM
/// (`HOST=0.0.0.0`): only the published port reaches the PC.
pub fn exec_args(
    vm: &str,
    folder: Option<&str>,
    line: &str,
    env: &[(String, String)],
) -> Vec<String> {
    let mut args: Vec<String> = vec!["exec".into()];
    for (k, v) in env {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args.push(vm.into());
    args.push("sh".into());
    args.push("-lc".into());
    args.push(match folder {
        Some(f) => format!("cd {} && {line}", shell_quote_posix(f)),
        None => line.to_owned(),
    });
    args
}

/// What serves a web action in the VM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Web {
    /// Apache with PHP of `line` (`php:<line>-apache`).
    Php { line: String },
    /// Files only (`httpd:2.4`).
    Static,
}

/// The container of a web action, in the foreground of `sh -lc` (it ends
/// with the action): `folder` of the workspace (relative, `/`; `.` the
/// whole) as Apache's document root, on `port` of the VM's network, a
/// container of the same name left by an earlier start removed first.
pub fn web_line(action: &str, web: &Web, folder: &str, port: u16) -> String {
    let container = shell_quote_posix(&format!("{PREFIX}web-{action}"));
    let root = format!("\"$PWD\"/{}", shell_quote_posix(folder));
    let (image, docroot) = match web {
        Web::Php { line } => (format!("php:{line}-apache"), "/var/www/html"),
        Web::Static => ("httpd:2.4".to_owned(), "/usr/local/apache2/htdocs"),
    };
    let listen = match web {
        Web::Php { .. } => format!(
            "sed -i 's/^Listen 80$/Listen {port}/' /etc/apache2/ports.conf && \
             sed -i 's/:80>/:{port}>/' /etc/apache2/sites-enabled/000-default.conf && \
             exec apache2-foreground"
        ),
        Web::Static => format!(
            "sed -i 's/^Listen 80$/Listen {port}/' /usr/local/apache2/conf/httpd.conf && \
             exec httpd-foreground"
        ),
    };
    format!(
        "docker rm -f {container} >/dev/null 2>&1; \
         exec docker run --rm --init --name {container} --network host \
         -v {root}:{docroot} {image} sh -c {}",
        shell_quote_posix(&listen)
    )
}

/// The project's MySQL in the VM, started once and kept (its data in a
/// volume of the VM): the version `line`, the project's database and user
/// with its password, on `port` of the VM's network.
pub fn mysql_line(line: &str, user: &str, password: &str, port: u16) -> String {
    let q = |s: &str| shell_quote_posix(s);
    let container = format!("{PREFIX}mysql");
    format!(
        "docker start {c} >/dev/null 2>&1 || docker run -d --name {c} --network host \
         -v {v}:/var/lib/mysql -e MYSQL_RANDOM_ROOT_PASSWORD=yes \
         -e MYSQL_DATABASE={u} -e MYSQL_USER={u} -e MYSQL_PASSWORD={p} \
         mysql:{l} --port={port} --bind-address=127.0.0.1 >/dev/null",
        c = q(&container),
        v = q(&format!("{container}-data")),
        u = q(user),
        p = q(password),
        l = q(line),
    )
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/vm.rs"]
mod tests;
