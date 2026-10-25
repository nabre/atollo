//! What phpMyAdmin (D-107) and mongo-express (D-147) share: a server of
//! the app's job on a loopback address of its own (`127.x.y.z`, new at
//! every start) and a port the system gives that no other part of Atollo
//! owns (D-141), one at a time, on the database version it logs in to,
//! opened with a new token in its address.

use std::fs;
use std::net::{Ipv4Addr, TcpListener};
use std::path::Path;
use std::process::Child;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use crate::kit::Kit;
use crate::os::jobs::Job;
use crate::os::temp::TempWork;

const READY_WITHIN: Duration = Duration::from_secs(15);

/// Ports tried before giving up on one that no part of Atollo owns.
const PORT_TRIES: usize = 20;

/// A running web admin.
pub(crate) struct Served {
    pub child: Child,
    pub host: Ipv4Addr,
    pub port: u16,
    /// The database port it logs in to.
    pub db_port: u16,
    pub token: String,
    // Dropped after the fields above: ends the server.
    pub _job: Job,
    // Dropped last, once the server has ended (phpMyAdmin's uploads).
    pub _keep: Option<TempWork>,
}

/// The one web admin of a kind (`name`, for the log), on the database
/// version it logs in to.
pub(crate) struct Slot {
    name: &'static str,
    running: Mutex<Option<Served>>,
}

impl Slot {
    pub const fn new(name: &'static str) -> Slot {
        Slot {
            name,
            running: Mutex::new(None),
        }
    }

    /// The address of the running server when it logs in to `db_port`;
    /// otherwise the old one ends and `start` gives a new one.
    pub fn open(&self, db_port: u16, start: impl FnOnce() -> Result<Served>) -> Result<String> {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = running.as_mut() {
            if matches!(s.child.try_wait(), Ok(None)) && s.db_port == db_port {
                return Ok(address(s.host, s.port, &s.token));
            }
        }
        *running = None;
        let served = start()?;
        let url = address(served.host, served.port, &served.token);
        *running = Some(served);
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
            crate::common::log::info(&format!("{}: stopped", self.name));
        }
    }

    /// Ends the server when it logs in to the database at `db_port` (a
    /// version that stops); one on another version stays.
    pub fn stop_for(&self, db_port: u16) {
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        if running.as_ref().is_some_and(|s| s.db_port == db_port) {
            *running = None;
            crate::common::log::info(&format!("{}: stopped", self.name));
        }
    }
}

/// The loopback address a web admin is served on, from random bytes: in
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

/// A loopback address of its own and a free port on it, from the system,
/// that no part of Atollo owns (`ports::ports_taken`, D-141); 127.0.0.1
/// when the system refuses the address (logged).
pub fn free_address(kit: &Kit, name: &str) -> Result<(Ipv4Addr, u16)> {
    let mut host = private_loopback(random_bytes()?);
    let taken = crate::common::ports::ports_taken(kit);
    // Ports owned by Atollo stay bound until the end: the system does not
    // give them again.
    let mut refused = Vec::new();
    for _ in 0..PORT_TRIES {
        let listener = match TcpListener::bind((host, 0)) {
            Ok(l) => l,
            Err(e) if host != Ipv4Addr::LOCALHOST => {
                crate::common::log::warn(&format!("{name}: {host} refused ({e}), using 127.0.0.1"));
                host = Ipv4Addr::LOCALHOST;
                continue;
            }
            Err(e) => return Err(e).context("free port"),
        };
        let addr = listener.local_addr()?;
        let std::net::IpAddr::V4(ip) = addr.ip() else {
            bail!("not an IPv4 address: {addr}");
        };
        if !taken.contains_key(&addr.port()) {
            return Ok((ip, addr.port()));
        }
        refused.push(listener);
    }
    bail!("{name}: no free port that Atollo does not use")
}

fn responds_at(host: Ipv4Addr, port: u16) -> bool {
    std::net::TcpStream::connect_timeout(&(host, port).into(), Duration::from_millis(400)).is_ok()
}

/// The address the app opens: the server with this start's token.
pub fn address(host: Ipv4Addr, port: u16, token: &str) -> String {
    format!("http://{host}:{port}/?atollo={token}")
}

/// Waits until `child` answers at `host:port`; its output (`log`) tells
/// why when it ends first. `what`: the server, for the error.
pub fn wait_ready(
    child: &mut Child,
    host: Ipv4Addr,
    port: u16,
    log: &Path,
    what: &str,
) -> Result<()> {
    let begun = Instant::now();
    while !responds_at(host, port) {
        if !matches!(child.try_wait(), Ok(None)) {
            let text = fs::read_to_string(log).unwrap_or_default();
            bail!("{what} ended: {}", text.trim());
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
    Ok(())
}

/// Ends a server that [`wait_ready`] gave up on: its job first, which
/// kills the whole group while the leader, not yet reaped, still names it
/// (`jobs::own_group`), then the leader is reaped, or on macOS and Linux it
/// would stay a zombie until the app exits (BUG-V7-14).
pub fn abandon(job: Job, mut child: Child) {
    drop(job);
    let _ = child.kill();
    let _ = child.wait();
}
