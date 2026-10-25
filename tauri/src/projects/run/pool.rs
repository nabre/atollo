//! The sites served by a shared pool (`servers/`) instead of a server of
//! their own.

use std::collections::HashSet;
use std::process::Child;
use std::time::Duration;

use crate::common::i18n::t as tr;
use crate::common::time::now_ms;
use crate::detect::Action;

use super::{key_of, Entry, Event, RunInfo, Runner, State};

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

impl Runner {
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
                    vm_run: None,
                    run: self.next_run(),
                },
            );
            info
        };
        let line = format!(
            "[Atollo] {}",
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

    /// Waits for a project's web server (`child`) and its `php` (php-cgi or
    /// php-fpm). When PHP ends first the action is stopped and ends in error, never
    /// left running with 502 for every `.php` (BUG-V3-01). The server's exit
    /// code, `None` when it was stopped for its php-cgi.
    pub(super) fn wait_with_php(
        &self,
        key: &str,
        run: u64,
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
                        .filter(|e| e.run == run && !e.stopped)
                        .and_then(|e| e.job.take())
                };
                // Stopped by the user: the job already ended both.
                let Some(job) = job else {
                    return child.wait().ok().and_then(|s| s.code());
                };
                let code = status
                    .code()
                    .map_or_else(|| "?".to_owned(), |c| c.to_string());
                self.append_to(
                    key,
                    Some(run),
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

    /// The pool serving the running action `key`, with its project and
    /// action ids (`None`: not served by a pool, or not running).
    pub fn pooled(&self, key: &str) -> Option<(String, String, String)> {
        let entries = self.entries();
        let e = entries.get(key)?;
        let pool = e.pool.clone().filter(|_| e.info.state == State::Running)?;
        Some((pool, e.info.project.clone(), e.info.action.clone()))
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
}
