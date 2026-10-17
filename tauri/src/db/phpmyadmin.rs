//! phpMyAdmin on the kit's MariaDB (DECISIONS.md D-107, in place of Adminer
//! of D-85): what EasyPHP and XAMPP give, from the Database page.
//!
//! - PHP's own server (`php -S`) of the kit's default PHP, on a loopback
//!   address of its own (`127.x.y.z`, new at every start) and a port the
//!   system gives: browsers share cookies between the ports of one host, so
//!   on 127.0.0.1 the token's cookie and phpMyAdmin's session would reach
//!   the projects' sites (their PHP could read them and log in as root);
//!   serving phpMyAdmin's folder through the router
//!   `data\kit\phpmyadmin.php` (`runtime/phpmyadmin.php`), started at the
//!   first "phpMyAdmin" and ended with the MariaDB version it logs in to or
//!   the app (the app's job);
//! - one at a time, on the version chosen (PLAN.md §13.2: each on its own
//!   port): opening it on another version starts it again there;
//! - a new token at every start: the address the app opens carries it,
//!   any request without it (or its cookie) gets 403, so the other users
//!   of the PC that reach 127.0.0.1 cannot use root's login;
//! - `config.inc.php` in phpMyAdmin's folder is the app's
//!   (`runtime/phpmyadmin.config.php`), written at every start: root's
//!   password, the MariaDB port and the cookie key come from the server's
//!   environment (`ATOLLO_*`), never the command line or the browser;
//! - sessions and phpMyAdmin's cache in `data\app\phpmyadmin`, uploaded
//!   files in `phpmyadmin\` of the PC's temporary folder (D-90), the
//!   server's output in `data\app\logs\phpmyadmin.log`.

use std::fs::{self, File};
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

use super::auth;
use crate::env::KitEnv;
use crate::fsx::write_atomic;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::os::temp::TempWork;
use crate::setup::catalog;

const READY_WITHIN: Duration = Duration::from_secs(15);

/// phpMyAdmin's `config.inc.php`, written into its folder.
const CONFIG: &str = include_str!("../../../runtime/phpmyadmin.config.php");

struct Server {
    child: Child,
    host: Ipv4Addr,
    port: u16,
    /// The MariaDB port it logs in to.
    db_port: u16,
    token: String,
    // Dropped after the fields above: ends PHP's server.
    _job: Job,
    // Dropped last, once PHP has ended: the uploads' folder.
    _uploads: TempWork,
}

#[derive(Default)]
pub struct PhpMyAdmin {
    running: Mutex<Option<Server>>,
}

/// `phpmyadmin\index.php` of the component.
fn entry(kit: &Kit) -> PathBuf {
    catalog::kit_path(kit, "phpmyadmin/index.php")
}

/// The loopback address phpMyAdmin is served on, from random bytes: in
/// 127.0.0.0/8 and never 127.0.0.1, where the projects' sites are. macOS
/// answers only on 127.0.0.1 unless an alias is added: there it stays.
pub fn private_loopback(random: [u8; 3]) -> Ipv4Addr {
    if cfg!(target_os = "macos") {
        return Ipv4Addr::LOCALHOST;
    }
    Ipv4Addr::new(127, 1 + random[0] % 254, random[1], 1 + random[2] % 254)
}

/// Three random bytes of a fresh token (64 hex digits).
fn random_bytes() -> Result<[u8; 3]> {
    let token = crate::editor::random_token()?;
    let byte = |i: usize| u8::from_str_radix(&token[2 * i..2 * i + 2], 16);
    Ok([byte(0)?, byte(1)?, byte(2)?])
}

/// A loopback address of its own and a free port on it, from the system;
/// 127.0.0.1 when the system refuses the address (logged).
fn free_address() -> Result<(Ipv4Addr, u16)> {
    let host = private_loopback(random_bytes()?);
    let listener = match TcpListener::bind((host, 0)) {
        Ok(l) => l,
        Err(e) if host != Ipv4Addr::LOCALHOST => {
            crate::log::warn(&format!(
                "phpmyadmin: {host} refused ({e}), using 127.0.0.1"
            ));
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).context("free port")?
        }
        Err(e) => return Err(e).context("free port"),
    };
    let addr = listener.local_addr()?;
    let std::net::IpAddr::V4(ip) = addr.ip() else {
        bail!("not an IPv4 address: {addr}");
    };
    Ok((ip, addr.port()))
}

fn responds_at(host: Ipv4Addr, port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&(host, port).into(), Duration::from_millis(400)).is_ok()
}

pub fn address(host: Ipv4Addr, port: u16, token: &str) -> String {
    format!("http://{host}:{port}/?atollo={token}")
}

impl PhpMyAdmin {
    /// The address that logs in to phpMyAdmin, on MariaDB at `db_port`, in
    /// `lang` (`it`, `en`...); starts PHP's server when it is not running or
    /// serves another port.
    pub fn open(&self, kit: &Kit, env: &KitEnv, db_port: u16, lang: &str) -> Result<String> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = running.as_mut() {
            if matches!(s.child.try_wait(), Ok(None)) && s.db_port == db_port {
                return Ok(address(s.host, s.port, &s.token));
            }
        }
        *running = None;
        let server = start(kit, env, db_port, lang)?;
        let url = address(server.host, server.port, &server.token);
        *running = Some(server);
        Ok(url)
    }

    pub fn stop(&self) {
        if self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .is_some()
        {
            crate::log::info("phpmyadmin: stopped");
        }
    }

    /// Ends phpMyAdmin when it logs in to the MariaDB at `db_port` (a
    /// version that stops); one on another version stays.
    pub fn stop_for(&self, db_port: u16) {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if running.as_ref().is_some_and(|s| s.db_port == db_port) {
            *running = None;
            crate::log::info("phpmyadmin: stopped");
        }
    }
}

fn start(kit: &Kit, env: &KitEnv, db_port: u16, lang: &str) -> Result<Server> {
    let index = entry(kit);
    let Some(root) = index.parent().filter(|_| index.is_file()) else {
        bail!("phpMyAdmin is not installed: {} missing", index.display());
    };
    // The router of this version of the app (only files that differ are
    // written).
    crate::setup::kit_files::write_all(kit)?;
    let router = kit.kit_files().join("phpmyadmin.php");
    let config = root.join("config.inc.php");
    if fs::read(&config).ok().as_deref() != Some(CONFIG.as_bytes()) {
        write_atomic(&config, CONFIG.as_bytes()).with_context(|| config.display().to_string())?;
    }
    let password = auth::load(kit)
        .mariadb_root
        .ok_or_else(|| anyhow!("MariaDB has no password of root yet"))?;
    let dir = kit.app_dir().join("phpmyadmin");
    let (sessions, cache) = (dir.join("sessions"), dir.join("tmp"));
    for d in [&sessions, &cache] {
        fs::create_dir_all(d).with_context(|| d.display().to_string())?;
    }
    // Uploaded files (an SQL file to import) pass through the PC's temporary
    // folder (D-90); the sessions stay in the kit.
    let uploads = TempWork::new(kit, "phpmyadmin").context("phpmyadmin uploads")?;
    let log = kit.logs().join("phpmyadmin.log");
    let out = File::create(&log).with_context(|| log.display().to_string())?;
    let token = crate::editor::random_token()?;
    let secret = crate::editor::random_token()?;
    let (host, port) = free_address()?;
    // The default PHP with its own php.ini and extensions, whatever line
    // the kit's environment names.
    let php_dir = catalog::kit_path(kit, "php");
    let mut cmd = jobs::kit_command(catalog::kit_path(kit, "php/php.exe"), env);
    cmd.env("CP_PHP", &php_dir)
        .env("PHPRC", crate::install::php_ini_dir(&kit.data(), &php_dir))
        .arg("-d")
        .arg(format!("session.save_path={}", sessions.display()))
        .arg("-d")
        .arg(format!("upload_tmp_dir={}", uploads.path().display()))
        // Large SQL files to import.
        .args(["-d", "upload_max_filesize=512M", "-d", "post_max_size=512M"])
        .args(["-d", "memory_limit=512M", "-d", "max_execution_time=0"])
        .arg("-S")
        .arg(format!("{host}:{port}"))
        .arg("-t")
        .arg(root)
        .arg(&router)
        .current_dir(root)
        .env("ATOLLO_PMA_ROOT", root)
        .env("ATOLLO_PMA_HOST", host.to_string())
        .env("ATOLLO_PMA_TOKEN", &token)
        .env("ATOLLO_PMA_SECRET", &secret)
        .env("ATOLLO_PMA_TMP", &cache)
        .env("ATOLLO_LANG", lang)
        .env("ATOLLO_DB_PORT", db_port.to_string())
        .env("ATOLLO_DB_PASSWORD", &password)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out.try_clone()?))
        .stderr(Stdio::from(out));
    let job = Job::new()?;
    let mut child = jobs::spawn_in(&mut cmd, &job).context("spawn php -S for phpMyAdmin")?;
    let begun = Instant::now();
    while !responds_at(host, port) {
        if !matches!(child.try_wait(), Ok(None)) {
            let text = fs::read_to_string(&log).unwrap_or_default();
            bail!("PHP's server ended: {}", text.trim());
        }
        if begun.elapsed() > READY_WITHIN {
            let _ = child.kill();
            bail!(
                "no answer on port {port} within {} s",
                READY_WITHIN.as_secs()
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    crate::log::info(&format!("phpmyadmin: started on {host}:{port}"));
    Ok(Server {
        child,
        host,
        port,
        db_port,
        token,
        _job: job,
        _uploads: uploads,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/db/phpmyadmin.rs"]
pub(crate) mod tests;
