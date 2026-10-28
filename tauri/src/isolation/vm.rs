//! A project in its microVM (D-167): one Docker Sandboxes sandbox per
//! project, `atollo-p-<project>-<hash>`, with the project's folder as its workspace
//! (the folder the commands start in). What runs there:
//!
//! - the agents only (D-183), all four in it (D-184): Claude Code, Codex,
//!   Pi and Antigravity (`agy`), their builds for Linux from the kit
//!   mounted read-only ([`agent_session_args`]); the project's code, its
//!   terminal and its web servers run on the PC;
//! - its databases stay on the PC, the kit's servers (D-171): the VM never
//!   sees their programs, settings or data, and reaches only their ports
//!   (`host.docker.internal`, [`host_port_allow`]) as the project's user;
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
    tagged_as(head, project, project)
}

/// [`tagged`] with the readable part from `shown` and the hash of `exact`
/// (a folder: its name, and its whole path).
fn tagged_as(head: &str, shown: &str, exact: &str) -> String {
    let hash = Sha256::digest(exact.as_bytes());
    // 96 bits: no name chosen on purpose (a cloned repository's) reaches
    // another project's sandbox by brute force.
    let hex = crate::common::hex(&hash[..12]);
    let room = NAME_MAX.saturating_sub(head.len() + hex.len() + 1);
    let slug: String = shown
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

/// The prefix of the sandboxes of folders that are not projects (D-187).
pub const FOLDER_PREFIX: &str = "atollo-f-";

/// The sandbox of the folder `dir` (its canonical path, checked by
/// `isolation::vm_workspace`): `atollo-f-<folder's name>-<hash of the
/// path>`, so two folders of the same name never share one (D-187).
pub fn folder_name(dir: &std::path::Path) -> String {
    let shown = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    tagged_as(FOLDER_PREFIX, &shown, &dir.to_string_lossy())
}

/// The prefix of the sandboxes an earlier Atollo made for the kit's
/// terminals (`atollo-t-<terminal>-<hash>`, D-168): the kit's terminals run
/// on the PC now (D-178), and those left are removed.
pub const TERMINAL_PREFIX: &str = "atollo-t-";

/// The prefix of the sandboxes an earlier Atollo made for Claude Code and
/// Codex (`atollo-a-<agent>-<project>-<hash>`, with `sbx`'s agent images):
/// every agent runs in its project's sandbox now (D-184), and those left
/// are removed.
pub const AGENT_PREFIX: &str = "atollo-a-";

/// Creates the sandbox: `sbx run -d --name <vm> [--kit <kit>] shell
/// <dir> [<folder>…] [<folder>:ro…]` (`shell`, the only image Atollo
/// uses: the project's sandbox and the tools sandbox),
/// its own kit (`kit`, D-169), the `writable` folders mounted too (the
/// tools sandbox's, D-180), then the `read_only` ones, where the VM cannot
/// change them.
pub fn start_args(
    vm: &str,
    dir: &Path,
    writable: &[std::path::PathBuf],
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

/// Mounts `host` read-write in the existing sandbox `vm`, at `guest`:
/// `sbx mount <vm> <host>:<guest>`, kept across restarts (an agent's
/// account, D-185) [unverified: the read-write form on a sandbox that
/// exists].
pub fn mount_rw_args(vm: &str, host: &Path, guest: &str) -> Vec<String> {
    vec![
        "mount".into(),
        vm.into(),
        format!("{}:{guest}", host.display()),
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

/// What an agent's build gets in the VM besides the tools: no update of its
/// own, the kit's build is mounted read-only and the kit updates it.
/// Antigravity's CLI and Claude Code have a switch for it; Codex and Pi
/// none (Codex only tells of a new version).
pub fn agent_vars(agent: &str) -> &'static [(&'static str, &'static str)] {
    match agent {
        "antigravity" => &[("AGY_CLI_DISABLE_AUTO_UPDATE", "true")],
        "claude" => &[("DISABLE_AUTOUPDATER", "1")],
        _ => &[],
    }
}

/// The session of `agent` (its id) in the project's sandbox `vm` (D-184):
/// its build for Linux (`program`, the VM's path of the kit's, mounted
/// read-only) started in `dir` (the VM's path) like a command of a
/// terminal, with `tools` ([`tools_options`]): the project's tools first on
/// the `PATH`, its database's variables; then [`agent_vars`]; `extra`: what
/// the user typed after the agent's command (a forwarder, LEVELS.md §6.8),
/// passed as it is. No key and no login of the kit: each agent signs in
/// inside the sandbox (its home is the VM's); the Gemini key, when the kit
/// holds one, reaches Google through the proxy of `sbx` (`secret gemini`),
/// never the VM.
pub fn agent_session_args(
    vm: &str,
    dir: &str,
    agent: &str,
    program: &str,
    tools: &crate::isolation::GuestTools,
    extra: &[String],
) -> Vec<String> {
    binary_session_args(vm, dir, program, tools, agent_vars(agent), extra)
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

/// The project's Python environment for the VM's Linux (D-188): when the
/// session names it (`ATOLLO_VENV`, `projects::envs::vm_vars`), made empty
/// with the kit's `uv` if it is not there yet, then first on the `PATH`
/// with `VIRTUAL_ENV`. Nothing of the project runs; a failure leaves the
/// session as it is.
pub const LOCAL_ENV: &str = concat!(
    r#"if [ -n "$ATOLLO_VENV" ]; then "#,
    r#"[ -d "$ATOLLO_VENV" ] || { command -v uv >/dev/null 2>&1 && uv venv -q "$ATOLLO_VENV" >/dev/null 2>&1; }; "#,
    r#"[ -f "$ATOLLO_VENV/pyvenv.cfg" ] && { VIRTUAL_ENV="$ATOLLO_VENV"; PATH="$ATOLLO_VENV/bin:$PATH"; export VIRTUAL_ENV PATH; }; "#,
    "fi"
);

/// The start of a session's shell line in the VM: [`TOOLS_FIRST`], then
/// [`MAKE_HOMES`], then [`LOCAL_ENV`].
pub fn session_start() -> String {
    format!("{TOOLS_FIRST}; {MAKE_HOMES}; {LOCAL_ENV}")
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
