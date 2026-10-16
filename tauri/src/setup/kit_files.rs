//! Files that the kit's programs need on disk, embedded in the exe and
//! written to `data\kit\` when missing or different (DECISIONS.md D-25).
//! They belong to the app: a hand edit is replaced at the next start.

use std::fs;

use anyhow::{Context, Result};

use crate::fsx::write_atomic;
use crate::kit::Kit;

const FILES: &[(&str, &str)] = &[
    ("loopback.js", include_str!("../../../runtime/loopback.js")),
    ("adminer.php", include_str!("../../../runtime/adminer.php")),
    ("profile.ps1", include_str!("../../../runtime/profile.ps1")),
    ("console.ps1", include_str!("../../../runtime/console.ps1")),
    ("bashrc", include_str!("../../../runtime/bashrc")),
    (r"bin\agy.cmd", include_str!("../../../runtime/bin/agy.cmd")),
    (r"bin\agy", include_str!("../../../runtime/bin/agy")),
    (
        r"bin\gradle.cmd",
        include_str!("../../../runtime/bin/gradle.cmd"),
    ),
    (r"bin\gradle", include_str!("../../../runtime/bin/gradle")),
];

pub fn write_all(kit: &Kit) -> Result<()> {
    let dir = kit.kit_files();
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    for (name, content) in FILES {
        let path = dir.join(name);
        // Batch files need CRLF, whatever the checkout did to the source.
        let content = if name.ends_with(".cmd") {
            content.replace("\r\n", "\n").replace('\n', "\r\n")
        } else {
            (*content).to_owned()
        };
        if fs::read(&path).ok().as_deref() != Some(content.as_bytes()) {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            write_atomic(&path, content.as_bytes()).with_context(|| path.display().to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};

    #[test]
    fn written_and_restored() {
        let tmp = TempDir::new("kitfiles");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        write_all(&kit).unwrap();
        let path = kit.kit_files().join("loopback.js");
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("atolloLoopback"));
        fs::write(&path, "changed").unwrap();
        write_all(&kit).unwrap();
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("atolloLoopback"));
    }
}
