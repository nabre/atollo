//! Adminer on the kit's MariaDB (DECISIONS.md D-85): what EasyPHP and XAMPP
//! give with phpMyAdmin, from the Database page.
//!
//! - PHP's own server (`php -S`) of the kit's default PHP, on 127.0.0.1 and a
//!   port the system gives, with the router `data\kit\adminer.php`
//!   (`kit-files/adminer.php`), started at the first "Adminer" and ended
//!   with MariaDB or the app (the app's job);
//! - a new token at every start: the address the app opens carries it,
//!   any request without it (or its cookie) gets 403, so the other users
//!   of the PC that reach 127.0.0.1 cannot use root's login;
//! - root's password and the MariaDB port in the server's environment
//!   (`ATOLLO_DB_*`), never on the command line or in the browser;
//! - sessions and Adminer's temporary files in `data\app\adminer`, the
//!   server's output in `data\app\logs\adminer.log`.

use std::fs::{self, File};
use std::net::{Ipv4Addr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

use crate::catalog;
use crate::db_auth;
use crate::env::KitEnv;
use crate::jobs::{self, Job};
use crate::kit::Kit;
use crate::run::responds;

const READY_WITHIN: Duration = Duration::from_secs(15);

struct Server {
    child: Child,
    port: u16,
    /// The MariaDB port it logs in to.
    db_port: u16,
    token: String,
    // Dropped last: ends PHP's server.
    _job: Job,
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
        crate::kit_files::write_all(kit)?;
    }
    let password = db_auth::load(kit)
        .mariadb_root
        .ok_or_else(|| anyhow!("MariaDB has no password of root yet"))?;
    let dir = kit.app_dir().join("adminer");
    fs::create_dir_all(&dir).with_context(|| dir.display().to_string())?;
    let log = kit.logs().join("adminer.log");
    let out = File::create(&log).with_context(|| log.display().to_string())?;
    let token = crate::editor::random_token()?;
    let port = free_port()?;
    let mut cmd = jobs::kit_command(catalog::kit_path(kit, "php/php.exe"), env);
    cmd.arg("-d")
        .arg(format!("session.save_path={}", dir.display()))
        .arg("-d")
        .arg(format!("upload_tmp_dir={}", dir.display()))
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
    let mut child = jobs::spawn(&mut cmd)?;
    let job = Job::new()?;
    job.assign(&child)?;
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
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn address_carries_the_token() {
        assert_eq!(address(7400, "ab12"), "http://127.0.0.1:7400/?atollo=ab12");
    }

    /// Adminer of the kit given by ATOLLO_TEST_KIT on its MariaDB already
    /// running at ATOLLO_TEST_MARIADB_PORT (an open Atollo, whose data the
    /// real test of `services.rs` must not start a second server on).
    #[test]
    fn real_adminer_on_running_mariadb() {
        let (Ok(root), Ok(db_port)) = (
            std::env::var("ATOLLO_TEST_KIT"),
            std::env::var("ATOLLO_TEST_MARIADB_PORT"),
        ) else {
            eprintln!("ATOLLO_TEST_KIT or ATOLLO_TEST_MARIADB_PORT not set: skipped");
            return;
        };
        let kit = Kit::at(PathBuf::from(root), crate::kit::Arch::detect());
        let env = KitEnv::build(
            &kit,
            &crate::settings::Settings::default(),
            "en",
            &crate::env::Host::capture(),
        );
        let adminer = Adminer::default();
        let url = adminer.open(&kit, &env, db_port.parse().unwrap()).unwrap();
        assert_logs_in(&url);
        adminer.stop();
        assert!(ureq::get(&url).call().is_err(), "stopped");
    }

    /// The checks of a real Adminer at `url` (the real test of `services.rs`):
    /// 403 without the token, then logged in as root with it.
    pub fn assert_logs_in(url: &str) {
        let port = url
            .trim_start_matches("http://127.0.0.1:")
            .split('/')
            .next()
            .unwrap()
            .to_owned();
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .proxy(None)
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .into();
        let bare = agent
            .get(&format!("http://127.0.0.1:{port}/"))
            .call()
            .unwrap();
        assert_eq!(bare.status().as_u16(), 403, "no token, no Adminer");
        let wrong = agent
            .get(&format!("http://127.0.0.1:{port}/?atollo=0"))
            .call()
            .unwrap();
        assert_eq!(wrong.status().as_u16(), 403, "wrong token");
        let entry = agent.get(url).call().unwrap();
        assert_eq!(entry.status().as_u16(), 302);
        let cookies: Vec<String> = entry
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .filter_map(|c| c.split(';').next().map(str::to_owned))
            .collect();
        let location = entry
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .unwrap()
            .to_owned();
        assert!(!location.contains("atollo="), "token left in {location}");
        let mut page = agent
            .get(&format!("http://127.0.0.1:{port}/{location}"))
            .header("Cookie", cookies.join("; "))
            .call()
            .unwrap();
        let status = page.status().as_u16();
        let body = page.body_mut().read_to_string().unwrap();
        assert_eq!(status, 200, "{body}");
        assert!(body.contains("db=mysql"), "databases listed: {body}");
    }
}
