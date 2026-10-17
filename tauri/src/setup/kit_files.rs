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
    for (name, content) in FILES {
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
