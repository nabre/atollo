//! Adminer on the kit's MariaDB (DECISIONS.md D-85): what EasyPHP and XAMPP
//! give with phpMyAdmin, from the Database page.
//!
//! - PHP's own server (`php -S`) of the kit's default PHP, on 127.0.0.1 and a
//!   port the system gives, with the router `data\kit\adminer.php`
//!   (`runtime/adminer.php`), started at the first "Adminer" and ended
//!   with MariaDB or the app (the app's job);
//! - a new token at every start: the address the app opens carries it,
//!   any request without it (or its cookie) gets 403, so the other users
//!   of the PC that reach 127.0.0.1 cannot use root's login;
//! - root's password and the MariaDB port in the server's environment
//!   (`ATOLLO_DB_*`), never on the command line or in the browser;
//! - sessions in `data\app\adminer`, uploaded files in `adminer\` of the
//!   PC's temporary folder (D-90), the server's output in
//!   `data\app\logs\adminer.log`.

use std::fs::{self, File};
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

use super::auth;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::os::temp::TempWork;
use crate::projects::run::responds;
use crate::setup::catalog;

const READY_WITHIN: Duration = Duration::from_secs(15);

struct Server {
    child: Child,
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
pub struct Adminer {
    running: Mutex<Option<Server>>,
}

/// `adminer\adminer.php` of the component.
fn script(kit: &Kit) -> PathBuf {
    catalog::kit_path(kit, "adminer/adminer.php")
}

/// A free port of 127.0.0.1, from the system.
fn free_port() -> Result<u16> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).context("free port")?;
    Ok(listener.local_addr()?.port())
}

pub fn address(port: u16, token: &str) -> String {
    format!("http://127.0.0.1:{port}/?atollo={token}")
}

impl Adminer {
    /// The address that logs in to Adminer, on MariaDB at `db_port`; starts
    /// PHP's server when it is not running or serves another port.
    pub fn open(&self, kit: &Kit, env: &KitEnv, db_port: u16) -> Result<String> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = running.as_mut() {
            if matches!(s.child.try_wait(), Ok(None)) && s.db_port == db_port {
                return Ok(address(s.port, &s.token));
            }
        }
        *running = None;
        let server = start(kit, env, db_port)?;
        let url = address(server.port, &server.token);
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
            crate::log::info("adminer: stopped");
        }
    }
}

fn start(kit: &Kit, env: &KitEnv, db_port: u16) -> Result<Server> {
    let adminer = script(kit);
    if !adminer.is_file() {
        bail!("Adminer is not installed: {} missing", adminer.display());
    }
    let router = kit.kit_files().join("adminer.php");
    if !router.is_file() {
        crate::setup::kit_files::write_all(kit)?;
    }
    let password = auth::load(kit)
        .mariadb_root
        .ok_or_else(|| anyhow!("MariaDB has no password of root yet"))?;
    let dir = kit.app_dir().join("adminer");
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    // Uploaded files (an SQL file to import) pass through the PC's temporary
    // folder (D-90); the sessions stay in the kit.
    let uploads = TempWork::new(kit, "adminer").context("adminer uploads")?;
    let log = kit.logs().join("adminer.log");
    let out = File::create(&log).with_context(|| log.display().to_string())?;
    let token = crate::editor::random_token()?;
    let port = free_port()?;
    // The default PHP with its own php.ini and extensions, whatever line
    // the kit's environment names.
    let php_dir = catalog::kit_path(kit, "php");
    let mut cmd = jobs::kit_command(catalog::kit_path(kit, "php/php.exe"), env);
    cmd.env("CP_PHP", &php_dir)
        .env("PHPRC", crate::install::php_ini_dir(&kit.data(), &php_dir))
        .arg("-d")
        .arg(format!("session.save_path={}", dir.display()))
        .arg("-d")
        .arg(format!("upload_tmp_dir={}", uploads.path().display()))
        .arg("-S")
        .arg(format!("127.0.0.1:{port}"))
        .arg("-t")
        .arg(&dir)
        .arg(&router)
        .current_dir(&dir)
        .env("ATOLLO_ADMINER", &adminer)
        .env("ATOLLO_ADMINER_TOKEN", &token)
        .env("ATOLLO_DB_PORT", db_port.to_string())
        .env("ATOLLO_DB_PASSWORD", &password)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out.try_clone()?))
        .stderr(Stdio::from(out));
    let job = Job::new()?;
    let mut child = jobs::spawn_in(&mut cmd, &job).context("spawn php -S for Adminer")?;
    let begun = Instant::now();
    while !responds(port) {
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
    crate::log::info(&format!("adminer: started on 127.0.0.1:{port}"));
    Ok(Server {
        child,
        port,
        db_port,
        token,
        _job: job,
        _uploads: uploads,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/db/adminer.rs"]
pub(crate) mod tests;
