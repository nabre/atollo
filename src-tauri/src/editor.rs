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

use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use windows::core::HSTRING;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Authorization::{
    ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};
use windows::Win32::Security::{SetFileSecurityW, DACL_SECURITY_INFORMATION, PSECURITY_DESCRIPTOR};

use crate::editor_cfg;
use crate::env::{mask_secrets, KitEnv};
use crate::jobs::{self, Job};
use crate::kit::Kit;

pub const PORTS: std::ops::RangeInclusive<u16> = 7000..=7099;
const START_TIMEOUT: Duration = Duration::from_secs(30);

struct Server {
    port: u16,
    token: String,
    child: Child,
    // Dropped last: ends the server and everything it started.
    _job: Job,
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
    pub extensions: crate::editor_ext::ExtState,
}

pub struct Editor {
    kit: Kit,
    env: Mutex<KitEnv>,
    server: Mutex<Option<Server>>,
    extensions: std::sync::Arc<Mutex<crate::editor_ext::ExtState>>,
}

/// Where the server program is.
pub fn server_dir(kit: &Kit) -> PathBuf {
    kit.apps().join("vscodium-server")
}

/// The files started (`out\server-main.js`, `node.exe`) are there: the
/// catalog's key and keys of `vscodium-server`.
pub fn is_installed(kit: &Kit) -> bool {
    crate::catalog::Catalog::get()
        .find("vscodium-server")
        .is_some_and(|c| c.is_installed(kit))
}

impl Editor {
    pub fn new(kit: Kit, env: KitEnv) -> Editor {
        Editor {
            kit,
            env: Mutex::new(env),
            server: Mutex::new(None),
            extensions: std::sync::Arc::default(),
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
            emulated: self.kit.arch() == crate::kit::Arch::Arm64,
            extensions: self
                .extensions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone(),
        }
    }

    /// The server's base URL and token, starting it when needed.
    pub fn ensure_running(&self) -> Result<(u16, String)> {
        let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(s) = guard.as_mut() {
            if matches!(s.child.try_wait(), Ok(None)) {
                return Ok((s.port, s.token.clone()));
            }
        }
        *guard = None;
        let server = self.start()?;
        let out = (server.port, server.token.clone());
        *guard = Some(server);
        let settings = crate::settings::Settings::load(&self.kit).settings;
        let net = crate::net::Net::new(&settings, None)
            .ok()
            .map(std::sync::Arc::new);
        crate::editor_ext::sync_in_background(
            self.kit.clone(),
            self.env().clone(),
            net,
            std::sync::Arc::clone(&self.extensions),
        );
        Ok(out)
    }

    pub fn stop(&self) {
        let mut guard = self.server.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(mut s) = guard.take() {
            let _ = s.child.kill();
        }
    }

    fn start(&self) -> Result<Server> {
        if !is_installed(&self.kit) {
            bail!("the editor is not installed");
        }
        let env = self.env();
        let system_root = PathBuf::from(env.get("SystemRoot").unwrap_or(r"C:\Windows"));
        match editor_cfg::write(&self.kit, &system_root)? {
            editor_cfg::Outcome::LeftAlone(why) => {
                crate::log::warn(&format!("editor settings left alone: {why}"))
            }
            other => crate::log::info(&format!("editor settings: {other:?}")),
        }
        let data_dir = self.kit.data().join("vscodium-server");
        fs::create_dir_all(&data_dir)?;
        let token = random_token()?;
        let token_file = data_dir.join("token");
        fs::write(&token_file, &token).with_context(|| token_file.display().to_string())?;
        if let Err(e) = restrict_to_owner(&token_file) {
            crate::log::warn(&format!("token file permissions: {e:#}"));
        }
        let log_path = self.kit.logs().join("editor.log");

        let mut last_error = None;
        // A port found free can be taken a moment later: try a few.
        for port in free_ports().take(3) {
            match self.start_on(port, &token, &token_file, &data_dir, &log_path) {
                Ok(server) => return Ok(server),
                Err(e) => {
                    crate::log::warn(&format!("editor on port {port}: {e:#}"));
                    last_error = Some(e);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| anyhow!("no free port in {PORTS:?}")))
    }

    fn start_on(
        &self,
        port: u16,
        token: &str,
        token_file: &Path,
        data_dir: &Path,
        log_path: &Path,
    ) -> Result<Server> {
        let dir = server_dir(&self.kit);
        let mut cmd = jobs::kit_command(dir.join("node.exe"), &self.env());
        cmd.arg(dir.join(r"out\server-main.js"))
            .args(["--host", "127.0.0.1", "--port", &port.to_string()])
            .arg("--connection-token-file")
            .arg(token_file)
            .arg("--server-data-dir")
            .arg(data_dir)
            .args(["--accept-server-license-terms", "--telemetry-level", "off"])
            // Workspace trust stays on (D-67): the editor asks once per
            // project and opens the ones not trusted in Restricted Mode.
            .env("DEVKIT_APP", "1")
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
        self.stop();
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

fn free_ports() -> impl Iterator<Item = u16> {
    PORTS.filter(|p| TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, *p)).is_ok())
}

pub fn random_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    // SAFETY: the buffer is valid for its length; system RNG, no algorithm handle.
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .context("BCryptGenRandom")?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Protected DACL with full access for the owner only. Fails on disks
/// without permissions (FAT32/exFAT): the caller only logs it.
fn restrict_to_owner(path: &Path) -> Result<()> {
    let sddl = HSTRING::from("D:P(A;;FA;;;OW)");
    let mut sd = PSECURITY_DESCRIPTOR::default();
    // SAFETY: valid SDDL string; `sd` receives a LocalAlloc'd descriptor freed below.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(&sddl, SDDL_REVISION_1, &mut sd, None)
    }
    .context("security descriptor")?;
    let file = HSTRING::from(path.as_os_str());
    // SAFETY: `sd` is a valid descriptor for the duration of the call.
    let result = unsafe { SetFileSecurityW(&file, DACL_SECURITY_INFORMATION, sd) };
    // SAFETY: frees the descriptor allocated above, once.
    unsafe { LocalFree(Some(HLOCAL(sd.0))) };
    result.ok().context("SetFileSecurityW")
}

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

/// `http://127.0.0.1:<port>/?tkn=<token>&folder=/C:/path/to/project`.
pub fn url(port: u16, token: &str, folder: Option<&Path>) -> String {
    let mut url = format!("http://127.0.0.1:{port}/?tkn={token}");
    if let Some(folder) = folder {
        let path = format!("/{}", folder.display().to_string().replace('\\', "/"));
        url.push_str("&folder=");
        url.push_str(&encode_query(&path));
    }
    url
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
mod tests {
    use super::*;

    #[test]
    fn url_encodes_odd_folders() {
        let u = url(
            7001,
            "ab",
            Some(Path::new(r"E:\Dev Kit (1) & co\projects\già")),
        );
        assert_eq!(u, "http://127.0.0.1:7001/?tkn=ab&folder=/E:/Dev%20Kit%20%281%29%20%26%20co/projects/gi%C3%A0");
        assert_eq!(url(7001, "ab", None), "http://127.0.0.1:7001/?tkn=ab");
    }

    #[test]
    fn tokens_are_random_hex() {
        let a = random_token().unwrap();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, random_token().unwrap());
    }

    /// Processes whose program lives under `dir`.
    fn processes_under(dir: &Path) -> Vec<u32> {
        use windows::Win32::System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
            TH32CS_SNAPPROCESS,
        };
        use windows::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        };
        let dir = dir.to_string_lossy().to_lowercase();
        let mut out = Vec::new();
        unsafe {
            let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0).unwrap();
            let mut e = PROCESSENTRY32W {
                dwSize: size_of::<PROCESSENTRY32W>() as u32,
                ..Default::default()
            };
            let mut ok = Process32FirstW(snap, &mut e).is_ok();
            while ok {
                if let Ok(h) =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, e.th32ProcessID)
                {
                    let mut buf = [0u16; 1024];
                    let mut len = buf.len() as u32;
                    if QueryFullProcessImageNameW(
                        h,
                        PROCESS_NAME_WIN32,
                        windows::core::PWSTR(buf.as_mut_ptr()),
                        &mut len,
                    )
                    .is_ok()
                    {
                        let path = String::from_utf16_lossy(&buf[..len as usize]).to_lowercase();
                        if path.starts_with(&dir) {
                            out.push(e.th32ProcessID);
                        }
                    }
                    let _ = windows::Win32::Foundation::CloseHandle(h);
                }
                ok = Process32NextW(snap, &mut e).is_ok();
            }
            let _ = windows::Win32::Foundation::CloseHandle(snap);
        }
        out
    }

    /// The real server (prompt, "editor.rs" test): needs a kit with the
    /// editor installed, given by DEVKIT_TEST_KIT (CI runs it after prepare).
    #[test]
    fn real_server_token_redirect_and_stop() {
        let Ok(root) = std::env::var("DEVKIT_TEST_KIT") else {
            eprintln!("DEVKIT_TEST_KIT not set: skipped");
            return;
        };
        let kit = Kit::at(PathBuf::from(root), crate::kit::Arch::detect());
        kit.prepare().unwrap();
        crate::kit_files::write_all(&kit).unwrap();
        let env = KitEnv::build(
            &kit,
            &crate::settings::Settings::default(),
            "en",
            &crate::env::Host::capture(),
        );
        let editor = Editor::new(kit.clone(), env);
        let (port, token) = editor.ensure_running().unwrap();
        assert!(PORTS.contains(&port));
        let agent = local_agent();
        let no_redirect = ureq::Agent::config_builder()
            .proxy(None)
            .http_status_as_error(false)
            .max_redirects(0)
            .build()
            .new_agent();
        let denied = agent
            .get(&format!("http://127.0.0.1:{port}/"))
            .call()
            .unwrap();
        assert_eq!(denied.status().as_u16(), 403, "no token: refused");
        let ok = no_redirect
            .get(&url(port, &token, Some(&kit.projects())))
            .call()
            .unwrap();
        assert!(
            ok.status().is_redirection(),
            "token: redirect, got {}",
            ok.status()
        );
        assert!(
            ok.headers()
                .get_all("set-cookie")
                .iter()
                .any(|c| c.to_str().unwrap_or("").contains(&token)),
            "cookie set"
        );
        // The workbench page, with the cookie the redirect set.
        let page = no_redirect
            .get(&format!("http://127.0.0.1:{port}/"))
            .header("Cookie", format!("vscode-tkn={token}"))
            .call()
            .unwrap()
            .body_mut()
            .read_to_string()
            .unwrap()
            .replace("&quot;", "\"");
        assert!(
            page.contains("\"enableWorkspaceTrust\":true"),
            "workspace trust on (D-67)"
        );
        assert!(
            !fs::read_to_string(kit.logs().join("editor.log"))
                .unwrap()
                .contains(&token),
            "token masked in log"
        );
        assert!(!processes_under(&server_dir(&kit)).is_empty());
        editor.stop();
        drop(editor);
        let start = Instant::now();
        while !processes_under(&server_dir(&kit)).is_empty() {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "server processes left"
            );
            std::thread::sleep(Duration::from_millis(200));
        }
        assert!(
            agent
                .get(&format!("http://127.0.0.1:{port}/"))
                .call()
                .is_err(),
            "port closed"
        );
    }

    #[test]
    fn ports_in_range() {
        let p = free_ports().next().unwrap();
        assert!(PORTS.contains(&p));
    }
}
