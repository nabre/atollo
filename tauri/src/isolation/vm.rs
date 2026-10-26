//! A project in its microVM (D-167): one Docker Sandboxes sandbox per
//! project, `atollo-p-<project>-<hash>`, with the project's folder as its workspace
//! (the folder the commands start in). What runs there:
//!
//! - the terminal: `sbx exec -it -w <folder> <vm> … bash -l`, the kit's
//!   tools for Linux first on the `PATH` (D-170, [`kit_terminal_args`]);
//! - an action's command line: `sbx exec -e … <vm> sh -lc '<line>'`, its
//!   process group noted in the VM ([`tracked`]) so that a stop ends it
//!   there ([`stop_args`]), its port published on the PC's `127.0.0.1`, or
//!   on the local network when Settings allow it (`sbx ports`);
//! - the web servers: containers of the VM's own Docker on the versions
//!   the project chose (`php:<line>-apache`, `httpd:2.4`);
//! - its databases stay on the PC, the kit's servers (D-171): the VM never
//!   sees their programs, settings or data, and reaches only their ports
//!   (`host.docker.internal`, [`host_port_allow`]) as the project's user;
//! - Claude Code and Codex: `sbx`'s own agents, a sandbox each
//!   (`atollo-a-claude-<project>-<hash>`), whose keys the proxy of `sbx` adds to
//!   their requests; Pi in the project's VM, its build for Linux from the
//!   kit (D-178). Antigravity, a desktop editor, never starts on a project.
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
    let hex = crate::common::hex(&hash[..12]);
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

/// The prefix of the sandboxes an earlier Atollo made for the kit's
/// terminals (`atollo-t-<terminal>-<hash>`, D-168): the kit's terminals run
/// on the PC now (D-178), and those left are removed.
pub const TERMINAL_PREFIX: &str = "atollo-t-";

/// Creates the sandbox: `sbx run -d --name <vm> [--kit <kit>] <agent>
/// <dir> [<folder>:ro…]` (`shell` for a project's or a terminal's,
/// `claude` or `codex` for an agent's), its own kit (`kit`, D-169), the
/// `read_only` folders mounted too, where the VM cannot change them.
pub fn start_args(
    vm: &str,
    dir: &Path,
    read_only: &[std::path::PathBuf],
    kit: Option<&Path>,
    agent: &str,
) -> Vec<String> {
    let mut args: Vec<String> = vec!["run".into(), "-d".into(), "--name".into(), vm.into()];
    if let Some(kit) = kit {
        args.push("--kit".into());
        args.push(kit.display().to_string());
    }
    args.push(agent.into());
    args.push(dir.display().to_string());
    args.extend(read_only.iter().map(|p| format!("{}:ro", p.display())));
    args
}

/// Starts an existing sandbox again, when stopped: `sbx run -d --name
/// <vm>`, its name only. `sbx` refuses any folder after the first for a
/// sandbox it has, even the very ones it was made with ("already exists
/// and can't be given new workspaces", tried with sbx 0.49.2); a missing
/// one goes in with `sbx mount`.
pub fn restart_args(vm: &str) -> Vec<String> {
    ["run", "-d", "--name", vm].map(String::from).into()
}

/// The PC as the VM reaches it: through the proxy of `sbx`, only on the
/// ports a sandbox is allowed ([`host_port_allow`]); `127.0.0.1` in the VM
/// is the VM.
pub const HOST: &str = "host.docker.internal";

/// Lets the sandbox `vm`, and it only, reach the PC's `127.0.0.1:<port>`
/// over TCP (a database server of the kit, D-171): every other port of the
/// PC stays closed. Adding it again is fine (`Already covered`).
pub fn host_port_allow(vm: &str, port: u16) -> Vec<String> {
    [
        "policy",
        "allow",
        "network",
        "--sandbox",
        vm,
        "--protocol",
        "tcp",
        &format!("localhost:{port}"),
    ]
    .map(String::from)
    .into()
}

/// Closes that port again for `vm` (`--force`: no question on a terminal).
pub fn host_port_close(vm: &str, port: u16) -> Vec<String> {
    [
        "policy",
        "rm",
        "network",
        "--sandbox",
        vm,
        "--resource",
        &format!("localhost:{port}"),
        "--force",
    ]
    .map(String::from)
    .into()
}

/// Mounts `host` read-only in the existing sandbox `vm`, at `guest` (its
/// path in the VM, [`guest_path`]): `sbx mount <vm> <host>:<guest>:ro`,
/// kept across restarts. The short `<host>:ro` names `ro` as the target.
pub fn mount_args(vm: &str, host: &Path, guest: &str) -> Vec<String> {
    vec![
        "mount".into(),
        vm.into(),
        format!("{}:{guest}:ro", host.display()),
    ]
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

/// The `sbx exec` options of `tools` (D-170, D-171): the file of the
/// variables that are never on a command line, the tools' `PATH`
/// (`ATOLLO_PATH`, put first by the line), their variables.
fn tools_options(tools: &crate::isolation::GuestTools) -> Vec<String> {
    let mut args = Vec::new();
    if let Some(file) = &tools.env_file {
        args.push("--env-file".into());
        args.push(file.display().to_string());
    }
    if !tools.path.is_empty() {
        args.push("-e".into());
        args.push(format!("ATOLLO_PATH={}", tools.path.join(":")));
    }
    for (k, v) in &tools.vars {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args
}

/// The kit's terminal in its VM: a login bash in `dir` (the VM's path),
/// with `tools` ([`tools_options`]): the kit's tools for Linux first on the
/// `PATH`, the VM's own programs after them.
pub fn kit_terminal_args(vm: &str, dir: &str, tools: &crate::isolation::GuestTools) -> Vec<String> {
    let mut args: Vec<String> = ["exec", "-it", "-w", dir].map(String::from).into();
    args.extend(tools_options(tools));
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

/// The agents that start on a project, all in the VM (D-178): Claude Code
/// and Codex in `sbx`'s sandboxes ([`sbx_agent`]), Pi in the project's own
/// VM ([`pi_session_args`]). Antigravity, a desktop editor, never does.
pub fn starts_in_vm(agent: &str) -> bool {
    sbx_agent(agent).is_some() || agent == "pi"
}

/// Pi in the project's VM `vm` (D-178): its build for Linux (`pi`, the
/// VM's path of the kit's, mounted read-only) started in `dir` (the VM's
/// path) like a command of the project's terminal, with `tools`
/// ([`tools_options`]): the project's tools first on the `PATH`, its
/// database's variables.
pub fn pi_session_args(
    vm: &str,
    dir: &str,
    pi: &str,
    tools: &crate::isolation::GuestTools,
) -> Vec<String> {
    let mut args: Vec<String> = ["exec", "-it", "-w", dir].map(String::from).into();
    args.extend(tools_options(tools));
    args.push(vm.into());
    args.push("sh".into());
    args.push("-c".into());
    args.push(format!("{TOOLS_FIRST}; exec {}", shell_quote_posix(pi)));
    args
}

/// The session of an agent in its sandbox `vm` (made by `Vm::start`):
/// `sbx run --name <vm>`, which attaches to the agent, with the project's
/// database variables (`--env-file`), the kit's tools for Linux first on
/// the `PATH` before the image's own (`image_path`, read in the sandbox:
/// `-e PATH` replaces it), and the tools' variables. `-e` and `--env-file`
/// apply to the agent's session also on a re-attach (sbx 0.49.2).
pub fn agent_session_args(
    vm: &str,
    image_path: &str,
    tools: &crate::isolation::GuestTools,
) -> Vec<String> {
    let mut args: Vec<String> = vec!["run".into(), "--name".into(), vm.into()];
    if let Some(file) = &tools.env_file {
        args.push("--env-file".into());
        args.push(file.display().to_string());
    }
    if !tools.path.is_empty() {
        args.push("-e".into());
        args.push(format!("PATH={}:{image_path}", tools.path.join(":")));
    }
    for (k, v) in &tools.vars {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args
}

/// The PC's address a port of the VM is published on: `127.0.0.1`, or
/// every card of the PC (`0.0.0.0`) when Settings let the projects'
/// servers be reached from the local network (`lan`, tried with sbx
/// 0.49.2).
fn host_ip(lan: bool) -> &'static str {
    if lan {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    }
}

/// Publishes `port` of the VM on the PC's [`host_ip`]`:<port>`.
pub fn publish_args(vm: &str, port: u16, lan: bool) -> Vec<String> {
    vec![
        "ports".into(),
        vm.into(),
        "--publish".into(),
        format!("{}:{port}:{port}", host_ip(lan)),
    ]
}

/// True when `sbx ports --publish`'s error `text` says `port` is published
/// already on the very address asked ([`host_ip`]), never another one.
pub fn published_already(text: &str, port: u16, lan: bool) -> bool {
    text.contains(&format!(" {}:{port}/", host_ip(lan))) && text.contains("already published")
}

/// Takes `port` back from the PC (idempotent: nothing published is fine).
pub fn unpublish_args(vm: &str, port: u16, lan: bool) -> Vec<String> {
    vec![
        "ports".into(),
        vm.into(),
        "--unpublish".into(),
        format!("{}:{port}:{port}", host_ip(lan)),
    ]
}

/// Where the actions running in a VM note their process group.
pub const RUNS: &str = "/tmp/atollo-runs";

/// The name of an action's note in [`RUNS`]: its id readable, then 12 hex
/// digits of its SHA-256 (two ids never share a note).
pub fn run_tag(action: &str) -> String {
    let readable: String = action
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .take(40)
        .collect();
    let hash = Sha256::digest(action.as_bytes());
    format!("{readable}-{}", crate::common::hex(&hash[..6]))
}

/// `line` noting its process group first: the `sh -lc` of `sbx exec` leads
/// a group of its own, which ending `sbx exec` on the PC does **not** stop
/// (tried with sbx 0.49.2); [`stop_args`] stops it from inside the VM.
pub fn tracked(tag: &str, line: &str) -> String {
    format!("mkdir -p {RUNS} && echo $$ > {RUNS}/{tag} && {line}")
}

/// Stops the action noted as `tag` inside `vm`: its whole process group,
/// `TERM` then `KILL` after five seconds, then its web container if it had
/// one (`container`). A note that is not a number above 1 is left alone:
/// the project's code can write it, and `kill -1` would reach every process
/// of the VM.
pub fn stop_args(vm: &str, tag: &str, container: Option<&str>) -> Vec<String> {
    let note = shell_quote_posix(&format!("{RUNS}/{tag}"));
    let mut script = format!(
        r#"f={note}; g=$(cat "$f" 2>/dev/null); rm -f "$f"; case "$g" in ''|*[!0-9]*|0|1) ;; *) kill -TERM -$g 2>/dev/null; i=0; while kill -0 -$g 2>/dev/null && [ $i -lt 5 ]; do sleep 1; i=$((i+1)); done; kill -KILL -$g 2>/dev/null ;; esac"#
    );
    if let Some(c) = container {
        script.push_str(&format!(
            "; docker rm -f {} >/dev/null 2>&1",
            shell_quote_posix(c)
        ));
    }
    script.push_str("; true");
    vec!["exec".into(), vm.into(), "sh".into(), "-c".into(), script]
}

/// A command line run in the VM: `sbx exec -e K=V … <vm> sh -lc '<line>'`,
/// in `folder` of the workspace when given (relative, `/`), with `tools`
/// ([`tools_options`]) then the action's variables (`env`), the kit's
/// tools first on the `PATH`, set before the `cd`: a folder that is not
/// there stops the line. A server in the VM listens on every card of the
/// VM (`HOST=0.0.0.0`): only the published port reaches the PC.
pub fn exec_args(
    vm: &str,
    folder: Option<&str>,
    line: &str,
    env: &[(String, String)],
    tools: &crate::isolation::GuestTools,
) -> Vec<String> {
    let path = &tools.path;
    let mut args: Vec<String> = vec!["exec".into()];
    args.extend(tools_options(tools));
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

/// One line of a creation (a framework's scaffolding, D-179) in the new
/// project's VM `vm`: the kit's PowerShell for the VM's Linux (`pwsh`, the
/// VM's path of the kit's, mounted read-only) runs the line `encoded`
/// (`-EncodedCommand`) in `dir` (the VM's path), with `tools`
/// ([`tools_options`]: the kit's tools first on the `PATH`, the file of
/// the variables that are never on a command line) and `quiet` (`CI=1`…).
/// Not interactive: its output is read through the pipes of `sbx exec`.
pub fn scaffold_args(
    vm: &str,
    dir: &str,
    pwsh: &str,
    encoded: &str,
    quiet: &[(&str, &str)],
    tools: &crate::isolation::GuestTools,
) -> Vec<String> {
    let mut args: Vec<String> = ["exec", "-w", dir].map(String::from).into();
    args.extend(tools_options(tools));
    for (k, v) in quiet {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args.push(vm.into());
    let script = format!(r#"{TOOLS_FIRST}; exec "$0" "$@""#);
    args.extend(
        [
            "sh",
            "-c",
            script.as_str(),
            pwsh,
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-EncodedCommand",
            encoded,
        ]
        .map(String::from),
    );
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

/// The name of the web container of `action` in its VM.
pub fn web_container(action: &str) -> String {
    format!("{PREFIX}web-{action}")
}

/// The container of a web action, in the foreground of `sh -lc` (it ends
/// with the action): `folder` of the workspace (relative, `/`; `.` the
/// whole) as Apache's document root, on `port` of the VM's network, a
/// container of the same name left by an earlier start removed first.
///
/// `aliases`: the URL and the folder in the VM of each path alias of the
/// site (`/media` → `/d/kit/projects/shop/storage/uploads`), checked by
/// `projects::alias` and only of the project itself (another project is
/// not in its VM): each mounted at `/srv/alias/<n>` with an `Alias` of
/// Apache. The `.localhost` domains need nothing: Apache answers every
/// name, and a browser takes `*.localhost` to the PC, where the port is
/// published.
pub fn web_line(
    action: &str,
    web: &Web,
    folder: &str,
    port: u16,
    aliases: &[(String, String)],
) -> String {
    let container = shell_quote_posix(&web_container(action));
    let root = format!("\"$PWD\"/{}", shell_quote_posix(folder));
    let (image, docroot, conf) = match web {
        Web::Php { line } => (
            format!("php:{line}-apache"),
            "/var/www/html",
            "/etc/apache2/conf-enabled/atollo-aliases.conf",
        ),
        Web::Static => (
            "httpd:2.4".to_owned(),
            "/usr/local/apache2/htdocs",
            "/usr/local/apache2/conf/httpd.conf",
        ),
    };
    let mut mounts = String::new();
    let mut alias_conf = String::new();
    for (n, (url, dir)) in aliases.iter().enumerate() {
        let inside = format!("/srv/alias/{n}");
        mounts.push_str(" -v ");
        mounts.push_str(&shell_quote_posix(&format!("{dir}:{inside}")));
        alias_conf.push_str(&format!(
            "Alias \"{url}\" \"{inside}\"\n<Directory \"{inside}\">\n    Require all granted\n</Directory>\n"
        ));
    }
    let aliases_in = if alias_conf.is_empty() {
        String::new()
    } else {
        format!(
            "printf '%s' {} >> {conf} && ",
            shell_quote_posix(&alias_conf)
        )
    };
    let listen = match web {
        Web::Php { .. } => format!(
            "{aliases_in}sed -i 's/^Listen 80$/Listen {port}/' /etc/apache2/ports.conf && \
             sed -i 's/:80>/:{port}>/' /etc/apache2/sites-enabled/000-default.conf && \
             exec apache2-foreground"
        ),
        Web::Static => format!(
            "{aliases_in}sed -i 's/^Listen 80$/Listen {port}/' /usr/local/apache2/conf/httpd.conf && \
             exec httpd-foreground"
        ),
    };
    format!(
        "docker rm -f {container} >/dev/null 2>&1; \
         exec docker run --rm --init --name {container} --network host \
         -v {root}:{docroot}{mounts} {image} sh -c {}",
        shell_quote_posix(&listen)
    )
}

#[cfg(test)]
#[path = "../../../test/rust/isolation/vm.rs"]
mod tests;
