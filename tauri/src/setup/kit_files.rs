//! Files that the kit's programs need on disk, embedded in the exe and
//! written to `data\kit\` when missing or different (DECISIONS.md D-25).
//! They belong to the app: a hand edit is replaced at the next start.

use std::fs;

use anyhow::{Context, Result};

use crate::common::fsx::{join_rel, write_atomic};
use crate::kit::Kit;

const FILES: &[(&str, &str)] = &[
    ("loopback.js", include_str!("../../../runtime/loopback.js")),
    (
        "phpmyadmin.php",
        include_str!("../../../runtime/phpmyadmin.php"),
    ),
    (
        "mongo-express.js",
        include_str!("../../../runtime/mongo-express.js"),
    ),
    (
        "php-router.php",
        include_str!("../../../runtime/php-router.php"),
    ),
    (
        "fcgi-guard.php",
        include_str!("../../../runtime/fcgi-guard.php"),
    ),
    ("profile.ps1", include_str!("../../../runtime/profile.ps1")),
    // The request clients of the levels (LEVELS.md §4.3): the editor's
    // tasks and agent profiles and the forwarders run through them.
    ("launch.ps1", include_str!("../../../runtime/launch.ps1")),
    ("launch.sh", include_str!("../../../runtime/launch.sh")),
    ("bashrc", include_str!("../../../runtime/bashrc")),
    (
        "bin/gradle.cmd",
        include_str!("../../../runtime/bin/gradle.cmd"),
    ),
    ("bin/gradle", include_str!("../../../runtime/bin/gradle")),
    // The app's command line as `atollo` in the kit's terminals.
    (
        "bin/atollo.cmd",
        include_str!("../../../runtime/bin/atollo.cmd"),
    ),
    ("bin/atollo", include_str!("../../../runtime/bin/atollo")),
];

/// Files and folders an older Atollo wrote and this one no longer does: the
/// agents' launchers that lived in `bin`, a folder the PC's shells keep on
/// their PATH (D-178); the editor's VM client `vm.ps1` and the stand-ins
/// of `no-agents`, replaced by the request clients and the forwarders; the
/// folder of the agents' launchers ([`AGENTS_DIR`]: Antigravity's with
/// their keyring round trip, Claude Code's with its token), since no agent
/// runs on the PC (D-179, LEVELS.md §7). Removed at every start.
const OBSOLETE: [&str; 6] = [
    "bin/agy.cmd",
    "bin/agy",
    "bin/claude",
    "vm.ps1",
    "no-agents",
    AGENTS_DIR,
];

/// The folder of the agents' launchers an earlier Atollo wrote in
/// `data/kit`: removed at start ([`OBSOLETE`]), and kept off every `PATH`
/// meanwhile (`levels::env::agent_dirs`).
pub const AGENTS_DIR: &str = "agents";

/// The folder of the forwarders, first on the PATH of the PC's shells
/// (`levels::env::FORWARD_DIR`): a command of an agent there asks the app to
/// run the agent in the project's VM, whatever the PC has installed after
/// it (LEVELS.md §6.8).
pub const FORWARD_DIR: &str = crate::levels::env::FORWARD_DIR;

/// What a forwarder says when the kit's PowerShell is missing (Windows'
/// `cmd`: its client is `launch.ps1`).
const NO_PWSH: &str =
    "Atollo: senza il PowerShell del kit gli agenti non si inoltrano alla VM: installalo da Strumenti.";

/// `forward\<cmd>.cmd`: `<cmd>` typed in `cmd` asks for `agent` through
/// `launch.ps1` (origin `forwarded`, the arguments as typed after `--`;
/// `cmd`'s `%*` may alter `"`, `%`, `^`: Q16). Jumps, not blocks: a block
/// breaks on a `)` of the arguments.
pub fn forward_cmd(cmd: &str, agent: &str) -> String {
    format!(
        "@echo off\r\n\
         rem Atollo: {cmd} runs {agent} in the project's VM: D-179, LEVELS.md 6.8.\r\n\
         where pwsh >nul 2>nul\r\n\
         if errorlevel 1 goto nopwsh\r\n\
         pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File \"%~dp0..\\launch.ps1\" -Origin forwarded -Agent {agent} -- %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :nopwsh\r\n\
         echo {NO_PWSH} 1>&2\r\n\
         exit /b 1\r\n"
    )
}

/// `forward/<cmd>.ps1`: the same in PowerShell, which passes the arguments
/// intact; `launch.ps1` runs in a PowerShell of its own (it sets variables
/// and exits), the one of this terminal.
pub fn forward_ps1(cmd: &str, agent: &str) -> String {
    format!(
        "# Atollo: {cmd} runs {agent} in the project's VM (D-179, LEVELS.md 6.8).\n\
         $pwsh = (Get-Process -Id $PID).Path\n\
         & $pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot '..' 'launch.ps1') -Origin forwarded -Agent {agent} -- @args\n\
         exit $LASTEXITCODE\n"
    )
}

/// `forward/<cmd>`: the same for sh, through `launch.sh` (run by `/bin/sh`,
/// whatever its mode on the kit's disk).
pub fn forward_sh(cmd: &str, agent: &str) -> String {
    format!(
        "#!/bin/sh\n\
         # Atollo: {cmd} runs {agent} in the project's VM (D-179, LEVELS.md 6.8).\n\
         exec /bin/sh \"$(dirname \"$0\")/../launch.sh\" forwarded {agent} \"$@\"\n"
    )
}

/// The read-only `atollo` of every sandbox (LEVELS.md §8.4), with its
/// texts: it prints `info.txt` or `info.json` beside its folder and
/// nothing else.
const VM_ATOLLO: &str = include_str!("../../../runtime/vm/atollo");

/// The folder of a sandbox's `guest` with the read-only `atollo`, first on
/// the VM's `PATH`.
pub const GUEST_BIN: &str = "bin";
/// The folder of a sandbox's `guest` with the agents' stand-ins, one
/// folder per agent (`agents/<id>`), so that a session leaves out the one
/// of the agent it runs (LEVELS.md §6.10).
pub const GUEST_AGENTS: &str = "agents";

/// The read-only `atollo` in `lang` (LEVELS.md §8.4): `atollo` prints
/// `info.txt`, `atollo --json` `info.json`; any other word says that the
/// kit is managed from the PC and exits 1. No network, no write, no secret.
pub fn vm_atollo(lang: &str) -> String {
    let refusal = crate::common::i18n::t(
        lang,
        "Il kit si gestisce dal PC: qui atollo mostra soltanto dove sei e cosa hai (atollo, atollo --json).",
        &[],
    );
    let missing = crate::common::i18n::t(
        lang,
        "Le informazioni di questa sandbox non ci sono ancora: le scrive Atollo quando prepara la VM.",
        &[],
    );
    VM_ATOLLO
        .replace("\r\n", "\n")
        .replace("@REFUSAL@", &crate::platform::sh_quote(&refusal))
        .replace("@MISSING@", &crate::platform::sh_quote(&missing))
}

/// The stand-in `cmd` of an agent inside a VM (LEVELS.md §6.10): a VM
/// cannot ask the PC to open an agent's sandbox, so it says where agents
/// start and exits 1.
pub fn vm_stand_in(cmd: &str, lang: &str) -> String {
    let text = crate::common::i18n::t(
        lang,
        "{0} non parte da qui. Gli agenti si avviano dalla pagina del progetto, dall'editor, o scrivendo il loro nome in un terminale del PC nella cartella del progetto.",
        &[cmd],
    );
    format!(
        "#!/bin/sh\n\
         # Atollo: {cmd} does not start inside a VM (D-179, LEVELS.md 6.10).\n\
         printf '%s\\n' {} >&2\n\
         exit 1\n",
        crate::platform::sh_quote(&text)
    )
}

/// Writes the read-only `atollo` and the agents' stand-ins in `guest` (a
/// sandbox's folder the VM sees read-only, `isolation::kit::guest_dir`):
/// LF, executable, only when they changed.
pub fn write_guest(guest: &std::path::Path, lang: &str) -> Result<()> {
    let mut files = vec![(format!("{GUEST_BIN}/atollo"), vm_atollo(lang))];
    for agent in crate::levels::Agent::ALL {
        for cmd in agent.commands() {
            files.push((
                format!("{GUEST_AGENTS}/{}/{cmd}", agent.id()),
                vm_stand_in(cmd, lang),
            ));
        }
    }
    for (name, content) in files {
        let path = join_rel(guest, &name);
        if fs::read(&path).ok().as_deref() == Some(content.as_bytes()) {
            let _ = crate::platform::files::set_executable(&path);
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| parent.display().to_string())?;
        }
        crate::platform::files::write_executable(&path, content.as_bytes())
            .with_context(|| path.display().to_string())?;
    }
    Ok(())
}

/// The wrapper the Claude Code extension would run through
/// `claudeCode.claudeProcessWrapper` (LEVELS.md §6.9, Q2): ready for the
/// spike by hand, not wired. Atollo never sets the setting and the
/// extension is uninstalled at every editor start (§10).
pub const CLAUDE_EXTENSION: &str = "claude-extension";

/// `forward\claude-extension.cmd`: as a forwarder, with origin
/// `editorExtension`; every word goes to the app, which drops the
/// extension's bundled binary (`levels::requests::agent_args`).
pub fn claude_extension_cmd() -> String {
    format!(
        "@echo off\r\n\
         rem Atollo: the Claude Code extension's agent in the project's VM: D-179, LEVELS.md 6.9, Q2, not wired.\r\n\
         where pwsh >nul 2>nul\r\n\
         if errorlevel 1 goto nopwsh\r\n\
         pwsh -NoLogo -NoProfile -ExecutionPolicy Bypass -File \"%~dp0..\\launch.ps1\" -Origin editorExtension -Extension anthropic.claude-code -Agent claude -- %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :nopwsh\r\n\
         echo {NO_PWSH} 1>&2\r\n\
         exit /b 1\r\n"
    )
}

/// `forward/claude-extension`: the same for sh, through `launch.sh`.
pub fn claude_extension_sh() -> String {
    "#!/bin/sh\n\
     # Atollo: the Claude Code extension's agent in the project's VM (D-179, LEVELS.md 6.9; Q2, not wired).\n\
     exec /bin/sh \"$(dirname \"$0\")/../launch.sh\" editorExtension anthropic.claude-code claude \"$@\"\n"
        .to_owned()
}

/// The MySQL programs the kit's terminal reaches logged in as `root`
/// (D-110).
pub const MYSQL_LAUNCHERS: [&str; 3] = ["mysql", "mysqldump", "mysqladmin"];

/// `bin\<stem>.cmd`: MySQL's `<stem>.exe` of `%CP_MYSQL%` with the kit's
/// login (`%ATOLLO_MYSQL_CNF%`, read after the other option files: it
/// wins). Jumps instead of blocks: `%*` inside parentheses breaks on a `)`
/// of the arguments.
pub fn mysql_launcher_cmd(stem: &str) -> String {
    format!(
        "@echo off\r\n\
         rem Atollo: {stem} of the kit's MySQL logged in as root with data\\app\\db\\mysql.cnf.\r\n\
         setlocal\r\n\
         if not exist \"%CP_MYSQL%\\bin\\{stem}.exe\" goto none\r\n\
         if not exist \"%ATOLLO_MYSQL_CNF%\" goto bare\r\n\
         \"%CP_MYSQL%\\bin\\{stem}.exe\" \"--defaults-extra-file=%ATOLLO_MYSQL_CNF%\" %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :bare\r\n\
         \"%CP_MYSQL%\\bin\\{stem}.exe\" %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :none\r\n\
         echo MySQL is not installed: Tools ^> Databases 1>&2\r\n\
         exit /b 1\r\n"
    )
}

/// `bin/<stem>`: the same as [`mysql_launcher_cmd`] for sh (macOS, Linux,
/// Git Bash).
pub fn mysql_launcher_sh(stem: &str) -> String {
    format!(
        "#!/bin/sh\n\
         # Atollo: {stem} of the kit's MySQL logged in as root (data/app/db/mysql.cnf).\n\
         if [ -x \"$CP_MYSQL/bin/{stem}\" ]; then\n\
         \x20   if [ -f \"$ATOLLO_MYSQL_CNF\" ]; then\n\
         \x20       exec \"$CP_MYSQL/bin/{stem}\" --defaults-extra-file=\"$ATOLLO_MYSQL_CNF\" \"$@\"\n\
         \x20   fi\n\
         \x20   exec \"$CP_MYSQL/bin/{stem}\" \"$@\"\n\
         fi\n\
         echo \"MySQL is not installed: Tools > Databases\" >&2\n\
         exit 1\n"
    )
}

/// The generated files: the MySQL launchers, `.cmd` and sh.
fn generated() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for stem in MYSQL_LAUNCHERS {
        out.push((format!("bin/{stem}.cmd"), mysql_launcher_cmd(stem)));
        out.push((format!("bin/{stem}"), mysql_launcher_sh(stem)));
    }
    for agent in crate::levels::Agent::ALL {
        for cmd in agent.commands() {
            let id = agent.id();
            out.push((format!("{FORWARD_DIR}/{cmd}.cmd"), forward_cmd(cmd, id)));
            out.push((format!("{FORWARD_DIR}/{cmd}.ps1"), forward_ps1(cmd, id)));
            out.push((format!("{FORWARD_DIR}/{cmd}"), forward_sh(cmd, id)));
        }
    }
    out.push((
        format!("{FORWARD_DIR}/{CLAUDE_EXTENSION}.cmd"),
        claude_extension_cmd(),
    ));
    out.push((
        format!("{FORWARD_DIR}/{CLAUDE_EXTENSION}"),
        claude_extension_sh(),
    ));
    out
}

/// Launchers run by a Unix shell (and Git Bash): mode 755 on Unix.
fn is_unix_bin(name: &str) -> bool {
    let folder = name.split('/').next().unwrap_or_default();
    let launcher = name.contains('/')
        && ["bin", FORWARD_DIR].contains(&folder)
        && !name.ends_with(".cmd")
        && !name.ends_with(".ps1");
    launcher || name == "launch.sh"
}

/// Files read by sh or bash, which take a CR as part of the line.
fn is_shell(name: &str) -> bool {
    name == "bashrc" || is_unix_bin(name)
}

/// `content` with the line endings `name` needs, whatever the checkout did
/// to the source: CRLF for batch files, LF for shell files, as is otherwise.
fn with_line_endings(name: &str, content: &str) -> String {
    let lf = content.replace("\r\n", "\n");
    if name.ends_with(".cmd") {
        lf.replace('\n', "\r\n")
    } else if is_shell(name) {
        lf
    } else {
        content.to_owned()
    }
}

pub fn write_all(kit: &Kit) -> Result<()> {
    let dir = kit.kit_files();
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let generated = generated();
    let all = FILES
        .iter()
        .map(|(n, c)| (*n, *c))
        .chain(generated.iter().map(|(n, c)| (n.as_str(), c.as_str())));
    for (name, content) in all {
        let path = join_rel(&dir, name);
        let content = with_line_endings(name, content);
        let is_unix_bin = is_unix_bin(name);
        if fs::read(&path).ok().as_deref() != Some(content.as_bytes()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            if is_unix_bin {
                crate::platform::files::write_executable(&path, content.as_bytes())
                    .with_context(|| path.display().to_string())?;
            } else {
                write_atomic(&path, content.as_bytes())
                    .with_context(|| path.display().to_string())?;
            }
        } else if is_unix_bin {
            let _ = crate::platform::files::set_executable(&path);
        }
    }
    for name in OBSOLETE {
        let path = join_rel(&dir, name);
        // A folder (never followed if it is a link: the link goes).
        let folder = fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir());
        let removed = if folder {
            fs::remove_dir_all(&path)
        } else {
            fs::remove_file(&path)
        };
        match removed {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(e).with_context(|| path.display().to_string());
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/setup/kit_files.rs"]
mod tests;
