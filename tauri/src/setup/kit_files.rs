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
    ("vm.ps1", include_str!("../../../runtime/vm.ps1")),
    ("bashrc", include_str!("../../../runtime/bashrc")),
    // The agents' launchers in a folder of their own (D-178): the kit's
    // environment has it on the PATH, the shells of the PC never
    // (`levels::env::strip_agents`).
    (
        "agents/agy.cmd",
        include_str!("../../../runtime/agents/agy.cmd"),
    ),
    ("agents/agy", include_str!("../../../runtime/agents/agy")),
    (
        "agents/claude",
        include_str!("../../../runtime/agents/claude"),
    ),
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

/// Files an older Atollo wrote and this one no longer does: the agents'
/// launchers that lived in `bin`, a folder the PC's shells keep on their
/// PATH (D-178). Removed at every start.
const OBSOLETE: [&str; 3] = ["bin/agy.cmd", "bin/agy", "bin/claude"];

/// The folder of the agents' launchers, in `data/kit`.
pub const AGENTS_DIR: &str = "agents";

/// The folder of the stand-ins first on the PATH of the PC's shells
/// (D-178), in `data/kit`: a command of an agent there says where the
/// agents run and starts nothing, whatever the PC has installed after it.
pub const NO_AGENTS_DIR: &str = "no-agents";

/// The commands of the agents the stand-ins take the place of.
pub const AGENT_COMMANDS: [&str; 5] = ["claude", "codex", "agy", "antigravity", "pi"];

/// The variable whose text a stand-in prints (the app's language).
pub const AGENTS_NOTICE_VAR: &str = "ATOLLO_AGENTS_NOTICE";

/// What a stand-in says when the shell has no [`AGENTS_NOTICE_VAR`].
const AGENTS_NOTICE_FALLBACK: &str =
    "Atollo: gli agenti girano solo nella VM: avviali dalla pagina del progetto o dall'editor.";

/// `no-agents\<stem>.cmd`: says where the agents run, exits with 1. Jumps,
/// not blocks: a block breaks on a `)` of the text.
pub fn no_agent_cmd(stem: &str) -> String {
    format!(
        "@echo off\r\n\
         rem Atollo: {stem} does not start in this shell (D-178).\r\n\
         if not defined {AGENTS_NOTICE_VAR} goto plain\r\n\
         echo %{AGENTS_NOTICE_VAR}% 1>&2\r\n\
         exit /b 1\r\n\
         :plain\r\n\
         echo {AGENTS_NOTICE_FALLBACK} 1>&2\r\n\
         exit /b 1\r\n"
    )
}

/// `no-agents/<stem>`: the same as [`no_agent_cmd`] for sh.
pub fn no_agent_sh(stem: &str) -> String {
    format!(
        "#!/bin/sh\n\
         # Atollo: {stem} does not start in this shell (D-178).\n\
         msg=${AGENTS_NOTICE_VAR}\n\
         [ -n \"$msg\" ] || msg=\"{AGENTS_NOTICE_FALLBACK}\"\n\
         printf '%s\\n' \"$msg\" >&2\n\
         exit 1\n"
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
    for stem in AGENT_COMMANDS {
        out.push((format!("{NO_AGENTS_DIR}/{stem}.cmd"), no_agent_cmd(stem)));
        out.push((format!("{NO_AGENTS_DIR}/{stem}"), no_agent_sh(stem)));
    }
    out
}

/// Launchers run by a Unix shell (and Git Bash): mode 755 on Unix.
fn is_unix_bin(name: &str) -> bool {
    let folder = name.split('/').next().unwrap_or_default();
    name.contains('/')
        && ["bin", AGENTS_DIR, NO_AGENTS_DIR].contains(&folder)
        && !name.ends_with(".cmd")
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
        match fs::remove_file(&path) {
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
