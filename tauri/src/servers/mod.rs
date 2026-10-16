//! Shared Web Server pools (Apache and Nginx VirtualHost pooling):
//! - Automatic pools grouped by `(engine, serverLine, phpLine)` from the
//!   projects in `projects\`, plus user-defined profiles in `data\app\servers.json`.
//! - Each pool writes its configuration and VirtualHost files in
//!   `data\servers\<pool_id>\` at every start or reload (never inside projects).
//! - Every attached project gets its own `.localhost` VirtualHost
//!   (`<project>.localhost` plus any custom `.localhost` domains) and portable
//!   relative `Alias` / `location` mappings.

use std::collections::BTreeMap;
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::detect::{self, set_site_with_options, SiteKind, SiteOptions};
use crate::env::KitEnv;
use crate::fsx::{read_json, write_atomic, write_json};
use crate::kit::{validate_project_name, Kit};
use crate::os::jobs::{self, Job};
use crate::projects::alias;
use crate::projects::apache::{self, fwd};
use crate::projects::nginx;
use crate::projects::run::is_port_free;
use crate::setup::catalog::Catalog;

const POOL_PORT_FROM: u16 = 8081;
const POOL_PORT_TO: u16 = 8999;
/// How long `list` reuses the scan of the projects (D-71 style): the 3 s
/// poll of the page refreshes only running/pid/port in the meantime.
const LIST_TTL: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebServerEngine {
    Apache,
    Nginx,
}

impl WebServerEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            WebServerEngine::Apache => "apache",
            WebServerEngine::Nginx => "nginx",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WebServerEngine::Apache => "Apache",
            WebServerEngine::Nginx => "Nginx",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedSite {
    pub project: String,
    pub folder: String,
    pub server_name: String,
    pub domains: Vec<String>,
    pub aliases: BTreeMap<String, String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServerPool {
    pub id: String,
    pub name: String,
    pub engine: WebServerEngine,
    pub server_line: Option<String>,
    pub php_line: Option<String>,
    pub port: u16,
    pub running: bool,
    pub pid: Option<u32>,
    pub manual: bool,
    pub sites: Vec<AttachedSite>,
    pub conf_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServerProfileInput {
    #[serde(default)]
    pub id: String,
    pub name: String,
    pub engine: WebServerEngine,
    #[serde(default)]
    pub server_line: Option<String>,
    #[serde(default)]
    pub php_line: Option<String>,
    #[serde(default)]
    pub port: Option<u16>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServersView {
    pub pools: Vec<WebServerPool>,
    pub apache_lines: Vec<String>,
    pub nginx_lines: Vec<String>,
    pub php_lines: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredServers {
    #[serde(default)]
    profiles: Vec<WebServerProfileInput>,
    #[serde(default)]
    ports: BTreeMap<String, u16>,
}

struct RunningPool {
    child: Child,
    port: u16,
    fcgi_port: Option<u16>,
    /// The pool's php-cgi (Nginx with PHP): when it dies the pool is
    /// stopped, never left "running" answering 502 to every `.php`.
    php: Option<Child>,
    _job: Job,
}

impl RunningPool {
    /// The server, and its php-cgi if it has one, are still running.
    fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
            && self
                .php
                .as_mut()
                .is_none_or(|p| matches!(p.try_wait(), Ok(None)))
    }

    /// Ends the server (and, with its Job, the rest), waiting for it so
    /// that its port is being released when this returns.
    fn end(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Takes out of `running` the pools whose server or php-cgi ended: they
/// are ended (outside the lock) by the caller.
fn take_dead(running: &mut BTreeMap<String, RunningPool>) -> Vec<(String, RunningPool)> {
    let dead: Vec<String> = running
        .iter_mut()
        .filter_map(|(id, r)| (!r.alive()).then(|| id.clone()))
        .collect();
    dead.into_iter()
        .filter_map(|id| running.remove(&id).map(|r| (id, r)))
        .collect()
}

/// Ends the pools [`take_dead`] took out, noting why in the log.
fn end_dead(dead: Vec<(String, RunningPool)>) {
    for (id, mut r) in dead {
        if matches!(r.child.try_wait(), Ok(None)) {
            crate::log::warn(&format!(
                "server pool {id}: php-cgi ended, the pool is stopped (start it again)"
            ));
        }
        r.end();
    }
}

/// Lock order (never taken the other way round, to rule out deadlocks):
/// `op_lock` (start/stop/reload, held for seconds) -> `store_lock` (one
/// load + change + save of `servers.json`) -> `running` (short lookups).
/// `store_lock` is never held while waiting for `op_lock`.
/// The last scan of `list` (before the running state is applied), with the
/// language it was made in. `generation` grows at every invalidation, so a
/// scan begun before a change never stores its stale result.
#[derive(Default)]
struct ListCache {
    generation: u64,
    entry: Option<(Instant, String, WebServersView)>,
}

pub struct WebServers {
    kit: Kit,
    running: Mutex<BTreeMap<String, RunningPool>>,
    op_lock: Mutex<()>,
    /// Held around every read-modify-write of `servers.json`, so that
    /// `list` (port assignment) and `save_profile`/`remove_profile` never
    /// overwrite each other's changes.
    store_lock: Mutex<()>,
    /// A leaf lock: held only to read or replace the cached scan.
    cache: Mutex<ListCache>,
}

fn store_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("servers.json")
}

fn load_store(kit: &Kit) -> StoredServers {
    read_json(&store_path(kit)).unwrap_or_default()
}

fn save_store(kit: &Kit, store: &StoredServers) -> Result<()> {
    write_json(&store_path(kit), store)
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for ch in s.trim().chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' {
            out.push(ch.to_ascii_lowercase());
        } else if (ch == '-' || ch.is_whitespace()) && !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_matches('-').to_owned()
}

fn auto_pool_id(
    engine: WebServerEngine,
    server_line: Option<&str>,
    php_line: Option<&str>,
) -> String {
    let srv = server_line.unwrap_or("default");
    match php_line {
        Some(php) => format!("{}-{srv}-php-{php}", engine.as_str()),
        None => format!("{}-{srv}", engine.as_str()),
    }
}

fn auto_pool_name(
    engine: WebServerEngine,
    server_line: Option<&str>,
    php_line: Option<&str>,
) -> String {
    let srv = match server_line {
        Some(l) => format!("{} {l}", engine.label()),
        None => engine.label().to_owned(),
    };
    match php_line {
        Some(p) => format!("{srv} · PHP {p}"),
        None => srv,
    }
}

/// Resolves the directory of a versioned component (`apache`, `nginx`, `php`) for `line`.
fn component_dir(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    if let Some(c) = Catalog::get().find(id) {
        if let Some(dir) = crate::versions::dir_for(kit, c, line) {
            return dir;
        }
    }
    kit.apps().join(id)
}

/// Generates Apache VirtualHost file content for one attached project in a pool.
fn apache_vhost_conf(
    port: u16,
    project_root: &Path,
    projects_root: &Path,
    doc_root: &Path,
    site: &AttachedSite,
) -> String {
    let doc_fwd = fwd(doc_root);
    let mut aliases_list = vec![site.server_name.clone()];
    for d in &site.domains {
        let d = d.trim().to_ascii_lowercase();
        if alias::valid_domain(&d) && !aliases_list.contains(&d) {
            aliases_list.push(d);
        }
    }
    let mut lines = vec![
        format!("<VirtualHost *:{port}>"),
        format!("    ServerName {}", site.server_name),
    ];
    if aliases_list.len() > 1 {
        lines.push(format!("    ServerAlias {}", aliases_list[1..].join(" ")));
    }
    lines.extend([
        format!("    DocumentRoot \"{doc_fwd}\""),
        format!("    <Directory \"{doc_fwd}\">"),
        "        Options Indexes FollowSymLinks".to_owned(),
        "        AllowOverride All".to_owned(),
        "        Require all granted".to_owned(),
        "    </Directory>".to_owned(),
    ]);
    lines.extend(apache::alias_directives(
        &format!("${{CP_PROJECTS_FWD}}/{}", site.project),
        &alias::resolve(project_root, projects_root, &site.aliases),
        "    ",
    ));
    lines.push("</VirtualHost>".to_owned());
    lines.push(String::new());
    lines.join("\r\n")
}

/// Generates the master `httpd.conf` for an Apache pool.
fn apache_pool_conf(base: &str, pool_dir: &Path, port: u16, lan: bool) -> Result<String> {
    let kit_part = apache::strip_kit_tail(base)?;
    let dir_fwd = fwd(pool_dir);
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    let default_htdocs = "${SRVROOT}/htdocs";
    let mut site = vec![
        "# --- Dev-Kit: shared Apache VirtualHost pool (written at every start) ---".to_owned(),
        format!("Listen {listen}"),
        format!("ServerName localhost:{port}"),
        format!("PidFile \"{dir_fwd}/httpd.pid\""),
        format!("ErrorLog \"{dir_fwd}/error.log\""),
        "<IfModule log_config_module>".to_owned(),
        format!("    CustomLog \"{dir_fwd}/access.log\" common"),
        "</IfModule>".to_owned(),
        format!("DocumentRoot \"{default_htdocs}\""),
        "DirectoryIndex index.php index.html".to_owned(),
    ];
    site.extend(apache::php_module_blocks("CP_PHP_FWD"));
    // The first VirtualHost answers any Host no other one names (an IP from
    // the LAN, a misspelt name): Apache's own page, never a project.
    site.extend([
        format!("<VirtualHost *:{port}>"),
        "    ServerName localhost".to_owned(),
        format!("    DocumentRoot \"{default_htdocs}\""),
        "</VirtualHost>".to_owned(),
    ]);
    site.push(format!("IncludeOptional \"{dir_fwd}/vhosts/*.conf\""));
    site.push(format!("IncludeOptional \"{dir_fwd}/conf.d/*.conf\""));
    Ok(format!(
        "{}\r\n\r\n{}\r\n",
        kit_part.trim_end(),
        site.join("\r\n")
    ))
}

/// Generates the master `nginx.conf` for an Nginx pool.
fn nginx_pool_conf(nginx_install: &Path, pool_dir: &Path, port: u16, lan: bool) -> String {
    let run_fwd = fwd(pool_dir);
    let nginx_fwd = fwd(nginx_install);
    let listen = if lan {
        port.to_string()
    } else {
        format!("127.0.0.1:{port}")
    };
    [
        "# --- Dev-Kit: shared Nginx VirtualHost pool (written at every start) ---".to_owned(),
        "worker_processes 1;".to_owned(),
        format!("pid \"{run_fwd}/nginx.pid\";"),
        format!("error_log \"{run_fwd}/error.log\";"),
        "events {".to_owned(),
        "    worker_connections 1024;".to_owned(),
        "}".to_owned(),
        "http {".to_owned(),
        format!("    include \"{nginx_fwd}/conf/mime.types\";"),
        "    default_type application/octet-stream;".to_owned(),
        format!("    access_log \"{run_fwd}/access.log\";"),
        format!("    client_body_temp_path \"{run_fwd}/temp/client_body\";"),
        format!("    proxy_temp_path \"{run_fwd}/temp/proxy\";"),
        format!("    fastcgi_temp_path \"{run_fwd}/temp/fastcgi\";"),
        format!("    uwsgi_temp_path \"{run_fwd}/temp/uwsgi\";"),
        format!("    scgi_temp_path \"{run_fwd}/temp/scgi\";"),
        "    sendfile on;".to_owned(),
        "    keepalive_timeout 65;".to_owned(),
        "    server_names_hash_bucket_size 64;".to_owned(),
        format!("    include \"{run_fwd}/vhosts/*.conf\";"),
        format!("    include \"{run_fwd}/conf.d/*.conf\";"),
        "    server {".to_owned(),
        format!("        listen {listen} default_server;"),
        "        server_name localhost;".to_owned(),
        format!("        root \"{nginx_fwd}/html\";"),
        "        index index.html index.htm;".to_owned(),
        "    }".to_owned(),
        "}".to_owned(),
        String::new(),
    ]
    .join("\r\n")
}

/// The document root of an attached site: its folder inside the project,
/// checked without following links (D-77). `None` (the site is not served,
/// with a warning in the log, as for a skipped alias) when the folder is
/// missing, outside the project or reached through a junction: never the
/// project root in its place, which would serve `.env`, `.git` and the like.
fn site_doc_root(project_root: &Path, site: &AttachedSite) -> Option<PathBuf> {
    match detect::inside_dir(project_root, &site.folder) {
        Ok(dir) => Some(dir),
        Err(_) => {
            crate::log::warn(&format!(
                "{}: site folder {} not served (missing, not a folder of the project, or through a link)",
                project_root.display(),
                site.folder
            ));
            None
        }
    }
}

/// True once `port` can be listened on again (checked every 100 ms), false
/// if it is still taken after `within`: after a stop, Windows may keep the
/// socket of the killed server for a moment.
fn wait_port_released(port: u16, within: Duration) -> bool {
    let begun = Instant::now();
    loop {
        if is_port_free(port) {
            return true;
        }
        if begun.elapsed() >= within {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// The last lines of the pool's error log, or of `stderr.log` (what the
/// server printed before its own log was open: a configuration syntax error).
fn pool_error_tail(pool_dir: &Path) -> String {
    for rel in ["error.log", "logs/error.log", "stderr.log"] {
        if let Ok(text) = fs::read_to_string(pool_dir.join(rel)) {
            let tail: Vec<&str> = text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if !tail.is_empty() {
                return format!(": {}", tail.join(" | "));
            }
        }
    }
    String::new()
}

impl WebServers {
    pub fn new(kit: Kit) -> Self {
        Self {
            kit,
            running: Mutex::new(BTreeMap::new()),
            op_lock: Mutex::new(()),
            store_lock: Mutex::new(()),
            cache: Mutex::new(ListCache::default()),
        }
    }

    pub fn running_count(&self) -> usize {
        let (dead, count) = {
            let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            let dead = take_dead(&mut running);
            (dead, running.len())
        };
        end_dead(dead);
        count
    }

    pub fn stop_all(&self) {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running.clear();
    }

    fn assign_or_get_port(
        &self,
        store: &mut StoredServers,
        dirty: &mut bool,
        id: &str,
        manual: Option<u16>,
    ) -> u16 {
        if let Some(p) = manual.filter(|p| *p >= 1024) {
            if store.ports.insert(id.to_owned(), p) != Some(p) {
                *dirty = true;
            }
            return p;
        }
        if let Some(&p) = store.ports.get(id) {
            return p;
        }
        let used: std::collections::BTreeSet<u16> = store.ports.values().copied().collect();
        for p in POOL_PORT_FROM..=POOL_PORT_TO {
            if !used.contains(&p) && is_port_free(p) {
                store.ports.insert(id.to_owned(), p);
                *dirty = true;
                return p;
            }
        }
        POOL_PORT_FROM
    }

    /// The pools for the "Server Web" page. With `fresh` false (the 3 s
    /// poll) the scan of the projects is reused for [`LIST_TTL`]; only the
    /// running state (running, pid, port) is read again every time.
    pub fn list(&self, lang: &str, fresh: bool) -> WebServersView {
        let (generation, cached) = {
            let cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
            let cached = cache
                .entry
                .as_ref()
                .filter(|(at, l, _)| !fresh && l == lang && at.elapsed() < LIST_TTL)
                .map(|(_, _, view)| view.clone());
            (cache.generation, cached)
        };
        let mut view = match cached {
            Some(view) => view,
            None => {
                let view = self.scan(lang, fresh);
                let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
                if cache.generation == generation {
                    cache.entry = Some((Instant::now(), lang.to_owned(), view.clone()));
                }
                view
            }
        };
        self.apply_running(&mut view);
        view
    }

    /// Drops the cached scan: the next `list` reads the projects again.
    pub fn invalidate(&self) {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        cache.generation = cache.generation.wrapping_add(1);
        cache.entry = None;
    }

    /// Marks the pools that are running, with their live pid and port.
    fn apply_running(&self, view: &mut WebServersView) {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        let dead = take_dead(&mut running);
        for pool in &mut view.pools {
            if let Some(r) = running.get(&pool.id) {
                pool.running = true;
                pool.pid = Some(r.child.id());
                pool.port = r.port;
                for s in &mut pool.sites {
                    s.url = format!("http://{}:{}", s.server_name, r.port);
                }
            }
        }
        drop(running);
        end_dead(dead);
    }

    /// Reads the profiles, the projects and the installed lines (the slow
    /// part of `list`), assigning ports to new pools. With `prune`, the
    /// ports of pools that no longer exist (and are not running) are
    /// dropped from `servers.json`.
    fn scan(&self, lang: &str, prune: bool) -> WebServersView {
        let apache_lines = crate::db::services::server_lines(&self.kit, "apache");
        let nginx_lines = crate::db::services::server_lines(&self.kit, "nginx");
        let php_lines = crate::db::services::server_lines(&self.kit, "php");

        let store_guard = self
            .store_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut store = load_store(&self.kit);
        let mut store_dirty = false;
        let mut pools_map: BTreeMap<String, WebServerPool> = BTreeMap::new();
        // Manual pools with a site that needs PHP.
        let mut php_wanted: std::collections::BTreeSet<String> = Default::default();

        for p in store.profiles.clone() {
            let port = self.assign_or_get_port(&mut store, &mut store_dirty, &p.id, p.port);
            let server_line = p.server_line.clone().or_else(|| match p.engine {
                WebServerEngine::Apache => apache_lines.first().cloned(),
                WebServerEngine::Nginx => nginx_lines.first().cloned(),
            });
            // PHP only when chosen, or (below) when a site of the pool needs it:
            // never a php-cgi for a pool of static sites.
            let php_line = p.php_line.clone();
            pools_map.insert(
                p.id.clone(),
                WebServerPool {
                    id: p.id.clone(),
                    name: p.name.clone(),
                    engine: p.engine,
                    server_line,
                    php_line,
                    port,
                    running: false,
                    pid: None,
                    manual: true,
                    sites: Vec::new(),
                    conf_dir: format!(r"data\servers\{}", p.id),
                },
            );
        }

        for proj_dir in crate::projects::folders(&self.kit) {
            let Some(proj_id) = proj_dir
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
            else {
                continue;
            };
            let detected = detect::detect(&proj_dir, lang);
            if !detected
                .actions
                .iter()
                .any(|a| a.apache_dir.is_some() || a.nginx_dir.is_some())
            {
                continue;
            }
            let choice = crate::versions::project_choice(&self.kit, &proj_id);
            let needs_php =
                detected.requires.iter().any(|r| r == "php") || choice.contains_key("php");

            for action in &detected.actions {
                let (engine, folder) = if let Some(f) = &action.apache_dir {
                    (WebServerEngine::Apache, f.clone())
                } else if let Some(f) = &action.nginx_dir {
                    (WebServerEngine::Nginx, f.clone())
                } else {
                    continue;
                };

                let target_pool_id = if let Some(explicit) = action
                    .server
                    .as_deref()
                    .filter(|s| pools_map.contains_key(*s))
                {
                    explicit.to_owned()
                } else {
                    let srv_line = choice
                        .get(engine.as_str())
                        .cloned()
                        .or_else(|| match engine {
                            WebServerEngine::Apache => apache_lines.first().cloned(),
                            WebServerEngine::Nginx => nginx_lines.first().cloned(),
                        });
                    let php_line = if needs_php {
                        choice
                            .get("php")
                            .cloned()
                            .or_else(|| php_lines.first().cloned())
                    } else {
                        None
                    };
                    let pid = auto_pool_id(engine, srv_line.as_deref(), php_line.as_deref());
                    if !pools_map.contains_key(&pid) {
                        let port =
                            self.assign_or_get_port(&mut store, &mut store_dirty, &pid, None);
                        let name = auto_pool_name(engine, srv_line.as_deref(), php_line.as_deref());
                        pools_map.insert(
                            pid.clone(),
                            WebServerPool {
                                id: pid.clone(),
                                name,
                                engine,
                                server_line: srv_line,
                                php_line,
                                port,
                                running: false,
                                pid: None,
                                manual: false,
                                sites: Vec::new(),
                                conf_dir: format!(r"data\servers\{pid}"),
                            },
                        );
                    }
                    pid
                };

                if needs_php {
                    php_wanted.insert(target_pool_id.clone());
                }
                if let Some(pool) = pools_map.get_mut(&target_pool_id) {
                    let server_name = format!("{proj_id}.localhost");
                    let url = format!("http://{server_name}:{}", pool.port);
                    pool.sites.push(AttachedSite {
                        project: proj_id.clone(),
                        folder,
                        server_name,
                        domains: action.domains.clone(),
                        aliases: action.aliases.clone(),
                        url,
                    });
                }
            }
        }

        for (id, pool) in &mut pools_map {
            if pool.manual && pool.php_line.is_none() && php_wanted.contains(id) {
                pool.php_line = php_lines.first().cloned();
            }
        }

        // If no pools exist yet but Apache or Nginx is installed, expose the default pool so the
        // user can start/inspect it from the "Server Web" page immediately.
        if pools_map.is_empty() && (!apache_lines.is_empty() || !nginx_lines.is_empty()) {
            let engine = if !apache_lines.is_empty() {
                WebServerEngine::Apache
            } else {
                WebServerEngine::Nginx
            };
            let srv_line = match engine {
                WebServerEngine::Apache => apache_lines.first().cloned(),
                WebServerEngine::Nginx => nginx_lines.first().cloned(),
            };
            // No project yet: no PHP project, so no php-cgi (BUG-05r).
            let php_line: Option<String> = None;
            let pid = auto_pool_id(engine, srv_line.as_deref(), php_line.as_deref());
            let port = self.assign_or_get_port(&mut store, &mut store_dirty, &pid, None);
            let name = auto_pool_name(engine, srv_line.as_deref(), php_line.as_deref());
            pools_map.insert(
                pid.clone(),
                WebServerPool {
                    id: pid.clone(),
                    name,
                    engine,
                    server_line: srv_line,
                    php_line,
                    port,
                    running: false,
                    pid: None,
                    manual: false,
                    sites: Vec::new(),
                    conf_dir: format!(r"data\servers\{pid}"),
                },
            );
        }

        if prune {
            // store_lock -> running: the documented lock order.
            let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            let before = store.ports.len();
            store
                .ports
                .retain(|id, _| pools_map.contains_key(id) || running.contains_key(id));
            store_dirty |= store.ports.len() != before;
        }
        if store_dirty {
            let _ = save_store(&self.kit, &store);
        }
        drop(store_guard);

        WebServersView {
            pools: pools_map.into_values().collect(),
            apache_lines,
            nginx_lines,
            php_lines,
        }
    }

    pub fn save_profile(&self, mut input: WebServerProfileInput, lang: &str) -> Result<()> {
        let name = input.name.trim().to_owned();
        if name.is_empty() {
            bail!(
                "{}",
                crate::i18n::t(lang, "Scrivi un nome per il profilo server.", &[])
            );
        }
        let id = if input.id.trim().is_empty() {
            slug(&name)
        } else {
            slug(&input.id)
        };
        if id.is_empty() {
            bail!(
                "{}",
                crate::i18n::t(lang, "Identificativo del server non valido.", &[])
            );
        }
        if let Some(p) = input.port {
            if p < 1024 {
                bail!(
                    "{}",
                    crate::i18n::t(lang, "Porta tra {0} e {1}", &["1024", "65535"])
                );
            }
        }
        input.id = id.clone();
        input.name = name;

        let _store = self
            .store_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut store = load_store(&self.kit);
        if let Some(p) = input.port {
            store.ports.insert(id.clone(), p);
        }
        if let Some(existing) = store.profiles.iter_mut().find(|p| p.id == id) {
            *existing = input;
        } else {
            store.profiles.push(input);
        }
        let saved = save_store(&self.kit, &store);
        self.invalidate();
        saved
    }

    pub fn remove_profile(&self, id: &str) -> Result<()> {
        // `stop` takes `op_lock`: done before `store_lock` (lock order).
        self.stop(id);
        let _store = self
            .store_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut store = load_store(&self.kit);
        store.profiles.retain(|p| p.id != id);
        store.ports.remove(id);
        let saved = save_store(&self.kit, &store);
        self.invalidate();
        saved
    }

    /// Writes `httpd.conf` / `nginx.conf` and all `vhosts/*.conf` for `pool`.
    fn write_pool_configs(
        &self,
        pool: &WebServerPool,
        fcgi_port: Option<u16>,
        lan: bool,
    ) -> Result<PathBuf> {
        let pool_dir = self.kit.data().join("servers").join(&pool.id);
        let vhosts_dir = pool_dir.join("vhosts");
        let conf_d_dir = pool_dir.join("conf.d");
        fs::create_dir_all(&vhosts_dir)?;
        fs::create_dir_all(&conf_d_dir)?;

        // Clear old generated vhost files before writing current ones.
        if let Ok(entries) = fs::read_dir(&vhosts_dir) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case("conf"))
                {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }

        match pool.engine {
            WebServerEngine::Apache => {
                let apache_dir = component_dir(&self.kit, "apache", pool.server_line.as_deref());
                let base = fs::read_to_string(apache_dir.join(r"conf\httpd.conf"))
                    .context("Apache httpd.conf")?;
                for site in &pool.sites {
                    let proj_root = self.kit.projects().join(&site.project);
                    let Some(doc_root) = site_doc_root(&proj_root, site) else {
                        continue;
                    };
                    let vhost = apache_vhost_conf(
                        pool.port,
                        &proj_root,
                        &self.kit.projects(),
                        &doc_root,
                        site,
                    );
                    write_atomic(
                        &vhosts_dir.join(format!("{}.conf", site.project)),
                        vhost.as_bytes(),
                    )?;
                }
                let main_conf = apache_pool_conf(&base, &pool_dir, pool.port, lan)?;
                let conf_path = pool_dir.join("httpd.conf");
                write_atomic(&conf_path, main_conf.as_bytes())?;
                Ok(conf_path)
            }
            WebServerEngine::Nginx => {
                nginx::prepare_dirs(&pool_dir)?;
                let nginx_dir = component_dir(&self.kit, "nginx", pool.server_line.as_deref());
                for site in &pool.sites {
                    let proj_root = self.kit.projects().join(&site.project);
                    let Some(doc_root) = site_doc_root(&proj_root, site) else {
                        continue;
                    };
                    let vhost = nginx::vhost_block(
                        &nginx_dir,
                        Some(&site.project),
                        &doc_root,
                        None,
                        pool.port,
                        fcgi_port,
                        lan,
                        &site.domains,
                        &alias::resolve(&proj_root, &self.kit.projects(), &site.aliases),
                        false,
                    );
                    write_atomic(
                        &vhosts_dir.join(format!("{}.conf", site.project)),
                        format!("{vhost}\r\n").as_bytes(),
                    )?;
                }
                let main_conf = nginx_pool_conf(&nginx_dir, &pool_dir, pool.port, lan);
                let conf_path = pool_dir.join("nginx.conf");
                write_atomic(&conf_path, main_conf.as_bytes())?;
                Ok(conf_path)
            }
        }
    }

    pub fn start(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.start_locked(id, base_env, lang)
    }

    fn start_locked(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let view = self.list(lang, true);
        let pool = view
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("Server pool not found: {id}"))?;
        if pool.running {
            return Ok(());
        }
        if !is_port_free(pool.port) {
            bail!(
                "{}",
                crate::i18n::t(
                    lang,
                    "La porta {0} è già usata da un altro programma: cambiala qui sotto.",
                    &[&pool.port.to_string()]
                )
            );
        }

        let mut env = base_env.clone();
        if let Some(php_line) = &pool.php_line {
            let php_dir = component_dir(&self.kit, "php", Some(php_line));
            env.set("CP_PHP", php_dir.display().to_string());
            env.set("CP_PHP_FWD", fwd(&php_dir));
        }
        env.set("CP_PROJECTS_FWD", fwd(&self.kit.projects()));
        let lan = env.get("DEVKIT_ALLOW_LAN") == Some("1");

        let pool_dir = self.kit.data().join("servers").join(&pool.id);
        let _ = fs::remove_file(pool_dir.join("error.log"));
        let _ = fs::remove_file(pool_dir.join("stderr.log"));

        let job = Job::new()?;
        let (mut cmd, fcgi_port, php) = match pool.engine {
            WebServerEngine::Apache => {
                let apache_dir = component_dir(&self.kit, "apache", pool.server_line.as_deref());
                let exe = apache_dir.join(r"bin\httpd.exe");
                if !exe.is_file() {
                    bail!("Apache is not installed");
                }
                env.set("CP_APACHE", apache_dir.display().to_string());
                env.set("CP_APACHE_FWD", fwd(&apache_dir));
                let conf_path = self.write_pool_configs(&pool, None, lan)?;
                let mut cmd = jobs::kit_command(&exe, &env);
                cmd.arg("-f").arg(&conf_path);
                (cmd, None, None)
            }
            WebServerEngine::Nginx => {
                let nginx_dir = component_dir(&self.kit, "nginx", pool.server_line.as_deref());
                let exe = nginx_dir.join("nginx.exe");
                if !exe.is_file() {
                    bail!("Nginx is not installed");
                }
                env.set("CP_NGINX", nginx_dir.display().to_string());
                env.set("CP_NGINX_FWD", fwd(&nginx_dir));
                let (fcgi, php) = if pool.php_line.is_some() {
                    nginx::spawn_php_cgi_if_present(&self.kit, &env, &job)
                        .map_or((None, None), |(port, child)| (Some(port), Some(child)))
                } else {
                    (None, None)
                };
                let conf_path = self.write_pool_configs(&pool, fcgi, lan)?;
                let mut cmd = jobs::kit_command(&exe, &env);
                cmd.arg("-p")
                    .arg(&pool_dir)
                    .arg("-c")
                    .arg(&conf_path)
                    .arg("-g")
                    .arg("daemon off;");
                (cmd, fcgi, php)
            }
        };

        // What the server prints (a syntax error in a configuration comes
        // before its error log is open) goes to stderr.log, for the error.
        let out_path = pool_dir.join("stderr.log");
        let out = fs::File::create(&out_path).with_context(|| out_path.display().to_string())?;
        cmd.stdin(Stdio::null())
            .stdout(Stdio::from(out.try_clone()?))
            .stderr(Stdio::from(out));
        let mut child = jobs::spawn_in(&mut cmd, &job)?;

        let addr = SocketAddr::from(([127, 0, 0, 1], pool.port));
        let mut ready = false;
        for _ in 0..60 {
            if TcpStream::connect_timeout(&addr, Duration::from_millis(80)).is_ok() {
                ready = true;
                break;
            }
            if let Some(status) = child.try_wait().context("wait web server")? {
                let detail = pool_error_tail(&pool_dir);
                bail!(
                    "{} exited prematurely ({status}){detail}",
                    pool.engine.label()
                );
            }
            std::thread::sleep(Duration::from_millis(80));
        }
        if !ready {
            let detail = pool_error_tail(&pool_dir);
            bail!(
                "{} did not start listening on port {}{detail}",
                pool.engine.label(),
                pool.port
            );
        }

        self.running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(
                id.to_owned(),
                RunningPool {
                    child,
                    port: pool.port,
                    fcgi_port,
                    php,
                    _job: job,
                },
            );
        Ok(())
    }

    pub fn stop(&self, id: &str) {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.stop_locked(id);
    }

    fn stop_locked(&self, id: &str) {
        let removed = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(id);
        // The server itself is killed and waited for before its Job is
        // dropped (which ends what it started, php-cgi included), so that
        // its port is being released when this returns.
        if let Some(pool) = removed {
            pool.end();
        }
    }

    pub fn reload(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.reload_locked(id, base_env, lang)
    }

    fn reload_locked(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let view = self.list(lang, true);
        let pool = view
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("Server pool not found: {id}"))?;
        let lan = base_env.get("DEVKIT_ALLOW_LAN") == Some("1");
        let existing_fcgi = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .and_then(|r| r.fcgi_port);

        let conf_path = self.write_pool_configs(&pool, existing_fcgi, lan)?;
        if !pool.running {
            return Ok(());
        }

        if pool.engine == WebServerEngine::Nginx {
            let nginx_dir = component_dir(&self.kit, "nginx", pool.server_line.as_deref());
            let exe = nginx_dir.join("nginx.exe");
            let pool_dir = self.kit.data().join("servers").join(&pool.id);
            let status = jobs::kit_command(&exe, base_env)
                .arg("-p")
                .arg(&pool_dir)
                .arg("-c")
                .arg(&conf_path)
                .args(["-s", "reload"])
                .status();
            if status.is_ok_and(|s| s.success()) {
                return Ok(());
            }
        }

        // Clean restart when graceful signal is unavailable (Apache standalone on Windows).
        self.stop_locked(id);
        // start_locked reports the port as taken if it is still not free.
        wait_port_released(pool.port, Duration::from_secs(5));
        self.start_locked(id, base_env, lang)
    }

    pub fn assign_project(
        &self,
        project: &str,
        folder: &str,
        engine: WebServerEngine,
        server_id: Option<String>,
        base_env: &KitEnv,
        lang: &str,
    ) -> Result<()> {
        validate_project_name(project).map_err(|e| anyhow!(e.to_string()))?;
        let proj_dir = self.kit.projects().join(project);
        if !crate::os::safe_path::is_dir(&self.kit.projects(), &proj_dir) {
            bail!(
                "{}",
                crate::i18n::t(lang, "Progetto non trovato: {0}", &[project])
            );
        }
        let detected = detect::detect(&proj_dir, lang);
        let existing = detected
            .actions
            .iter()
            .find(|a| a.apache_dir.is_some() || a.nginx_dir.is_some());
        let opts = SiteOptions {
            server: server_id.filter(|s| !s.trim().is_empty()),
            domains: existing.map(|a| a.domains.clone()).unwrap_or_default(),
            aliases: existing.map(|a| a.aliases.clone()).unwrap_or_default(),
        };
        let kind = match engine {
            WebServerEngine::Apache => SiteKind::Apache,
            WebServerEngine::Nginx => SiteKind::Nginx,
        };
        let folder_clean = if folder.trim().is_empty() {
            "."
        } else {
            folder.trim()
        };
        set_site_with_options(&proj_dir, folder_clean, kind, &opts, lang)?;
        self.invalidate();

        // Reload any running pools so the new VirtualHost routing takes effect immediately.
        let running_ids: Vec<String> = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .keys()
            .cloned()
            .collect();
        for rid in running_ids {
            let _ = self.reload(&rid, base_env, lang);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../test/rust/servers/mod.rs"]
mod tests;
