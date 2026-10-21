//! The packages of the conda environments that cannot follow the kit
//! (D-92: R, Ruby off Windows, MongoDB on Linux, SQLite on Linux ARM64,
//! make of C/C++ off Windows), kept in the kit so that a move creates them
//! again without the network (D-118).
//!
//! `data/app/conda/<target>/`, one folder per target, never mixed:
//! - `pkgs/<file>`: the archives (`.conda`, `.tar.bz2`) as conda-forge
//!   publishes them, one copy shared by every list of the target;
//! - `locks/<id>@<line>.txt`: the explicit list of the line's environment
//!   (`@EXPLICIT`, then `url#sha256` in micromamba's order);
//! - `locks/<id>@<line>.json`: version, date, micromamba, file, size and
//!   SHA-256 of every archive.
//!
//! An installation from the network (`keep`) copies the archives out of
//! micromamba's cache in the work folder, each checked against the SHA-256
//! of `conda-meta` (conda-forge's metadata, which micromamba checked when it
//! downloaded). After a move (`reinstall`) every archive is checked again
//! against the list, and the list against its record; only then, with copies
//! in the work folder checked once more, `micromamba create --offline`
//! makes the environment again in its folder, followed by the component's
//! own steps (`Installer::after_conda`). Anything missing or different:
//! the installation from the network as before, with a warning. What no
//! list names any more goes (`sweep`).
//!
//! Files: `mod` (the folders and names), `list` (explicit lists, records,
//! the check before an offline installation), `keep` (the archives kept
//! after an installation from the network), `reinstall` (the environment
//! made again after a move), `sweep` (what no list names, the Settings
//! page), `launchers` (`kit-bin`, D-125).

mod keep;
mod launchers;
mod list;
mod reinstall;
mod sweep;

use std::path::PathBuf;

use crate::kit::Kit;

pub(super) use keep::{keep, wanted};
pub use launchers::{launchers_of_older_installations, write_launchers, LAUNCHERS};
pub use reinstall::reinstall;
pub use sweep::{forget, free, status, sweep, Status};

/// The only channel of the kit's environments (`micromamba_create_args`).
const CHANNEL: &str = "https://conda.anaconda.org/conda-forge/";

/// `data/app/conda/<target>`: the kept packages and lists of the kit's
/// target.
pub fn root(kit: &Kit) -> PathBuf {
    kit.app_dir().join("conda").join(kit.target().as_str())
}

fn pkgs_dir(kit: &Kit) -> PathBuf {
    root(kit).join("pkgs")
}

fn locks_dir(kit: &Kit) -> PathBuf {
    root(kit).join("locks")
}

/// `<id>@<line>`: the name of the lists of `line` of component `id`.
pub fn lock_name(id: &str, line: &str) -> String {
    format!("{id}@{line}")
}

#[cfg(test)]
#[path = "../../../../test/rust/install/conda/mod.rs"]
mod tests;
