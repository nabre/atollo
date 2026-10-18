//! The running pools: Apache, Nginx or Caddy (and PHP's FastCGI server) in
//! the app's Job Object, started, stopped and reloaded; sites attached and
//! detached by their projects.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::net::{SocketAddr, TcpStream};
use std::path::Path;
use std::process::{Child, Stdio};
use std::sync::PoisonError;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};

use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs::{self, Job};
use crate::projects::apache::{self, fwd};
use crate::projects::caddy;
use crate::projects::fastcgi::{self, FcgiAddr};
use crate::projects::nginx::{self, FastCgi};
use crate::projects::run::is_port_free;
use crate::setup::catalog::Catalog;

use super::config::component_dir;
use super::pools::load_store;
use super::{WebServerEngine, WebServerPool, WebServers};

/// How long a start waits for a taken port before calling it used by
/// another program.
const PORT_BUSY_GRACE: Duration = Duration::from_secs(2);

pub(super) struct RunningPool {
    pub(super) child: Child,
    pub(super) port: u16,
    pub(super) fcgi: Option<FastCgi>,
    /// The PHP line the pool was started with (`None`: no PHP).
    pub(super) php_line: Option<String>,
    /// The pool's php-cgi (Nginx with PHP; php-fpm for Caddy off Windows):
    /// when it dies the pool is stopped, never left "running" answering
    /// 502 to every `.php`.
    pub(super) php: Option<Child>,
    /// Caddy's owner-only admin socket, which `caddy reload` talks to
    /// (none: a reload restarts the pool).
    pub(super) admin: Option<caddy::AdminSocket>,
    /// The pool as it was started: what the page shows of it when it is
    /// no longer in the scan (an orphan, defect 5).
    pub(super) started: WebServerPool,
    /// Started from the "Server Web" page or with the app (not by a
    /// project): never stopped because its last project stopped.
    pub(super) by_user: bool,
    /// The projects' actions started on it (`project`, `action`): when the
    /// last one stops, a pool started by them and not kept alive stops.
    pub(super) attached: BTreeSet<(String, String)>,
    pub(super) _job: Job,
}

/// What a reload compares of a running pool with what it should be now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LivePool {
    pub(super) port: u16,
    pub(super) php_line: Option<String>,
    pub(super) fcgi: Option<FastCgi>,
}

/// A reload of a running pool needs a full stop and start (not Nginx's
/// graceful `-s reload` or `caddy reload`, which keep their php-cgi and
/// port): Apache (no
/// graceful signal on Windows), another port, another PHP line, or PHP
/// wanted with no php-cgi running (a manual pool that just got its first
/// PHP site: else every `.php` answers 503 until a manual restart).
pub(super) fn needs_restart(pool: &WebServerPool, live: &LivePool) -> bool {
    pool.engine == WebServerEngine::Apache
        || pool.port != live.port
        || pool.php_line != live.php_line
        || (pool.php_line.is_some() && live.fcgi.is_none())
}

impl RunningPool {
    /// The server, and its php-cgi if it has one, are still running.
    pub(super) fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
            && self
                .php
                .as_mut()
                .is_none_or(|p| matches!(p.try_wait(), Ok(None)))
    }

    /// Ends the server (and, with its Job, the rest), waiting for it so
    /// that its port is being released when this returns.
    pub(super) fn end(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A running pool stops when a project leaves it (PLAN.md §13.1): only one
/// started by projects (`by_user` false), with none of them `left`, and not
/// a profile «Mantieni attivo anche senza progetti» (an automatic pool
/// never is). One started from the page or with the app is the user's to stop.
pub(super) fn stops_when_left(by_user: bool, left: usize, keep_alive: bool) -> bool {
    !by_user && left == 0 && !keep_alive
}

/// Takes out of `running` the pools whose server or php-cgi ended: they
/// are ended (outside the lock) by the caller.
pub(super) fn take_dead(running: &mut BTreeMap<String, RunningPool>) -> Vec<(String, RunningPool)> {
    let dead: Vec<String> = running
        .iter_mut()
        .filter_map(|(id, r)| (!r.alive()).then(|| id.clone()))
        .collect();
    dead.into_iter()
        .filter_map(|id| running.remove(&id).map(|r| (id, r)))
        .collect()
}

/// Ends the pools [`take_dead`] took out, noting why in the log.
pub(super) fn end_dead(dead: Vec<(String, RunningPool)>) {
    for (id, mut r) in dead {
        if matches!(r.child.try_wait(), Ok(None)) {
            crate::log::warn(&format!(
                "server pool {id}: php-cgi ended, the pool is stopped (start it again)"
            ));
        }
        r.end();
    }
}

/// True once `port` can be listened on again (checked every 100 ms), false
/// if it is still taken after `within`: after a stop, Windows may keep the
/// socket of the killed server for a moment.
pub(super) fn wait_port_released(port: u16, within: Duration) -> bool {
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

/// `addr` takes a connection now: a TCP port of 127.0.0.1, or a Unix
/// socket (php-fpm on macOS and Linux).
pub(super) fn fcgi_answers(addr: &FcgiAddr) -> bool {
    match addr {
        FcgiAddr::Tcp(port) => {
            let addr = SocketAddr::from(([127, 0, 0, 1], *port));
            TcpStream::connect_timeout(&addr, Duration::from_millis(80)).is_ok()
        }
        #[cfg(unix)]
        FcgiAddr::Unix(sock) => std::os::unix::net::UnixStream::connect(&sock.path).is_ok(),
        // php-fpm runs on macOS and Linux only; php-cgi listens on TCP.
        #[cfg(not(unix))]
        FcgiAddr::Unix(sock) => sock.path.exists(),
    }
}

/// True once `child` (php-cgi or php-fpm) answers on `addr`, or is still
/// running after `within` (slow, not failed); false if it ended: its
/// FastCGI port is chosen free and let go before it starts, and another
/// program may have taken it in between (defect 9). In that case the port
/// answers at once, before php-cgi fails to bind it: an answer counts only
/// if php-cgi is still running a moment after.
pub(super) fn php_cgi_listening(child: &mut Child, addr: &FcgiAddr, within: Duration) -> bool {
    let begun = Instant::now();
    loop {
        if fcgi_answers(addr) {
            std::thread::sleep(Duration::from_millis(250));
            return matches!(child.try_wait(), Ok(None));
        }
        if !matches!(child.try_wait(), Ok(None)) {
            return false;
        }
        if begun.elapsed() >= within {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// The pool's php-cgi with its FastCGI port, started again on another
/// port (up to 3 times) when it ends at once because its port was taken.
pub(super) fn spawn_php_cgi(
    kit: &Kit,
    env: &KitEnv,
    job: &Job,
) -> (Option<FastCgi>, Option<Child>) {
    spawn_with_retries(|| nginx::spawn_php_cgi_if_present(kit, env, job))
}

/// The FastCGI server of a Caddy pool (php-cgi on Windows, php-fpm with
/// its `fpm.conf` in `pool_dir` and its private socket elsewhere), retried
/// as [`spawn_php_cgi`].
pub(super) fn spawn_fastcgi(
    kit: &Kit,
    env: &KitEnv,
    job: &Job,
    pool_dir: &Path,
    id: &str,
) -> (Option<FastCgi>, Option<Child>) {
    spawn_with_retries(|| fastcgi::spawn_fastcgi_if_present(kit, env, job, pool_dir, id))
}

fn spawn_with_retries(
    mut spawn: impl FnMut() -> Option<(FastCgi, Child)>,
) -> (Option<FastCgi>, Option<Child>) {
    for _ in 0..3 {
        let Some((fcgi, mut child)) = spawn() else {
            break;
        };
        if php_cgi_listening(&mut child, &fcgi.addr, Duration::from_secs(5)) {
            return (Some(fcgi), Some(child));
        }
        crate::log::warn(&format!(
            "PHP's FastCGI server ended at once on {}: trying again",
            fcgi.addr.describe()
        ));
    }
    (None, None)
}

/// The last lines of the pool's error log, or of `stderr.log` (what the
/// server printed before its own log was open: a configuration syntax error).
pub(super) fn pool_error_tail(pool_dir: &Path) -> String {
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
    /// Starts the profiles marked «Avvia con Atollo» (when the app opens),
    /// as if from the "Server Web" page; a failure only goes to the log.
    pub fn autostart(&self, env: &KitEnv, lang: &str) {
        // A fresh list when the app opens: the ports of pools gone since
        // the last session leave `servers.json`.
        self.list(lang, true);
        let ids: Vec<String> = load_store(&self.kit)
            .profiles
            .into_iter()
            .filter(|p| p.autostart)
            .map(|p| p.id)
            .collect();
        for id in ids {
            if let Err(e) = self.start(&id, env, lang) {
                crate::log::warn(&format!("server pool {id}: automatic start: {e:#}"));
            }
        }
    }

    /// The site (`project`, `action`) is served by its pool, which is
    /// started (or reloaded, when the site was left out of it) as needed:
    /// «Avvia» of a project's site. Returns the pool's id and live port,
    /// and the site's ServerName.
    pub fn attach(
        &self,
        project: &str,
        action: &str,
        base_env: &KitEnv,
        lang: &str,
    ) -> Result<(String, u16, String)> {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        let key = (project.to_owned(), action.to_owned());
        let was_detached = self
            .detached
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&key);
        // Before the files are written: a site only detected goes in the
        // pool while its project runs it.
        let was_started = !self
            .started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.clone());
        let undo = || {
            if was_detached {
                self.detached
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .insert(key.clone());
            }
            if !was_started {
                self.started
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .remove(&key);
            }
        };
        let Some((pool, site)) = self.pool_of(project, action, lang) else {
            undo();
            bail!("No server pool serves {project} ({action})");
        };
        // In the running configuration already, unless it was left out (or
        // only detected and not started before) or came after the last start
        // or reload.
        let written = (site.chosen || was_started)
            && self
                .running
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get(&pool.id)
                .is_some_and(|r| {
                    r.started
                        .sites
                        .iter()
                        .any(|s| s.project == project && s.action == action)
                });
        let result = if pool.running {
            if was_detached || !written {
                self.reload_locked(&pool.id, base_env, lang)
            } else {
                Ok(())
            }
        } else {
            self.start_locked(&pool.id, base_env, lang, false)
        };
        if let Err(e) = result {
            undo();
            return Err(e);
        }
        let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
        let r = running
            .get_mut(&pool.id)
            .ok_or_else(|| anyhow!("Server pool not running: {}", pool.id))?;
        r.attached.insert(key);
        Ok((pool.id, r.port, site.server_name))
    }

    /// The project's site (`project`, `action`) stopped: its VirtualHost
    /// leaves the pool `id` with a reload; a pool started by projects that
    /// is left with none stops, unless its profile keeps it alive
    /// (PLAN.md §13.1).
    pub fn detach(
        &self,
        id: &str,
        project: &str,
        action: &str,
        base_env: &KitEnv,
        lang: &str,
    ) -> Result<()> {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        let key = (project.to_owned(), action.to_owned());
        self.detached
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.clone());
        self.started
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&key);
        let (by_user, left) = {
            let mut running = self.running.lock().unwrap_or_else(PoisonError::into_inner);
            let Some(r) = running.get_mut(id) else {
                return Ok(());
            };
            r.attached.remove(&key);
            (r.by_user, r.attached.len())
        };
        let keep_alive = load_store(&self.kit)
            .profiles
            .iter()
            .any(|p| p.id == id && p.keep_alive);
        if stops_when_left(by_user, left, keep_alive) {
            self.stop_locked(id);
            Ok(())
        } else {
            self.reload_locked(id, base_env, lang)
        }
    }

    /// Starts the pool `id` from the "Server Web" page (or with the app): it
    /// then stays on when its projects stop.
    pub fn start(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.start_locked(id, base_env, lang, true)
    }

    /// `by_user`: started from the page or with the app, not by a project.
    pub(super) fn start_locked(
        &self,
        id: &str,
        base_env: &KitEnv,
        lang: &str,
        by_user: bool,
    ) -> Result<()> {
        let view = self.list_all(lang, true);
        let pool = view
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("Server pool not found: {id}"))?;
        if pool.running {
            if by_user {
                if let Some(r) = self
                    .running
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get_mut(id)
                {
                    r.by_user = true;
                }
            }
            return Ok(());
        }
        // A pool just ended (a dead one ended by `list`, outside `op_lock`)
        // may still hold its socket for a moment: not "used by another
        // program" yet.
        if !wait_port_released(pool.port, PORT_BUSY_GRACE) {
            bail!(
                "{}",
                crate::i18n::t(
                    lang,
                    "La porta {0} è già usata da un altro programma: cambiala qui sotto.",
                    &[&pool.port.to_string()]
                )
            );
        }

        // The pool's lines are on disk (defect 4): a missing one would start
        // the default line in its place, a "PHP 7.4" pool running PHP 8.
        let catalog = Catalog::get();
        for (cid, name, line) in [
            (
                pool.engine.as_str(),
                pool.engine.label(),
                pool.server_line.as_deref(),
            ),
            ("php", "PHP", pool.php_line.as_deref()),
        ] {
            let (Some(line), Some(c)) = (line, catalog.find(cid)) else {
                continue;
            };
            if !crate::versions::is_present(&self.kit, c, line) {
                bail!(
                    "{}",
                    crate::i18n::t(
                        lang,
                        "{0} {1} non è installato nel kit: installa questa versione o scegline un'altra.",
                        &[name, line]
                    )
                );
            }
        }

        let mut env = base_env.clone();
        if let Some(php_line) = &pool.php_line {
            let php_dir = component_dir(&self.kit, "php", Some(php_line));
            env.set_php(&self.kit.data(), &php_dir);
        }
        env.set("CP_PROJECTS_FWD", fwd(&self.kit.projects()));
        let lan = env.get("ATOLLO_ALLOW_LAN") == Some("1");

        let pool_dir = self.kit.data().join("servers").join(&pool.id);
        let _ = fs::remove_file(pool_dir.join("error.log"));
        let _ = fs::remove_file(pool_dir.join("stderr.log"));

        let job = Job::new()?;
        let mut admin = None;
        let (mut cmd, fcgi, php) = match pool.engine {
            WebServerEngine::Apache => {
                let apache_dir = component_dir(&self.kit, "apache", pool.server_line.as_deref());
                let exe = apache::httpd_exe(&self.kit, &apache_dir);
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
                let exe = nginx::nginx_exe(&self.kit, &nginx_dir);
                if !exe.is_file() {
                    bail!("Nginx is not installed");
                }
                env.set("CP_NGINX", nginx_dir.display().to_string());
                env.set("CP_NGINX_FWD", fwd(&nginx_dir));
                let (fcgi, php) = if pool.php_line.is_some() {
                    match spawn_php_cgi(&self.kit, &env, &job) {
                        // A PHP pool never runs without its PHP: every
                        // `.php` would answer 503, with nothing watched.
                        (None, _) => bail!("php-cgi did not start (see the log)"),
                        started => started,
                    }
                } else {
                    (None, None)
                };
                let conf_path = self.write_pool_configs(&pool, fcgi.as_ref(), lan)?;
                let mut cmd = jobs::kit_command(&exe, &env);
                cmd.arg("-p")
                    .arg(&pool_dir)
                    .arg("-c")
                    .arg(&conf_path)
                    .arg("-g")
                    .arg("daemon off;");
                (cmd, fcgi, php)
            }
            WebServerEngine::Caddy => {
                let caddy_dir = component_dir(&self.kit, "caddy", pool.server_line.as_deref());
                let exe = caddy::caddy_exe(&self.kit, &caddy_dir);
                if !exe.is_file() {
                    bail!("Caddy is not installed");
                }
                env.set("CP_CADDY", caddy_dir.display().to_string());
                let (fcgi, php) = if pool.php_line.is_some() {
                    match spawn_fastcgi(&self.kit, &env, &job, &pool_dir, &pool.id) {
                        // A PHP pool never runs without its PHP (as Nginx).
                        (None, _) => bail!("PHP's FastCGI server did not start (see the log)"),
                        started => started,
                    }
                } else {
                    (None, None)
                };
                admin = caddy::admin_socket(&self.kit, &pool.id);
                let conf_path = self.write_pool_configs_with(
                    &pool,
                    fcgi.as_ref(),
                    lan,
                    admin.as_ref().map(|a| a.path.as_path()),
                )?;
                let mut cmd = caddy::caddy_command(&exe, &pool_dir, &env);
                cmd.args(caddy::run_args(&conf_path));
                caddy::with_key(&mut cmd, fcgi.as_ref());
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
                    fcgi,
                    php_line: pool.php_line.clone(),
                    php,
                    admin,
                    started: pool,
                    by_user,
                    attached: BTreeSet::new(),
                    _job: job,
                },
            );
        Ok(())
    }

    pub fn stop(&self, id: &str) {
        let _op = self.op_lock.lock().unwrap_or_else(PoisonError::into_inner);
        self.stop_locked(id);
    }

    pub(super) fn stop_locked(&self, id: &str) {
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

    pub(super) fn reload_locked(&self, id: &str, base_env: &KitEnv, lang: &str) -> Result<()> {
        let view = self.list_all(lang, true);
        let pool = view
            .pools
            .into_iter()
            .find(|p| p.id == id)
            .ok_or_else(|| anyhow!("Server pool not found: {id}"))?;
        if pool.orphan {
            // Nothing to reload it with: its combination is gone.
            bail!(
                "{}",
                crate::i18n::t(
                    lang,
                    "Questo pool non corrisponde più ai progetti e ai profili: fermalo.",
                    &[]
                )
            );
        }
        let lan = base_env.get("ATOLLO_ALLOW_LAN") == Some("1");
        let live = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .map(|r| LivePool {
                port: r.port,
                php_line: r.php_line.clone(),
                fcgi: r.fcgi.clone(),
            });
        let Some(live) = live else {
            // Not running: its files are ready for the next start.
            self.write_pool_configs(&pool, None, lan)?;
            return Ok(());
        };

        let admin = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            // Its folder checked again just before `caddy reload` connects:
            // one put in its place is never talked to (a restart instead).
            .and_then(|r| r.admin.as_ref())
            .filter(|a| {
                let ok = a.still_private();
                if !ok {
                    crate::log::warn(&format!(
                        "server pool {id}: {}: no longer a private folder, the pool is restarted",
                        a.path.display()
                    ));
                }
                ok
            })
            .map(|a| a.path.clone());
        if !needs_restart(&pool, &live) {
            // Nginx or Caddy with the same PHP and port: a graceful reload,
            // the sites that stay are served without a pause.
            let pool_dir = self.kit.data().join("servers").join(&pool.id);
            let status = match pool.engine {
                WebServerEngine::Caddy => match admin {
                    Some(admin) => {
                        let conf_path = self.write_pool_configs_with(
                            &pool,
                            live.fcgi.as_ref(),
                            lan,
                            Some(&admin),
                        )?;
                        let caddy_dir =
                            component_dir(&self.kit, "caddy", pool.server_line.as_deref());
                        let exe = caddy::caddy_exe(&self.kit, &caddy_dir);
                        caddy::caddy_command(&exe, &pool_dir, base_env)
                            .args(caddy::reload_args(&conf_path, &admin))
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                    }
                    None => Err(std::io::Error::other("no admin endpoint")),
                },
                _ => {
                    let conf_path = self.write_pool_configs(&pool, live.fcgi.as_ref(), lan)?;
                    let nginx_dir = component_dir(&self.kit, "nginx", pool.server_line.as_deref());
                    let exe = nginx::nginx_exe(&self.kit, &nginx_dir);
                    jobs::kit_command(&exe, base_env)
                        .arg("-p")
                        .arg(&pool_dir)
                        .arg("-c")
                        .arg(&conf_path)
                        .args(["-s", "reload"])
                        .status()
                }
            };
            if status.is_ok_and(|s| s.success()) {
                if let Some(r) = self
                    .running
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .get_mut(id)
                {
                    r.started = pool;
                }
                return Ok(());
            }
        }

        // A clean restart: Apache (no graceful signal on Windows), a pool
        // whose PHP or port changed, or a failed Nginx or Caddy reload. Who started
        // it and its projects carry over to the new process.
        let (by_user, attached) = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(id)
            .map(|r| (r.by_user, r.attached.clone()))
            .unwrap_or_default();
        self.stop_locked(id);
        // start_locked reports the port as taken if it is still not free.
        wait_port_released(live.port, Duration::from_secs(5));
        self.start_locked(id, base_env, lang, by_user)?;
        if let Some(r) = self
            .running
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get_mut(id)
        {
            r.attached = attached;
        }
        Ok(())
    }
}
