//! The command-line tools of the servers: run with a time limit, logged in
//! as the administrator.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Result};

use super::{auth, bin, Services};
use crate::env::KitEnv;
use crate::os::jobs;

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

impl Services {
    /// A tool of MariaDB logged in as `root` (the kit's `[client]` login).
    pub(crate) fn mariadb_tool(&self, exe: &str, env: &KitEnv) -> Command {
        let mut cmd = jobs::kit_command(bin(&self.kit, "mariadb", exe), env);
        // It must be the first option.
        cmd.arg(format!(
            "--defaults-extra-file={}",
            auth::client_ini(&self.kit).display()
        ));
        cmd
    }

    /// mongodump/mongorestore logged in as `admin`, the password in a config
    /// file of the private folder (never on the command line), removed after.
    pub(super) fn mongo_tool(
        &self,
        exe: &str,
        env: &KitEnv,
        port: u16,
    ) -> Result<(Command, PathBuf)> {
        let admin = auth::load(&self.kit)
            .mongodb_admin
            .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
        let config = auth::dir(&self.kit).join(format!("{exe}-{}.yaml", std::process::id()));
        auth::write_private_text(&config, &format!("password: {admin}\n"))?;
        let mut cmd = jobs::kit_command(self.kit.apps().join(r"mongodb-tools\bin").join(exe), env);
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
