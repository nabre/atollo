//! The kit's PHP as a FastCGI server, shared by Nginx and Caddy: on Windows
//! `php-cgi.exe -b 127.0.0.1:<port>`, on macOS and Linux `php-fpm` (the
//! static-php-cli build has no php-cgi) with a pool written at every start,
//! listening on an owner-only Unix socket ([`PrivateSocket`]), never on
//! TCP: php-fpm, unlike php-cgi, takes `PHP_VALUE` and `PHP_ADMIN_VALUE`
//! from whoever connects, which can turn the guard off. Every request must
//! also carry a random key, new at every start, that
//! `runtime/fcgi-guard.php` (prepended to every script) checks (D-99
//! «Resta»): the only defence of php-cgi's TCP port, a second one of
//! php-fpm's socket.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Arc;

use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::{Kit, Os};
use crate::os::jobs::{self, Job};
use crate::projects::apache::fwd;
use crate::projects::caddy::{self, PrivateSocket};

/// The FastCGI parameter that carries the key of [`FastCgi`], checked by
/// `runtime\fcgi-guard.php` (D-99 «Resta»).
pub const FCGI_KEY_PARAM: &str = "DEVKIT_FCGI_KEY";

/// Where a PHP FastCGI server listens: a port of 127.0.0.1 (php-cgi, which
/// ignores `PHP_VALUE`), or an owner-only Unix socket (php-fpm on macOS and
/// Linux), whose folder is removed when the last copy of this is dropped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FcgiAddr {
    Tcp(u16),
    Unix(Arc<PrivateSocket>),
}

impl FcgiAddr {
    /// The upstream of Caddy's `php_fastcgi`: `127.0.0.1:<port>` or
    /// `"unix/<path>"` (the path was checked usable in a Caddyfile).
    pub fn caddy_upstream(&self) -> String {
        match self {
            FcgiAddr::Tcp(port) => format!("127.0.0.1:{port}"),
            FcgiAddr::Unix(sock) => format!("\"{}\"", caddy::admin_address(&sock.path)),
        }
    }

    /// The address of Nginx's `fastcgi_pass`: `127.0.0.1:<port>` or
    /// `unix:<path>`.
    pub fn nginx_pass(&self) -> String {
        match self {
            FcgiAddr::Tcp(port) => format!("127.0.0.1:{port}"),
            FcgiAddr::Unix(sock) => format!("unix:{}", fwd(&sock.path)),
        }
    }

    /// The handler of Apache's `mod_proxy_fcgi` (D-116):
    /// `proxy:fcgi://127.0.0.1:<port>` or
    /// `proxy:unix:<path>|fcgi://localhost` (2.4.10 and later).
    pub fn apache_handler(&self) -> String {
        match self {
            FcgiAddr::Tcp(port) => format!("proxy:fcgi://127.0.0.1:{port}"),
            FcgiAddr::Unix(sock) => format!("proxy:unix:{}|fcgi://localhost", fwd(&sock.path)),
        }
    }

    /// The address as the log says it.
    pub fn describe(&self) -> String {
        match self {
            FcgiAddr::Tcp(port) => format!("port {port}"),
            FcgiAddr::Unix(sock) => format!("socket {}", sock.path.display()),
        }
    }
}

/// A running PHP FastCGI server as the web server reaches it: its address
/// ([`FcgiAddr`]) and the random key the web server sends with every
/// request. php-cgi has no authentication and Nginx for Windows speaks
/// FastCGI over TCP only (no Unix socket, no named pipe): without the key
/// the kit's guard, prepended to every script, answers 403 and runs nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastCgi {
    pub addr: FcgiAddr,
    pub key: String,
}

impl FastCgi {
    /// A FastCGI server on `port` of 127.0.0.1 with `key`.
    pub fn tcp(port: u16, key: impl Into<String>) -> Self {
        FastCgi {
            addr: FcgiAddr::Tcp(port),
            key: key.into(),
        }
    }
}

/// The FastCGI program of the PHP in `dir` for Nginx: `php-cgi.exe` on
/// Windows, `bin/php-cgi` on macOS and Linux (BUG-V3-08).
pub fn php_cgi_exe(kit: &Kit, dir: &Path) -> PathBuf {
    if kit.os().is_windows() {
        dir.join("php-cgi.exe")
    } else {
        dir.join("bin").join("php-cgi")
    }
}

/// The FastCGI server Caddy runs for the PHP in `dir` on `os`:
/// `php-cgi.exe` on Windows, `sbin/php-fpm` of static-php-cli elsewhere.
pub fn fastcgi_exe(os: Os, dir: &Path) -> PathBuf {
    if os.is_windows() {
        dir.join("php-cgi.exe")
    } else {
        dir.join("sbin").join("php-fpm")
    }
}

/// Resolves the PHP directory (`php-cgi`) from `env` (`CP_PHP`) or the kit's default line.
pub fn php_dir(kit: &Kit, env: &KitEnv) -> Option<PathBuf> {
    php_dir_with(kit, env, |dir| php_cgi_exe(kit, dir))
}

/// The PHP directory whose `program` exists: `CP_PHP` of `env`, else the
/// kit's default line.
fn php_dir_with(kit: &Kit, env: &KitEnv, program: impl Fn(&Path) -> PathBuf) -> Option<PathBuf> {
    let dir = env
        .get("CP_PHP")
        .map(PathBuf::from)
        .filter(|d| program(d).is_file())
        .unwrap_or_else(|| crate::setup::catalog::kit_path(kit, "php"));
    program(&dir).is_file().then_some(dir)
}

/// A new key for a FastCGI server; `None` (in the log) when it cannot be had.
fn new_key(what: &str) -> Option<String> {
    match crate::editor::random_token() {
        Ok(key) => Some(key),
        Err(e) => {
            crate::log::warn(&format!("{what}: no FastCGI key: {e:#}"));
            None
        }
    }
}

/// A new key and a free port of 127.0.0.1 for a FastCGI server; `None`
/// (in the log) when either cannot be had.
fn key_and_port(what: &str) -> Option<(String, u16)> {
    let key = new_key(what)?;
    let port = match std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .and_then(|l| l.local_addr())
    {
        Ok(addr) => addr.port(),
        Err(e) => {
            crate::log::warn(&format!("{what}: no free FastCGI port: {e}"));
            return None;
        }
    };
    Some((key, port))
}

/// The kit's guard, `None` (in the log) when it is missing: never a
/// FastCGI server that runs whatever it is asked.
fn guard_file(kit: &Kit, what: &str) -> Option<PathBuf> {
    let guard = kit.kit_files().join("fcgi-guard.php");
    if guard.is_file() {
        Some(guard)
    } else {
        crate::log::warn(&format!("{what}: {} is missing", guard.display()));
        None
    }
}

/// Spawns `php-cgi.exe -b 127.0.0.1:<port>` inside `job` when PHP is
/// installed, returning its [`FastCgi`] and the process, whose death the
/// caller watches (a server without its php-cgi answers 502 to every `.php`).
/// php-cgi gets a new key (`-d devkit.fcgi_key`), the kit's guard as
/// `auto_prepend_file`, and `FCGI_WEB_SERVER_ADDRS=127.0.0.1` (PHP's own
/// list of the web servers it accepts). `None` when PHP is not installed,
/// the guard is missing or the spawn failed (in the log).
pub fn spawn_php_cgi_if_present(kit: &Kit, env: &KitEnv, job: &Job) -> Option<(FastCgi, Child)> {
    let php = php_dir(kit, env)?;
    let cgi = php_cgi_exe(kit, &php);
    let guard = guard_file(kit, "php-cgi")?;
    let (key, port) = key_and_port("php-cgi")?;
    let mut cmd = jobs::kit_command(&cgi, env);
    cmd.arg("-b")
        .arg(format!("127.0.0.1:{port}"))
        .arg("-c")
        .arg(crate::install::php_ini_dir(&kit.data(), &php))
        .args(guard_args(&guard, &key))
        .env("PHP_FCGI_MAX_REQUESTS", "0")
        .env("FCGI_WEB_SERVER_ADDRS", "127.0.0.1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match jobs::spawn_in(&mut cmd, job) {
        Ok(child) => Some((FastCgi::tcp(port, key), child)),
        Err(e) => {
            crate::log::warn(&format!("php-cgi {}: {e:#}", cgi.display()));
            None
        }
    }
}

/// The php-cgi arguments of the guard: the key as an ini value of the
/// command line (`get_cfg_var`, out of reach of a request and of
/// `.user.ini`) and the guard in double quotes with `/`, so that a space or
/// a `;` of the kit's path stays in the ini value.
pub fn guard_args(guard: &Path, key: &str) -> [String; 4] {
    [
        "-d".to_owned(),
        format!("devkit.fcgi_key={key}"),
        "-d".to_owned(),
        format!("auto_prepend_file=\"{}\"", fwd(guard)),
    ]
}

/// The `php-fpm` configuration of one web server (`fpm.conf` in its folder
/// `run_dir`): in the foreground, its pid and log there, one pool on the
/// Unix socket `socket` (owner-only: `listen.mode = 0600`, in a folder only
/// the user can enter; never a TCP port, where any local program could send
/// `PHP_ADMIN_VALUE` and turn the guard off), workers started on demand,
/// the kit's environment passed to the scripts (`clear_env = no`: `PATH`,
/// `CP_*`, `DB_PORT`), and the files PHP runs for Apache and Nginx
/// (`.php`, `.phtml`, `.phar`).
pub fn fpm_conf(run_dir: &Path, socket: &Path) -> String {
    let run = fwd(run_dir);
    [
        "; Dev-Kit: php-fpm of this web server (written at every start)",
        "[global]",
        &format!("pid = \"{run}/php-fpm.pid\""),
        &format!("error_log = \"{run}/php-fpm.log\""),
        "daemonize = no",
        "",
        "[www]",
        &format!("listen = \"{}\"", fwd(socket)),
        "listen.mode = 0600",
        "pm = ondemand",
        "pm.max_children = 16",
        "pm.process_idle_timeout = 30s",
        "pm.max_requests = 500",
        "clear_env = no",
        "catch_workers_output = yes",
        "decorate_workers_output = no",
        "security.limit_extensions = .php .phtml .phar",
        "",
    ]
    .join("\n")
}

/// The arguments of `php-fpm` for the pool in `conf`: in the foreground,
/// the line's `php.ini` folder, and the guard with its key (as php-cgi's,
/// [`guard_args`]).
pub fn fpm_args(conf: &Path, ini_dir: &Path, guard: &Path, key: &str) -> Vec<String> {
    let mut args = vec![
        "--nodaemonize".to_owned(),
        "--fpm-config".to_owned(),
        conf.display().to_string(),
        "-c".to_owned(),
        ini_dir.display().to_string(),
    ];
    args.extend(guard_args(guard, key));
    args
}

/// Spawns the FastCGI server Caddy talks to, inside `job`: php-cgi on
/// Windows ([`spawn_php_cgi_if_present`]), `php-fpm` with `fpm.conf` in
/// `run_dir` on macOS and Linux, on a new [`PrivateSocket`] of the instance
/// `id`. `None` when PHP is not installed, the guard is missing, no private
/// socket can be had or the spawn failed (in the log): never php-fpm on TCP.
pub fn spawn_fastcgi_if_present(
    kit: &Kit,
    env: &KitEnv,
    job: &Job,
    run_dir: &Path,
    id: &str,
) -> Option<(FastCgi, Child)> {
    if kit.os().is_windows() {
        return spawn_php_cgi_if_present(kit, env, job);
    }
    let os = kit.os();
    let php = php_dir_with(kit, env, |d| fastcgi_exe(os, d))?;
    let fpm = fastcgi_exe(os, &php);
    let guard = guard_file(kit, "php-fpm")?;
    let key = new_key("php-fpm")?;
    let socket = Arc::new(caddy::private_socket(kit, id, "php-fpm", "fpm.sock")?);
    let conf = run_dir.join("fpm.conf");
    if let Err(e) = fs::create_dir_all(run_dir)
        .and_then(|()| write_atomic(&conf, fpm_conf(run_dir, &socket.path).as_bytes()))
    {
        crate::log::warn(&format!("php-fpm: {}: {e}", conf.display()));
        return None;
    }
    let ini = crate::install::php_ini_dir(&kit.data(), &php);
    let mut cmd = jobs::kit_command(&fpm, env);
    cmd.args(fpm_args(&conf, &ini, &guard, &key))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    match jobs::spawn_in(&mut cmd, job) {
        Ok(child) => Some((
            FastCgi {
                addr: FcgiAddr::Unix(socket),
                key,
            },
            child,
        )),
        Err(e) => {
            crate::log::warn(&format!("php-fpm {}: {e:#}", fpm.display()));
            None
        }
    }
}

#[cfg(test)]
#[path = "../../../test/rust/projects/fastcgi.rs"]
mod tests;
