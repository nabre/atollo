//! Starting and stopping the projects' actions (port of nabre/dev-kit
//! `core/processes.js` and `core/panel/ports.js`).
//!
//! - every action runs in a job of its own, nested in the app's: stopping it
//!   ends the whole tree (npm → node → esbuild…), closing the app ends all;
//! - commands go through `cmd.exe /d /s /c` with the kit's environment plus
//!   `PORT`, `ROCKET_PORT`, `BROWSER=none`, `PYTHONUNBUFFERED=1`; a `static`
//!   action runs this exe as a file server (`Dev-Kit.exe serve`);
//! - web actions keep their port from one start to the next (bookmarks,
//!   cookies, localStorage stay valid): 7100-7999, remembered in
//!   `data\app\ports.json`, or the one the user locked;
//! - output: last 3000 lines per action, without ANSI colours; a web action
//!   is "running" once its port answers, or the port it prints if it ignores
//!   the one assigned.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::detect::{self, Action, Kind};
use crate::env::KitEnv;
use crate::jobs::{self, Job};
use crate::kit::{write_atomic, Kit};

const MAX_LINES: usize = 3000;
pub const PORT_FROM: u16 = 7100;
pub const PORT_TO: u16 = 7999;

static ANSI: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"\x1b\[[0-?]*[ -/]*[@-~]|\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)|\x1b[=>()][0-9A-Za-z]?")
        .ok()
});
static LOCAL_URL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"https?://(?:localhost|127\.0\.0\.1|0\.0\.0\.0|\[::1?\]):(\d{2,5})[^\s'"<>)]*"#)
        .ok()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Starting,
    Running,
    Finished,
    Error,
    Stopped,
}

/// What the interface sees of a started action.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInfo {
    pub key: String,
    pub project: String,
    pub action: String,
    pub label: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    pub command: String,
    pub port: Option<u16>,
    pub url: Option<String>,
    pub state: State,
    /// Milliseconds since 1970.
    pub start: u64,
    pub end: Option<u64>,
    pub code: Option<i32>,
    pub pid: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Event {
    State(RunInfo),
    Log { key: String, lines: Vec<String> },
}

pub type Notify = Arc<dyn Fn(Event) + Send + Sync>;

struct Entry {
    info: RunInfo,
    lines: Vec<String>,
    /// Dropped to stop: ends every process of the action.
    job: Option<Job>,
    stopped: bool,
    actual_port: Option<u16>,
    detected_url: Option<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ports {
    pub ports: BTreeMap<String, u16>,
    /// Ports chosen by the user: never moved to another one.
    pub locked: Vec<String>,
}

pub struct Runner {
    kit: Kit,
    entries: Mutex<HashMap<String, Entry>>,
    ports: Mutex<Ports>,
    /// Ports being probed for a start: two actions started together differ.
    pending: Mutex<HashSet<u16>>,
    notify: Notify,
}

pub fn key_of(project: &str, action: &str) -> String {
    format!("{project}|{action}")
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

fn ports_file(kit: &Kit) -> PathBuf {
    kit.app_dir().join("ports.json")
}

/// Something answers on the port (IPv4 or IPv6 loopback).
pub fn responds(port: u16) -> bool {
    let t = Duration::from_millis(400);
    TcpStream::connect_timeout(&SocketAddr::from((Ipv4Addr::LOCALHOST, port)), t).is_ok()
        || TcpStream::connect_timeout(&SocketAddr::from((Ipv6Addr::LOCALHOST, port)), t).is_ok()
}

fn can_listen(addr: SocketAddr) -> bool {
    match TcpListener::bind(addr) {
        Ok(_) => true,
        // No IPv6 on this PC: not a reason to skip the port.
        Err(e) => !matches!(
            e.kind(),
            std::io::ErrorKind::AddrInUse | std::io::ErrorKind::PermissionDenied
        ),
    }
}

pub fn is_port_free(port: u16) -> bool {
    !responds(port)
        && can_listen(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        && can_listen(SocketAddr::from((Ipv6Addr::LOCALHOST, port)))
}

/// Text without colours, split in lines (`\r\n`, `\n`, `\r`); the last,
/// incomplete piece stays in `rest`.
fn split_lines(rest: &mut String, chunk: &str) -> Vec<String> {
    rest.push_str(chunk);
    let mut lines: Vec<String> = rest.split(['\n', '\r']).map(str::to_owned).collect();
    *rest = lines.pop().unwrap_or_default();
    // "\r\n" leaves empty pieces between \r and \n: dropped.
    let had_crlf = chunk.contains("\r\n");
    if had_crlf {
        lines.retain(|l| !l.is_empty());
    }
    if rest.len() > 4000 {
        lines.push(std::mem::take(rest));
    }
    lines
        .into_iter()
        .map(|l| match ANSI.as_ref() {
            Some(re) => re.replace_all(&l, "").into_owned(),
            None => l,
        })
        .collect()
}

impl Runner {
    pub fn new(kit: Kit, notify: Notify) -> Runner {
        let ports = std::fs::read_to_string(ports_file(&kit))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Runner {
            kit,
            entries: Mutex::new(HashMap::new()),
            ports: Mutex::new(ports),
            pending: Mutex::new(HashSet::new()),
            notify,
        }
    }

    fn entries(&self) -> std::sync::MutexGuard<'_, HashMap<String, Entry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn list(&self) -> Vec<RunInfo> {
        let mut out: Vec<RunInfo> = self.entries().values().map(|e| e.info.clone()).collect();
        out.sort_by(|a, b| a.key.cmp(&b.key));
        out
    }

    pub fn log(&self, key: &str) -> Vec<String> {
        self.entries()
            .get(key)
            .map(|e| e.lines.clone())
            .unwrap_or_default()
    }

    pub fn ports(&self) -> Ports {
        self.ports
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn save_ports(&self, ports: &Ports) -> Result<()> {
        write_atomic(
            &ports_file(&self.kit),
            serde_json::to_string_pretty(ports)?.as_bytes(),
        )?;
        Ok(())
    }

    fn alive(&self, key: &str) -> bool {
        self.entries()
            .get(key)
            .is_some_and(|e| matches!(e.info.state, State::Starting | State::Running))
    }

    /// The port of a web action: the one it had (if free), or the first free
    /// one in the range. A locked port in use is an error, never moved.
    pub fn assign_port(&self, key: &str, lang: &str) -> Result<u16> {
        let ports = self.ports();
        if let Some(&p) = ports.ports.get(key) {
            if is_port_free(p) {
                return Ok(p);
            }
            if ports.locked.iter().any(|k| k == key) {
                bail!(
                    "{}",
                    tr(lang, "La porta {0} è occupata da un altro programma. Chiudilo o cambia la porta.", &[&p.to_string()])
                );
            }
        }
        let mut used: HashSet<u16> = ports.ports.values().copied().collect();
        used.extend(self.entries().values().filter_map(|e| e.info.port));
        for n in PORT_FROM..=PORT_TO {
            if used.contains(&n) {
                continue;
            }
            if !self
                .pending
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .insert(n)
            {
                continue;
            }
            let free = is_port_free(n);
            if free {
                let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
                ports.ports.insert(key.to_owned(), n);
                let snapshot = ports.clone();
                drop(ports);
                if let Err(e) = self.save_ports(&snapshot) {
                    crate::log::warn(&format!("ports.json: {e:#}"));
                }
            }
            self.pending
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&n);
            if free {
                return Ok(n);
            }
        }
        bail!(
            "{}",
            tr(
                lang,
                "Nessuna porta libera tra {0} e {1}",
                &[&PORT_FROM.to_string(), &PORT_TO.to_string()]
            )
        )
    }

    /// The user's port for an action (locked), or `None` to go back to an
    /// automatic one.
    pub fn set_port(&self, key: &str, port: Option<u16>, lang: &str) -> Result<Ports> {
        if !key.contains('|') {
            bail!("invalid key");
        }
        let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
        ports.locked.retain(|k| k != key);
        match port {
            None => {
                ports.ports.remove(key);
            }
            Some(n) => {
                if n < 1024 {
                    bail!("{}", tr(lang, "Porta tra {0} e {1}", &["1024", "65535"]));
                }
                if crate::editor::PORTS.contains(&n) {
                    let (from, to) = (crate::editor::PORTS.start(), crate::editor::PORTS.end());
                    bail!(
                        "{}",
                        tr(
                            lang,
                            "Le porte da {0} a {1} sono dell'editor",
                            &[&from.to_string(), &to.to_string()]
                        )
                    );
                }
                if let Some((other, _)) = ports.ports.iter().find(|(k, v)| **v == n && *k != key) {
                    bail!(
                        "{}",
                        tr(
                            lang,
                            "Porta già assegnata a {0}",
                            &[&other.replace('|', " → ")]
                        )
                    );
                }
                ports.ports.insert(key.to_owned(), n);
                ports.locked.push(key.to_owned());
            }
        }
        let snapshot = ports.clone();
        drop(ports);
        self.save_ports(&snapshot)?;
        Ok(snapshot)
    }

    /// Starts `action` of `project` (folder `dir`). Already running: the
    /// same process, not a second one.
    pub fn start(
        self: &Arc<Self>,
        id: &str,
        dir: &Path,
        action: &Action,
        env: &KitEnv,
        lang: &str,
    ) -> Result<RunInfo> {
        let key = key_of(id, &action.id);
        if self.alive(&key) {
            return self
                .entries()
                .get(&key)
                .map(|e| e.info.clone())
                .ok_or_else(|| anyhow!("gone"));
        }
        let port = if action.kind == Kind::Web {
            Some(self.assign_port(&key, lang)?)
        } else {
            None
        };
        let fill = |s: &str| s.replace("{port}", &port.map(|p| p.to_string()).unwrap_or_default());
        // Settings > "Permetti ai server dei progetti..." (DEVKIT_ALLOW_LAN).
        let lan = env.get("DEVKIT_ALLOW_LAN") == Some("1");
        let listen = |s: &str| on_network(fill(s), lan);
        let cwd = match &action.folder {
            Some(f) => detect::inside_dir(dir, f)?,
            None => dir.to_path_buf(),
        };

        let (mut cmd, shown) = if let Some(folder) = &action.static_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            let exe = std::env::current_exe()?;
            let mut cmd = jobs::kit_command(&exe, env);
            cmd.arg("serve")
                .arg(&root)
                .arg(port.unwrap_or(PORT_FROM).to_string());
            (
                cmd,
                format!("Dev-Kit.exe serve {folder} {}", port.unwrap_or_default()),
            )
        } else {
            let command = listen(&action.command);
            let system_root = env.get("SystemRoot").unwrap_or(r"C:\Windows");
            let mut cmd = jobs::kit_command(Path::new(system_root).join(r"System32\cmd.exe"), env);
            cmd.raw_arg(format!("/d /s /c \"{command}\""));
            (cmd, command)
        };
        cmd.current_dir(&cwd)
            .env("BROWSER", "none")
            .env("PYTHONUNBUFFERED", "1")
            .env("DEVKIT_LANG", lang)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(p) = port {
            cmd.env("PORT", p.to_string())
                .env("ROCKET_PORT", p.to_string());
        }
        for (k, v) in &action.env {
            cmd.env(k, listen(v));
        }

        let url = (action.kind == Kind::Web)
            .then(|| fill(action.url.as_deref().unwrap_or("http://localhost:{port}/")));
        let mut info = RunInfo {
            key: key.clone(),
            project: id.to_owned(),
            action: action.id.clone(),
            label: action.label.clone(),
            kind: action.kind,
            command: shown.clone(),
            port,
            url,
            state: if action.kind == Kind::Web {
                State::Starting
            } else {
                State::Running
            },
            start: now_ms(),
            end: None,
            code: None,
            pid: None,
        };
        let mut header = vec![format!("> {shown}")];
        header.push(match port {
            Some(p) => format!(
                "  {}   {}",
                tr(lang, "cartella: {0}", &[&cwd.display().to_string()]),
                tr(lang, "porta: {0}", &[&p.to_string()])
            ),
            None => format!(
                "  {}",
                tr(lang, "cartella: {0}", &[&cwd.display().to_string()])
            ),
        });
        header.push(String::new());

        let job = Job::new()?;
        let spawned = jobs::spawn_in(&mut cmd, &job);
        let mut child: Child = match spawned {
            Ok(c) => c,
            Err(e) => {
                info.state = State::Error;
                info.end = Some(now_ms());
                header.push(format!(
                    "[!] {}",
                    tr(lang, "Avvio non riuscito: {0}", &[&format!("{e:#}")])
                ));
                self.entries().insert(
                    key.clone(),
                    Entry {
                        info: info.clone(),
                        lines: header.clone(),
                        job: None,
                        stopped: false,
                        actual_port: None,
                        detected_url: None,
                    },
                );
                (self.notify)(Event::Log {
                    key: key.clone(),
                    lines: header,
                });
                (self.notify)(Event::State(info.clone()));
                return Ok(info);
            }
        };
        info.pid = Some(child.id());
        self.entries().insert(
            key.clone(),
            Entry {
                info: info.clone(),
                lines: Vec::new(),
                job: Some(job),
                stopped: false,
                actual_port: None,
                detected_url: None,
            },
        );
        self.append(&key, header, false);
        (self.notify)(Event::State(info.clone()));

        let mut readers = Vec::new();
        for stream in [
            child
                .stdout
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
            child
                .stderr
                .take()
                .map(|s| Box::new(s) as Box<dyn Read + Send>),
        ]
        .into_iter()
        .flatten()
        {
            let (me, key) = (Arc::clone(self), key.clone());
            readers.push(std::thread::spawn(move || me.read_stream(&key, stream)));
        }
        let (me, key2, lang2) = (Arc::clone(self), key.clone(), lang.to_owned());
        std::thread::spawn(move || {
            let code = child.wait().ok().and_then(|s| s.code());
            for r in readers {
                let _ = r.join();
            }
            me.finish(&key2, code, &lang2);
        });
        if action.kind == Kind::Web {
            let (me, key3, lang3) = (Arc::clone(self), key.clone(), lang.to_owned());
            std::thread::spawn(move || me.watch_port(&key3, &lang3));
        }
        Ok(info)
    }

    fn read_stream(&self, key: &str, mut stream: Box<dyn Read + Send>) {
        let mut buf = vec![0u8; 16 * 1024];
        let mut pending: Vec<u8> = Vec::new();
        let mut rest = String::new();
        loop {
            let n = match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => n,
            };
            pending.extend_from_slice(&buf[..n]);
            // Keep an incomplete UTF-8 sequence for the next read.
            let valid = match std::str::from_utf8(&pending) {
                Ok(_) => pending.len(),
                Err(e) if e.error_len().is_none() => e.valid_up_to(),
                Err(_) => pending.len(),
            };
            let text = String::from_utf8_lossy(&pending[..valid]).into_owned();
            pending.drain(..valid);
            let lines = split_lines(&mut rest, &text);
            if !lines.is_empty() {
                self.append(key, lines, true);
            }
        }
        let tail = String::from_utf8_lossy(&pending).into_owned() + &rest;
        let tail = tail.trim_end_matches(['\r', '\n']).to_owned();
        if !tail.is_empty() {
            self.append(key, vec![tail], true);
        }
    }

    /// `scan`: look for the address a web server prints (not in the app's
    /// own lines, which may hold one in the command).
    fn append(&self, key: &str, lines: Vec<String>, scan: bool) {
        {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            e.lines.extend(lines.iter().cloned());
            if e.lines.len() > MAX_LINES {
                let extra = e.lines.len() - MAX_LINES;
                e.lines.drain(..extra);
            }
            if scan
                && e.info.kind == Kind::Web
                && e.info.state == State::Starting
                && e.actual_port.is_none()
            {
                if let Some(re) = LOCAL_URL.as_ref() {
                    for line in &lines {
                        let Some(c) = re.captures(line) else { continue };
                        let Some(p) = c.get(1).and_then(|m| m.as_str().parse::<u16>().ok()) else {
                            continue;
                        };
                        if Some(p) == e.info.port {
                            continue;
                        }
                        e.actual_port = Some(p);
                        let url = c[0].trim_end_matches(['.', ',', ';', ':', '!', '?']);
                        let url = ["0.0.0.0", "[::1]", "[::]", "127.0.0.1"]
                            .iter()
                            .fold(url.to_owned(), |u, h| u.replacen(h, "localhost", 1));
                        e.detected_url = Some(url);
                        break;
                    }
                }
            }
        }
        (self.notify)(Event::Log {
            key: key.to_owned(),
            lines,
        });
    }

    fn watch_port(&self, key: &str, lang: &str) {
        loop {
            std::thread::sleep(Duration::from_millis(700));
            let (state, port, actual) = match self.entries().get(key) {
                Some(e) => (e.info.state, e.info.port, e.actual_port),
                None => return,
            };
            if state != State::Starting {
                return;
            }
            if port.is_some_and(responds) {
                self.ready(key, None, lang);
                return;
            }
            if let Some(a) = actual.filter(|a| responds(*a)) {
                self.ready(key, Some(a), lang);
                return;
            }
        }
    }

    fn ready(&self, key: &str, actual: Option<u16>, lang: &str) {
        let (info, line) = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            if e.info.state != State::Starting {
                return;
            }
            let mut line = Vec::new();
            if let Some(a) = actual {
                line.push(format!(
                    "[Dev-Kit] {}",
                    tr(
                        lang,
                        "il server usa la porta {0} invece della {1}",
                        &[&a.to_string(), &e.info.port.unwrap_or_default().to_string()]
                    )
                ));
                e.info.url = e.detected_url.clone();
            }
            e.info.state = State::Running;
            line.push(format!(
                "[Dev-Kit] {}",
                tr(
                    lang,
                    "server pronto: {0}",
                    &[e.info.url.as_deref().unwrap_or_default()]
                )
            ));
            (e.info.clone(), line)
        };
        self.append(key, line, false);
        (self.notify)(Event::State(info));
    }

    fn finish(&self, key: &str, code: Option<i32>, lang: &str) {
        let info = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            e.job = None;
            e.info.code = code;
            e.info.end = Some(now_ms());
            e.info.state = if e.stopped {
                State::Stopped
            } else if code == Some(0) {
                State::Finished
            } else {
                State::Error
            };
            e.info.clone()
        };
        let line = match code {
            Some(c) => tr(lang, "processo terminato, codice {0}", &[&c.to_string()]),
            None => tr(lang, "processo terminato", &[]),
        };
        self.append(key, vec![format!("[{line}]")], false);
        (self.notify)(Event::State(info));
    }

    /// Stops an action and everything it started.
    pub fn stop(&self, key: &str) {
        let job = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            e.stopped = true;
            e.job.take()
        };
        // Closing the job's last handle ends its processes (KILL_ON_JOB_CLOSE).
        drop(job);
    }

    pub fn stop_all(&self) {
        let jobs: Vec<Job> = self
            .entries()
            .values_mut()
            .filter_map(|e| {
                e.stopped = true;
                e.job.take()
            })
            .collect();
        drop(jobs);
    }

    /// Actions still running (for the close confirmation).
    pub fn running(&self) -> usize {
        self.entries()
            .values()
            .filter(|e| matches!(e.info.state, State::Starting | State::Running))
            .count()
    }

    /// Forgets the output of an action that ended.
    pub fn clear(&self, key: &str) {
        let mut entries = self.entries();
        if entries.get(key).is_some_and(|e| e.job.is_none()) {
            entries.remove(key);
        }
    }
}

fn tr(lang: &str, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang, source, args)
}

/// A command (or a variable) found by the app listens on 127.0.0.1; with
/// the local network allowed, on every interface.
fn on_network(s: String, lan: bool) -> String {
    if lan {
        s.replace("127.0.0.1", "0.0.0.0")
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::{tests::TempDir, Arch};
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn local_network_setting() {
        let cmd = "php -S 127.0.0.1:7101 -t public".to_owned();
        assert_eq!(on_network(cmd.clone(), false), cmd);
        assert_eq!(on_network(cmd, true), "php -S 0.0.0.0:7101 -t public");
    }

    #[test]
    fn lines_and_colours() {
        let mut rest = String::new();
        assert_eq!(split_lines(&mut rest, "a\r\nb"), ["a"]);
        assert_eq!(rest, "b");
        assert_eq!(
            split_lines(&mut rest, "c\n\x1b[32mok\x1b[0m\n"),
            ["bc", "ok"]
        );
        assert_eq!(split_lines(&mut rest, "50%\r60%\r"), ["50%", "60%"]);
        assert!(rest.is_empty());
    }

    #[test]
    fn ports_are_kept_and_locked() {
        let tmp = TempDir::new("run-ports");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let r = Runner::new(kit.clone(), Arc::new(|_| {}));
        let a = r.assign_port("p|a", "it").unwrap();
        let b = r.assign_port("p|b", "it").unwrap();
        assert_ne!(a, b);
        assert_eq!(r.assign_port("p|a", "it").unwrap(), a);
        // Kept across restarts.
        let r2 = Runner::new(kit, Arc::new(|_| {}));
        assert_eq!(r2.ports().ports["p|b"], b);
        assert!(r2.set_port("p|c", Some(a), "it").is_err());
        assert!(r2.set_port("p|c", Some(80), "it").is_err());
        assert!(
            r2.set_port("p|c", Some(7050), "it").is_err(),
            "the editor's"
        );
        assert!(r2.set_port("nokey", Some(9000), "it").is_err());
        let p = r2.set_port("p|c", Some(PORT_TO), "it").unwrap();
        assert_eq!(p.locked, ["p|c"]);
        // A locked port in use is an error, not another port.
        let busy = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let busy_port = busy.local_addr().unwrap().port();
        r2.set_port("p|d", Some(busy_port.max(1024)), "it").unwrap();
        if busy_port >= 1024 {
            assert!(r2.assign_port("p|d", "it").is_err());
        }
        r2.set_port("p|c", None, "it").unwrap();
        assert!(!r2.ports().ports.contains_key("p|c"));
    }

    #[test]
    fn runs_and_stops_commands() {
        let tmp = TempDir::new("run-cmd");
        let kit = Kit::at(tmp.0.clone(), Arch::X64);
        kit.prepare().unwrap();
        let proj = kit.projects().join("demo");
        std::fs::create_dir_all(&proj).unwrap();
        let events = Arc::new(AtomicUsize::new(0));
        let ev = Arc::clone(&events);
        let kit2 = kit.clone();
        let r = Arc::new(Runner::new(
            kit,
            Arc::new(move |_| {
                ev.fetch_add(1, Ordering::Relaxed);
            }),
        ));
        let env = KitEnv::build(
            &kit2,
            &crate::settings::Settings::default(),
            "it",
            &crate::env::Host::capture(),
        );
        let action = Action::for_test("echo", "echo %PORT% hello & exit /b 3", Kind::Command);
        let info = r.start("demo", &proj, &action, &env, "it").unwrap();
        assert_eq!(info.state, State::Running);
        let key = info.key.clone();
        let start = std::time::Instant::now();
        while r.list()[0].state == State::Running {
            assert!(start.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(50));
        }
        let done = r.list().into_iter().find(|i| i.key == key).unwrap();
        assert_eq!(done.state, State::Error);
        assert_eq!(done.code, Some(3));
        assert!(
            r.log(&key).iter().any(|l| l.contains("hello")),
            "{:?}",
            r.log(&key)
        );
        assert!(events.load(Ordering::Relaxed) >= 3);

        // A server that never ends: stopped through its job.
        let long = Action::for_test("wait", "ping -n 60 127.0.0.1", Kind::Command);
        let info = r.start("demo", &proj, &long, &env, "it").unwrap();
        assert_eq!(r.running(), 1);
        r.stop(&info.key);
        let start = std::time::Instant::now();
        while r.running() > 0 {
            assert!(start.elapsed() < Duration::from_secs(10));
            std::thread::sleep(Duration::from_millis(50));
        }
        assert_eq!(
            r.list().iter().find(|i| i.key == info.key).unwrap().state,
            State::Stopped
        );
    }
}
