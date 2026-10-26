//! OS-aware binary, script, and launcher path resolution across all targets.
//!
//! Domain modules (`db`, `editor`, `agents`, `servers`, `projects`) ask this
//! module for executable paths instead of hardcoding `.exe`, `.cmd`, `.bat`,
//! or Windows backslashes.

use std::path::{Path, PathBuf};

use crate::kit::{Kit, Os};

/// Executable file name on `os` (`"node.exe"` on Windows, `"node"` on Unix).
pub fn exe_name(os: Os, stem: &str) -> String {
    format!("{stem}{}", os.exe_suffix())
}

/// Command wrapper file name on `os` (`"agy.cmd"` on Windows, `"agy"` on Unix).
pub fn cmd_name(os: Os, stem: &str) -> String {
    format!("{stem}{}", os.cmd_suffix())
}

/// Executable inside `<dir>/bin/<stem>[.exe]` for `os`.
pub fn db_bin_in(os: Os, dir: &Path, stem: &str) -> PathBuf {
    dir.join("bin").join(exe_name(os, stem))
}

/// Database server or client binary inside `data/apps/<target>/<folder>/bin/<stem>[.exe]`:
/// the folder and stem come from `db::DbEngine` (`client_component`,
/// `tools_component`, `client_bin`, `dump_bin`…), e.g. `mongosh/bin/mongosh`.
pub fn db_bin(kit: &Kit, folder: &str, stem: &str) -> PathBuf {
    db_bin_in(kit.os(), &kit.apps().join(folder), stem)
}

/// Bundled Node.js binary inside `vscodium-server` (`node.exe` on Windows, `node` on Unix).
pub fn editor_node(kit: &Kit, server_dir: &Path) -> PathBuf {
    server_dir.join(exe_name(kit.os(), "node"))
}

/// Entry script `out/server-main.js` inside `vscodium-server`.
pub fn editor_server_main(server_dir: &Path) -> PathBuf {
    server_dir.join("out").join("server-main.js")
}
