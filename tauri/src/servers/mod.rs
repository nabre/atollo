//! Shared Web Server pools (Caddy and Apache VirtualHost pooling):
//! - Automatic pools grouped by `(engine, serverLine, phpLine)` from the
//!   projects in `projects\`, plus user-defined profiles in `data\app\servers.json`.
//! - Each pool writes its configuration and VirtualHost files in
//!   `data\servers\<pool_id>\` at every start or reload (never inside projects).
//! - Every attached project gets its own `.localhost` VirtualHost
//!   (`<project>.localhost` plus any custom `.localhost` domains) and portable
//!   relative `Alias` / `location` mappings.
//!
//! Files: `mod` (types, the page's list), `pools` (servers.json, automatic
//! pools, scan), `config` (generated configuration), `process` (start, stop,
//! reload, attach), `migrate` (the pools and sites of Nginx to Caddy, D-120).

mod config;
pub mod migrate;
mod pools;
mod process;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use crate::kit::Kit;
pub(crate) use pools::{host_label, pool_ports};
use process::{end_dead, take_dead, RunningPool};

/// How long `list` reuses the scan of the projects (D-71 style): the 3 s
/// poll of the page refreshes only running/pid/port in the meantime.
const LIST_TTL: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WebServerEngine {
    Apache,
    /// The kit's web server on every system (D-111); a profile of Nginx,
    /// no longer in the kit, reads as Caddy (D-120, `migrate.rs`).
    #[serde(alias = "nginx")]
    Caddy,
}

impl WebServerEngine {
    pub const ALL: [WebServerEngine; 2] = [WebServerEngine::Apache, WebServerEngine::Caddy];

    /// Its id, also the id of its component in the catalog.
    pub fn as_str(self) -> &'static str {
        match self {
            WebServerEngine::Apache => "apache",
            WebServerEngine::Caddy => "caddy",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            WebServerEngine::Apache => "Apache",
            WebServerEngine::Caddy => "Caddy",
        }
    }

    /// The engine of a site of a project, from the folder its action
    /// names (`apache`, `caddy` in `panel.json`), with that folder.
    pub fn of_action(action: &crate::detect::Action) -> Option<(WebServerEngine, String)> {
        if let Some(f) = &action.apache_dir {
            Some((WebServerEngine::Apache, f.clone()))
        } else {
            action
                .caddy_dir
                .as_ref()
                .map(|f| (WebServerEngine::Caddy, f.clone()))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedSite {
    pub project: String,
    /// The id of the project's action this site comes from.
    #[serde(default)]
    pub action: String,
    pub folder: String,
    pub server_name: String,
    pub domains: Vec<String>,
    pub aliases: BTreeMap<String, String>,
    pub url: String,
    /// The project chose this server: a site written in `panel.json` (or
    /// assigned to the pool), not one only detected (`.htaccess`, a PHP
    /// project offered Apache). One that is not goes in the pool only while
    /// a project runs it (D-101).
    #[serde(default)]
    pub chosen: bool,
    /// Its project was stopped in this session ([`WebServers::detach`]):
    /// listed, but left out of the pool's files until it starts again.
    #[serde(default)]
    pub detached: bool,
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
    /// Running, but no longer one of the pools of the profiles and the
    /// projects: shown as it was started so that it can be stopped. Since
    /// the automatic ids name their lines (SC-2) a running automatic pool is
    /// always rebuilt from its id, so only a pool whose id is neither a
    /// profile's nor an automatic one can be left like this.
    pub orphan: bool,
    /// The profile as saved (a manual pool), for the page to save it again
    /// with its options changed; `None` for an automatic pool, which stops
    /// with its last project and never starts with the app.
    pub profile: Option<WebServerProfileInput>,
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
    /// «Mantieni attivo anche senza progetti»: started by a project, the
    /// pool stays on when its last project stops. On by default (a profile
    /// of an older `servers.json` behaves as before).
    #[serde(default = "keep_alive_default")]
    pub keep_alive: bool,
    /// «Avvia con Atollo»: started when the app opens.
    #[serde(default)]
    pub autostart: bool,
}

fn keep_alive_default() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebServersView {
    pub pools: Vec<WebServerPool>,
    pub apache_lines: Vec<String>,
    pub caddy_lines: Vec<String>,
    pub php_lines: Vec<String>,
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
    /// The sites (`project`, `action`) whose project was stopped: their
    /// VirtualHost is left out of the pool until the project starts again
    /// (D-101). A leaf lock, like `cache`.
    detached: Mutex<BTreeSet<(String, String)>>,
    /// The sites (`project`, `action`) started by their project in this
    /// session ([`Self::attach`]): with the sites the projects chose, the
    /// only ones a pool lists and serves ([`serves`]). A leaf lock.
    started: Mutex<BTreeSet<(String, String)>>,
}

/// A site is in its pool when its project chose that server, or while its
/// project runs it (D-101): never every PHP project that
/// was only offered Apache.
fn serves(site: &AttachedSite, started: &BTreeSet<(String, String)>) -> bool {
    site.chosen || started.contains(&(site.project.clone(), site.action.clone()))
}

/// The pools as the page shows them: only the sites [`serves`] keeps, and
/// no automatic pool left with none (unless it runs), but one, so that a kit
/// with Caddy or Apache and no site still has a pool to start.
fn shown_pools(
    pools: Vec<WebServerPool>,
    started: &BTreeSet<(String, String)>,
) -> Vec<WebServerPool> {
    let mut first_auto = None;
    let mut out = Vec::new();
    for mut pool in pools {
        pool.sites.retain(|s| serves(s, started));
        if pool.manual || pool.running || !pool.sites.is_empty() {
            out.push(pool);
        } else if first_auto.is_none() {
            first_auto = Some(pool);
        }
    }
    if out.is_empty() {
        out.extend(first_auto);
    }
    out
}

impl WebServers {
    pub fn new(kit: Kit) -> Self {
        Self {
            kit,
            running: Mutex::new(BTreeMap::new()),
            op_lock: Mutex::new(()),
            store_lock: Mutex::new(()),
            cache: Mutex::new(ListCache::default()),
            detached: Mutex::new(BTreeSet::new()),
            started: Mutex::new(BTreeSet::new()),
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

    /// The pool `id` runs (server and php-cgi alive). Light: only the
    /// `running` lock, no scan; a dead pool is ended by the next `list`.
    pub fn is_running(&self, id: &str) -> bool {
        self.running_port(id).is_some()
    }

    pub fn stop_all(&self) {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running.clear();
    }

    /// The pools for the "Server Web" page ([`shown_pools`]). With `fresh`
    /// false (the 3 s poll) the scan of the projects is reused for
    /// [`LIST_TTL`]; only the running state (running, pid, port) is read
    /// again every time.
    pub fn list(&self, lang: &str, fresh: bool) -> WebServersView {
        let mut view = self.list_all(lang, fresh);
        let started = self.started_sites();
        let detached = self
            .detached
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        view.pools = shown_pools(view.pools, &started);
        for site in view.pools.iter_mut().flat_map(|p| p.sites.iter_mut()) {
            site.detached = detached.contains(&(site.project.clone(), site.action.clone()));
        }
        view
    }

    fn started_sites(&self) -> BTreeSet<(String, String)> {
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Every pool with every site that may go in it (also those of the
    /// projects not running): what a start, a reload or [`Self::pool_of`]
    /// works out the pool of a site from.
    fn list_all(&self, lang: &str, fresh: bool) -> WebServersView {
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

    /// The pool serving `action_id` of `project`, with that site (its URL
    /// with the live port when the pool runs). Always from a fresh scan:
    /// a cached one may still put the site in the pool it was moved from
    /// (`panel.json` edited by hand, or before an invalidation).
    pub fn pool_of(
        &self,
        project: &str,
        action_id: &str,
        lang: &str,
    ) -> Option<(WebServerPool, AttachedSite)> {
        self.list_all(lang, true)
            .pools
            .into_iter()
            .find_map(|pool| {
                if pool.orphan {
                    return None;
                }
                let site = pool
                    .sites
                    .iter()
                    .find(|s| s.project == project && s.action == action_id)
                    .cloned()?;
                Some((pool, site))
            })
    }

    /// The live port of the pool `id` when it runs (server and php-cgi
    /// alive). Light: only the `running` lock.
    pub fn running_port(&self, id: &str) -> Option<u16> {
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        running
            .get_mut(id)
            .and_then(|r| r.alive().then_some(r.port))
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
        // Running pools the scan does not have (defect 5; an automatic one
        // only until the next scan rebuilds it from its id): shown as they
        // were started, never left running out of sight.
        for (id, r) in running.iter() {
            if !view.pools.iter().any(|p| &p.id == id) {
                view.pools.push(WebServerPool {
                    running: true,
                    pid: Some(r.child.id()),
                    port: r.port,
                    orphan: true,
                    ..r.started.clone()
                });
            }
        }
        drop(running);
        end_dead(dead);
    }
}

#[cfg(test)]
#[path = "../../../test/rust/servers/mod.rs"]
mod tests;
