//! Caddy, the kit's web server on every system (D-111): a
//! project's own Caddy, with a `Caddyfile` written at every start in
//! `data\projects\<project>\caddy\` (never in the project), and the text of
//! the shared pools' `Caddyfile` (`servers/config.rs`).
//!
//! Caddy reads no `.htaccess`: the site gets rules of its own, front
//! controller (`index.php`, as WordPress,
//! Drupal and Laravel want it), hidden files and `.inc`/`.php3`…`.php8`
//! refused with 404, aliases checked on disk. PHP is the kit's FastCGI
//! server (`projects/fastcgi.rs`: php-cgi on Windows, php-fpm elsewhere),
//! sent the start's key with every request (`env ATOLLO_FCGI_KEY`), which
//! Caddy reads from its own environment, never from its configuration.
//!
//! A pool has its own admin endpoint on an owner-only Unix socket
//! ([`AdminSocket`]; two pools never meet on Caddy's `:2019`, and no TCP
//! port lets another program read or replace the configuration), used by
//! `caddy reload` to change the sites without stopping the others. A
//! project's own Caddy has none (`admin off`). The data and config folders
//! are in the instance's folder, nothing goes to the user's profile.

//!
//! The parts: `socket` (the private Unix sockets of the admin endpoints
//! and of php-fpm, sensitive code kept apart), `conf` (the Caddyfile's
//! text), `command` (the command lines and the process).

mod command;
mod conf;
mod socket;

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use crate::env::KitEnv;
use crate::kit::{Kit, Os};
use crate::projects::apache::fwd;

pub use command::{
    caddy_command, command, reload_args, reload_failure, run_args, validate_args, with_key,
};
pub use conf::{
    admin_address, pool_conf, resolve_aliases, site_block, site_routes, skipped_aliases,
};
pub use socket::{
    admin_socket, is_private_dir, private_socket, socket_bases, socket_parent, socket_path_usable,
    AdminSocket, PrivateSocket,
};

/// The server program of the Caddy in `dir` on `os`: `caddy.exe` on
/// Windows, `caddy` elsewhere, at the root of its folder (D-111).
pub fn caddy_exe_for(os: Os, dir: &Path) -> PathBuf {
    dir.join(if os.is_windows() {
        "caddy.exe"
    } else {
        "caddy"
    })
}

/// [`caddy_exe_for`] on the kit's system.
pub fn caddy_exe(kit: &Kit, dir: &Path) -> PathBuf {
    caddy_exe_for(kit.os(), dir)
}

/// The Caddy folder from `env` (`CP_CADDY`, a version chosen for the
/// project) or the kit's default line.
pub fn caddy_dir(kit: &Kit, env: &KitEnv) -> PathBuf {
    env.get("CP_CADDY")
        .map(PathBuf::from)
        .filter(|d| caddy_exe(kit, d).is_file())
        .unwrap_or_else(|| {
            let resolved = crate::setup::kit_path(kit, "caddy");
            if caddy_exe(kit, &resolved).is_file() {
                resolved
            } else {
                kit.apps().join("caddy")
            }
        })
}

/// The folder of the project's Caddy files.
pub fn dir(kit: &Kit, project: &str) -> Result<PathBuf> {
    crate::projects::state::ProjectState::of(kit, project)
        .map(|s| s.caddy_dir())
        .ok_or_else(|| anyhow!("not a project name: {project}"))
}

/// `dir` holds a `.caddy` file: only then is it imported (`import` of a
/// pattern that matches nothing stops Caddy).
pub fn has_snippets(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries.flatten().any(|e| {
            e.path()
                .extension()
                .is_some_and(|x| x.eq_ignore_ascii_case("caddy"))
        })
    })
}

/// A path as a Caddyfile token: with `/`, in double quotes. `None` when it
/// holds a character Caddy would read as something else (`"`, `\`, `{` and
/// `}` of a placeholder, a line break): such a folder is not served.
pub fn quoted_path(path: &Path) -> Option<String> {
    let text = fwd(path);
    if text.is_empty() || text.contains(['"', '\\', '{', '}', '\n', '\r', '`']) {
        return None;
    }
    Some(format!("\"{text}\""))
}

#[cfg(test)]
#[path = "../../../../test/rust/projects/caddy.rs"]
mod tests;
