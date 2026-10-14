//! Files that the kit's programs need on disk, embedded in the exe and
//! written to `data\kit\` when missing or different (DECISIONS.md D-25).
//! They belong to the app: a hand edit is replaced at the next start.

use std::fs;

use anyhow::{Context, Result};

use crate::kit::{write_atomic, Kit};

const FILES: &[(&str, &str)] = &[
    ("loopback.js", include_str!("../../kit-files/loopback.js")),
    ("profile.ps1", include_str!("../../kit-files/profile.ps1")),
    ("bashrc", include_str!("../../kit-files/bashrc")),
];

pub fn write_all(kit: &Kit) -> Result<()> {
    let dir = kit.kit_files();
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    for (name, content) in FILES {
        let path = dir.join(name);
        if fs::read(&path).ok().as_deref() != Some(content.as_bytes()) {
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
            .contains("devkitLoopback"));
        fs::write(&path, "changed").unwrap();
        write_all(&kit).unwrap();
        assert!(fs::read_to_string(&path)
            .unwrap()
            .contains("devkitLoopback"));
    }
}
