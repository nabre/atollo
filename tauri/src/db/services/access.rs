//! Passwords (D-74): the administrator's, set at the first start, and the
//! projects' own databases and users.

use std::time::Duration;

use anyhow::{anyhow, bail, Context, Result};

use super::tools::run_with_input;
use super::{auth, bin, bin_at, check_line, line_port, project_line, Services, SERVERS};
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
    /// before D-74 (no password yet), and MariaDB's `[client]` login (for
    /// the version the server runs, `main`; the others get the same
    /// password, every version of a server has one administrator).
    pub(super) fn secure(
        &self,
        id: &str,
        line: Option<&str>,
        main: bool,
        env: &KitEnv,
        port: u16,
        fresh: bool,
    ) -> Result<()> {
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
                        .env("ATOLLO_DB_PASSWORD", &password);
                    run_with_input(cmd, "", Duration::from_secs(60)).context(
                        "MongoDB already has users that Atollo does not know: \
                         its administrator could not be created",
                    )?;
                    secrets.mongodb_admin = Some(password.clone());
                }
                _ => {
                    // No password yet: log in without one, ignoring my.ini.
                    let mut cmd =
                        jobs::kit_command(bin_at(&self.kit, id, line, "mariadb.exe"), env);
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
        // Every MariaDB tool reads `client.ini` (`mariadb_tool_at`, with its
        // own `--port`): written for the server's port, also when another
        // version is the first one ever started.
        if id == "mariadb" && main {
            auth::write_client_ini(&self.kit, port, &password)?;
        } else if id == "mariadb" {
            if !auth::client_ini(&self.kit).is_file() {
                auth::write_client_ini(&self.kit, line_port(&self.kit, id, None), &password)?;
            }
            // The login of this version on its port: `MYSQL_HOME` of the
            // projects on it (`project_mysql_home`).
            if let Some(l) = line {
                auth::write_line_client_ini(&self.kit, l, port, &password)?;
            }
        }
        Ok(())
    }

    /// The project's databases already created, at the ports of the
    /// versions they are on. A database whose name cannot be resolved
    /// ([`auth::project_db_name`]) is left out, with a warning.
    pub fn project_dbs(&self, project: &str) -> Vec<auth::ProjectDb> {
        auth::project_passwords(&self.kit, project)
            .iter()
            .filter_map(|(engine, pw)| {
                let name = auth::project_db_name(&self.kit, project, engine)
                    .map_err(|e| crate::log::warn(&format!("{project}: {engine}: {e:#}")))
                    .ok()?;
                let line = project_line(&self.kit, project, engine);
                let port = line_port(&self.kit, engine, line.as_deref());
                Some(auth::project_db(engine, &name, pw, port))
            })
            .collect()
    }

    /// Starts the version of `engine` the project's database is on.
    pub fn start_for(&self, project: &str, engine: &str, env: &KitEnv) -> Result<u16> {
        let line = project_line(&self.kit, project, engine);
        self.start_line(engine, line.as_deref(), env)
    }

    /// The version of `engine` for the project's database (None: the one
    /// the server runs), in its `db.json` (PLAN.md §13.2). A project that
    /// has a database already gets it, with the same name and password,
    /// on the version chosen (started for it); its data stays on the
    /// version it was on (a copy and a restore move it).
    pub fn project_db_line(
        &self,
        project: &str,
        engine: &str,
        line: Option<&str>,
        env: &KitEnv,
    ) -> Result<Option<auth::ProjectDb>> {
        if !SERVERS.contains(&engine) {
            bail!("unknown server {engine}");
        }
        if let Some(l) = line {
            check_line(&self.kit, engine, l)?;
        }
        let previous = auth::project_line(&self.kit, project, engine)?;
        let has_db = auth::try_project_password(&self.kit, project, engine)?.is_some();
        if has_db {
            // A database created before the versions were kept: the one it
            // is on now, for the deletion of the project.
            if let Some(on) = project_line(&self.kit, project, engine) {
                auth::add_project_used_line(&self.kit, project, engine, &on)?;
            }
        }
        auth::set_project_line(&self.kit, project, engine, line)?;
        if !has_db {
            return Ok(None);
        }
        let moved = self
            .start_for(project, engine, env)
            .and_then(|_| self.project_db_create(project, engine));
        if moved.is_err() {
            // The database and its user are still on the version it was
            // on: `db.json` goes back to it, so DB_PORT, `.env`, export and
            // restore keep finding them.
            if let Err(e) = auth::set_project_line(&self.kit, project, engine, previous.as_deref())
            {
                crate::log::warn(&format!("{project}: {engine}: version not restored: {e:#}"));
            }
        }
        moved.map(Some)
    }

    /// Creates (or repairs) the project's database and user on the running
    /// version it is on; name and password stay the same once generated.
    /// They are stored before the server gets them, and an unreadable
    /// `db.json`, a name of the server's own or one another project uses
    /// stop here (`auth::ensure_project_db`): a failed call is repaired by
    /// running again, never a password lost nor another project's user
    /// altered.
    pub fn project_db_create(&self, project: &str, engine: &str) -> Result<auth::ProjectDb> {
        let line = project_line(&self.kit, project, engine);
        let (port, env) = self.running_at(engine, line.as_deref())?;
        let (name, password) = auth::ensure_project_db(&self.kit, project, engine)
            .context("the passwords of the project's databases")?;
        match engine {
            "mongodb" => {
                let admin = auth::load(&self.kit)
                    .mongodb_admin
                    .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
                let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(port.to_string())
                    .args(["--eval", auth::MONGO_PROJECT_JS])
                    .env("ATOLLO_DB_ADMIN", admin)
                    .env("ATOLLO_DB_NAME", &name)
                    .env("ATOLLO_DB_PASSWORD", &password);
                run_with_input(cmd, "", Duration::from_secs(60))?;
            }
            _ => {
                let cmd = self.mariadb_tool_at("mariadb.exe", &env, line.as_deref(), port);
                run_with_input(
                    cmd,
                    &auth::project_sql(&name, &password),
                    Duration::from_secs(60),
                )?;
            }
        }
        // The deletion of the project drops it from every version it is on.
        if let Some(l) = &line {
            if let Err(e) = auth::add_project_used_line(&self.kit, project, engine, l) {
                crate::log::warn(&format!("{project}: {engine} {l}: version not kept: {e:#}"));
            }
        }
        Ok(auth::project_db(engine, &name, &password, port))
    }

    /// Drops the project's database and user from `engine` (PLAN-DATA.md
    /// WP-5), as the administrator: the server is started first when it is
    /// stopped, and stopped again after. Database and user may be missing.
    /// Their name is the project's ([`auth::project_db_name`]); refused when
    /// another project of the kit uses it too.
    #[allow(dead_code)] // the tests; `projects::delete` resolves the names first
    pub fn project_db_drop(&self, project: &str, engine: &str, env: &KitEnv) -> Result<()> {
        let name = auth::project_db_name(&self.kit, project, engine)?;
        auth::check_name_free(&self.kit, project, engine, &name)?;
        let line = project_line(&self.kit, project, engine);
        self.db_drop_at(engine, line.as_deref(), &name, env)
    }

    /// Drops the database and user `name` from version `line` of `engine`
    /// (None: the one the server runs), a name already resolved and checked
    /// by the caller (`projects::delete`); a server's own name or one that
    /// is not a name is refused here as well.
    pub fn db_drop_at(
        &self,
        engine: &str,
        line: Option<&str>,
        name: &str,
        env: &KitEnv,
    ) -> Result<()> {
        if !auth::is_valid_name(name) || auth::is_reserved(engine, name) {
            bail!("{engine}: {name} is not a project's database: not dropped");
        }
        let was_running = self.running_at(engine, line).is_ok();
        self.start_line(engine, line, env)?;
        let result = self.drop_on_running(engine, line, name);
        if !was_running {
            self.stop_line(engine, line);
        }
        result
    }

    fn drop_on_running(&self, engine: &str, line: Option<&str>, name: &str) -> Result<()> {
        let (port, env) = self.running_at(engine, line)?;
        match engine {
            "mongodb" => {
                let admin = auth::load(&self.kit)
                    .mongodb_admin
                    .ok_or_else(|| anyhow!("MongoDB has no administrator yet"))?;
                let mut cmd = jobs::kit_command(bin(&self.kit, "mongosh", "mongosh.exe"), &env);
                cmd.args(["--quiet", "--host", "127.0.0.1", "--port"])
                    .arg(port.to_string())
                    .args(["--eval", auth::MONGO_DROP_JS])
                    .env("ATOLLO_DB_ADMIN", admin)
                    .env("ATOLLO_DB_NAME", name);
                run_with_input(cmd, "", Duration::from_secs(60))?;
            }
            _ => {
                let cmd = self.mariadb_tool_at("mariadb.exe", &env, line, port);
                run_with_input(cmd, &auth::project_drop_sql(name), Duration::from_secs(60))?;
            }
        }
        crate::log::info(&format!("{engine}: database and user {name} dropped"));
        Ok(())
    }
}
