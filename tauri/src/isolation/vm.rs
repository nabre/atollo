//! A project in its microVM (D-167): one Docker Sandboxes sandbox per
//! project, `atollo-p-<project>-<hash>`, with the project's folder as its workspace
//! (the folder the commands start in). What runs there:
//!
//! - the terminal: `sbx exec -it -w <folder> <vm> … bash -l`, the kit's
//!   tools for Linux first on the `PATH` (D-170, [`kit_terminal_args`]);
//! - an action's command line: `sbx exec -e … <vm> sh -lc '<line>'`, its
//!   port published on the PC's `127.0.0.1` (`sbx ports`);
//! - the web servers and MySQL: containers of the VM's own Docker on the
//!   versions the project chose (`php:<line>-apache`, `httpd:2.4`,
//!   `mysql:<line>`), on the VM's network, so `127.0.0.1:<DB_PORT>` of the
//!   project's `.env` reaches its database there as it does on the PC;
//! - Claude Code and Codex: `sbx`'s own agents, a sandbox each
//!   (`atollo-a-claude-<project>-<hash>`), whose keys the proxy of `sbx` adds to
//!   their requests. The other agents have no sandbox: not started.
//!
//! The words given to `sh -lc` are quoted for the VM's shell (POSIX):
//! never text of the PC's shell.

use std::path::Path;

use sha2::{Digest, Sha256};

use crate::platform::sh_quote as shell_quote_posix;

/// The prefix of everything Atollo names in the VMs.
pub const PREFIX: &str = "atollo-";

/// `project` readable in a name `sbx` takes (`[a-z0-9.-]`, at most 40
/// characters), then the first 24 hex digits of the SHA-256 of its exact
/// name: two projects never share a sandboxes' name, and so never a
/// workspace (`My Shop` and `my-shop` look the same, their hashes not).
fn tag(project: &str) -> String {
    tag_of(project, project)
}

/// [`tag`] with the readable part from `readable` and the hash of `exact`.
fn tag_of(readable: &str, exact: &str) -> String {
    let slug: String = readable
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .take(40)
        .collect();
    let hash = Sha256::digest(exact.as_bytes());
    // 96 bits: no name chosen on purpose (a cloned repository's) reaches
    // another project's sandbox by brute force.
    let hex: String = hash[..12].iter().map(|b| format!("{b:02x}")).collect();
    let slug = slug.trim_matches(['-', '.']);
    if slug.is_empty() {
        hex
    } else {
        format!("{slug}-{hex}")
    }
}

/// The sandbox of `project`: `atollo-p-<project>-<hash>`.
pub fn name(project: &str) -> String {
    format!("{PREFIX}p-{}", tag(project))
}

/// The sandbox of `agent` for `project`, in a namespace of its own:
/// `atollo-a-<agent>-<project>-<hash>`, never a project's.
pub fn agent_name(project: &str, agent: &str) -> String {
    format!("{PREFIX}a-{agent}-{}", tag(project))
}

/// The sandbox of the kit's terminal `terminal` (its id) in `dir`:
/// `atollo-t-<terminal>-<hash>`, the hash of both, so each folder has its
/// own and never a project's (`atollo-p-`).
pub fn terminal_name(terminal: &str, dir: &Path) -> String {
    let exact = format!("{terminal}\n{}", dir.display());
    format!("{PREFIX}t-{}", tag_of(terminal, &exact))
}

/// Creates the sandbox the first time, starts it again when stopped:
/// `sbx run -d --name <vm> [--kit <kit>] shell <dir> [<folder>:ro…]`, its
/// own kit when it is made (`kit`, D-169), the `read_only` folders mounted
/// too, where the VM cannot change them.
pub fn start_args(
    vm: &str,
    dir: &Path,
    read_only: &[std::path::PathBuf],
    kit: Option<&Path>,
) -> Vec<String> {
    let mut args: Vec<String> = vec!["run".into(), "-d".into(), "--name".into(), vm.into()];
    if let Some(kit) = kit {
        args.push("--kit".into());
        args.push(kit.display().to_string());
    }
    args.push("shell".into());
    args.push(dir.display().to_string());
    args.extend(read_only.iter().map(|p| format!("{}:ro", p.display())));
    args
}

/// The global network policy Atollo gives the kit's `sbx` before its first
/// VM (scelta dell'utente, D-167): Docker's recommended one, the usual
/// traffic of development (AI services, package registries) and nothing
/// else.
pub const POLICY_ARGS: [&str; 3] = ["policy", "init", "balanced"];

/// The Linux of the microVM on a PC of `host`: Docker Sandboxes runs one of
/// the PC's processor (`linux-x64` on Windows x64, `linux-arm64` on Apple
/// silicon); `None` where there is no VM.
pub fn guest_target(host: crate::kit::Target) -> Option<crate::kit::Target> {
    use crate::kit::Target;
    match host {
        Target::WIN32_X64 | Target::LINUX_X64 => Some(Target::LINUX_X64),
        Target::DARWIN_ARM64 | Target::LINUX_ARM64 => Some(Target::LINUX_ARM64),
        _ => None,
    }
}

/// Where the folder `path` of the PC is in the VM: the same path on macOS
/// and Linux; on Windows `D:\a\b` is `/d/a/b` (the drive in lower case, as
/// `sbx` mounts it). `None` for a path without a drive (a network share).
pub fn guest_path(windows: bool, path: &Path) -> Option<String> {
    let text = path.to_string_lossy();
    if !windows {
        return Some(text.into_owned());
    }
    let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
    let mut chars = text.chars();
    let (drive, colon) = (chars.next()?, chars.next()?);
    if !drive.is_ascii_alphabetic() || colon != ':' {
        return None;
    }
    let rest = chars.as_str().replace('\\', "/");
    let rest = rest.trim_end_matches('/');
    Some(format!("/{}{rest}", drive.to_ascii_lowercase()))
}

/// The kit's terminal in its VM: a login bash in `dir` (the VM's path),
/// the kit's tools for Linux first on the `PATH` (`path`, VM paths) and
/// their variables (`vars`), the VM's own programs after them.
pub fn kit_terminal_args(
    vm: &str,
    dir: &str,
    path: &[String],
    vars: &[(String, String)],
) -> Vec<String> {
    let mut args: Vec<String> = ["exec", "-it", "-w", dir].map(String::from).into();
    args.push("-e".into());
    args.push(format!("ATOLLO_PATH={}", path.join(":")));
    for (k, v) in vars {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args.push(vm.into());
    args.extend(
        [
            "sh",
            "-c",
            r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH" exec bash -l"#,
        ]
        .map(String::from),
    );
    args
}

/// The agents `sbx` runs in a sandbox of their own: Atollo's id → `sbx`'s.
pub fn sbx_agent(agent: &str) -> Option<&'static str> {
    match agent {
        "claude" => Some("claude"),
        "codex" => Some("codex"),
        _ => None,
    }
}

/// `agent` in its sandbox for `project`: `sbx run --name <agent_name>
/// <agent> <dir>` (created the first time, attached after).
pub fn agent_args(project: &str, agent: &str, dir: &Path) -> Option<Vec<String>> {
    let id = sbx_agent(agent)?;
    Some(vec![
        "run".into(),
        "--name".into(),
        agent_name(project, id),
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
/// action's variables, and the kit's tools (`path`, D-170) first on the
/// `PATH`, set before the `cd`: a folder that is not there stops the line.
/// A server in the VM listens on every card of the VM (`HOST=0.0.0.0`):
/// only the published port reaches the PC.
pub fn exec_args(
    vm: &str,
    folder: Option<&str>,
    line: &str,
    env: &[(String, String)],
    path: &[String],
) -> Vec<String> {
    let mut args: Vec<String> = vec!["exec".into()];
    if !path.is_empty() {
        args.push("-e".into());
        args.push(format!("ATOLLO_PATH={}", path.join(":")));
    }
    for (k, v) in env {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args.push(vm.into());
    args.push("sh".into());
    args.push("-lc".into());
    let line = match folder {
        Some(f) => format!("cd {} && {line}", shell_quote_posix(f)),
        None => line.to_owned(),
    };
    args.push(if path.is_empty() {
        line
    } else {
        format!("{TOOLS_FIRST}; {line}")
    });
    args
}

/// The kit's tools first on the `PATH` (`ATOLLO_PATH`, given with `-e`).
pub const TOOLS_FIRST: &str = r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$PATH"; export PATH"#;

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
