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
use crate::setup::catalog::Catalog;

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

/// The data of a version that has it yet (MariaDB's `mysql`, MySQL's data
/// dictionary `mysql.ibd`, MongoDB's `WiredTiger`).
fn has_data(engine: DbEngine, dir: &std::path::Path) -> bool {
    match engine {
        DbEngine::MariaDb => dir.join("mysql").is_dir(),
        DbEngine::MySql => dir.join("mysql.ibd").is_file(),
        DbEngine::MongoDb => dir.join("WiredTiger").is_file(),
    }
}

/// The versions of `engine` with data in `data\db\<id>\<line>`, newest
/// first: for MariaDB, those whose data has not moved to MySQL yet (D-117).
pub(crate) fn lines_with_data(kit: &Kit, engine: DbEngine) -> Vec<String> {
    let mut lines: Vec<String> = fs::read_dir(data_root(kit, engine.id()))
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (crate::versions::valid_line(&name) && has_data(engine, &e.path())).then_some(name)
        })
        .collect();
    lines.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
    lines
}

/// The data of `line` the first time it starts: a copy of that of the
/// nearest older version, if the server can take it (MariaDB from any
/// older one; MongoDB from the same or the previous major only). The old
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
    // MariaDB opens the data of any older version, MySQL that of the LTS
    // before (the kit's lines are LTS: 8.4 to 9.7) and upgrades it at its
    // first start; MongoDB only that of the same or the previous major.
    let one_major_at_a_time = match engine {
        DbEngine::MariaDb | DbEngine::MySql => false,
        DbEngine::MongoDb => true,
    };
    if one_major_at_a_time
        && wanted.first().copied().unwrap_or(0) > key.first().copied().unwrap_or(0) + 1
    {
        crate::log::warn(&format!(
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
    if target.exists() {
        crate::net::archive::remove_dir(&target)?;
    }
    copy_dir(&root.join(&from), &target)?;
    crate::log::info(&format!("{id}: data of {from} copied for {line}"));
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
            .find_internal(engine.component())
            .and_then(|c| crate::versions::default_line(kit, c))
        else {
            continue;
        };
        let moving = kit.data().join("db").join(format!("{id}.devkit-move"));
        let moved = fs::rename(&root, &moving)
            .and_then(|()| fs::create_dir_all(&root))
            .and_then(|()| fs::rename(&moving, root.join(&line)));
        match moved {
            Ok(()) => crate::log::info(&format!(
                "{id}: data moved to {}",
                root.join(&line).display()
            )),
            Err(e) => {
                if moving.exists() && !root.join(&line).exists() {
                    let _ = fs::remove_dir(&root);
                    let _ = fs::rename(&moving, &root);
                }
                crate::log::warn(&format!("{id}: data not moved ({e})"));
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
                let mut secrets = auth::load(kit);
                let password = match secrets.mysql_root.clone() {
                    Some(p) => p,
                    None => {
                        let p = auth::new_password()?;
                        secrets.mysql_root = Some(p.clone());
                        auth::save(kit, &secrets)?;
                        p
                    }
                };
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
            // A socket that does not fit its data folder goes to the kit's
            // own private folder of the PC's temporary one, never /tmp.
            let fallback = mysql::socket_dir(kit.system_temp());
            let socket = mysql::socket_path(kit.os(), &data, port, fallback.as_deref())?;
            if let Some(dir) = socket.as_ref().and_then(|s| s.parent()) {
                if dir != data {
                    fs::create_dir_all(dir)?;
                    crate::router::fs::restrict_dir_to_owner(dir)?;
                }
            }
            let cnf = mysql::server_cnf(kit.os(), &basedir, &data, port, socket.as_deref());
            if fs::read_to_string(data.join("my.cnf")).ok().as_deref() != Some(cnf.as_str()) {
                crate::fsx::write_atomic(&data.join("my.cnf"), cnf.as_bytes())?;
            }
            return Ok((data, fresh));
        }
        match engine.install_db_bin() {
            Some(install) if fresh => {
                // mariadb-install-db wants a folder that does not exist yet.
                if data.is_dir() && fs::read_dir(&data)?.next().is_none() {
                    fs::remove_dir(&data)?;
                }
                if let Some(parent) = data.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut cmd =
                    jobs::kit_command(bin_at(&self.kit, engine.component(), line, install), env);
                cmd.arg(format!("--datadir={}", data.display()))
                    .arg(format!("--port={port}"));
                run_tool(cmd, Duration::from_secs(120)).context(install)?;
            }
            _ => {}
        }
        fs::create_dir_all(&data)?;
        Ok((data, fresh))
    }

    /// After the data of `from` was copied for `line`: MongoDB takes the
    /// features of the new major (its next one needs it); MariaDB's system
    /// tables are brought up to date by hand (`mariadb-upgrade`, D-73).
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
            DbEngine::MariaDb => {
                crate::log::info(&format!(
                    "mariadb {line}: data of {from}; run mariadb-upgrade to update the system tables"
                ));
                return;
            }
            // MySQL upgrades the data dictionary and the system tables
            // itself at the first start of the new version.
            DbEngine::MySql => {
                crate::log::info(&format!("mysql {line}: data of {from} upgraded"));
                return;
            }
            DbEngine::MongoDb => {}
        }
        let mut cmd = jobs::kit_command(client_at(&self.kit, engine, None), env);
        cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
            .arg(port.to_string())
            .args(["--eval", auth::MONGO_FCV_JS])
            .env(
                "DEVKIT_DB_ADMIN",
                auth::load(&self.kit).mongodb_admin.unwrap_or_default(),
            )
            .env("DEVKIT_DB_FCV", line);
        match run_tool(cmd, Duration::from_secs(60)) {
            Ok(_) => crate::log::info(&format!("mongodb {line}: feature compatibility {line}")),
            Err(e) => crate::log::warn(&format!(
                "mongodb {line}: setFeatureCompatibilityVersion: {e:#}"
            )),
        }
    }
}

#[cfg(test)]
#[path = "../../../../test/rust/db/services/data.rs"]
mod tests;
