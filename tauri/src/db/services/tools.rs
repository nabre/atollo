//! The command-line tools of the servers: run with a time limit, logged in
//! as the administrator.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};

use super::{auth, bin_at, Services};
use crate::db::DbEngine;
use crate::env::KitEnv;
use crate::os::jobs;
use crate::router::bins::exe_name;

/// Runs a tool with `input` on its stdin (SQL with passwords: never on the
/// command line) and waits at most `limit`; fails with its error output.
pub(super) fn run_with_input(mut cmd: Command, input: &str, limit: Duration) -> Result<()> {
    use std::io::{Read, Write};
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    let mut child = jobs::spawn(&mut cmd)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let mut err = String::new();
            if let Some(mut e) = child.stderr.take() {
                let _ = e.read_to_string(&mut err);
            }
            return if status.success() {
                Ok(())
            } else {
                Err(anyhow!("{}", err.trim()))
            };
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Runs a tool of a server and waits at most `limit`.
pub(super) fn run_tool(mut cmd: Command, limit: Duration) -> Result<()> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = jobs::spawn(&mut cmd)?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(anyhow!("{:?}: exit {status}", cmd.get_program()))
            };
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Runs a tool of a server with its output appended to `log` (the server's
/// own: MySQL's initialization tells there why it failed) and waits at
/// most `limit`.
pub(super) fn run_logged(mut cmd: Command, log: &std::path::Path, limit: Duration) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)?;
    cmd.stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone()?))
        .stderr(Stdio::from(file));
    let mut child = jobs::spawn(&mut cmd)?;
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                let text = std::fs::read_to_string(log).unwrap_or_default();
                let tail: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
                Err(anyhow!(
                    "exit {status}: {}",
                    tail[tail.len().saturating_sub(3)..].join("\n")
                ))
            };
        }
        if start.elapsed() > limit {
            let _ = child.kill();
            bail!("{:?}: no answer", cmd.get_program());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// The text of a project's own login for a SQL `engine` (only for its
/// tool, then removed): MariaDB's as before, CRLF; MySQL's with LF and TCP.
pub(super) fn project_login_text(
    engine: DbEngine,
    user: &str,
    password: &str,
    port: u16,
) -> String {
    match engine {
        DbEngine::MySql => format!(
            "[client]\nuser={user}\npassword={password}\nhost=127.0.0.1\nport={port}\nprotocol=TCP\n"
        ),
        DbEngine::MariaDb | DbEngine::MongoDb => format!(
            "[client]\r\nuser={user}\r\npassword={password}\r\nhost=127.0.0.1\r\nport={port}\r\n"
        ),
    }
}

impl Services {
    /// A tool of a SQL `engine` (`stem`: [`DbEngine::client_bin`],
    /// [`DbEngine::dump_bin`]…) logged in as `root` (the kit's `[client]`
    /// login), of version `line` (None: the one the server runs) on `port`:
    /// the `[client]` login names the port of the version the server runs
    /// only (PLAN.md §13.2). MySQL reads its login alone
    /// (`--defaults-file`, D-110): MariaDB's `my.ini` in `MYSQL_HOME` and
    /// the PC's option files never reach it.
    pub(crate) fn sql_tool_at(
        &self,
        engine: DbEngine,
        stem: &str,
        env: &KitEnv,
        line: Option<&str>,
        port: u16,
    ) -> Command {
        debug_assert!(engine.is_sql(), "{engine} has no [client] login");
        let mut cmd = jobs::kit_command(bin_at(&self.kit, engine.component(), line, stem), env);
        // It must be the first option; the port from here wins over it.
        match engine {
            DbEngine::MySql => cmd.arg(format!(
                "--defaults-file={}",
                auth::mysql_cnf(&self.kit).display()
            )),
            DbEngine::MariaDb | DbEngine::MongoDb => cmd.arg(format!(
                "--defaults-extra-file={}",
                auth::client_ini(&self.kit).display()
            )),
        };
        cmd.arg(format!("--port={port}"));
        cmd
    }

    /// A tool of a SQL `engine` logged in as the project's own user, which
    /// has rights on its database only: for SQL the kit did not write (a
    /// copy brought from outside), so a `USE other` or `mysql.*` in it
    /// fails. Its login is in a file of the private folder read instead of
    /// every other (`--defaults-file`: root's `my.ini` is not read), removed
    /// after.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn sql_as_project(
        &self,
        engine: DbEngine,
        stem: &str,
        env: &KitEnv,
        line: Option<&str>,
        port: u16,
        user: &str,
        password: &str,
    ) -> Result<(Command, PathBuf)> {
        let program = bin_at(&self.kit, engine.component(), line, stem);
        // The client's own commands off (`\!`, `system`, `source`, `tee`):
        // the SQL may come from elsewhere. A client without the option
        // never gets it (fail closed).
        let help = jobs::kit_command(&program, env)
            .arg("--help")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        let Some(guard) = super::mysql::restore_guard(engine, &help) else {
            bail!(
                "{}: this {engine} client cannot turn its own commands off (`--commands=OFF`, \
                 `--sandbox`): update {engine} to restore a copy",
                program.display()
            );
        };
        let exe = exe_name(self.kit.os(), stem);
        let login = auth::dir(&self.kit).join(format!("{exe}-{}.cnf", std::process::id()));
        auth::write_private_text(&login, &project_login_text(engine, user, password, port))?;
        let mut cmd = jobs::kit_command(program, env);
        // It must be the first option.
        cmd.arg(format!("--defaults-file={}", login.display()))
            .arg(guard);
        Ok((cmd, login))
    }

    /// MongoDB's `stem` ([`DbEngine::dump_bin`], [`DbEngine::restore_bin`])
    /// logged in as `admin`, the password in a config file of the private
    /// folder (never on the command line), removed after.
    pub(super) fn mongo_tool(
        &self,
        stem: &str,
        env: &KitEnv,
        port: u16,
    ) -> Result<(Command, PathBuf)> {
        let admin = auth::load(&self.kit)
            .mongodb_admin
            .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
        let exe = exe_name(self.kit.os(), stem);
        let config = auth::dir(&self.kit).join(format!("{exe}-{}.yaml", std::process::id()));
        auth::write_private_text(&config, &format!("password: {admin}\n"))?;
        let tools = DbEngine::MongoDb.tools_component();
        let mut cmd = jobs::kit_command(bin_at(&self.kit, tools, None, stem), env);
        cmd.arg(format!("--config={}", config.display()))
            .args([
                "--host=127.0.0.1",
                "--username=admin",
                "--authenticationDatabase=admin",
            ])
            .arg(format!("--port={port}"));
        Ok((cmd, config))
    }
}
