//! Starting and stopping the projects' actions (port of nabre/dev-kit
//! `core/processes.js` and `core/panel/ports.js`).
//!
//! - every action runs in a job of its own, nested in the app's: stopping it
//!   ends the whole tree (npm → node → esbuild…), closing the app ends all;
//! - commands go through `cmd.exe /d /s /c` (`/bin/sh -c` on macOS and
//!   Linux, `jobs::shell_command`) with the kit's environment plus
//!   `PORT`, `ROCKET_PORT`, `DB_PORT` (its MySQL database), `BROWSER=none`, `PYTHONUNBUFFERED=1`; a `static`
//!   action runs this exe as a file server (`Dev-Kit.exe serve`), an
//!   `apache` action the kit's httpd with a configuration of the project,
//!   `caddy` actions the same with Caddy and PHP;
//! - web actions keep their port from one start to the next (bookmarks,
//!   cookies, localStorage stay valid): 7100-7999, remembered in
//!   `data\projects\<project>\ports.json`, or the one the user locked;
//! - output: last 3000 lines per action, without ANSI colours; a web action
//!   is "running" once its port answers, or the port it prints if it ignores
//!   the one assigned.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Read;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::detect::{self, Action, Kind};
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::state::{self, ProjectState};

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
    /// URLs of the site's aliases that were left out of its configuration.
    pub skipped_aliases: Vec<String>,
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
    /// Served by this shared pool (`servers/mod.rs`): no process of its own.
    pool: Option<String>,
}

/// The `server` of a site in `panel.json` that keeps its own httpd/caddy
/// instead of a pool; never the id of a server profile.
pub const DEDICATED_SERVER: &str = "dedicated";

/// How «Avvia» serves an Apache or Caddy site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiteMode {
    /// Its own httpd/caddy on the action's port: any other action, or a
    /// site whose `server` is [`DEDICATED_SERVER`].
    Dedicated,
    /// The automatic pool of the project's lines (engine, server line, PHP
    /// line), started or reloaded (D-101): no `server`, or `auto`.
    Auto,
    /// The shared pool chosen in `panel.json` (`server`).
    Pool(String),
}

/// A site assigned to a server profile is served by that pool, one with no
/// `server` (or `auto`) by the automatic pool of its lines, one marked
/// `dedicated` (and every action that is not an Apache/Caddy site) on its own.
pub fn site_mode(action: &Action) -> SiteMode {
    if action.web_server_dir().is_none() {
        return SiteMode::Dedicated;
    }
    match action
        .server
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "auto")
    {
        Some(DEDICATED_SERVER) => SiteMode::Dedicated,
        Some(id) => SiteMode::Pool(id.to_owned()),
        None => SiteMode::Auto,
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ports {
    pub ports: BTreeMap<String, u16>,
    /// Ports chosen by the user: never moved to another one.
    pub locked: Vec<String>,
}

/// `ports.json` of one project (`data\projects\<project>\`): its actions'
/// ports, by action id, and those locked. Keys of a newer version stay.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct ProjectPorts {
    ports: BTreeMap<String, u16>,
    locked: Vec<String>,
    #[serde(flatten)]
    other: serde_json::Map<String, serde_json::Value>,
}

pub struct Runner {
    kit: Kit,
    entries: Mutex<HashMap<String, Entry>>,
    ports: Mutex<Ports>,
    /// What the project files hold now, to write only those that changed.
    saved: Mutex<BTreeMap<String, ProjectPorts>>,
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

/// The ports files of all projects, in memory under `project|action` keys,
/// and as they are on disk.
fn load_ports(kit: &Kit) -> (Ports, BTreeMap<String, ProjectPorts>) {
    let mut ports = Ports::default();
    let mut saved = BTreeMap::new();
    for project in state::all(kit) {
        let Some(file) = ProjectState::of(kit, &project).map(|s| s.ports_file()) else {
            continue;
        };
        let Some(p) = crate::fsx::read_json::<ProjectPorts>(&file) else {
            continue;
        };
        for (action, &port) in &p.ports {
            ports.ports.insert(key_of(&project, action), port);
        }
        ports
            .locked
            .extend(p.locked.iter().map(|a| key_of(&project, a)));
        saved.insert(project, p);
    }
    (ports, saved)
}

/// `project|action` keys split by project: `(project, action)`.
fn split_key(key: &str) -> Option<(&str, &str)> {
    key.split_once('|')
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
pub(crate) fn split_lines(rest: &mut String, chunk: &str) -> Vec<String> {
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
        let (ports, saved) = load_ports(&kit);
        Runner {
            kit,
            entries: Mutex::new(HashMap::new()),
            ports: Mutex::new(ports),
            saved: Mutex::new(saved),
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

    /// Writes the `ports.json` of the projects whose ports changed since the
    /// last save, each atomically; the others are not touched. The first
    /// failure is returned after trying them all.
    fn save_ports(&self) -> Result<()> {
        // `saved` is held while writing, so saves never overtake each other
        // and always write the ports as they are now.
        let mut saved = self.saved.lock().unwrap_or_else(PoisonError::into_inner);
        let ports = self.ports();
        let mut now: BTreeMap<String, ProjectPorts> = BTreeMap::new();
        for (key, &port) in &ports.ports {
            if let Some((project, action)) = split_key(key) {
                now.entry(project.to_owned())
                    .or_default()
                    .ports
                    .insert(action.to_owned(), port);
            }
        }
        for key in &ports.locked {
            if let Some((project, action)) = split_key(key) {
                now.entry(project.to_owned())
                    .or_default()
                    .locked
                    .push(action.to_owned());
            }
        }
        // a project that lost all its ports is written empty
        for project in saved.keys() {
            now.entry(project.clone()).or_default();
        }
        let mut result = Ok(());
        for (project, mut p) in now {
            let before = saved.get(&project);
            p.other = before.map(|b| b.other.clone()).unwrap_or_default();
            if before == Some(&p) || (before.is_none() && p == ProjectPorts::default()) {
                continue;
            }
            let Some(state) = ProjectState::of(&self.kit, &project) else {
                continue;
            };
            // nothing left to say and no folder: not created just for that
            if p.ports.is_empty() && p.locked.is_empty() && !state.dir().exists() {
                saved.remove(&project);
                continue;
            }
            match crate::fsx::write_json(&state.ports_file(), &p) {
                Ok(()) => {
                    saved.insert(project, p);
                }
                Err(e) => {
                    crate::log::warn(&format!("ports of {project}: {e:#}"));
                    if result.is_ok() {
                        result = Err(e);
                    }
                }
            }
        }
        result
    }

    /// A process of the action runs (one served by a pool has none: a
    /// dedicated start replaces it).
    fn alive(&self, key: &str) -> bool {
        self.entries().get(key).is_some_and(|e| {
            e.pool.is_none() && matches!(e.info.state, State::Starting | State::Running)
        })
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
                drop(ports);
                // a failure is already in the log
                let _ = self.save_ports();
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
        self.save_ports()?;
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

        let job = Job::new()?;
        // The FastCGI server of a project's Caddy (or of its Apache off
        // Windows), watched with the server.
        let mut php: Option<Child> = None;
        // The FastCGI server of a project's Caddy, kept until the action
        // ends: php-fpm's private socket folder goes with its last copy.
        let mut fcgi_kept: Option<crate::projects::fastcgi::FastCgi> = None;
        let (mut cmd, shown) = if let Some(folder) = &action.static_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // The AppImage, not its inner program: `kit_command` clears
            // APPIMAGE, and the image's mount is not this program's to keep.
            let exe = crate::kit::app_exe().ok_or_else(|| anyhow!("current exe"))?;
            let mut cmd = jobs::kit_command(&exe, env);
            cmd.arg("serve")
                .arg(&root)
                .arg(port.unwrap_or(PORT_FROM).to_string());
            (
                cmd,
                format!("Dev-Kit.exe serve {folder} {}", port.unwrap_or_default()),
            )
        } else if let Some(folder) = &action.apache_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // Off Windows PHP is php-fpm (D-116), in the action's job and
            // watched with the server like Caddy's; on Windows the module.
            let fcgi = if self.kit.os().is_windows() {
                None
            } else {
                let run_dir = crate::projects::apache::dir(&self.kit, id)?;
                crate::projects::fastcgi::spawn_fastcgi_if_present(
                    &self.kit,
                    env,
                    &job,
                    &run_dir,
                    &format!("project {id}"),
                )
                .map(|(fcgi, child)| {
                    php = Some(child);
                    fcgi
                })
            };
            fcgi_kept = fcgi.clone();
            let cmd = crate::projects::apache::command(
                &self.kit,
                id,
                dir,
                &root,
                port.unwrap_or(PORT_FROM),
                fcgi.as_ref(),
                lan,
                &action.domains,
                &action.aliases,
                env,
            )?;
            let shown = if self.kit.os().is_windows() {
                format!("httpd.exe -f data\\projects\\{id}\\apache\\httpd.conf ({folder})")
            } else {
                format!("httpd -f data/projects/{id}/apache/httpd.conf -DFOREGROUND ({folder})")
            };
            (cmd, shown)
        } else if let Some(folder) = &action.caddy_dir {
            let root = detect::inside_dir(&cwd, folder)?;
            // Its PHP (php-cgi, or php-fpm off Windows) lives in the action's
            // job too, watched with the server.
            let run_dir = crate::projects::caddy::dir(&self.kit, id)?;
            let fcgi = crate::projects::fastcgi::spawn_fastcgi_if_present(
                &self.kit,
                env,
                &job,
                &run_dir,
                &format!("project {id}"),
            )
            .map(|(fcgi, child)| {
                php = Some(child);
                fcgi
            });
            // php-fpm's socket folder lives as long as the action.
            fcgi_kept = fcgi.clone();
            let cmd = crate::projects::caddy::command(
                &self.kit,
                id,
                dir,
                &root,
                port.unwrap_or(PORT_FROM),
                fcgi.as_ref(),
                lan,
                &action.domains,
                &action.aliases,
                env,
            )?;
            (
                cmd,
                format!("caddy run --config data/projects/{id}/caddy/Caddyfile ({folder})"),
            )
        } else {
            let command = listen(&action.command);
            (jobs::shell_command(&command, env), command)
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
        // The port of the MySQL version its database is on
        // (D-102), unless a `.env` sets another one: dotenv loaders
        // keep a variable already there, and the user's own port would be
        // lost.
        if let Some(p) = crate::db::services::project_db_port(&self.kit, id) {
            let p = p.to_string();
            let own = [dir, cwd.as_path()]
                .into_iter()
                .filter_map(|d| dotenv_value(d, "DB_PORT"))
                .any(|v| v != p);
            if !own {
                cmd.env("DB_PORT", p);
                // `mysql` of its scripts logs in on that version too
                // (`DEVKIT_MYSQL_CNF`).
                for (name, value) in crate::db::services::project_client_env(&self.kit, id) {
                    cmd.env(name, value);
                }
            }
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
            skipped_aliases: if action.apache_dir.is_some() {
                crate::projects::alias::skipped(dir, &self.kit.projects(), &action.aliases)
            } else if action.caddy_dir.is_some() {
                crate::projects::caddy::skipped_aliases(dir, &self.kit.projects(), &action.aliases)
            } else {
                Vec::new()
            },
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
                        pool: None,
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
                pool: None,
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
            let _fcgi = fcgi_kept;
            let code = match php {
                Some(php) => me.wait_with_php(&key2, &mut child, php, &lang2),
                None => child.wait().ok().and_then(|s| s.code()),
            };
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

    /// `action` of `project` served by the running pool `pool` at `url`:
    /// shown as running with no process of its own. A dedicated server of
    /// the action still running is kept (and returned) instead.
    pub fn attach_pool(
        &self,
        project: &str,
        action: &Action,
        pool: &str,
        port: u16,
        url: &str,
        lang: &str,
    ) -> RunInfo {
        let key = key_of(project, &action.id);
        let info = {
            let mut entries = self.entries();
            if let Some(e) = entries.get(&key).filter(|e| {
                e.pool.is_none() && matches!(e.info.state, State::Starting | State::Running)
            }) {
                return e.info.clone();
            }
            let info = RunInfo {
                key: key.clone(),
                project: project.to_owned(),
                action: action.id.clone(),
                label: action.label.clone(),
                kind: action.kind,
                command: tr(lang, "pool condiviso {0}", &[pool]),
                port: Some(port),
                url: Some(format!("{}/", url.trim_end_matches('/'))),
                state: State::Running,
                start: now_ms(),
                end: None,
                code: None,
                pid: None,
                skipped_aliases: Vec::new(),
            };
            entries.insert(
                key.clone(),
                Entry {
                    info: info.clone(),
                    lines: Vec::new(),
                    job: None,
                    stopped: false,
                    actual_port: None,
                    detected_url: None,
                    pool: Some(pool.to_owned()),
                },
            );
            info
        };
        let line = format!(
            "[NEXKORA] {}",
            tr(
                lang,
                "servito dal pool condiviso {0}: {1}",
                &[pool, info.url.as_deref().unwrap_or_default()]
            )
        );
        self.append(&key, vec![line], false);
        (self.notify)(Event::State(info.clone()));
        info
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
    /// A line of the kit (`[NEXKORA] …`) in the log of the run `key`.
    pub fn note(&self, key: &str, text: &str) {
        self.append(key, vec![format!("[NEXKORA] {text}")], false);
    }

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
                    "[NEXKORA] {}",
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
                "[NEXKORA] {}",
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

    /// Waits for a project's web server (`child`) and its `php` (php-cgi or
    /// php-fpm). When PHP ends first the action is stopped and ends in error, never
    /// left running with 502 for every `.php` (BUG-V3-01). The server's exit
    /// code, `None` when it was stopped for its php-cgi.
    fn wait_with_php(
        &self,
        key: &str,
        child: &mut Child,
        mut php: Child,
        lang: &str,
    ) -> Option<i32> {
        loop {
            if let Ok(Some(status)) = child.try_wait() {
                return status.code();
            }
            if let Ok(Some(status)) = php.try_wait() {
                let job = {
                    let mut entries = self.entries();
                    entries
                        .get_mut(key)
                        .filter(|e| !e.stopped)
                        .and_then(|e| e.job.take())
                };
                // Stopped by the user: the job already ended both.
                let Some(job) = job else {
                    return child.wait().ok().and_then(|s| s.code());
                };
                let code = status
                    .code()
                    .map_or_else(|| "?".to_owned(), |c| c.to_string());
                self.append(
                    key,
                    vec![format!(
                        "[!] {}",
                        tr(
                            lang,
                            "php-cgi terminato (codice {0}): server fermato",
                            &[&code]
                        )
                    )],
                    false,
                );
                // Closing the job's last handle ends the server too.
                drop(job);
                let _ = child.wait();
                return None;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
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

    /// The pool serving the running action `key`, with its project and
    /// action ids (`None`: not served by a pool, or not running).
    pub fn pooled(&self, key: &str) -> Option<(String, String, String)> {
        let entries = self.entries();
        let e = entries.get(key)?;
        let pool = e.pool.clone().filter(|_| e.info.state == State::Running)?;
        Some((pool, e.info.project.clone(), e.info.action.clone()))
    }

    /// Stops an action and everything it started. An action served by a
    /// shared pool is only marked stopped here: the caller takes its site
    /// out of the pool (`WebServers::detach`).
    pub fn stop(&self, key: &str) {
        let (job, unpooled) = {
            let mut entries = self.entries();
            let Some(e) = entries.get_mut(key) else {
                return;
            };
            e.stopped = true;
            let unpooled = (e.pool.is_some() && e.info.state == State::Running).then(|| {
                e.info.state = State::Stopped;
                e.info.end = Some(now_ms());
                e.info.clone()
            });
            (e.job.take(), unpooled)
        };
        // Closing the job's last handle ends its processes (KILL_ON_JOB_CLOSE).
        drop(job);
        if let Some(info) = unpooled {
            (self.notify)(Event::State(info));
        }
    }

    pub fn stop_all(&self) {
        let jobs: Vec<Job> = self
            .entries()
            .values_mut()
            .filter_map(|e| {
                e.stopped = true;
                if e.pool.is_some() && e.info.state == State::Running {
                    e.info.state = State::Stopped;
                    e.info.end = Some(now_ms());
                }
                e.job.take()
            })
            .collect();
        drop(jobs);
    }

    /// Actions still running (for the close confirmation); those served by
    /// a pool are counted with the pools.
    pub fn running(&self) -> usize {
        self.entries()
            .values()
            .filter(|e| {
                e.pool.is_none() && matches!(e.info.state, State::Starting | State::Running)
            })
            .count()
    }

    /// Marks stopped the actions served by a pool that no longer runs
    /// (`pool_running` false: stopped from the "Server Web" page, or
    /// dead), which would otherwise stay "running" and never be cleared.
    pub fn reconcile_pools(&self, pool_running: impl Fn(&str) -> bool) {
        // The pools are asked without `entries` held (no lock inside another).
        let pools: HashSet<String> = self
            .entries()
            .values()
            .filter(|e| e.info.state == State::Running)
            .filter_map(|e| e.pool.clone())
            .collect();
        let gone: HashSet<String> = pools.into_iter().filter(|p| !pool_running(p)).collect();
        if gone.is_empty() {
            return;
        }
        let ended: Vec<RunInfo> = {
            let mut entries = self.entries();
            entries
                .values_mut()
                .filter(|e| e.info.state == State::Running)
                .filter(|e| e.pool.as_ref().is_some_and(|p| gone.contains(p)))
                .map(|e| {
                    e.info.state = State::Stopped;
                    e.info.end = Some(now_ms());
                    e.info.clone()
                })
                .collect()
        };
        for info in ended {
            (self.notify)(Event::State(info));
        }
    }

    /// Forgets the output of an action that ended.
    pub fn clear(&self, key: &str) {
        let mut entries = self.entries();
        if entries.get(key).is_some_and(|e| {
            e.job.is_none() && (e.pool.is_none() || e.info.state != State::Running)
        }) {
            entries.remove(key);
        }
    }

    /// The ports of `from` are now `to`'s, whose files were moved on disk
    /// (`state::adopt`): nothing is written, and a later save does not
    /// replace the moved `ports.json`.
    pub fn rename_project(&self, from: &str, to: &str) {
        // Same order as `save_ports`: `saved`, then `ports`.
        let mut saved = self.saved.lock().unwrap_or_else(PoisonError::into_inner);
        let mut ports = self.ports.lock().unwrap_or_else(PoisonError::into_inner);
        let rekey = |key: &String| match split_key(key) {
            Some((p, action)) if p == from => key_of(to, action),
            _ => key.clone(),
        };
        ports.ports = std::mem::take(&mut ports.ports)
            .into_iter()
            .map(|(k, port)| (rekey(&k), port))
            .collect();
        ports.locked = ports.locked.iter().map(rekey).collect();
        if let Some(p) = saved.remove(from) {
            saved.insert(to.to_owned(), p);
        }
    }
}

fn tr(lang: &str, source: &str, args: &[&str]) -> String {
    crate::i18n::t(lang, source, args)
}

/// The value of `key` in the `.env` of `dir` (`KEY=value`, `export KEY=`,
/// quotes taken off), when set there.
pub(crate) fn dotenv_value(dir: &Path, key: &str) -> Option<String> {
    let text = crate::os::safe_path::read_text(dir, &dir.join(".env"))?;
    text.lines().rev().find_map(|line| {
        let line = line.trim();
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (k, v) = line.split_once('=')?;
        (!k.starts_with('#') && k.trim() == key).then(|| {
            let v = v.trim();
            v.strip_prefix('"')
                .and_then(|v| v.strip_suffix('"'))
                .or_else(|| v.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                .unwrap_or(v)
                .to_owned()
        })
    })
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
#[path = "../../../test/rust/projects/run.rs"]
mod tests;
