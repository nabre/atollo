//! Caddy, the kit's web server on every system (Q-3, D-111, WP-3.3): a
//! project's own Caddy, with a `Caddyfile` written at every start in
//! `data\projects\<project>\caddy\` (never in the project), and the text of
//! the shared pools' `Caddyfile` (`servers/config.rs`).
//!
//! Caddy reads no `.htaccess`: the site gets the rules Nginx's site has
//! (`projects/nginx.rs`), front controller (`index.php`, as WordPress,
//! Drupal and Laravel want it), hidden files and `.inc`/`.php3`…`.php8`
//! refused with 404, aliases checked on disk. PHP is the kit's FastCGI
//! server (`projects/fastcgi.rs`: php-cgi on Windows, php-fpm elsewhere),
//! sent the start's key with every request (`env DEVKIT_FCGI_KEY`), which
//! Caddy reads from its own environment, never from its configuration.
//!
//! A pool has its own admin endpoint on an owner-only Unix socket
//! ([`AdminSocket`]; two pools never meet on Caddy's `:2019`, and no TCP
//! port lets another program read or replace the configuration), used by
//! `caddy reload` to change the sites without stopping the others. A
//! project's own Caddy has none (`admin off`). The data and config folders
//! are in the instance's folder, nothing goes to the user's profile.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, bail, Result};

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::{Kit, Os};
use crate::os::jobs;
use crate::projects::alias::{self, look_inside, valid_domain, Inside, SiteAlias};
use crate::projects::apache::{fwd, PHP_NEVER, PHP_RUN};
use crate::projects::fastcgi::{FastCgi, FCGI_KEY_PARAM};

/// The server program of the Caddy in `dir` on `os`: `caddy.exe` on
/// Windows, `caddy` elsewhere, at the root of its folder (WP-3.1).
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
            let resolved = crate::setup::catalog::kit_path(kit, "caddy");
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
    if !meta.is_dir() || meta.file_type().is_symlink() || crate::os::safe_path::is_reparse(&meta) {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no preconditions.
        let me = unsafe { libc::geteuid() };
        meta.uid() == me && meta.mode() & 0o077 == 0
    }
    #[cfg(not(unix))]
    {
        true
    }
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
        // `%TEMP%\Dev-Kit\<id>`: the user's own.
        out.extend(temp);
    } else {
        // The host's temporary folder itself (`/tmp`), not the kit's
        // `Dev-Kit/<id>` in it, which another user may have made first.
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
    if !meta.is_dir() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        // SAFETY: geteuid has no preconditions.
        let me = unsafe { libc::geteuid() };
        let sticky = meta.mode() & 0o1000 != 0;
        let owner_ok = meta.uid() == me || meta.uid() == 0;
        owner_ok && (sticky || (meta.uid() == me && meta.mode() & 0o022 == 0))
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Makes the folder `base/name` for a socket: created here and now (never
/// one that exists: `create_dir`, not `create_dir_all`, refused when it is
/// there or is a link), mode `0700` from the start on Unix, owner-only DACL
/// on Windows, then checked ([`is_private_dir`]) before it is used. A
/// folder that fails the check is removed when it is ours.
pub fn private_leaf(base: &Path, name: &str) -> std::io::Result<PathBuf> {
    let leaf = base.join(name);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        fs::DirBuilder::new().mode(0o700).create(&leaf)?;
    }
    #[cfg(not(unix))]
    fs::DirBuilder::new().create(&leaf)?;
    #[cfg(windows)]
    if let Err(e) = crate::router::fs::restrict_dir_to_owner(&leaf) {
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
fn current_uid() -> Option<u32> {
    #[cfg(unix)]
    {
        // SAFETY: geteuid has no preconditions.
        Some(unsafe { libc::geteuid() })
    }
    #[cfg(not(unix))]
    {
        None
    }
}

/// The kit's folder for sockets in `base` ([`socket_parent_name`]), made
/// owner-only (`0700`) when it is not there; `None` when it is there and
/// not private ([`is_private_dir`]: another user's, a link, open to
/// others): that base is not used, since its owner could rename a leaf and
/// put another folder or socket in its place.
pub fn socket_parent(base: &Path) -> Option<PathBuf> {
    #[cfg(windows)]
    let _ = fs::create_dir_all(base);
    if !base_trusted(base) {
        return None;
    }
    let parent = base.join(socket_parent_name(current_uid()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        let _ = fs::DirBuilder::new().mode(0o700).create(&parent);
    }
    #[cfg(not(unix))]
    {
        let _ = fs::DirBuilder::new().create(&parent);
    }
    let private = is_private_dir(&parent);
    #[cfg(unix)]
    if private {
        sweep_stale_leaves(&parent, STALE_LEAF_AGE);
    }
    private.then_some(parent)
}

/// How old a leaf of the sockets' folder must be before [`sweep_stale_leaves`]
/// looks at it: a new one gets its socket within seconds of being made.
#[cfg(unix)]
pub const STALE_LEAF_AGE: std::time::Duration = std::time::Duration::from_secs(600);

/// Removes the `c-*` leaves of the sockets' folder `parent` that a crash
/// left behind ([`PrivateSocket`] removes its own when the instance ends):
/// older than `min_age`, and with no socket in them that a server still
/// listens on (a connect is refused or finds nothing). The leaves of a
/// running Caddy or php-fpm, of this app or of another kit of the same
/// user, answer and stay. Unix only: on Windows the folder is the kit's own
/// in the user's temporary folder.
#[cfg(unix)]
pub fn sweep_stale_leaves(parent: &Path, min_age: std::time::Duration) {
    use std::os::unix::net::UnixStream;
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
        let alive = fs::read_dir(&leaf)
            .into_iter()
            .flatten()
            .flatten()
            .any(|f| {
                let is_socket = {
                    use std::os::unix::fs::FileTypeExt;
                    f.file_type().is_ok_and(|t| t.is_socket())
                };
                is_socket && UnixStream::connect(f.path()).is_ok()
            });
        if !alive {
            match fs::remove_dir_all(&leaf) {
                Ok(()) => crate::log::info(&format!(
                    "caddy: {}: left by an instance that ended, removed",
                    leaf.display()
                )),
                Err(e) => crate::log::warn(&format!("caddy: {}: {e}", leaf.display())),
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
                crate::log::warn(&format!("{what} {id}: no random name: {e:#}"));
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
            crate::log::warn(&format!(
                "{what} {id}: {}: not a private folder, not used",
                base.join(socket_parent_name(current_uid())).display()
            ));
            continue;
        };
        match PrivateSocket::in_leaf(&parent, &format!("c-{random}"), file) {
            Ok(sock) => return Some(sock),
            Err(e) => crate::log::warn(&format!("{what} {id}: socket folder: {e}")),
        }
    }
    crate::log::warn(&format!("{what} {id}: no private folder for its socket"));
    None
}

/// The admin socket of the instance `id`, `None` (in the log; the instance
/// then runs with `admin off` and a reload restarts it) when no folder of
/// [`socket_bases`] gives a private one with a path short enough.
pub fn admin_socket(kit: &Kit, id: &str) -> Option<AdminSocket> {
    let sock = private_socket(kit, id, "caddy", "a.sock");
    if sock.is_none() {
        crate::log::warn(&format!(
            "caddy {id}: no admin socket: admin off, a reload restarts the pool"
        ));
    }
    sock
}

/// The address of the admin socket `path` for Caddy: `unix/<path>`, with
/// `/` (`unix//tmp/…` on Unix, `unix/C:/…` on Windows).
pub fn admin_address(path: &Path) -> String {
    format!("unix/{}", fwd(path))
}

/// The global options of an instance whose folder is `run_dir`: its admin
/// endpoint on the socket `admin` (owner-only), else none (`admin off`: a
/// project's own Caddy, which restarts with its action, or a file written
/// only for the next start); no automatic HTTPS, nothing saved outside the
/// configuration, its storage in its folder.
pub fn global_options(admin: Option<&Path>, run_dir: &Path) -> Result<Vec<String>> {
    let storage = quoted_path(&run_dir.join("storage"))
        .ok_or_else(|| anyhow!("{}: not usable in a Caddyfile", run_dir.display()))?;
    let admin = admin.filter(|p| socket_path_usable(p));
    Ok(vec![
        "{".to_owned(),
        match admin {
            Some(path) => format!("\tadmin \"{}|0600\"", admin_address(path)),
            None => "\tadmin off".to_owned(),
        },
        "\tauto_https off".to_owned(),
        "\tpersist_config off".to_owned(),
        format!("\tstorage file_system {storage}"),
        "}".to_owned(),
    ])
}

/// The PHP of a site (or of an alias) at `indent`: the FastCGI server, the
/// key of this start and the files PHP runs ([`PHP_RUN`]). The key is not
/// in the text: Caddy reads it from its own environment at every request
/// (`{env.DEVKIT_FCGI_KEY}`, set by [`with_key`]), so neither the file on
/// disk nor the configuration Caddy holds contains it.
fn php_lines(fcgi: &FastCgi, indent: &str) -> Vec<String> {
    vec![
        format!("{indent}php_fastcgi {} {{", fcgi.addr.caddy_upstream()),
        format!("{indent}\tenv {FCGI_KEY_PARAM} {{env.{FCGI_KEY_PARAM}}}"),
        format!("{indent}\tsplit .php .phtml .phar"),
        format!("{indent}}}"),
    ]
}

/// The routes of a site, in the order they run (`route`): hidden files and
/// folders (`.env`, `.git/`) but `.well-known`, and the files of
/// [`PHP_NEVER`], refused with 404 everywhere, aliases included; without
/// PHP the files of [`PHP_RUN`] refused with 503 (never sent as text); the
/// aliases (their folders listed, PHP run in them as Apache does); then
/// the document root with PHP's front controller and the files. `None`
/// when the document root cannot be written in a Caddyfile; an alias that
/// cannot is left out (in the log).
pub fn site_routes(
    doc_root: &Path,
    aliases: &[SiteAlias],
    fcgi: Option<&FastCgi>,
    indent: &str,
) -> Option<Vec<String>> {
    let root = quoted_path(doc_root)?;
    let i = indent;
    let mut lines = vec![
        format!("{i}route {{"),
        format!("{i}\t@hidden {{"),
        format!("{i}\t\tpath_regexp /\\."),
        format!("{i}\t\tnot path /.well-known /.well-known/*"),
        format!("{i}\t}}"),
        format!("{i}\trespond @hidden 404"),
        format!("{i}\t@never path_regexp (?i){PHP_NEVER}"),
        format!("{i}\trespond @never 404"),
    ];
    if fcgi.is_none() {
        lines.extend([
            format!("{i}\t@php path_regexp (?i){PHP_RUN}"),
            format!("{i}\trespond @php 503"),
        ]);
    }
    let inner = format!("{i}\t\t");
    for a in aliases {
        let Some(target) = quoted_path(&a.dir) else {
            crate::log::warn(&format!(
                "alias {} not served by Caddy: {} cannot be written in a Caddyfile",
                a.url,
                a.dir.display()
            ));
            continue;
        };
        lines.push(format!("{i}\tredir {} {}/ 308", a.url, a.url));
        lines.push(format!("{i}\thandle_path {}/* {{", a.url));
        lines.push(format!("{inner}root * {target}"));
        if let Some(f) = fcgi {
            lines.extend(php_lines(f, &inner));
        }
        lines.push(format!("{inner}file_server browse"));
        lines.push(format!("{i}\t}}"));
    }
    lines.push(format!("{i}\troot * {root}"));
    if let Some(f) = fcgi {
        lines.extend(php_lines(f, &format!("{i}\t")));
    }
    lines.push(format!("{i}\tfile_server"));
    lines.push(format!("{i}}}"));
    Some(lines)
}

/// A site block: the addresses (`http://<name>:<port>`, or every `Host` with
/// `names` empty), on 127.0.0.1 only out of LAN mode, then `body`.
pub fn site_block(port: u16, lan: bool, names: &[String], body: &[String]) -> Vec<String> {
    let addresses = if names.is_empty() {
        format!("http://:{port}")
    } else {
        names
            .iter()
            .map(|n| format!("http://{n}:{port}"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let mut lines = vec![format!("{addresses} {{")];
    if !lan {
        lines.push("\tbind 127.0.0.1".to_owned());
    }
    lines.extend(body.iter().cloned());
    lines.push("}".to_owned());
    lines
}

/// Against DNS rebinding (a page of another site that renames itself to
/// 127.0.0.1 and reads the project): a `Host` no site names, or none, gets
/// 421, as from a project's own Nginx (D-98). Caddy tries this block last.
fn refuse_other_hosts(port: u16) -> Vec<String> {
    site_block(port, false, &[], &["\trespond 421".to_owned()])
}

/// The local `Host` names a project's own Caddy answers besides
/// `localhost` and the project's domains out of LAN mode (SEC-V3-03).
const LOCAL_HOSTS: [&str; 3] = ["127.0.0.1", "[::1]", "*.localhost"];

/// The names a site answers: `<project>.localhost` and its valid domains
/// (lowercase, once each); a project's own Caddy (`dedicated`) also
/// `localhost` and, out of LAN mode, [`LOCAL_HOSTS`].
pub fn site_names(
    project: Option<&str>,
    domains: &[String],
    dedicated: bool,
    lan: bool,
) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    if let Some(p) = project.filter(|p| !p.is_empty()) {
        names.push(format!("{p}.localhost"));
    }
    for d in domains {
        let d = d.trim().to_ascii_lowercase();
        if valid_domain(&d) && !names.contains(&d) {
            names.push(d);
        }
    }
    if dedicated || names.is_empty() {
        names.push("localhost".to_owned());
    }
    if dedicated && !lan {
        names.extend(LOCAL_HOSTS.iter().map(|h| (*h).to_owned()));
    }
    names
}

/// The complete `Caddyfile` of a project's own Caddy: no admin endpoint
/// (it restarts with its action, never reloads); in LAN mode the site
/// answers any `Host` (an IP of the PC included), else only its names and
/// the local ones, every other `Host` refused with 421.
#[allow(clippy::too_many_arguments)]
pub fn project_conf(
    project: Option<&str>,
    doc_root: &Path,
    run_dir: &Path,
    port: u16,
    fcgi: Option<&FastCgi>,
    lan: bool,
    domains: &[String],
    aliases: &[SiteAlias],
) -> Result<String> {
    let routes = site_routes(doc_root, aliases, fcgi, "\t").ok_or_else(|| {
        anyhow!(
            "{}: this folder cannot be served by Caddy (its path holds \" \\ {{ or }})",
            doc_root.display()
        )
    })?;
    let mut lines =
        vec!["# --- Dev-Kit: this project's Caddy (written at every start) ---".to_owned()];
    lines.extend(global_options(None, run_dir)?);
    let names = if lan {
        Vec::new()
    } else {
        site_names(project, domains, true, lan)
    };
    lines.extend(site_block(port, lan, &names, &routes));
    if !lan {
        lines.extend(refuse_other_hosts(port));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// The local names, besides `localhost`, that reach a pool's own page out
/// of LAN mode (never `*.localhost`, which are the projects' sites).
const POOL_LOCAL_HOSTS: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The `Caddyfile` of a shared pool: `sites` (blocks of [`site_block`]),
/// the pool's own page for its local names (any `Host` in LAN mode, an IP
/// of the PC included), any other `Host` refused with 421; the user's own
/// `conf.d\*.caddy` imported when there is any.
pub fn pool_conf(
    pool_dir: &Path,
    port: u16,
    admin: Option<&Path>,
    lan: bool,
    sites: &[Vec<String>],
    user_snippets: bool,
) -> Result<String> {
    let mut lines =
        vec!["# --- Dev-Kit: shared Caddy pool (written at every start or reload) ---".to_owned()];
    lines.extend(global_options(admin, pool_dir)?);
    if user_snippets {
        if let Some(glob) = quoted_path(&pool_dir.join("conf.d").join("*.caddy")) {
            lines.push(format!("import {glob}"));
        }
    }
    for site in sites {
        lines.extend(site.iter().cloned());
    }
    let page = ["\trespond \"Dev-Kit: Caddy pool\" 200".to_owned()];
    if lan {
        lines.extend(site_block(port, lan, &[], &page));
    } else {
        let names: Vec<String> = POOL_LOCAL_HOSTS.iter().map(|h| (*h).to_owned()).collect();
        lines.extend(site_block(port, lan, &names, &page));
        lines.extend(refuse_other_hosts(port));
    }
    lines.push(String::new());
    Ok(lines.join("\n"))
}

/// The aliases [`alias::resolve`] keeps that Caddy can serve: those whose
/// folder has no link or junction inside ([`look_inside`]), since Caddy's
/// file server follows them on every system; the others are left out with
/// a warning in the log.
pub fn resolve_aliases(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<SiteAlias> {
    alias::resolve(project_root, projects_root, aliases)
        .into_iter()
        .filter(|a| match look_inside(&a.dir) {
            Inside::Clean => true,
            Inside::Link(p) => {
                crate::log::warn(&format!(
                    "{}: alias {} not served by Caddy: {} is a link, a junction or unreadable (Caddy would follow it)",
                    project_root.display(),
                    a.url,
                    p.display()
                ));
                false
            }
            Inside::TooLarge => {
                crate::log::warn(&format!(
                    "{}: alias {} not served by Caddy: its folder is too large to check for links (over {} levels or {} entries)",
                    project_root.display(),
                    a.url,
                    alias::INSIDE_DEPTH,
                    alias::INSIDE_ENTRIES
                ));
                false
            }
        })
        .collect()
}

/// The URLs of the aliases [`resolve_aliases`] leaves out, for the
/// interface to say so.
pub fn skipped_aliases(
    project_root: &Path,
    projects_root: &Path,
    aliases: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = alias::skipped(project_root, projects_root, aliases);
    for (url, target) in aliases {
        let url = url.trim();
        if out.iter().any(|u| u == url) {
            continue;
        }
        let one = BTreeMap::from([(url.to_owned(), target.clone())]);
        let clean = alias::resolve(project_root, projects_root, &one)
            .iter()
            .all(|a| look_inside(&a.dir) == Inside::Clean);
        if !clean {
            out.push(url.to_owned());
        }
    }
    out
}

/// `caddy run` of the `Caddyfile` `conf`.
pub fn run_args(conf: &Path) -> Vec<OsString> {
    vec![
        "run".into(),
        "--config".into(),
        conf.as_os_str().to_owned(),
        "--adapter".into(),
        "caddyfile".into(),
    ]
}

/// `caddy reload` of `conf` on the instance whose admin socket is `admin`:
/// the new sites are served, the others go on without a pause.
pub fn reload_args(conf: &Path, admin: &Path) -> Vec<OsString> {
    let mut args = run_args(conf);
    args[0] = "reload".into();
    args.push("--address".into());
    args.push(admin_address(admin).into());
    args
}

/// The reason of a failed `caddy reload` from its `stderr`, for the log:
/// the last non-empty lines (Caddy ends with the error, an adapt error
/// names the Caddyfile's line), at most 2000 characters.
pub fn reload_failure(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim_end)
        .filter(|l| !l.trim().is_empty())
        .collect();
    let tail = lines[lines.len().saturating_sub(5)..].join(" | ");
    if tail.is_empty() {
        return "no output".to_owned();
    }
    let mut chars = tail.chars();
    let short: String = chars.by_ref().take(2000).collect();
    if chars.next().is_some() {
        format!("{short}...")
    } else {
        short
    }
}

/// The key of `fcgi` in the environment of the Caddy that runs (never in
/// its configuration, [`php_lines`]): the only place it reads it from.
pub fn with_key(cmd: &mut Command, fcgi: Option<&FastCgi>) {
    if let Some(f) = fcgi {
        cmd.env(FCGI_KEY_PARAM, &f.key);
    }
}

/// `caddy` with the kit's environment, its data and config folders in the
/// instance's folder `run_dir` (Caddy reads `XDG_DATA_HOME` and
/// `XDG_CONFIG_HOME` before the system's folders, on Windows too).
pub fn caddy_command(exe: &Path, run_dir: &Path, env: &KitEnv) -> Command {
    let mut cmd = jobs::kit_command(exe, env);
    cmd.env("XDG_DATA_HOME", run_dir.join("data"))
        .env("XDG_CONFIG_HOME", run_dir.join("config"));
    cmd
}

/// Prepares the project's `Caddyfile` and returns the `Command` that starts
/// its Caddy.
#[allow(clippy::too_many_arguments)]
pub fn command(
    kit: &Kit,
    project: &str,
    project_root: &Path,
    doc_root: &Path,
    port: u16,
    fcgi: Option<&FastCgi>,
    lan: bool,
    domains: &[String],
    aliases: &BTreeMap<String, String>,
    env: &KitEnv,
) -> Result<Command> {
    let caddy = caddy_dir(kit, env);
    let exe = caddy_exe(kit, &caddy);
    if !exe.is_file() {
        bail!("Caddy is not installed (Strumenti > Caddy)");
    }
    let run_dir = dir(kit, project)?;
    fs::create_dir_all(&run_dir)?;
    let conf = run_dir.join("Caddyfile");
    let content = project_conf(
        Some(project),
        doc_root,
        &run_dir,
        port,
        fcgi,
        lan,
        domains,
        &resolve_aliases(project_root, &kit.projects(), aliases),
    )?;
    write_atomic(&conf, content.as_bytes())?;
    let mut cmd = caddy_command(&exe, &run_dir, env);
    cmd.args(run_args(&conf));
    with_key(&mut cmd, fcgi);
    Ok(cmd)
}

#[cfg(test)]
#[path = "../../../test/rust/projects/caddy.rs"]
mod tests;
