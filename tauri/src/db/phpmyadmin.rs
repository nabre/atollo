//! phpMyAdmin on the kit's SQL server, MySQL (DECISIONS.md
//! D-107, in place of Adminer of D-85; D-110): what EasyPHP and XAMPP give,
//! from the Database page.
//!
//! - PHP's own server (`php -S`) of the kit's default PHP, on a loopback
//!   address of its own (`127.x.y.z`, new at every start) and a port the
//!   system gives (`web_admin`, shared with mongo-express, D-147): browsers
//!   share cookies between the ports of one host, so on 127.0.0.1 the
//!   token's cookie and phpMyAdmin's session would reach the projects'
//!   sites (their PHP could read them and log in as root); serving
//!   phpMyAdmin's folder through the router
//!   `data\kit\phpmyadmin.php` (`runtime/phpmyadmin.php`), started at the
//!   first "phpMyAdmin" and ended with the MySQL version it logs in to or
//!   the app (the app's job);
//! - one at a time, on the version chosen (D-102: each on its own
//!   port): opening it on another version starts it again there;
//! - a new token at every start: the address the app opens carries it,
//!   any request without it (or its cookie) gets 403, so the other users
//!   of the PC that reach 127.0.0.1 cannot use root's login;
//! - `config.inc.php` in phpMyAdmin's folder is the app's
//!   (`runtime/phpmyadmin.config.php`), written at every start: root's
//!   password, the MySQL port and the cookie key come from the server's
//!   environment (`ATOLLO_*`), never the command line or the browser;
//! - sessions and phpMyAdmin's cache in `data\app\phpmyadmin`, uploaded
//!   files in `phpmyadmin\` of the PC's temporary folder (D-90), the
//!   server's output in `data\app\logs\phpmyadmin.log`.

use std::fs::{self, File};
use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{bail, Context, Result};

#[cfg(test)]
use super::web_admin::address;
// Its test does not run on macOS, whose loopback answers on 127.0.0.1 only.
#[cfg(all(test, not(target_os = "macos")))]
use super::web_admin::private_loopback;
use super::web_admin::{abandon, free_address, wait_ready, Served, Slot};
use crate::common::fsx::write_atomic;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::os::temp::TempWork;
use crate::setup::catalog;

/// phpMyAdmin's `config.inc.php`, written into its folder.
const CONFIG: &str = include_str!("../../../runtime/phpmyadmin.config.php");

pub struct PhpMyAdmin {
    slot: Slot,
}

impl Default for PhpMyAdmin {
    fn default() -> Self {
        PhpMyAdmin {
            slot: Slot::new("phpmyadmin"),
        }
    }
}

/// `phpmyadmin\index.php` of the component.
fn entry(kit: &Kit) -> PathBuf {
    catalog::kit_path(kit, "phpmyadmin/index.php")
}

impl PhpMyAdmin {
    /// The address that logs in to phpMyAdmin, on MySQL at `db_port`, in
    /// `lang` (`it`, `en`...); starts PHP's server when it is not running or
    /// serves another port.
    /// `password`: root's of the server on `db_port`.
    pub fn open(
        &self,
        kit: &Kit,
        env: &KitEnv,
        db_port: u16,
        password: &str,
        lang: &str,
    ) -> Result<String> {
        self.slot
            .open(db_port, || start(kit, env, db_port, password, lang))
    }

    pub fn stop(&self) {
        self.slot.stop();
    }

    /// Ends phpMyAdmin when it logs in to the MySQL at `db_port` (a
    /// version that stops); one on another version stays.
    pub fn stop_for(&self, db_port: u16) {
        self.slot.stop_for(db_port);
    }
}

fn start(kit: &Kit, env: &KitEnv, db_port: u16, password: &str, lang: &str) -> Result<Served> {
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
    let (host, port) = free_address(kit, "phpmyadmin")?;
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
        .env("ATOLLO_DB_PASSWORD", password)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out.try_clone()?))
        .stderr(Stdio::from(out));
    let job = Job::new()?;
    let mut child = jobs::spawn_in(&mut cmd, &job).context("spawn php -S for phpMyAdmin")?;
    if let Err(e) = wait_ready(&mut child, host, port, &log, "PHP's server") {
        abandon(job, child);
        return Err(e);
    }
    crate::common::log::info(&format!("phpmyadmin: started on {host}:{port}"));
    Ok(Served {
        child,
        host,
        port,
        db_port,
        token,
        _job: job,
        _keep: Some(uploads),
    })
}

#[cfg(test)]
#[path = "../../../test/rust/db/phpmyadmin.rs"]
pub(crate) mod tests;
