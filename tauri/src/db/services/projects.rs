//! The versions the projects' databases are on: their ports and logins, and the deletion of a project.

use std::path::PathBuf;

use anyhow::{Context, Result};

use super::{auth, installed, line_dir, line_port, server_line, server_lines, Services};
use crate::db::DbEngine;
use crate::env::KitEnv;
use crate::kit::Kit;

/// The version of `engine` the project's database is on: the one chosen
/// for it (`lines` of its `db.json`), otherwise the one the server runs.
pub fn project_line(kit: &Kit, project: &str, engine: &str) -> Option<String> {
    auth::project_line(kit, project, engine)
        .unwrap_or_else(|e| {
            crate::log::warn(&format!("{project}: {engine}: {e:#}"));
            None
        })
        .or_else(|| server_line(kit, engine))
}

/// The SQL server the project's database is on, when it has one (a
/// password stored for MySQL).
fn project_sql_db(kit: &Kit, project: &str) -> Option<DbEngine> {
    auth::project_passwords(kit, project)
        .contains_key(DbEngine::MySql.id())
        .then_some(DbEngine::MySql)
}

/// The port of the project's SQL database, when it has one: the `DB_PORT`
/// of its programs (D-102).
pub fn project_db_port(kit: &Kit, project: &str) -> Option<u16> {
    let id = project_sql_db(kit, project)?.id();
    Some(line_port(
        kit,
        id,
        project_line(kit, project, id).as_deref(),
    ))
}

/// The login of the project's programs when its SQL database is on another
/// version than the server's, written at that version's start (`mysql` of
/// its scripts then logs in on its port): `ATOLLO_MYSQL_CNF` (read by the
/// kit's launchers, `setup/kit_files.rs`) and `CP_MYSQL`, the folder of that
/// version. Empty: the kit's own (`env/`), on the server's port.
pub fn project_client_env(kit: &Kit, project: &str) -> Vec<(&'static str, PathBuf)> {
    let Some(engine) = project_sql_db(kit, project) else {
        return Vec::new();
    };
    let id = engine.id();
    let Some(line) = auth::project_line(kit, project, id).ok().flatten() else {
        return Vec::new();
    };
    if Some(&line) == server_line(kit, id).as_ref() {
        return Vec::new();
    }
    let cnf = auth::mysql_line_cnf(kit, &line);
    if cnf.is_file() {
        vec![
            ("ATOLLO_MYSQL_CNF", cnf),
            ("CP_MYSQL", line_dir(kit, id, Some(&line))),
        ]
    } else {
        Vec::new()
    }
}

/// The installed versions of `id`, other than the one the server runs, that
/// a project's database is on (`lines` of its `db.json`, with a database
/// created): those the automatic start starts beside the server. Newest
/// first; a version chosen and then uninstalled is left out.
pub fn lines_in_use(kit: &Kit, id: &str) -> Vec<String> {
    let main = server_line(kit, id);
    let wanted: std::collections::BTreeSet<String> = crate::projects::state::all(kit)
        .into_iter()
        .filter(|p| auth::project_passwords(kit, p).contains_key(id))
        .filter_map(|p| auth::project_line(kit, &p, id).ok().flatten())
        .filter(|l| Some(l) != main.as_ref())
        .collect();
    server_lines(kit, id)
        .into_iter()
        .filter(|l| wanted.contains(l))
        .collect()
}

/// The versions of `engine` to drop the project's database from: every one
/// it was created on (`used` of its `db.json`) and the one it is on now,
/// in that order, each once.
pub fn project_drop_lines(kit: &Kit, project: &str, engine: &str) -> Vec<String> {
    let mut lines = auth::project_used_lines(kit, project, engine).unwrap_or_else(|e| {
        crate::log::warn(&format!("{project}: {engine}: {e:#}"));
        Vec::new()
    });
    if let Some(now) = project_line(kit, project, engine) {
        lines.push(now);
    }
    let mut seen = std::collections::BTreeSet::new();
    lines.retain(|l| seen.insert(l.clone()));
    lines
}

/// The kit's servers for the deletion of `project` (D-89),
/// started with `env` when stopped: every version its databases are on
/// ([`project_drop_lines`]); a version no longer installed is skipped and
/// named in the log.
pub struct ProjectDrops<'a> {
    pub services: &'a Services,
    pub env: KitEnv,
    pub project: String,
}

impl crate::projects::ProjectDatabases for ProjectDrops<'_> {
    fn installed(&self, engine: &str) -> bool {
        installed(&self.services.kit, engine)
    }

    fn drop_db(&self, engine: &str, database: &str) -> Result<()> {
        let kit = &self.services.kit;
        let lines = project_drop_lines(kit, &self.project, engine);
        if lines.is_empty() {
            return self.services.db_drop_at(engine, None, database, &self.env);
        }
        let installed = server_lines(kit, engine);
        for line in &lines {
            if installed.contains(line) {
                self.services
                    .db_drop_at(engine, Some(line), database, &self.env)
                    .with_context(|| format!("{engine} {line}"))?;
            } else {
                crate::log::warn(&format!(
                    "{}: {engine} {line} not installed: database and user {database} left there",
                    self.project
                ));
            }
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../../../test/rust/db/services/projects.rs"]
mod tests;
