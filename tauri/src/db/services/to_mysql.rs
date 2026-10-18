//! The kit's servers as the move of MariaDB's data to MySQL uses them
//! (`db::migrate_engine`, D-117): MariaDB on its own port, MySQL aside on
//! a port the system gives, the tools logged in as `root` (D-74, D-110).

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{bail, Result};

use super::tools::{run_query, run_with_files, run_with_input};
use super::{server_lines, Services};
use crate::db::migrate_engine::Servers;
use crate::db::DbEngine;
use crate::env::KitEnv;
use crate::kit::Kit;
use crate::os::jobs;

/// How long a dump or a restore may take: a development kit's databases,
/// on a stick too.
const BIG: Duration = Duration::from_secs(4 * 3600);
const SHORT: Duration = Duration::from_secs(120);

/// A port the system gives on 127.0.0.1, free now.
fn free_port() -> Result<u16> {
    let l = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    Ok(l.local_addr()?.port())
}

/// [`Services`] with the environment of the app, for the move.
pub struct ToMysql<'a> {
    pub services: &'a Services,
    pub env: KitEnv,
}

impl ToMysql<'_> {
    fn client(&self, engine: DbEngine, line: &str, stem: &str) -> Result<std::process::Command> {
        let (port, env) = self.services.running_at(engine, Some(line))?;
        Ok(self
            .services
            .sql_tool_at(engine, stem, &env, Some(line), port))
    }
}

impl Servers for ToMysql<'_> {
    fn kit(&self) -> &Kit {
        &self.services.kit
    }

    fn installed_lines(&self, engine: DbEngine) -> Vec<String> {
        server_lines(&self.services.kit, engine.id())
    }

    fn start(&self, engine: DbEngine, line: &str) -> Result<bool> {
        if self.running(engine, line) {
            return Ok(true);
        }
        match engine {
            DbEngine::MySql => {
                self.services
                    .start_aside(engine, line, &self.env, free_port()?)?;
            }
            DbEngine::MariaDb | DbEngine::MongoDb => {
                self.services.start_engine(engine, Some(line), &self.env)?;
            }
        }
        Ok(false)
    }

    fn stop(&self, engine: DbEngine, line: &str) {
        self.services.stop_engine(engine, Some(line));
    }

    fn running(&self, engine: DbEngine, line: &str) -> bool {
        self.services.running_at(engine, Some(line)).is_ok()
    }

    fn sql(&self, engine: DbEngine, line: &str, sql: &str) -> Result<()> {
        run_with_input(self.client(engine, line, engine.client_bin())?, sql, SHORT)
    }

    fn query(&self, engine: DbEngine, line: &str, sql: &str) -> Result<String> {
        let mut cmd = self.client(engine, line, engine.client_bin())?;
        cmd.args(["--batch", "--skip-column-names"]);
        run_query(cmd, sql, BIG)
    }

    fn dump(&self, line: &str, databases: &[String], to: &Path) -> Result<()> {
        let engine = DbEngine::MariaDb;
        let mut cmd = self.client(engine, line, engine.dump_bin())?;
        // One consistent snapshot with routines, triggers and events; binary
        // columns as hexadecimal, so no character set touches them.
        cmd.args([
            "--single-transaction",
            "--routines",
            "--events",
            "--triggers",
            "--hex-blob",
            "--default-character-set=utf8mb4",
            "--databases",
        ])
        .args(databases);
        run_with_files(cmd, None, Some(to), BIG)
    }

    fn restore(&self, line: &str, from: &Path) -> Result<()> {
        let engine = DbEngine::MySql;
        let (port, env) = self.services.running_at(engine, Some(line))?;
        let program = super::bin_at(&self.services.kit, engine.component(), Some(line), "mysql");
        // The client's own commands off, as for a copy (D-110): a `\!` in a
        // row of the dump runs nothing.
        let help = jobs::kit_command(&program, &env)
            .arg("--help")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let Some(guard) = super::mysql::restore_guard(engine, &help) else {
            bail!(
                "{}: this MySQL client cannot turn its own commands off: update MySQL",
                program.display()
            );
        };
        let mut cmd =
            self.services
                .sql_tool_at(engine, engine.client_bin(), &env, Some(line), port);
        cmd.arg(guard).arg("--default-character-set=utf8mb4");
        run_with_files(cmd, Some(from), None, BIG)
    }

    fn serve(&self, line: &str) -> Result<()> {
        self.services
            .start_engine(DbEngine::MySql, Some(line), &self.env)
            .map(|_| ())
    }
}
