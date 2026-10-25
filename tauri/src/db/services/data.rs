//! The data of each version a server runs, `data\db\<id>\<line>` (D-81):
//! created at the first start (MySQL with `mysqld --initialize --init-file`
//! and the kit's `my.cnf`, `mysql.rs`), copied forward from an older
//! version, moved from the single folder of an older kit.

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Context, Result};

#[cfg(test)]
use super::bin;
use super::tools::{run_logged, run_tool};
use super::{auth, bin_at, client_at, line_dir, log_path, mysql, server_line, Services};
use crate::db::DbEngine;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs;
use crate::projects::caddy;
use crate::setup::Catalog;

/// The folder of MySQL's socket when it does not fit in the data folder
/// ([`mysql::socket_dir`]): in the first base of
/// [`caddy::socket_bases`] whose private folder of sockets
/// ([`caddy::socket_parent`]: the user's own, checked for owner, links and
/// rights, in a parent nobody else can rename in) gives a short enough
/// path. Made owner-only from the start and checked again
/// ([`caddy::is_private_dir`]) before it is used: one another user made
/// first, or a link, is refused (BUG-V7-11). `None` (in the log) when no
/// base gives one: the server does not start rather than use a shared one.
fn private_socket_dir(kit: &Kit, port: u16) -> Option<PathBuf> {
    for base in caddy::socket_bases(kit) {
        let Some(parent) = caddy::socket_parent(&base) else {
            continue;
        };
        let dir = mysql::socket_dir(&parent);
        if mysql::fallback_socket(&dir, port).is_none() {
            continue;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            let _ = fs::DirBuilder::new().mode(0o700).create(&dir);
        }
        #[cfg(not(unix))]
        {
            let _ = fs::DirBuilder::new().create(&dir);
        }
        if caddy::is_private_dir(&dir) {
            return Some(dir);
        }
        crate::common::log::warn(&format!(
            "mysql: {}: not a private folder, not used for the socket",
            dir.display()
        ));
    }
    None
}

/// `data\db\<id>`: the data of every version of the server.
fn data_root(kit: &Kit, id: &str) -> PathBuf {
    kit.data().join("db").join(id)
}

/// The data of the version the server runs, `data\db\<id>\<line>`: data
/// of a newer version does not open in an older one (D-81).
pub fn data_dir(kit: &Kit, id: &str) -> PathBuf {
    data_dir_at(kit, id, None)
}

/// The data of version `line` (None: the one the server runs).
pub fn data_dir_at(kit: &Kit, id: &str, line: Option<&str>) -> PathBuf {
    match line.map(str::to_owned).or_else(|| server_line(kit, id)) {
        Some(line) => data_root(kit, id).join(line),
        None => data_root(kit, id),
    }
}

/// The data of a version that has it yet (MySQL's data
/// dictionary `mysql.ibd`, MongoDB's `WiredTiger`).
fn has_data(engine: DbEngine, dir: &std::path::Path) -> bool {
    match engine {
        DbEngine::MySql => dir.join("mysql.ibd").is_file(),
        DbEngine::MongoDb => dir.join("WiredTiger").is_file(),
    }
}

/// The data of `line` the first time it starts: a copy of that of the
/// nearest older version, if the server can take it (MongoDB from the same
/// or the previous major only). The old
/// data stays as it was, for its own version. Returns the version copied.
/// Refused while that version is among `running` (D-102: versions
/// run side by side): its files are in use.
pub(super) fn seed(kit: &Kit, id: &str, line: &str, running: &[String]) -> Result<Option<String>> {
    let engine = DbEngine::parse(id)?;
    let root = data_root(kit, id);
    let target = root.join(line);
    if has_data(engine, &target) {
        return Ok(None);
    }
    let wanted = crate::install::version_key(line);
    let mut older: Vec<(Vec<u64>, String)> = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let key = crate::install::version_key(&name);
            (crate::versions::valid_line(&name) && key < wanted && has_data(engine, &e.path()))
                .then_some((key, name))
        })
        .collect();
    older.sort();
    let Some((key, from)) = older.pop() else {
        return Ok(None);
    };
    // MySQL opens the data of the LTS before (the kit's lines are LTS: 8.4 to 9.7) and upgrades it at its
    // first start; MongoDB only that of the same or the previous major.
    let one_major_at_a_time = match engine {
        DbEngine::MySql => false,
        DbEngine::MongoDb => true,
    };
    if one_major_at_a_time
        && wanted.first().copied().unwrap_or(0) > key.first().copied().unwrap_or(0) + 1
    {
        crate::common::log::warn(&format!(
            "{id} {line}: the data of {from} cannot jump more than one major version; \
             starting with empty data (the {from} data stays)"
        ));
        return Ok(None);
    }
    if running.contains(&from) {
        bail!(
            "{id} {line} starts the first time with a copy of the data of {from}: \
             stop {id} {from} first"
        );
    }
    // Copied beside, then renamed (BUG-V6-09): a copy cut short (a full or
    // pulled out stick) is never a data folder, whose `mysql.ibd` would
    // skip the copy at the next start and open what is half there. What a
    // cut copy left goes first.
    let partial = root.join(format!("{line}.partial"));
    if partial.exists() {
        crate::net::archive::remove_dir(&partial)?;
    }
    if target.exists() {
        crate::net::archive::remove_dir(&target)?;
    }
    if let Err(e) = copy_dir(&root.join(&from), &partial) {
        let _ = crate::net::archive::remove_dir(&partial);
        return Err(e.context(format!("{id} {line}: the copy of the data of {from}")));
    }
    fs::rename(&partial, &target).with_context(|| target.display().to_string())?;
    crate::common::log::info(&format!("{id}: data of {from} copied for {line}"));
    Ok(Some(from))
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)?.flatten() {
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target).with_context(|| target.display().to_string())?;
        }
    }
    Ok(())
}

/// Data of an older kit, `data\db\<id>` itself: into the folder of the
/// installed version, `data\db\<id>\<line>`. At start (versions::migrate).
pub fn migrate_data(kit: &Kit) {
    for engine in DbEngine::ALL {
        let id = engine.id();
        let root = data_root(kit, id);
        if !has_data(engine, &root) {
            continue;
        }
        let Some(line) = Catalog::get()
            .find(engine.component())
            .and_then(|c| crate::versions::default_line(kit, c))
        else {
            continue;
        };
        let moving = kit.data().join("db").join(format!("{id}.atollo-move"));
        let moved = fs::rename(&root, &moving)
            .and_then(|()| fs::create_dir_all(&root))
            .and_then(|()| fs::rename(&moving, root.join(&line)));
        match moved {
            Ok(()) => crate::common::log::info(&format!(
                "{id}: data moved to {}",
                root.join(&line).display()
            )),
            Err(e) => {
                if moving.exists() && !root.join(&line).exists() {
                    let _ = fs::remove_dir(&root);
                    let _ = fs::rename(&moving, &root);
                }
                crate::common::log::warn(&format!("{id}: data not moved ({e})"));
            }
        }
    }
}

impl Services {
    /// Creates the data folder the first time; true when it is new (no
    /// password set in it yet).
    pub(super) fn prepare(
        &self,
        engine: DbEngine,
        line: Option<&str>,
        env: &KitEnv,
        port: u16,
    ) -> Result<(PathBuf, bool)> {
        let data = data_dir_at(&self.kit, engine.id(), line);
        let fresh = !has_data(engine, &data);
        if engine == DbEngine::MySql {
            let basedir = line_dir(&self.kit, engine.component(), line);
            let kit = &self.kit;
            mysql::ensure_server_libs(kit.os(), kit.arch(), &basedir)?;
            if fresh {
                // `--initialize` wants a folder that is missing or empty: the
                // kit's own my.cnf of a failed start goes; anything else is
                // never removed (it may be someone's data).
                if data.is_dir() {
                    let _ = fs::remove_file(data.join("my.cnf"));
                    if fs::read_dir(&data)?.next().is_some() {
                        bail!(
                            "{}: not a MySQL data folder (no mysql.ibd) and not empty: \
                             move its files away, then start MySQL again",
                            data.display()
                        );
                    }
                    fs::remove_dir(&data)?;
                }
                if let Some(parent) = data.parent() {
                    fs::create_dir_all(parent)?;
                }
                // root's password, stored before any server hears of it
                // (every version has the same administrator), set by the
                // initialization itself: root is never without one.
                // A `secrets.json` that cannot be read stops here: a new
                // password over it would lose the other versions' root
                // (SEC-V6-01).
                let password = auth::update_secrets(kit, |secrets| {
                    Ok(match secrets.mysql_root.clone() {
                        Some(p) => p,
                        None => {
                            let p = auth::new_password()?;
                            secrets.mysql_root = Some(p.clone());
                            p
                        }
                    })
                })?;
                let init = auth::dir(kit).join(format!("mysql-init-{}.sql", std::process::id()));
                auth::write_private_text(&init, &auth::mysql_root_sql(&password))?;
                let mut cmd = jobs::kit_command(
                    bin_at(kit, engine.component(), line, engine.server_bin()),
                    env,
                );
                cmd.args(mysql::init_args(&basedir, &data, &init));
                let log = log_path(kit, engine.id(), line);
                let done = run_logged(cmd, &log, Duration::from_secs(300));
                let _ = fs::remove_file(&init);
                if let Err(e) = done {
                    // A half-made folder (it was missing or empty): never
                    // started with a root of unknown password.
                    if data.is_dir() {
                        let _ = crate::net::archive::remove_dir(&data);
                    }
                    return Err(e.context("mysqld --initialize"));
                }
            }
            fs::create_dir_all(&data)?;
            // A socket that does not fit its data folder goes to the user's
            // private folder of sockets, never /tmp itself; the folder is
            // only looked for (and made) when it is needed.
            let socket = match mysql::socket_path(kit.os(), &data, port, None) {
                Ok(socket) => socket,
                Err(_) => {
                    let fallback = private_socket_dir(kit, port);
                    mysql::socket_path(kit.os(), &data, port, fallback.as_deref())?
                }
            };
            let cnf = mysql::server_cnf(kit.os(), &basedir, &data, port, socket.as_deref());
            if fs::read_to_string(data.join("my.cnf")).ok().as_deref() != Some(cnf.as_str()) {
                crate::common::fsx::write_atomic(&data.join("my.cnf"), cnf.as_bytes())?;
            }
            return Ok((data, fresh));
        }
        fs::create_dir_all(&data)?;
        Ok((data, fresh))
    }

    /// After the data of `from` was copied for `line`: MongoDB takes the
    /// features of the new major (its next one needs it).
    pub(super) fn after_copy(
        &self,
        engine: DbEngine,
        env: &KitEnv,
        port: u16,
        from: &str,
        line: &str,
    ) {
        let major = |l: &str| l.split('.').next().unwrap_or("").to_owned();
        if major(from) == major(line) {
            return;
        }
        match engine {
            // MySQL upgrades the data dictionary and the system tables
            // itself at the first start of the new version.
            DbEngine::MySql => {
                crate::common::log::info(&format!("mysql {line}: data of {from} upgraded"));
                return;
            }
            DbEngine::MongoDb => {}
        }
        let mut cmd = jobs::kit_command(client_at(&self.kit, engine, None), env);
        cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
            .arg(port.to_string())
            .args(["--eval", auth::MONGO_FCV_JS])
            .env(
                "ATOLLO_DB_ADMIN",
                auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
            )
            .env("ATOLLO_DB_FCV", line);
        match run_tool(cmd, Duration::from_secs(60)) {
            Ok(_) => {
                crate::common::log::info(&format!("mongodb {line}: feature compatibility {line}"))
            }
            Err(e) => crate::common::log::warn(&format!(
                "mongodb {line}: setFeatureCompatibilityVersion: {e:#}"
            )),
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/db/services/data.rs"]
mod tests;
