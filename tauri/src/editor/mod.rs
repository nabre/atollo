//! Editor and consoles: the VSCodium web server (this file), its windows
//! (`window`), settings (`cfg`), extensions (`ext`) and theme (`theme`) and
//! the consoles (`terminal`, a console window, on macOS and Linux a session
//! of the app's terminal, `pty`; `terminals`, the Terminals page: saved
//! consoles with their own tools and versions; `picker`, the folder and file
//! windows of macOS and Linux).
//!
//! The integrated editor: the VSCodium web server, one per session.
//!
//! - listens on 127.0.0.1 only, on a free port in 7000-7099 (the projects'
//!   servers use 7100-7999);
//! - a random 32-byte token per session, in `data\vscodium-server\token`
//!   (readable by the user only where the disk has permissions), passed with
//!   `--connection-token-file`, never on the command line;
//! - started as `node.exe out\server-main.js` (what `bin\codium-server.cmd`
//!   does, without cmd.exe and its quoting), with the kit environment, in a
//!   job of its own: stopping the server ends its whole tree;
//! - output in `data\app\logs\editor.log`, with the token masked.

pub mod cfg;
#[cfg(windows)]
pub mod conpty;
pub mod ext;
pub mod pty;
pub mod terminal;
pub mod terminals;
pub mod theme;
pub mod window;

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
#[cfg(windows)]
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

use crate::env::{mask_secrets, KitEnv};
use crate::kit::{Kit, Os};
use crate::platform::jobs::{self, Job};

pub const PORTS: std::ops::RangeInclusive<u16> = 7000..=7099;
const START_TIMEOUT: Duration = Duration::from_secs(30);

struct Server {
    port: u16,
    token: String,
    child: Child,
    // Dropped last: ends the server and everything it started.
    _job: Job,
}

impl Server {
    /// Ends the server and its tree. The killed program is then waited
    /// for, on a thread of its own (a kit on a slow disk can keep it a
    /// moment): never reaped, it would stay a zombie on macOS and Linux
    /// until the app exits (BUG-V7-20).
    fn end(self) {
        let Server {
            mut child, _job, ..
        } = self;
        let _ = child.kill();
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub installed: bool,
    pub running: bool,
    pub port: Option<u16>,
    /// The server is x64 only: emulated on ARM64.
    pub emulated: bool,
    /// Recommended extensions being installed in the background.
    pub extensions: ext::ExtState,
}

pub struct Editor {
    kit: Kit,
    env: Mutex<KitEnv>,
    /// Held only to look at, clear or store the server, never across a
    /// start: `status()` runs on the main thread (the 5 s poll, the close
    /// button) and must not wait up to 90 s for a slow start (BUG-V7-15).
    server: Mutex<Option<Server>>,
    /// Held for a whole start, so that two opens start one server; waited
    /// for by `stop_and_wait()`, never by `status()`.
    starting: Mutex<()>,
    /// Bumped by `stop()`: a start that was running when the server was
    /// stopped gives up (no next port, no files written) and ends the
    /// server it made instead of storing it.
    stops: AtomicU64,
    extensions: std::sync::Arc<Mutex<ext::ExtState>>,
    /// The background install of extensions: ends with the server, which
    /// it shares the program folder with (an update could not replace it).
    extensions_job: ext::JobSlot,
    /// Told what was installed, to offer a restart in the editor windows.
    on_installed: Mutex<Option<ext::OnInstalled>>,
}

/// Where the server program is.
pub fn server_dir(kit: &Kit) -> PathBuf {
    kit.apps().join("vscodium-server")
}

/// The files started (`out\server-main.js`, `node.exe`) are there: the
/// catalog's key and keys of `vscodium-server`.
pub fn is_installed(kit: &Kit) -> bool {
    crate::setup::Catalog::get()
        .find("vscodium-server")
        .is_some_and(|c| c.is_installed(kit))
}

impl Editor {
    pub fn new(kit: Kit, env: KitEnv) -> Editor {
        Editor {
            kit,
            env: Mutex::new(env),
            server: Mutex::new(None),
            starting: Mutex::new(()),
            stops: AtomicU64::new(0),
            extensions: std::sync::Arc::default(),
            extensions_job: ext::JobSlot::default(),
            on_installed: Mutex::new(None),
        }
    }

    fn env(&self) -> KitEnv {
        self.env.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// New environment for the next start of the server.
    pub fn set_env(&self, env: KitEnv) {
        *self.env.lock().unwrap_or_else(|e| e.into_inner()) = env;
    }

    pub fn status(&self) -> Status {
        let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
        let alive = guard
            .as_mut()
            .is_some_and(|s| matches!(s.child.try_wait(), Ok(None)));
        if !alive {
            *guard = None;
        }
        Status {
            installed: is_installed(&self.kit),
            running: alive,
            port: guard.as_ref().map(|s| s.port),
            // The x64 build only where the catalog says so (Windows ARM64):
            // native on macOS and Linux ARM64.
            emulated: crate::setup::Catalog::get()
                .find("vscodium-server")
                .is_some_and(|c| c.is_emulated_on(self.kit.target())),
            extensions: self
                .extensions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        }
    }

    /// The server's base URL and token, starting it when needed.
    pub fn ensure_running(&self) -> Result<(u16, String)> {
        let _starting = self.starting.lock().unwrap_or_else(|e| e.into_inner());
        {
            let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(s) = guard.as_mut() {
                if matches!(s.child.try_wait(), Ok(None)) {
                    // A project opened: its extensions may be new.
                    self.sync_extensions_now(false);
                    return Ok((s.port, s.token.clone()));
                }
            }
            *guard = None;
        }
        let stops = self.stops.load(Ordering::SeqCst);
        // Without the server lock: status() answers "not running" meanwhile.
        let server = self.start(stops)?;
        let out = (server.port, server.token.clone());
        {
            let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
            if self.stops.load(Ordering::SeqCst) != stops {
                // Stopped while it started (an update, the app closing):
                // what was asked for no longer holds.
                server.end();
                bail!("the editor was stopped while it started");
            }
            *guard = Some(server);
        }
        self.sync_extensions_now(true);
        Ok(out)
    }

    pub fn set_on_installed(&self, f: ext::OnInstalled) {
        *self.on_installed.lock().unwrap_or_else(|e| e.into_inner()) = Some(f);
    }

    /// Installs the extensions the projects now want, if the server runs
    /// (otherwise at its start). False when an install is already running:
    /// ask again later.
    pub fn sync_extensions(&self) -> bool {
        let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
        if guard
            .as_mut()
            .is_some_and(|s| matches!(s.child.try_wait(), Ok(None)))
        {
            return self.sync_extensions_now(false);
        }
        true
    }

    /// `retry`: the server (re)started, failed extensions are tried again.
    fn sync_extensions_now(&self, retry: bool) -> bool {
        let settings = crate::common::settings::Settings::load(&self.kit).settings;
        let net = crate::net::Net::new(&settings, None)
            .ok()
            .map(std::sync::Arc::new);
        ext::sync_in_background(
            self.kit.clone(),
            self.env(),
            net,
            std::sync::Arc::clone(&self.extensions),
            std::sync::Arc::clone(&self.extensions_job),
            retry,
            self.on_installed
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        )
    }

    pub fn stop(&self) {
        let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
        // Under the server lock, so a start in progress either stored its
        // server before (ended here) or sees the count change after.
        self.stops.fetch_add(1, Ordering::SeqCst);
        if let Some(s) = guard.take() {
            s.end();
        }
        // Closing the job ends the install and what it started; it is
        // retried at the next start.
        drop(
            self.extensions_job
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take(),
        );
    }

    /// `stop()`, then the start in progress, if any, waited for: it gives
    /// up within a few seconds and its server is ended. For the eject and
    /// the app's exit, off the main thread, before the kit's processes are
    /// swept: no server is started from the drive after that (BUG-V7-15).
    pub fn stop_and_wait(&self) {
        self.stop();
        drop(self.starting.lock().unwrap_or_else(|e| e.into_inner()));
    }

    /// A `stop()` came after `stops` was read.
    fn stopped(&self, stops: u64) -> bool {
        self.stops.load(Ordering::SeqCst) != stops
    }

    /// `stops`: the count when the start was asked for; when it changes the
    /// start gives up before its next step.
    fn start(&self, stops: u64) -> Result<Server> {
        if self.stopped(stops) {
            bail!("the editor was stopped while it started");
        }
        if !is_installed(&self.kit) {
            bail!("the editor is not installed");
        }
        let env = self.env();
        let system_root = PathBuf::from(env.get("SystemRoot").unwrap_or(r"C:\Windows"));
        match cfg::write(&self.kit, &system_root)? {
            cfg::Outcome::LeftAlone(why) => {
                crate::common::log::warn(&format!("editor settings left alone: {why}"))
            }
            other => crate::common::log::info(&format!("editor settings: {other:?}")),
        }
        if self.stopped(stops) {
            bail!("the editor was stopped while it started");
        }
        if let Err(e) = theme::ensure(&self.kit, &env) {
            crate::common::log::warn(&format!("editor theme: {e:#}"));
        }
        if self.stopped(stops) {
            bail!("the editor was stopped while it started");
        }
        let data_dir = self.kit.data().join("vscodium-server");
        fs::create_dir_all(&data_dir)?;
        let token = random_token()?;
        let token_file = data_dir.join("token");
        // A new file each start, owner-only before the token is in it: never
        // the old one rewritten, which another account's DACL (the kit
        // used on another PC) would refuse (PORT-V6-01, BUG-V6-33).
        crate::platform::files::replace_private(&token_file, &token)?;
        let log_path = self.kit.logs().join("editor.log");

        let mut last_error = None;
        // A port found free can be taken a moment later: try a few.
        for port in free_ports().take(3) {
            if self.stopped(stops) {
                bail!("the editor was stopped while it started");
            }
            match self.start_on(port, stops, &token, &token_file, &data_dir, &log_path) {
                Ok(server) => return Ok(server),
                Err(e) => {
                    crate::common::log::warn(&format!("editor on port {port}: {e:#}"));
                    last_error = Some(e);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow!("no free port in {PORTS:?}")))
    }

    fn start_on(
        &self,
        port: u16,
        stops: u64,
        token: &str,
        token_file: &Path,
        data_dir: &Path,
        log_path: &Path,
    ) -> Result<Server> {
        let dir = server_dir(&self.kit);
        let mut cmd = jobs::kit_command(
            crate::router::bins::editor_node(&self.kit, &dir),
            &self.env(),
        );
        cmd.arg(crate::router::bins::editor_server_main(&dir))
            .args(["--host", "127.0.0.1", "--port", &port.to_string()])
            .arg("--connection-token-file")
            .arg(token_file)
            .arg("--server-data-dir")
            .arg(data_dir)
            .args(["--accept-server-license-terms", "--telemetry-level", "off"])
            // Workspace trust stays on (D-67): the editor asks once per
            // project and opens the ones not trusted in Restricted Mode.
            .env("ATOLLO_APP", "1")
            .current_dir(self.kit.projects())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let job = Job::new()?;
        let mut child = jobs::spawn_in(&mut cmd, &job)?;
        append_log(log_path, &format!("--- start on 127.0.0.1:{port}"));
        if let Some(out) = child.stdout.take() {
            pump(out, log_path.to_path_buf(), token.to_owned());
        }
        if let Some(err) = child.stderr.take() {
            pump(err, log_path.to_path_buf(), token.to_owned());
        }

        let agent = local_agent();
        let started = Instant::now();
        loop {
            if self.stopped(stops) {
                // Ended and reaped here, as at the timeout: the job closing
                // with this function ends what it started.
                let _ = child.kill();
                let _ = child.wait();
                bail!("the editor was stopped while it started");
            }
            if let Ok(Some(status)) = child.try_wait() {
                bail!(
                    "the editor server stopped at start ({status}); see {}",
                    log_path.display()
                );
            }
            if agent
                .get(&format!("http://127.0.0.1:{port}/version"))
                .call()
                .is_ok()
            {
                return Ok(Server {
                    port,
                    token: token.to_owned(),
                    child,
                    _job: job,
                });
            }
            if started.elapsed() > START_TIMEOUT {
                let _ = child.kill();
                // Reaped here, on the start's own thread (BUG-V7-20).
                let _ = child.wait();
                bail!(
                    "the editor server did not answer within {} s; see {}",
                    START_TIMEOUT.as_secs(),
                    log_path.display()
                );
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        // At the app's exit: a start in progress, if any, ends first.
        self.stop_and_wait();
    }
}

/// HTTP client for the local server: no proxy, statuses are not errors.
fn local_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .proxy(None)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(2)))
        .build()
        .new_agent()
}

/// The editor's own range, bound on IPv4 only: unlike
/// [`crate::common::ports::is_port_free`], which also asks who answers and binds
/// IPv6; kept as it is, so the editor picks the same ports as before.
fn free_ports() -> impl Iterator<Item = u16> {
    PORTS.filter(|p| TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, *p)).is_ok())
}

pub fn random_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    #[cfg(windows)]
    // SAFETY: the buffer is valid for its length; system RNG, no algorithm handle.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .context("BCryptGenRandom")?;
    #[cfg(unix)]
    fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .context("/dev/urandom")?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

// restrict_to_owner: protected DACL with full access for the owner only
// (mode 600 on Unix); it fails on disks without permissions (FAT32/exFAT):
// the caller only logs it. restrict_dir_to_owner: the same for a folder,
// inherited by every file and folder created in it afterwards (temporary
// files included), so they are private from the start; on Unix the folder
// gets mode 700 and what is inside cannot be reached.
pub(crate) use crate::platform::files::{restrict_dir_to_owner, restrict_to_owner};

fn append_log(path: &Path, line: &str) {
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(f, "{line}");
    }
}

/// Copies the server's output to the log, masking the token.
fn pump(stream: impl Read + Send + 'static, log: PathBuf, token: String) {
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            append_log(&log, &mask_secrets(&line.replace(&token, "***")));
        }
    });
}

/// `http://127.0.0.1:<port>/?tkn=<token>&folder=/C:/path/to/project`
/// (`folder=/home/me/kit/projects/app` on macOS and Linux).
pub fn url(port: u16, token: &str, folder: Option<&Path>) -> String {
    url_on(Os::current(), port, token, folder)
}

/// [`url`] for a folder of `os`.
pub fn url_on(os: Os, port: u16, token: &str, folder: Option<&Path>) -> String {
    let mut url = format!("http://127.0.0.1:{port}/?tkn={token}");
    if let Some(folder) = folder {
        url.push_str("&folder=");
        url.push_str(&encode_query(&folder_param_on(os, folder)));
    }
    url
}

/// `folder` of `os` as the server's `folder=` wants it: on Windows forward
/// slashes after one more `/` (`/C:/x`, `///server/share/x`), as always; a
/// Unix path as it is, since it has its `/` already (`//home/...` would be
/// another path) and a `\` is a character of its names.
pub fn folder_param_on(os: Os, folder: &Path) -> String {
    let path = folder.display().to_string();
    if os.is_windows() {
        format!("/{}", path.replace('\\', "/"))
    } else if path.starts_with('/') {
        path
    } else {
        format!("/{path}")
    }
}

fn encode_query(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
#[path = "../../../test/rust/editor/mod.rs"]
mod tests;
