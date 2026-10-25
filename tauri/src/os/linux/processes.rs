//! The PC's processes (`platform::processes`) from `/proc`.

use std::path::PathBuf;

use crate::platform::processes::Proc;

/// The parent of a process from `/proc/<pid>/stat`: the fourth field, after
/// the name in parentheses (which may itself hold spaces and parentheses).
fn stat_parent(stat: &str) -> Option<u32> {
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace().nth(1)?.parse().ok()
}

/// Every process with its program path (those the user can read).
pub fn all() -> Vec<Proc> {
    let Ok(dir) = std::fs::read_dir("/proc") else {
        return Vec::new();
    };
    dir.flatten()
        .filter_map(|e| {
            let pid: u32 = e.file_name().to_str()?.parse().ok()?;
            let path = image(pid)?;
            let stat = std::fs::read_to_string(e.path().join("stat")).ok()?;
            Some(Proc {
                pid,
                parent: stat_parent(&stat).unwrap_or(0),
                name: path.file_name()?.to_string_lossy().into_owned(),
                path,
            })
        })
        .collect()
}

/// The program of process `pid`, if the user can read it.
pub fn image(pid: u32) -> Option<PathBuf> {
    std::fs::read_link(format!("/proc/{pid}/exe")).ok()
}

#[cfg(test)]
#[path = "../../../../test/rust/os/linux/processes.rs"]
mod tests;
