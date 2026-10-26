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
