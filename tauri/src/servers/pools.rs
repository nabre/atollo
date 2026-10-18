//! The pools of the Web Servers page: `data\app\servers.json` (profiles
//! and remembered ports), the automatic pools grouped from the projects,
//! the names of their sites, and the scan that builds the page's view.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::PoisonError;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

use crate::detect::{self, set_site_with_options, SiteKind, SiteOptions};
use crate::env::KitEnv;
use crate::fsx::{read_json, write_json};
use crate::kit::{validate_project_name, Kit};
use crate::projects::run::is_port_free;

use super::{
    AttachedSite, WebServerEngine, WebServerPool, WebServerProfileInput, WebServers, WebServersView,
};

pub(super) const POOL_PORT_FROM: u16 = 8081;
pub(super) const POOL_PORT_TO: u16 = 8999;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StoredServers {
    #[serde(default)]
    pub(super) profiles: Vec<WebServerProfileInput>,
    #[serde(default)]
    pub(super) ports: BTreeMap<String, u16>,
}

pub(super) fn store_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("servers.json")
}

pub(super) fn load_store(kit: &Kit) -> StoredServers {
    read_json(&store_path(kit)).unwrap_or_default()
}

pub(super) fn save_store(kit: &Kit, store: &StoredServers) -> Result<()> {
    write_json(&store_path(kit), store)
}

pub(super) fn slug(s: &str) -> String {
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

/// A DNS label for a `.localhost` name (`alias::valid_domain`): lowercase
/// ASCII letters and digits, anything else as one `-`, none at the edges;
/// `site` when nothing is left.
pub(super) fn host_label(s: &str) -> String {
    let mut out = String::new();
    for ch in s.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    let out = out.trim_end_matches('-');
    if out.is_empty() {
        "site".to_owned()
    } else {
        out.chars()
            .take(50)
            .collect::<String>()
            .trim_end_matches('-')
            .to_owned()
    }
}

/// Gives every site of a pool its own `ServerName` (BUG-V3-02): the main
/// site of a project (its `site` action, else its first one in the pool)
/// `<project>.localhost`, every other `<action>.<project>.localhost`. A name
/// already taken in the pool (two projects whose names give the same label)
/// gets `-2`, `-3`... on its first label. The VirtualHost file is named after
/// it, so two sites never write the same file either.
pub(super) fn name_sites(sites: &mut [AttachedSite], port: u16) {
    let mut used: BTreeSet<String> = Default::default();
    for i in 0..sites.len() {
        let project = sites[i].project.clone();
        let main = sites
            .iter()
            .position(|s| s.project == project && s.action == detect::SITE_ACTION)
            .or_else(|| sites.iter().position(|s| s.project == project));
        let (first, rest) = if main == Some(i) {
            (host_label(&project), "localhost".to_owned())
        } else {
            (
                host_label(&sites[i].action),
                format!("{}.localhost", host_label(&project)),
            )
        };
        let mut name = format!("{first}.{rest}");
        let mut n = 2;
        while used.contains(&name) {
            name = format!("{first}-{n}.{rest}");
            n += 1;
        }
        used.insert(name.clone());
        sites[i].url = format!("http://{name}:{port}");
        sites[i].server_name = name;
    }
}

/// The Apache/Nginx sites of a project that go in a pool, each project
/// once per site (D-101, defect 7): every one written in
/// `panel.json`; the detected ones (`.htaccess`, `nginx.conf`) only when
/// there is none of those, and then only the first, so that a project with
/// a `site` action, or with both files, is not in two pools at once.
pub(super) fn pooled_actions(actions: &[detect::Action]) -> Vec<&detect::Action> {
    let web = |a: &&detect::Action| a.web_server_dir().is_some();
    let custom: Vec<&detect::Action> = actions.iter().filter(web).filter(|a| a.custom).collect();
    if custom.is_empty() {
        actions.iter().filter(web).take(1).collect()
    } else {
        custom
    }
}

/// The VirtualHost file of a site in `vhosts\`: one per `ServerName`, unique
/// in the pool ([`name_sites`]).
pub(super) fn vhost_file(site: &AttachedSite) -> String {
    format!("{}.conf", site.server_name)
}

/// `data\servers\<id>` as the page shows it, with the system's separator.
fn conf_dir_text(id: &str) -> String {
    let sep = std::path::MAIN_SEPARATOR;
    format!("data{sep}servers{sep}{id}")
}

/// The id of the automatic pool of (engine, server line, PHP line), SC-2:
/// `apache-2.4_php-7.4`, `nginx-1.27` (no PHP), `apache-default` (no line
/// installed). It names the lines it runs, so it can be read back
/// ([`parse_auto_pool_id`]) whatever is installed later.
pub(super) fn auto_pool_id(
    engine: WebServerEngine,
    server_line: Option<&str>,
    php_line: Option<&str>,
) -> String {
    let srv = server_line.unwrap_or("default");
    match php_line {
        Some(php) => format!("{}-{srv}_php-{php}", engine.as_str()),
        None => format!("{}-{srv}", engine.as_str()),
    }
}

/// The engine and lines an automatic pool id names: the id of
/// [`auto_pool_id`], or the one of before SC-2 (`apache-2.4-php-7.4`).
/// `None` for any other id (a profile's).
pub(super) fn parse_auto_pool_id(id: &str) -> Option<PoolLines> {
    let (engine, rest) = WebServerEngine::ALL
        .into_iter()
        .find_map(|e| Some((e, id.strip_prefix(e.as_str())?.strip_prefix('-')?)))?;
    let (srv, php) = match rest
        .split_once("_php-")
        .or_else(|| rest.split_once("-php-"))
    {
        Some((srv, php)) => (srv, Some(php)),
        None => (rest, None),
    };
    let line = |l: &str| crate::versions::valid_line(l).then(|| l.to_owned());
    let srv = match srv {
        "default" => None,
        l => Some(line(l)?),
    };
    let php = match php {
        Some(p) => Some(line(p)?),
        None => None,
    };
    Some((engine, srv, php))
}

/// An id a site's `server` in `panel.json` gives another meaning than a
/// profile: `auto`, `dedicated`, or an automatic pool's.
pub(super) fn reserved_id(id: &str) -> bool {
    id == "auto" || id == crate::projects::run::DEDICATED_SERVER || parse_auto_pool_id(id).is_some()
}

/// The engine and lines of `id` when it is an automatic pool id as
/// [`auto_pool_id`] writes it today.
pub(super) type PoolLines = (WebServerEngine, Option<String>, Option<String>);

pub(super) fn canonical_auto_pool(id: &str) -> Option<PoolLines> {
    parse_auto_pool_id(id)
        .filter(|(e, srv, php)| auto_pool_id(*e, srv.as_deref(), php.as_deref()) == id)
}

/// Moves the ports stored under the automatic ids of before SC-2
/// (`apache-2.4-php-7.4`) to the new ones (`apache-2.4_php-7.4`), so that a
/// pool keeps its port; a profile's id is never touched. True if any moved.
pub(super) fn migrate_port_ids(store: &mut StoredServers) -> bool {
    let profiles: BTreeSet<&str> = store.profiles.iter().map(|p| p.id.as_str()).collect();
    let moves: Vec<(String, String)> = store
        .ports
        .keys()
        .filter(|id| !profiles.contains(id.as_str()))
        .filter_map(|id| {
            let (engine, srv, php) = parse_auto_pool_id(id)?;
            let new = auto_pool_id(engine, srv.as_deref(), php.as_deref());
            (&new != id).then(|| (id.clone(), new))
        })
        .collect();
    let mut moved = false;
    for (old, new) in moves {
        if let Some(port) = store.ports.remove(&old) {
            store.ports.entry(new).or_insert(port);
            moved = true;
        }
    }
    moved
}

/// An automatic pool with no sites yet.
pub(super) fn auto_pool(
    engine: WebServerEngine,
    server_line: Option<String>,
    php_line: Option<String>,
    port: u16,
) -> WebServerPool {
    let id = auto_pool_id(engine, server_line.as_deref(), php_line.as_deref());
    WebServerPool {
        name: auto_pool_name(engine, server_line.as_deref(), php_line.as_deref()),
        conf_dir: conf_dir_text(&id),
        id,
        engine,
        server_line,
        php_line,
        port,
        running: false,
        pid: None,
        manual: false,
        sites: Vec::new(),
        orphan: false,
        profile: None,
    }
}

pub(super) fn auto_pool_name(
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

impl WebServers {
    pub(super) fn assign_or_get_port(
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
        // A stored port another pool also has (a profile given that port
        // by hand before profiles' ports were checked) is given up for a
        // new one, so files of before are mended at the first scan.
        if let Some(&p) = store.ports.get(id) {
            if !store.ports.iter().any(|(other, q)| *q == p && other != id) {
                return p;
            }
        }
        let used: BTreeSet<u16> = store.ports.values().copied().collect();
        // A free port no other pool has; failing that (every one taken by
        // other programs), one no other pool has, which the start reports
        // as busy: never the port of another pool (defect 6).
        let p = (POOL_PORT_FROM..=POOL_PORT_TO)
            .find(|p| !used.contains(p) && is_port_free(*p))
            .or_else(|| (POOL_PORT_FROM..=POOL_PORT_TO).find(|p| !used.contains(p)))
            .unwrap_or(POOL_PORT_FROM);
        store.ports.insert(id.to_owned(), p);
        *dirty = true;
        p
    }

    /// Reads the profiles, the projects and the installed lines (the slow
    /// part of `list`), assigning ports to new pools. With `prune`, the
    /// ports of pools that no longer exist (and are not running) are
    /// dropped from `servers.json`.
    pub(super) fn scan(&self, lang: &str, prune: bool) -> WebServersView {
        let apache_lines = crate::db::services::server_lines(&self.kit, "apache");
        let nginx_lines = crate::db::services::server_lines(&self.kit, "nginx");
        let caddy_lines = crate::db::services::server_lines(&self.kit, "caddy");
        let php_lines = crate::db::services::server_lines(&self.kit, "php");
        let newest = |engine: WebServerEngine| match engine {
            WebServerEngine::Apache => apache_lines.first().cloned(),
            WebServerEngine::Nginx => nginx_lines.first().cloned(),
            WebServerEngine::Caddy => caddy_lines.first().cloned(),
        };

        let store_guard = self
            .store_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut store = load_store(&self.kit);
        let mut store_dirty = migrate_port_ids(&mut store);
        let mut pools_map: BTreeMap<String, WebServerPool> = BTreeMap::new();
        // Manual pools with a site that needs PHP.
        let mut php_wanted: BTreeSet<String> = Default::default();

        for p in store.profiles.clone() {
            let port = self.assign_or_get_port(&mut store, &mut store_dirty, &p.id, p.port);
            let server_line = p.server_line.clone().or_else(|| newest(p.engine));
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
                    conf_dir: conf_dir_text(&p.id),
                    orphan: false,
                    profile: Some(p.clone()),
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
                .any(|a| a.web_server_dir().is_some())
            {
                continue;
            }
            let choice = crate::versions::project_choice(&self.kit, &proj_id);
            let needs_php =
                detected.requires.iter().any(|r| r == "php") || choice.contains_key("php");

            for action in pooled_actions(&detected.actions) {
                let Some((engine, folder)) = WebServerEngine::of_action(action) else {
                    continue;
                };

                let explicit = action.server.as_deref().map(str::trim);
                if explicit == Some(crate::projects::run::DEDICATED_SERVER) {
                    // Its own server, never a pool's.
                    continue;
                }
                let target_pool_id = if let Some(profile) =
                    explicit.filter(|s| pools_map.get(*s).is_some_and(|p| p.manual))
                {
                    profile.to_owned()
                } else {
                    // An automatic pool named in `panel.json` (also by its id
                    // of before SC-2) runs the lines it names; any other site
                    // goes to the pool of the project's lines (the newest on
                    // disk for one not chosen, as the kit runs it).
                    let (srv_line, php_line) = match explicit
                        .and_then(parse_auto_pool_id)
                        .filter(|(e, _, _)| *e == engine)
                    {
                        Some((_, srv, php)) => (srv, php),
                        None => {
                            let srv_line = choice
                                .get(engine.as_str())
                                .cloned()
                                .or_else(|| newest(engine));
                            let php_line = if needs_php {
                                choice
                                    .get("php")
                                    .cloned()
                                    .or_else(|| php_lines.first().cloned())
                            } else {
                                None
                            };
                            (srv_line, php_line)
                        }
                    };
                    let pid = auto_pool_id(engine, srv_line.as_deref(), php_line.as_deref());
                    if !pools_map.contains_key(&pid) {
                        let port =
                            self.assign_or_get_port(&mut store, &mut store_dirty, &pid, None);
                        pools_map.insert(pid.clone(), auto_pool(engine, srv_line, php_line, port));
                    }
                    pid
                };

                if needs_php {
                    php_wanted.insert(target_pool_id.clone());
                }
                if let Some(pool) = pools_map.get_mut(&target_pool_id) {
                    // Named, with its URL, by `name_sites` below.
                    pool.sites.push(AttachedSite {
                        project: proj_id.clone(),
                        action: action.id.clone(),
                        folder,
                        server_name: String::new(),
                        domains: action.domains.clone(),
                        aliases: action.aliases.clone(),
                        url: String::new(),
                        chosen: action.custom,
                        detached: false,
                    });
                }
            }
        }

        for pool in pools_map.values_mut() {
            name_sites(&mut pool.sites, pool.port);
        }

        for (id, pool) in &mut pools_map {
            if pool.manual && pool.php_line.is_none() && php_wanted.contains(id) {
                pool.php_line = php_lines.first().cloned();
            }
        }

        // If no pools exist yet but a web server is installed, expose the default pool so the
        // user can start/inspect it from the "Server Web" page immediately: Apache's as
        // before, else Caddy's, else Nginx's.
        let first_engine = [
            WebServerEngine::Apache,
            WebServerEngine::Caddy,
            WebServerEngine::Nginx,
        ]
        .into_iter()
        .find(|e| newest(*e).is_some());
        if let (true, Some(engine)) = (pools_map.is_empty(), first_engine) {
            let srv_line = newest(engine);
            // No project yet: no PHP project, so no php-cgi (BUG-05r).
            let pid = auto_pool_id(engine, srv_line.as_deref(), None);
            let port = self.assign_or_get_port(&mut store, &mut store_dirty, &pid, None);
            pools_map.insert(pid, auto_pool(engine, srv_line, None, port));
        }

        {
            // store_lock -> running: the documented lock order.
            let running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            // A running automatic pool no project goes to any more (a project
            // moved to a newer line): its id names its lines (SC-2), so it is
            // still a pool of the list, to stop or to reload, never an orphan.
            for id in running.keys() {
                if pools_map.contains_key(id) {
                    continue;
                }
                if let Some((engine, srv, php)) = canonical_auto_pool(id) {
                    let port = self.assign_or_get_port(&mut store, &mut store_dirty, id, None);
                    pools_map.insert(id.clone(), auto_pool(engine, srv, php, port));
                }
            }
        }

        if prune {
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
            caddy_lines,
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
            // A new profile never takes an id a site's `server` means
            // otherwise (`auto`, `dedicated`, an automatic pool's, as from
            // the name "Apache 2.4").
            let id = slug(&name);
            if reserved_id(&id) {
                format!("{id}-profile")
            } else {
                id
            }
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

        // The ports of pools gone since (a project removed, another line)
        // leave `servers.json` first: never "already taken" by a pool that
        // does not exist any more.
        self.scan(lang, true);
        let _store = self
            .store_lock
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let mut store = load_store(&self.kit);
        if let Some(p) = input.port {
            // One port per pool (defect 6): two pools on one port, the
            // second never starts and its sites answer from the first.
            if let Some((other, _)) = store.ports.iter().find(|(o, q)| **q == p && **o != id) {
                bail!(
                    "{}",
                    crate::i18n::t(
                        lang,
                        "La porta {0} è già del server {1}: scegline un'altra.",
                        &[&p.to_string(), other]
                    )
                );
            }
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
        // `op_lock` for the whole removal (then `store_lock`, the lock
        // order): a start in between would bring the pool back to life
        // after it was stopped, with its profile gone.
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.stop_locked(id);
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
            .find(|a| a.web_server_dir().is_some());
        let opts = SiteOptions {
            server: server_id.filter(|s| !s.trim().is_empty()),
            domains: existing.map(|a| a.domains.clone()).unwrap_or_default(),
            aliases: existing.map(|a| a.aliases.clone()).unwrap_or_default(),
        };
        let kind = match engine {
            WebServerEngine::Apache => SiteKind::Apache,
            WebServerEngine::Nginx => SiteKind::Nginx,
            WebServerEngine::Caddy => SiteKind::Caddy,
        };
        let folder_clean = if folder.trim().is_empty() {
            "."
        } else {
            folder.trim()
        };
        // The pools serving the project before and after the change are the
        // only ones to reload (defect 8: not every running pool).
        let mut touched = self.pools_serving(project, lang);
        set_site_with_options(&proj_dir, folder_clean, kind, &opts, lang)?;
        self.invalidate();
        touched.extend(self.pools_serving(project, lang));

        let mut failed = None;
        for id in touched {
            if !self.is_running(&id) {
                continue;
            }
            if let Err(e) = self.reload(&id, base_env, lang) {
                crate::log::warn(&format!("server pool {id}: reload: {e:#}"));
                failed.get_or_insert(e);
            }
        }
        failed.map_or(Ok(()), Err)
    }

    /// The ids of the pools (orphans aside) with a site of `project`.
    pub(super) fn pools_serving(&self, project: &str, lang: &str) -> BTreeSet<String> {
        self.list_all(lang, true)
            .pools
            .into_iter()
            .filter(|p| !p.orphan && p.sites.iter().any(|s| s.project == project))
            .map(|p| p.id)
            .collect()
    }
}
