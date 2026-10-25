//! Processes of the PC whose program lives in a given folder (agents of the
//! kit, programs to close before removing the drive, other kits). Each
//! system lists them its own way (`os/*/processes.rs`): Windows with
//! ToolHelp, Linux from `/proc`, macOS with `libproc`.

use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use crate::os::current as sys;
use crate::os::current::processes as imp;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Proc {
    pub pid: u32,
    pub parent: u32,
    pub name: String,
    pub path: PathBuf,
}

/// A path of this PC as its usual file system compares it: without case on
/// Windows (NTFS) and macOS (APFS by default), exact on Linux, where
/// `VSCODIUM-SERVER` and `vscodium-server` are two folders (D-123).
pub(crate) fn path_key(path: &str) -> String {
    if sys::PATHS_IGNORE_CASE {
        path.to_lowercase()
    } else {
        path.to_owned()
    }
}

/// `folder` as a prefix ending with the separator ([`path_key`]).
pub(crate) fn prefix_of(folder: &Path) -> String {
    format!(
        "{}{MAIN_SEPARATOR}",
        path_key(
            folder
                .display()
                .to_string()
                .trim_end_matches(MAIN_SEPARATOR)
        )
    )
}

pub(crate) fn is_under(p: &Proc, prefix: &str) -> bool {
    path_key(&p.path.display().to_string()).starts_with(prefix)
}

/// The processes in `all` whose program is in `folder`.
pub fn running_from(all: &[Proc], folder: &Path) -> Vec<u32> {
    let prefix = prefix_of(folder);
    all.iter()
        .filter(|p| is_under(p, &prefix))
        .map(|p| p.pid)
        .collect()
}

/// Every process with its program path (those the user can query).
pub fn all() -> Vec<Proc> {
    imp::all()
}

/// Processes whose program is under `dir` (case as [`path_key`]).
pub fn running_under(dir: &Path) -> Vec<Proc> {
    let prefix = prefix_of(dir);
    all().into_iter().filter(|p| is_under(p, &prefix)).collect()
}

#[cfg(test)]
#[path = "../../../test/rust/platform/processes.rs"]
mod tests;
