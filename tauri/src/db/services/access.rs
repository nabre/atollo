//! Passwords (D-74): the administrator's, set at the first start, and the
//! projects' own databases and users.

use std::time::Duration;

use anyhow::{anyhow, Context, Result};

use super::tools::run_with_input;
use super::{auth, bin, Services};
use crate::env::KitEnv;
use crate::os::jobs;

pub(super) fn admin_user(id: &str) -> &'static str {
    match id {
        "mongodb" => "admin",
        _ => "root",
    }
}

pub(super) fn admin_password(secrets: &auth::Secrets, id: &str) -> Option<String> {
    match id {
        "mongodb" => secrets.mongodb_admin.clone(),
        _ => secrets.mariadb_root.clone(),
    }
}

impl Services {
    /// The administrator's password, set on a new data folder or on one made
    /// before D-74 (no password yet), and MariaDB's `[client]` login.
    pub(super) fn secure(&self, id: &str, env: &KitEnv, port: u16, fresh: bool) -> Result<()> {
        let mut secrets = auth::load(&self.kit);
        let known = admin_password(&secrets, id);
        let password = match &known {
            Some(p) => p.clone(),
            None => auth::new_password()?,
        };
        if fresh || known.is_none() {
            match id {
                "mongodb" => {
                    let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), env);
                    cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                        .arg(port.to_string())
                        .args(["--eval", auth::MONGO_ADMIN_JS])
                        .env("DEVKIT_DB_PASSWORD", &password);
                    run_with_input(cmd, "", Duration::from_secs(60)).context(
                        "MongoDB already has users that NEXKORA does not know: \
                         its administrator could not be created",
                    )?;
                    secrets.mongodb_admin = Some(password.clone());
                }
                _ => {
                    // No password yet: log in without one, ignoring my.ini.
                    let mut cmd = jobs::kit_command(bin(&self.kit, id, "mariadb.exe"), env);
                    cmd.args([
                        "--no-defaults",
                        "--user=root",
                        "--host=127.0.0.1",
                        "--protocol=tcp",
                    ])
                    .arg(format!("--port={port}"));
                    run_with_input(
                        cmd,
                        &auth::root_password_sql(&password),
                        Duration::from_secs(60),
                    )
                    .context("MariaDB: setting the password of root")?;
                    secrets.mariadb_root = Some(password.clone());
                }
            }
            auth::save(&self.kit, &secrets)?;
            crate::log::info(&format!("{id}: administrator password set"));
        }
        if id == "mariadb" {
            auth::write_client_ini(&self.kit, port, &password)?;
        }
        Ok(())
    }

    /// The project's databases already created, at the servers' ports.
    pub fn project_dbs(&self, project: &str) -> Vec<auth::ProjectDb> {
        auth::project_passwords(&self.kit, project)
            .iter()
            .map(|(engine, pw)| auth::project_db(engine, project, pw, self.port(engine)))
            .collect()
    }

    /// Creates (or repairs) the project's database and user on a running
    /// server; the password stays the same once generated. It is stored
    /// before the server gets it, and an unreadable `db.json` stops here: a
    /// failed call is repaired by running again, never a password lost.
    pub fn project_db_create(&self, project: &str, engine: &str) -> Result<auth::ProjectDb> {
        let (port, env) = self.running_server(engine)?;
        let password = auth::ensure_project_password(&self.kit, project, engine)
            .context("the passwords of the project's databases")?;
        let name = auth::db_name(project);
        match engine {
            "mongodb" => {
                let admin = auth::load(&self.kit)
                    .mongodb_admin
                    .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
                let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(port.to_string())
                    .args(["--eval", auth::MONGO_PROJECT_JS])
                    .env("DEVKIT_DB_ADMIN", admin)
                    .env("DEVKIT_DB_NAME", &name)
                    .env("DEVKIT_DB_PASSWORD", &password);
                run_with_input(cmd, "", Duration::from_secs(60))?;
            }
            _ => {
                let cmd = self.mariadb_tool("mariadb.exe", &env);
                run_with_input(
                    cmd,
                    &auth::project_sql(&name, &password),
                    Duration::from_secs(60),
                )?;
            }
        }
        Ok(auth::project_db(engine, project, &password, port))
    }

    /// Drops the project's database and user from `engine` (PLAN-DATA.md
    /// WP-5), as the administrator: the server is started first when it is
    /// stopped, and stopped again after. Database and user may be missing.
    pub fn project_db_drop(&self, project: &str, engine: &str, env: &KitEnv) -> Result<()> {
        let was_running = self.running_server(engine).is_ok();
        self.start(engine, env)?;
        let result = self.drop_on_running(project, engine);
        if !was_running {
            self.stop(engine);
        }
        result
    }

    fn drop_on_running(&self, project: &str, engine: &str) -> Result<()> {
        let (port, env) = self.running_server(engine)?;
        let name = auth::db_name(project);
        match engine {
            "mongodb" => {
                let admin = auth::load(&self.kit)
                    .mongodb_admin
                    .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
                let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(port.to_string())
                    .args(["--eval", auth::MONGO_DROP_JS])
                    .env("DEVKIT_DB_ADMIN", admin)
                    .env("DEVKIT_DB_NAME", &name);
                run_with_input(cmd, "", Duration::from_secs(60))?;
            }
            _ => {
                let cmd = self.mariadb_tool("mariadb.exe", &env);
                run_with_input(cmd, &auth::project_drop_sql(&name), Duration::from_secs(60))?;
            }
        }
        crate::log::info(&format!("{engine}: database and user {name} dropped"));
        Ok(())
    }
}
