//! Files that the kit's programs need on disk, embedded in the exe and
//! written to `data\kit\` when missing or different (DECISIONS.md D-25).
//! They belong to the app: a hand edit is replaced at the next start.

use std::fs;

use anyhow::{Context, Result};

use crate::fsx::{join_rel, write_atomic};
use crate::kit::Kit;

const FILES: &[(&str, &str)] = &[
    ("loopback.js", include_str!("../../../runtime/loopback.js")),
    (
        "phpmyadmin.php",
        include_str!("../../../runtime/phpmyadmin.php"),
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
    ("bashrc", include_str!("../../../runtime/bashrc")),
    ("bin/agy.cmd", include_str!("../../../runtime/bin/agy.cmd")),
    ("bin/agy", include_str!("../../../runtime/bin/agy")),
    (
        "bin/gradle.cmd",
        include_str!("../../../runtime/bin/gradle.cmd"),
    ),
    ("bin/gradle", include_str!("../../../runtime/bin/gradle")),
];

/// The MySQL programs the kit's terminal reaches logged in as `root`
/// (D-110), with MariaDB's program of the same use for a kit without MySQL.
pub const MYSQL_LAUNCHERS: [(&str, &str); 3] = [
    ("mysql", "mariadb"),
    ("mysqldump", "mariadb-dump"),
    ("mysqladmin", "mariadb-admin"),
];

/// `bin\<stem>.cmd`: MySQL's `<stem>.exe` of `%CP_MYSQL%` with the kit's
/// login (`%DEVKIT_MYSQL_CNF%`, read after the other option files: it wins
/// over MariaDB's `my.ini` in `MYSQL_HOME`), or MariaDB's `mariadb` program
/// when the kit has no MySQL (its login comes from `MYSQL_HOME`, as
/// before). Jumps instead of blocks: `%*` inside parentheses breaks on a
/// `)` of the arguments.
pub fn mysql_launcher_cmd(stem: &str, mariadb: &str) -> String {
    format!(
        "@echo off\r\n\
         rem Dev-Kit: {stem} of the kit's MySQL logged in as root with data\\app\\db\\mysql.cnf,\r\n\
         rem or MariaDB's {mariadb} on a kit without MySQL.\r\n\
         setlocal\r\n\
         if not exist \"%CP_MYSQL%\\bin\\{stem}.exe\" goto mariadb\r\n\
         if not exist \"%DEVKIT_MYSQL_CNF%\" goto bare\r\n\
         \"%CP_MYSQL%\\bin\\{stem}.exe\" \"--defaults-extra-file=%DEVKIT_MYSQL_CNF%\" %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :bare\r\n\
         \"%CP_MYSQL%\\bin\\{stem}.exe\" %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :mariadb\r\n\
         if not exist \"%CP_MARIADB%\\bin\\{mariadb}.exe\" goto none\r\n\
         \"%CP_MARIADB%\\bin\\{mariadb}.exe\" %*\r\n\
         exit /b %ERRORLEVEL%\r\n\
         :none\r\n\
         echo MySQL is not installed: Tools ^> Databases 1>&2\r\n\
         exit /b 1\r\n"
    )
}

/// `bin/<stem>`: the same as [`mysql_launcher_cmd`] for sh (macOS, Linux,
/// Git Bash).
pub fn mysql_launcher_sh(stem: &str, mariadb: &str) -> String {
    format!(
        "#!/bin/sh\n\
         # Dev-Kit: {stem} of the kit's MySQL logged in as root (data/app/db/mysql.cnf),\n\
         # or MariaDB's {mariadb} on a kit without MySQL.\n\
         if [ -x \"$CP_MYSQL/bin/{stem}\" ]; then\n\
         \x20   if [ -f \"$DEVKIT_MYSQL_CNF\" ]; then\n\
         \x20       exec \"$CP_MYSQL/bin/{stem}\" --defaults-extra-file=\"$DEVKIT_MYSQL_CNF\" \"$@\"\n\
         \x20   fi\n\
         \x20   exec \"$CP_MYSQL/bin/{stem}\" \"$@\"\n\
         fi\n\
         if [ -x \"$CP_MARIADB/bin/{mariadb}\" ]; then\n\
         \x20   exec \"$CP_MARIADB/bin/{mariadb}\" \"$@\"\n\
         fi\n\
         echo \"MySQL is not installed: Tools > Databases\" >&2\n\
         exit 1\n"
    )
}

/// The generated files: the MySQL launchers, `.cmd` and sh.
fn generated() -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (stem, mariadb) in MYSQL_LAUNCHERS {
        out.push((format!("bin/{stem}.cmd"), mysql_launcher_cmd(stem, mariadb)));
        out.push((format!("bin/{stem}"), mysql_launcher_sh(stem, mariadb)));
    }
    out
}

/// Launchers run by a Unix shell (and Git Bash): mode 755 on Unix.
fn is_unix_bin(name: &str) -> bool {
    name.starts_with("bin/") && !name.ends_with(".cmd")
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
                crate::router::fs::write_executable(&path, content.as_bytes())
                    .with_context(|| path.display().to_string())?;
            } else {
                write_atomic(&path, content.as_bytes())
                    .with_context(|| path.display().to_string())?;
            }
        } else if is_unix_bin {
            let _ = crate::router::fs::set_executable(&path);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/setup/kit_files.rs"]
mod tests;
