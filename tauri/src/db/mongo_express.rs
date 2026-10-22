//! mongo-express on the kit's MongoDB (DECISIONS.md D-147): MongoDB in the
//! browser, what phpMyAdmin is for MySQL (D-107), from the Databases page.
//!
//! - the kit's Node.js runs `data\kit\mongo-express.js`
//!   (`runtime/mongo-express.js`), which loads mongo-express's middleware
//!   behind the same gate as phpMyAdmin's router: mongo-express's own
//!   server knows only a fixed user and password (basic auth), turned off;
//! - on a loopback address of its own and a port that no other part of
//!   Atollo owns (`web_admin`, D-141), in the app's job, one at a time on
//!   the MongoDB version chosen, ended with it, «Ferma tutto», the removal
//!   of the drive or the app;
//! - a new token at every start: the address the app opens carries it, the
//!   gate turns it into an HttpOnly, SameSite=Strict cookie, any request
//!   without it, or for another Host, gets 403;
//! - the password of MongoDB's `admin` and the token reach the starter in
//!   its environment only (`ATOLLO_DB_PASSWORD`, `ATOLLO_ME_TOKEN`), which
//!   it empties before loading mongo-express: never the command line, a
//!   file, the address or the browser;
//! - working folder `data\app\mongo-express` (empty: mongo-express reads a
//!   `.env` there), output in `data\app\logs\mongo-express.log`.

use std::fs::{self, File};
use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};

use super::web_admin::{abandon, free_address, wait_ready, Served, Slot};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::setup::catalog;

/// The starter the app writes to `data\kit\` (`setup/kit_files.rs`).
pub const STARTER: &str = "mongo-express.js";

pub struct MongoExpress {
    slot: Slot,
}

impl Default for MongoExpress {
    fn default() -> Self {
        MongoExpress {
            slot: Slot::new("mongo-express"),
        }
    }
}

/// `mongo-express\app.js` of the component.
fn entry(kit: &Kit) -> PathBuf {
    catalog::kit_path(kit, "mongo-express/app.js")
}

impl MongoExpress {
    /// The address that opens mongo-express logged in to MongoDB at
    /// `db_port` as `admin` (`password`); starts it when it is not running
    /// or serves another port.
    pub fn open(&self, kit: &Kit, env: &KitEnv, db_port: u16, password: &str) -> Result<String> {
        self.slot
            .open(db_port, || start(kit, env, db_port, password))
    }

    pub fn stop(&self) {
        self.slot.stop();
    }

    /// Ends mongo-express when it logs in to the MongoDB at `db_port` (a
    /// version that stops); one on another version stays.
    pub fn stop_for(&self, db_port: u16) {
        self.slot.stop_for(db_port);
    }
}

/// One start of mongo-express.
pub struct Launch<'a> {
    /// The kit's Node.js.
    pub node: &'a Path,
    /// `data\kit\mongo-express.js`.
    pub starter: &'a Path,
    /// The private working folder.
    pub work: &'a Path,
    /// mongo-express's folder.
    pub root: &'a Path,
    pub host: Ipv4Addr,
    pub port: u16,
    pub db_port: u16,
    pub password: &'a str,
    pub token: &'a str,
}

/// The command of `launch`: the secrets in the environment, nothing but
/// the starter as argument.
pub fn command(env: &KitEnv, launch: &Launch) -> Command {
    let mut cmd = jobs::kit_command(launch.node, env);
    cmd.arg(launch.starter)
        .current_dir(launch.work)
        .env("NODE_ENV", "production")
        .env("ATOLLO_ME_ROOT", launch.root)
        .env("ATOLLO_ME_HOST", launch.host.to_string())
        .env("ATOLLO_ME_PORT", launch.port.to_string())
        .env("ATOLLO_ME_TOKEN", launch.token)
        .env("ATOLLO_DB_PORT", launch.db_port.to_string())
        .env("ATOLLO_DB_PASSWORD", launch.password)
        .stdin(Stdio::null());
    cmd
}

fn start(kit: &Kit, env: &KitEnv, db_port: u16, password: &str) -> Result<Served> {
    let app = entry(kit);
    let Some(root) = app.parent().filter(|_| app.is_file()) else {
        bail!("mongo-express is not installed: {} missing", app.display());
    };
    if password.is_empty() {
        bail!("MongoDB has no password of admin yet");
    }
    let node = catalog::kit_path(kit, "node/node.exe");
    if !node.is_file() {
        bail!("Node.js is not installed: {} missing", node.display());
    }
    // The starter of this version of the app (only files that differ are
    // written).
    crate::setup::kit_files::write_all(kit)?;
    let starter = kit.kit_files().join(STARTER);
    let work = kit.app_dir().join("mongo-express");
    fs::create_dir_all(&work).with_context(|| work.display().to_string())?;
    fs::create_dir_all(kit.logs()).with_context(|| kit.logs().display().to_string())?;
    let log = kit.logs().join("mongo-express.log");
    let out = File::create(&log).with_context(|| log.display().to_string())?;
    let token = crate::editor::random_token()?;
    let (host, port) = free_address(kit, "mongo-express")?;
    let mut cmd = command(
        env,
        &Launch {
            node: &node,
            starter: &starter,
            work: &work,
            root,
            host,
            port,
            db_port,
            password,
            token: &token,
        },
    );
    cmd.stdout(Stdio::from(out.try_clone()?))
        .stderr(Stdio::from(out));
    let job = Job::new()?;
    let mut child = jobs::spawn_in(&mut cmd, &job).context("spawn node for mongo-express")?;
    if let Err(e) = wait_ready(&mut child, host, port, &log, "mongo-express") {
        abandon(job, child);
        return Err(e);
    }
    crate::log::info(&format!("mongo-express: started on {host}:{port}"));
    Ok(Served {
        child,
        host,
        port,
        db_port,
        token,
        _job: job,
        _keep: None,
    })
}

#[cfg(test)]
#[path = "../../../test/rust/db/mongo_express.rs"]
pub(crate) mod tests;
