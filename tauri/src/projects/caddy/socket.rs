//! The private Unix sockets of the kit's servers (D-111): Caddy's admin
//! endpoint of a pool and the php-fpm a Caddy talks to, each in an
//! owner-only folder of its own, checked before the server binds.

use std::fs;
use std::path::{Path, PathBuf};

use crate::kit::{Kit, Os};
use crate::projects::apache::fwd;

use super::quoted_path;

/// The longest admin socket path written: macOS takes 104 bytes in
/// `sun_path`, Linux and Windows 108; a longer one means no admin endpoint.
pub const SOCKET_PATH_MAX: usize = 100;

/// A Unix domain socket of one instance in a folder of its own: Caddy's
/// admin endpoint of a pool ([`admin_socket`]) and, on macOS and Linux,
/// the php-fpm a Caddy talks to (`projects/fastcgi.rs`). Never a TCP port,
/// which any local program of any user could reach: to read Caddy's
/// configuration or load another one (D-111), or to send php-fpm the
/// FastCGI parameters `PHP_VALUE`/`PHP_ADMIN_VALUE`, which turn off the
/// key guard. The socket is created owner-only (`|0600` for Caddy,
/// `listen.mode = 0600` for php-fpm, ignored by Windows) in a folder of its
/// own ([`private_leaf`]): new, with a random name, owner-only and checked
/// before the server binds, in a per-user folder ([`socket_bases`],
/// [`socket_parent`]). Short enough for `sun_path`; the folder is removed
/// when this is dropped (the instance ends).
#[derive(Debug)]
pub struct PrivateSocket {
    pub path: PathBuf,
    dir: PathBuf,
}

/// Caddy's admin endpoint of a pool ([`PrivateSocket`]).
pub type AdminSocket = PrivateSocket;

impl PrivateSocket {
    /// The socket `file` in a new private folder `parent/name`
    /// ([`private_leaf`]).
    pub fn in_leaf(parent: &Path, name: &str, file: &str) -> std::io::Result<Self> {
        let dir = private_leaf(parent, name)?;
        Ok(PrivateSocket {
            path: dir.join(file),
            dir,
        })
    }

    /// Its folder is still the private one made for it ([`is_private_dir`]):
    /// checked again before a client connects (`caddy reload`), so that a
    /// folder put in its place is never talked to.
    pub fn still_private(&self) -> bool {
        is_private_dir(&self.dir) && self.path.parent() == Some(self.dir.as_path())
    }
}

impl PartialEq for PrivateSocket {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for PrivateSocket {}

impl Drop for PrivateSocket {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

/// The socket path `path` can be written in a Caddyfile and fits `sun_path`.
pub fn socket_path_usable(path: &Path) -> bool {
    quoted_path(path).is_some() && fwd(path).len() <= SOCKET_PATH_MAX
}

/// A folder only its owner can enter: owned by the current user, not a
/// link, nobody else with any right on it (Unix: mode `0700` or tighter).
/// On Windows the folders are per user (`%TEMP%`) and get an owner-only
/// DACL ([`private_leaf`]): only that it is a folder and not a link.
pub fn is_private_dir(path: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(path) else {
        return false;
    };
    meta.is_dir()
        && !meta.file_type().is_symlink()
        && !crate::platform::safe_path::is_reparse(&meta)
        && crate::platform::files::owner_only(&meta)
}

/// Where the admin sockets may go, best first. On Linux the session's
/// `$XDG_RUNTIME_DIR`, on macOS the per-user `$TMPDIR`, each only when it
/// is private ([`is_private_dir`]); then the host's temporary folder: the
/// kit's own in it on Windows (per user), the folder itself on Unix (maybe
/// a shared `/tmp`, where [`socket_parent`] and the leaf keep it private).
pub fn socket_bases(kit: &Kit) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let var = match kit.os() {
        Os::Linux => Some("XDG_RUNTIME_DIR"),
        Os::Darwin => Some("TMPDIR"),
        Os::Win32 => None,
    };
    if let Some(dir) = var.and_then(std::env::var_os).map(PathBuf::from) {
        if dir.is_absolute() && is_private_dir(&dir) {
            out.push(dir);
        }
    }
    let temp = kit.system_temp().map(Path::to_path_buf);
    if kit.os().is_windows() {
        // `%TEMP%\Atollo\<id>`: the user's own.
        out.extend(temp);
    } else {
        // The host's temporary folder itself (`/tmp`), not the kit's
        // `Atollo/<id>` in it, which another user may have made first.
        out.extend(
            temp.as_deref()
                .and_then(|t| t.parent()?.parent())
                .map(Path::to_path_buf),
        );
    }
    out
}

/// A folder the kit's folder of sockets may be made in: on Unix one only
/// the user can write in (theirs, no write for group and others), or a
/// sticky one (`/tmp`: nobody renames another's entries) of root or the
/// user; on Windows any folder (not a link: [`is_private_dir`] of the
/// parent checks what matters).
pub fn base_trusted(base: &Path) -> bool {
    let Ok(meta) = fs::symlink_metadata(base) else {
        return false;
    };
    meta.is_dir() && crate::platform::files::safe_parent(&meta)
}

/// Makes the folder `base/name` for a socket: created here and now (never
/// one that exists: `create_dir`, not `create_dir_all`, refused when it is
/// there or is a link), mode `0700` from the start on Unix, owner-only DACL
/// on Windows, then checked ([`is_private_dir`]) before it is used. A
/// folder that fails the check is removed when it is ours.
pub fn private_leaf(base: &Path, name: &str) -> std::io::Result<PathBuf> {
    let leaf = base.join(name);
    crate::platform::files::create_dir_owner_only(&leaf)?;
    if let Err(e) = crate::platform::files::restrict_dir_to_owner(&leaf) {
        let _ = fs::remove_dir(&leaf);
        return Err(std::io::Error::other(format!("{e:#}")));
    }
    if !is_private_dir(&leaf) {
        let _ = fs::remove_dir(&leaf);
        return Err(std::io::Error::other(format!(
            "{}: not a private folder",
            leaf.display()
        )));
    }
    Ok(leaf)
}

/// The name of the kit's folder for sockets in a base of [`socket_bases`]:
/// per user on Unix (`atollo-caddy-<uid>`), where the base may be a shared
/// `/tmp`, so that another user cannot have made it first; `atollo-caddy`
/// on Windows, whose temporary folder is the user's own.
pub fn socket_parent_name(uid: Option<u32>) -> String {
    match uid {
        Some(uid) => format!("atollo-caddy-{uid}"),
        None => "atollo-caddy".to_owned(),
    }
}

/// The current user's id on Unix, `None` on Windows.
pub(super) fn current_uid() -> Option<u32> {
    crate::platform::host::user_id()
}

/// The kit's folder for sockets in `base` ([`socket_parent_name`]), made
/// owner-only (`0700`) when it is not there; `None` when it is there and
/// not private ([`is_private_dir`]: another user's, a link, open to
/// others): that base is not used, since its owner could rename a leaf and
/// put another folder or socket in its place.
pub fn socket_parent(base: &Path) -> Option<PathBuf> {
    // On Windows the base is the kit's own folder in the temporary one,
    // maybe not made yet; on Unix it is there already.
    let _ = fs::create_dir_all(base);
    if !base_trusted(base) {
        return None;
    }
    let parent = base.join(socket_parent_name(current_uid()));
    let _ = crate::platform::files::create_dir_owner_only(&parent);
    let private = is_private_dir(&parent);
    if private {
        sweep_stale_leaves(&parent, STALE_LEAF_AGE);
    }
    private.then_some(parent)
}

/// How old a leaf of the sockets' folder must be before [`sweep_stale_leaves`]
/// looks at it: a new one gets its socket within seconds of being made.
pub const STALE_LEAF_AGE: std::time::Duration = std::time::Duration::from_secs(600);

/// Removes the `c-*` leaves of the sockets' folder `parent` that a crash
/// left behind ([`PrivateSocket`] removes its own when the instance ends):
/// older than `min_age`, and with no socket in them that a server still
/// listens on (a connect is refused or finds nothing). The leaves of a
/// running Caddy or php-fpm, of this app or of another kit of the same
/// user, answer and stay. Nothing on Windows, where the app cannot ask a
/// socket (`platform::sockets`) and the folder is the kit's own in the
/// user's temporary folder.
pub fn sweep_stale_leaves(parent: &Path, min_age: std::time::Duration) {
    let Ok(entries) = fs::read_dir(parent) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_str().is_some_and(|n| n.starts_with("c-")) {
            continue;
        }
        let leaf = entry.path();
        if !is_private_dir(&leaf) {
            continue;
        }
        let old = fs::symlink_metadata(&leaf)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age >= min_age);
        if !old {
            continue;
        }
        if crate::platform::sockets::live_socket_in(&leaf) == Some(false) {
            match fs::remove_dir_all(&leaf) {
                Ok(()) => crate::common::log::info(&format!(
                    "caddy: {}: left by an instance that ended, removed",
                    leaf.display()
                )),
                Err(e) => crate::common::log::warn(&format!("caddy: {}: {e}", leaf.display())),
            }
        }
    }
}

/// A new [`PrivateSocket`] named `file` for the instance `id` (`what` says
/// which, in the log), in the first folder of [`socket_bases`] that gives a
/// private one with a path short enough; `None` (in the log) when none does.
pub fn private_socket(kit: &Kit, id: &str, what: &str, file: &str) -> Option<PrivateSocket> {
    for base in socket_bases(kit) {
        let random = match crate::editor::random_token() {
            Ok(t) => t.chars().take(12).collect::<String>(),
            Err(e) => {
                crate::common::log::warn(&format!("{what} {id}: no random name: {e:#}"));
                return None;
            }
        };
        let probe = base
            .join(socket_parent_name(current_uid()))
            .join(format!("c-{random}"))
            .join(file);
        if !socket_path_usable(&probe) {
            continue;
        }
        let Some(parent) = socket_parent(&base) else {
            crate::common::log::warn(&format!(
                "{what} {id}: {}: not a private folder, not used",
                base.join(socket_parent_name(current_uid())).display()
            ));
            continue;
        };
        match PrivateSocket::in_leaf(&parent, &format!("c-{random}"), file) {
            Ok(sock) => return Some(sock),
            Err(e) => crate::common::log::warn(&format!("{what} {id}: socket folder: {e}")),
        }
    }
    crate::common::log::warn(&format!("{what} {id}: no private folder for its socket"));
    None
}

/// The admin socket of the instance `id`, `None` (in the log; the instance
/// then runs with `admin off` and a reload restarts it) when no folder of
/// [`socket_bases`] gives a private one with a path short enough.
pub fn admin_socket(kit: &Kit, id: &str) -> Option<AdminSocket> {
    let sock = private_socket(kit, id, "caddy", "a.sock");
    if sock.is_none() {
        crate::common::log::warn(&format!(
            "caddy {id}: no admin socket: admin off, a reload restarts the pool"
        ));
    }
    sock
}
