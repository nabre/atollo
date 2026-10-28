//! A project in its microVM (D-167): one Docker Sandboxes sandbox per
//! project, `atollo-p-<project>-<hash>`, with the project's folder as its workspace
//! (the folder the commands start in). What runs there:
//!
//! - the agents only (D-183): the project's code, its terminal and its web
//!   servers run on the PC;
//! - its databases stay on the PC, the kit's servers (D-171): the VM never
//!   sees their programs, settings or data, and reaches only their ports
//!   (`host.docker.internal`, [`host_port_allow`]) as the project's user;
//! - Claude Code and Codex: `sbx`'s own agents, a sandbox each
//!   (`atollo-a-claude-<project>-<hash>`), whose keys the proxy of `sbx` adds to
//!   their requests; Pi and Antigravity (`agy`) in the project's VM, their
//!   builds for Linux from the kit (D-178, D-179).
//!
//! The words given to `sh -lc` are quoted for the VM's shell (POSIX):
//! never text of the PC's shell.

use std::path::Path;

use sha2::{Digest, Sha256};

/// The prefix of everything Atollo names in the VMs.
pub const PREFIX: &str = "atollo-";

/// The longest name of a sandbox: `sbx kit add` makes its swap container
/// `<name>-swap-<8 hex>`, and Docker takes at most 63 characters ("name:
/// must be at most 63 characters", tried with sbx 0.50.0).
pub const NAME_MAX: usize = 63 - "-swap-".len() - 8;

/// `head` then `project` readable in a name `sbx` takes (`[a-z0-9.-]`, cut
/// to stay within [`NAME_MAX`]), then the first 24 hex digits of the
/// SHA-256 of its exact name: two projects never share a sandboxes' name,
/// and so never a workspace (`My Shop` and `my-shop` look the same, their
/// hashes not).
fn tagged(head: &str, project: &str) -> String {
    let hash = Sha256::digest(project.as_bytes());
    // 96 bits: no name chosen on purpose (a cloned repository's) reaches
    // another project's sandbox by brute force.
    let hex = crate::common::hex(&hash[..12]);
    let room = NAME_MAX.saturating_sub(head.len() + hex.len() + 1);
    let slug: String = project
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .take(room)
        .collect();
    let slug = slug.trim_matches(['-', '.']);
    if slug.is_empty() {
        format!("{head}{hex}")
    } else {
        format!("{head}{slug}-{hex}")
    }
}

/// The sandbox of `project`: `atollo-p-<project>-<hash>`.
pub fn name(project: &str) -> String {
    tagged(&format!("{PREFIX}p-"), project)
}

/// The sandbox of `agent` for `project`, in a namespace of its own:
/// `atollo-a-<agent>-<project>-<hash>`, never a project's.
pub fn agent_name(project: &str, agent: &str) -> String {
    tagged(&format!("{PREFIX}a-{agent}-"), project)
}

/// The prefix of the sandboxes an earlier Atollo made for the kit's
/// terminals (`atollo-t-<terminal>-<hash>`, D-168): the kit's terminals run
/// on the PC now (D-178), and those left are removed.
pub const TERMINAL_PREFIX: &str = "atollo-t-";

/// Creates the sandbox: `sbx run -d --name <vm> [--kit <kit>] <agent>
/// <dir> [<folder>…] [<folder>:ro…]` (`shell` for a project's, a
/// terminal's or the tools sandbox's, `claude` or `codex` for an agent's),
/// its own kit (`kit`, D-169), the `writable` folders mounted too (the
/// tools sandbox's, D-180), then the `read_only` ones, where the VM cannot
/// change them.
pub fn start_args(
    vm: &str,
    dir: &Path,
    writable: &[std::path::PathBuf],
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
    args.extend(writable.iter().map(|p| p.display().to_string()));
    args.extend(read_only.iter().map(|p| format!("{}:ro", p.display())));
    args
}

/// Lets the sandbox `vm`, and it only, reach `hosts` over TCP (the tools
/// sandbox's installers, D-180): `sbx policy allow network --sandbox <vm>
/// <host,host…>`, a rule of the sandbox's own policy, gone with it.
pub fn hosts_allow(vm: &str, hosts: &[&str]) -> Vec<String> {
    [
        "policy",
        "allow",
        "network",
        "--sandbox",
        vm,
        &hosts.join(","),
    ]
    .map(String::from)
    .into()
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

/// «Verifica strumenti» in the sandbox `vm` (LEVELS.md §8.2): `sbx exec`
/// with the session's `tools` (their `PATH` and variables, never the file
/// of the databases' variables: the check needs no password) and the check
/// `script`, which reads and prints only.
pub fn check_args(vm: &str, tools: &crate::isolation::GuestTools, script: &str) -> Vec<String> {
    let tools = crate::isolation::GuestTools {
        env_file: None,
        ..tools.clone()
    };
    let mut args: Vec<String> = vec!["exec".into()];
    args.extend(tools_options(&tools));
    args.push(vm.into());
    args.push("sh".into());
    args.push("-c".into());
    args.push(script.into());
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

/// An agent's own build for Linux (`program`, the VM's path of the kit's,
/// mounted read-only) started in the project's VM `vm`, in `dir` (the VM's
/// path), like a command of the project's terminal: `tools`
/// ([`tools_options`]), then `vars`, then `extra` after the program.
fn binary_session_args(
    vm: &str,
    dir: &str,
    program: &str,
    tools: &crate::isolation::GuestTools,
    vars: &[(&str, &str)],
    extra: &[String],
) -> Vec<String> {
    let mut args: Vec<String> = ["exec", "-it", "-w", dir].map(String::from).into();
    args.extend(tools_options(tools));
    for (k, v) in vars {
        args.push("-e".into());
        args.push(format!("{k}={v}"));
    }
    args.push(vm.into());
    args.push("sh".into());
    args.push("-c".into());
    let start = session_start();
    args.push(format!(r#"{start}; exec "$0" "$@""#));
    args.push(program.into());
    args.extend(extra.iter().cloned());
    args
}

/// Pi in the project's VM `vm` (D-178): its build for Linux (`pi`, the
/// VM's path of the kit's, mounted read-only) started in `dir` (the VM's
/// path) like a command of the project's terminal, with `tools`
/// ([`tools_options`]): the project's tools first on the `PATH`, its
/// database's variables; `extra`: what the user typed after `pi` (a
/// forwarder, LEVELS.md §6.8), passed to Pi as they are.
pub fn pi_session_args(
    vm: &str,
    dir: &str,
    pi: &str,
    tools: &crate::isolation::GuestTools,
    extra: &[String],
) -> Vec<String> {
    binary_session_args(vm, dir, pi, tools, &[], extra)
}

/// What Antigravity's CLI gets in the VM besides the tools: no update of
/// its own (the kit's build is mounted read-only; the kit updates it).
pub const AGY_VARS: [(&str, &str); 1] = [("AGY_CLI_DISABLE_AUTO_UPDATE", "true")];

/// Antigravity's CLI in the project's VM `vm` (D-179, LEVELS.md §9): as
/// [`pi_session_args`] with its build for Linux (`agy`) and [`AGY_VARS`].
/// No key and no login of the kit: the Gemini key, when the kit holds one,
/// reaches Google through the proxy of `sbx` (`secret gemini`), never the
/// VM; without it `agy` signs in inside the VM.
pub fn agy_session_args(
    vm: &str,
    dir: &str,
    agy: &str,
    tools: &crate::isolation::GuestTools,
    extra: &[String],
) -> Vec<String> {
    binary_session_args(vm, dir, agy, tools, &AGY_VARS, extra)
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

/// The kit's tools first on the `PATH` (`ATOLLO_PATH`, given with `-e`),
/// then what the user installs in the VM's home ([`VM_HOMES`]), then the
/// VM's own programs. Only variables: «Verifica strumenti» starts with it
/// and writes nothing; a session adds [`MAKE_HOMES`] ([`session_start`]).
pub const TOOLS_FIRST: &str = concat!(
    r#"PATH="$ATOLLO_PATH${ATOLLO_PATH:+:}$HOME/.cargo/bin:$HOME/.gem/bin:$HOME/.npm-global/bin:$PATH"; "#,
    r#"GEM_HOME="${GEM_HOME:-$HOME/.gem}"; NPM_CONFIG_PREFIX="${NPM_CONFIG_PREFIX:-$HOME/.npm-global}"; "#,
    r#"R_LIBS_USER="${R_LIBS_USER:-$HOME/R/library}"; "#,
    "export PATH GEM_HOME NPM_CONFIG_PREFIX R_LIBS_USER"
);

/// R's library made in the VM's home: R ignores an `R_LIBS_USER` that is
/// not there, and `Rscript` cannot ask to make it.
pub const MAKE_HOMES: &str = r#"mkdir -p "$R_LIBS_USER" 2>/dev/null"#;

/// The start of a session's shell line in the VM: [`TOOLS_FIRST`], then
/// [`MAKE_HOMES`].
pub fn session_start() -> String {
    format!("{TOOLS_FIRST}; {MAKE_HOMES}")
}

/// Where the user's own installations go in the VM (D-181): the kit's
/// folders are read-only there, so `cargo install`, `gem install`,
/// `npm install -g` and R's `install.packages` write in the VM's home
/// (`~/.cargo`, `~/.gem`, `~/.npm-global`, `~/R/library`), set by
/// [`TOOLS_FIRST`] unless the session gives them. The kit's own values for
/// them never reach the VM (`isolation::guest_view`).
pub const VM_HOMES: [&str; 4] = ["CARGO_HOME", "GEM_HOME", "NPM_CONFIG_PREFIX", "R_LIBS_USER"];

#[cfg(test)]
#[path = "../../../test/rust/isolation/vm.rs"]
mod tests;
