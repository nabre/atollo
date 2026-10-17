//! The database servers the kit runs (PLAN-PIATTAFORME.md §4.3): one enum
//! instead of the ids `"mariadb"` / `"mongodb"` matched as strings, so a new
//! engine is a compile error at every place that has to know it.
//!
//! The ids stay the strings saved in `services.json`, `db.json`,
//! `secrets.json` and sent to the interface (serde, [`DbEngine::id`]); the
//! binaries are stems, the router adds `.exe` on Windows
//! (`router::bins::db_bin_in`).

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

/// A database server the kit runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DbEngine {
    MariaDb,
    MongoDb,
}

impl DbEngine {
    /// Every engine, in the order of the Database page.
    pub const ALL: [DbEngine; 2] = [DbEngine::MariaDb, DbEngine::MongoDb];

    /// The id saved in the kit's files and used by the interface.
    pub const fn id(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongodb",
        }
    }

    /// The engine of `id`; None for anything else (`sqlite`, `apache`…).
    pub fn from_id(id: &str) -> Option<DbEngine> {
        DbEngine::ALL.into_iter().find(|e| e.id() == id)
    }

    /// [`DbEngine::from_id`], or the error the servers give for an unknown id.
    pub fn parse(id: &str) -> Result<DbEngine> {
        match DbEngine::from_id(id) {
            Some(e) => Ok(e),
            None => bail!("unknown server {id}"),
        }
    }

    /// The catalog component of the server (versioned: one folder per line).
    pub const fn component(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongodb",
        }
    }

    /// The port of the version the server runs when none is set.
    pub const fn default_port(self) -> u16 {
        match self {
            DbEngine::MariaDb => 3306,
            DbEngine::MongoDb => 27017,
        }
    }

    /// The server program, in [`DbEngine::component`].
    pub const fn server_bin(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadbd",
            DbEngine::MongoDb => "mongod",
        }
    }

    /// The program that creates a new data folder before the first start
    /// (in [`DbEngine::component`]); None: the server creates it itself.
    pub const fn install_db_bin(self) -> Option<&'static str> {
        match self {
            DbEngine::MariaDb => Some("mariadb-install-db"),
            DbEngine::MongoDb => None,
        }
    }

    /// The interactive client, in [`DbEngine::client_component`].
    pub const fn client_bin(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongosh",
        }
    }

    /// The component the client is in: the server's own line, or a
    /// separate unversioned one (`mongosh`).
    pub const fn client_component(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongosh",
        }
    }

    /// The program that exports a database, in [`DbEngine::tools_component`].
    pub const fn dump_bin(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb-dump",
            DbEngine::MongoDb => "mongodump",
        }
    }

    /// The program that puts an export back, in [`DbEngine::tools_component`]
    /// (the SQL client reads the dump on stdin).
    pub const fn restore_bin(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongorestore",
        }
    }

    /// The component of the dump and restore programs.
    pub const fn tools_component(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "mariadb",
            DbEngine::MongoDb => "mongodb-tools",
        }
    }

    /// The program that shuts the server down; None: the client does it
    /// (mongosh `shutdownServer`).
    pub const fn admin_bin(self) -> Option<&'static str> {
        match self {
            DbEngine::MariaDb => Some("mariadb-admin"),
            DbEngine::MongoDb => None,
        }
    }

    /// The end of the name of a copy of a project's database (D-78):
    /// `<name>-<stamp><ext>` in its backup folder.
    pub const fn backup_ext(self) -> &'static str {
        match self {
            DbEngine::MariaDb => ".sql",
            DbEngine::MongoDb => ".archive.gz",
        }
    }

    /// The engine a copy named `file` is for ([`DbEngine::backup_ext`]);
    /// None: not a database copy.
    pub fn of_backup(file: &str) -> Option<DbEngine> {
        DbEngine::ALL
            .into_iter()
            .find(|e| file.ends_with(e.backup_ext()))
    }

    /// A SQL server (MySQL protocol, `mysql://` URLs, `DB_*` of Laravel).
    pub const fn is_sql(self) -> bool {
        match self {
            DbEngine::MariaDb => true,
            DbEngine::MongoDb => false,
        }
    }

    /// The administrator the kit logs in as (D-74).
    pub const fn admin_user(self) -> &'static str {
        match self {
            DbEngine::MariaDb => "root",
            DbEngine::MongoDb => "admin",
        }
    }

    /// Databases and users of the server itself: a project named after one
    /// would create, alter or drop it (`DROP DATABASE mysql`, the `root`
    /// user; MongoDB's `admin` holds every user of the server).
    pub const fn reserved_names(self) -> &'static [&'static str] {
        match self {
            DbEngine::MariaDb => &[
                "mysql",
                "information_schema",
                "performance_schema",
                "sys",
                "root",
                "mariadb_sys",
            ],
            DbEngine::MongoDb => &["admin", "local", "config"],
        }
    }

    /// The server's own databases, marked in the content explorer (D-79).
    pub const fn system_databases(self) -> &'static [&'static str] {
        match self {
            DbEngine::MariaDb => &["information_schema", "mysql", "performance_schema", "sys"],
            DbEngine::MongoDb => &["admin", "config", "local"],
        }
    }

    /// The administrator's connection string (no password in it).
    pub fn admin_url(self, port: u16) -> String {
        match self {
            DbEngine::MariaDb => format!("mysql://root@127.0.0.1:{port}/"),
            DbEngine::MongoDb => format!("mongodb://admin@127.0.0.1:{port}/?authSource=admin"),
        }
    }

    /// A project's connection string: its database and user `name`.
    pub fn project_url(self, name: &str, password: &str, port: u16) -> String {
        match self {
            DbEngine::MariaDb => format!("mysql://{name}:{password}@127.0.0.1:{port}/{name}"),
            DbEngine::MongoDb => {
                format!("mongodb://{name}:{password}@127.0.0.1:{port}/{name}?authSource={name}")
            }
        }
    }
}

impl std::fmt::Display for DbEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

#[cfg(test)]
#[path = "../../../test/rust/db/engine.rs"]
mod tests;
