//! The SQL and mongosh scripts the servers get: MySQL's `root`, the projects' users, MongoDB's admin.

/// SQL that gives MySQL's `root` its password while `mysqld --initialize`
/// makes the data folder (its `--init-file`: `root@localhost` only, with a
/// random expired password until then), and the same `root` for
/// 127.0.0.1: the kit connects over TCP, and a TCP login from 127.0.0.1
/// matches `root@localhost` only when the server resolves that address to
/// the name `localhost`, which the kit does not rely on.
pub fn mysql_root_sql(password: &str) -> String {
    format!(
        "ALTER USER 'root'@'localhost' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
         CREATE USER IF NOT EXISTS 'root'@'127.0.0.1' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
         ALTER USER 'root'@'127.0.0.1' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
         GRANT ALL PRIVILEGES ON *.* TO 'root'@'127.0.0.1' WITH GRANT OPTION;\n\
         GRANT PROXY ON ''@'' TO 'root'@'127.0.0.1' WITH GRANT OPTION;\n\
         FLUSH PRIVILEGES;\n"
    )
}

/// SQL that creates (or brings back to `password`) a project's database and
/// user on MySQL (D-110), with every right on that database only:
/// `caching_sha2_password`, the default of 8.4 and the only one left in
/// 9.x, for `localhost` and `127.0.0.1` (the server listens on 127.0.0.1
/// only). In a database-level GRANT `_` is a wildcard (`my_app` would also
/// match `my2app`): escaped there.
pub fn project_sql(name: &str, password: &str) -> String {
    let mut sql = format!("CREATE DATABASE IF NOT EXISTS `{name}` CHARACTER SET utf8mb4;\n");
    let pattern = name.replace('_', "\\_");
    for host in ["localhost", "127.0.0.1"] {
        sql.push_str(&format!(
            "CREATE USER IF NOT EXISTS '{name}'@'{host}' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
             ALTER USER '{name}'@'{host}' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
             GRANT ALL PRIVILEGES ON `{pattern}`.* TO '{name}'@'{host}';\n"
        ));
    }
    sql.push_str("FLUSH PRIVILEGES;\n");
    sql
}

/// mongosh script for a project's user (names and passwords from the
/// environment: `ATOLLO_DB_ADMIN`, `ATOLLO_DB_NAME`, `ATOLLO_DB_PASSWORD`).
pub const MONGO_PROJECT_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); \
    const n = process.env.ATOLLO_DB_NAME, p = process.env.ATOLLO_DB_PASSWORD; \
    const d = db.getSiblingDB(n); \
    if (d.getUser(n)) { d.updateUser(n, { pwd: p }); } \
    else { d.createUser({ user: n, pwd: p, roles: [{ role: 'dbOwner', db: n }] }); }";

/// SQL that drops a project's database and user (D-89); both
/// may be missing already.
pub fn project_drop_sql(name: &str) -> String {
    format!(
        "DROP DATABASE IF EXISTS `{name}`;\n\
         DROP USER IF EXISTS '{name}'@'localhost', '{name}'@'127.0.0.1', '{name}'@'::1';\n\
         FLUSH PRIVILEGES;\n"
    )
}

/// mongosh script that drops a project's user and database (names and the
/// administrator's password from the environment, as `MONGO_PROJECT_JS`).
pub const MONGO_DROP_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); \
    const n = process.env.ATOLLO_DB_NAME, d = db.getSiblingDB(n); \
    if (d.getUser(n)) { d.dropUser(n); } \
    d.dropDatabase();";

/// mongosh script for the first `admin` (MongoDB's localhost exception: it
/// works while the server has no users).
pub const MONGO_ADMIN_JS: &str = "db.getSiblingDB('admin').createUser({ user: 'admin', \
    pwd: process.env.ATOLLO_DB_PASSWORD, roles: ['root'] })";

pub const MONGO_SHUTDOWN_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); a.shutdownServer()";

/// mongosh script after data of the previous major version was copied:
/// the features of the new one (`ATOLLO_DB_FCV`, `9.0`), as MongoDB's
/// upgrade guide asks; the next major needs it.
pub const MONGO_FCV_JS: &str = "const a = db.getSiblingDB('admin'); \
    a.auth('admin', process.env.ATOLLO_DB_ADMIN); \
    a.adminCommand({ setFeatureCompatibilityVersion: process.env.ATOLLO_DB_FCV, confirm: true })";
