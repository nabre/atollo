//! MariaDB's data moved to MySQL (DECISIONS.md D-117), once, at the first
//! start of a kit that has MariaDB data: MySQL is the kit's only SQL server.
//!
//! For each version of MariaDB with data (`data\db\mariadb\<line>`), one at
//! a time, the version of the server first: started on its own port (the
//! sites' one) and made read-only for the projects' users, the
//! databases dumped one by one (`mariadb-dump --routines --events
//! --triggers`, the server's own databases, users included, left out),
//! each dump filtered for MySQL (`filter`, which stops rather than guess),
//! a MySQL version started aside on a port of its own, the projects' users
//! created with their passwords of `db.json`, each dump restored by an
//! account that reaches its database only (the project's user, or one made
//! for the restore and dropped after: never `root`), then databases,
//! tables, views, routines, triggers, events and the rows of every table
//! compared. Then the switch, last:
//! MariaDB and the MySQL used aside stop, `services.json`, `secrets.json`
//! and the projects' `db.json` are copied (`<file>.pre-mysql`) and
//! rewritten (`mariadb` → `mysql`, MySQL on MariaDB's ports),
//! `data\db\mariadb` becomes `data\db\mariadb.pre-mysql` (the dumps in it),
//! MySQL starts where MariaDB ran. MariaDB's programs go at the next start
//! (`versions::disk`), its data stays until the user deletes it.
//!
//! - **Nothing is lost.** MariaDB's data is never written by the move; a
//!   step that fails stops it with the error and the dump kept, MariaDB
//!   active as before (read-only off), the configuration as it was (the
//!   copies put back). The step is tried again at the next start or from
//!   the Database page.
//! - **Repeatable.** Each run starts the versions again from their dump:
//!   MariaDB may have taken writes since an earlier run. The databases an
//!   earlier, interrupted run restored on MySQL (`restored`, saved before
//!   the restore) are dropped first, and only those: a database MySQL
//!   already has is a conflict, never overwritten.
//! - **Resumed.** Once the switch begins (`switching`), MariaDB does not
//!   start again (`Services::start_engine`): an interrupted switch ends at
//!   the next start from where it stopped, every rewrite done once.
//! - Passwords never reach the log or the state file.

pub mod filter;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};

use super::{auth, DbEngine};
use crate::kit::Kit;

/// The servers as the move uses them; the app's are `db::services`
/// (`services/to_mysql.rs`), the tests' a fake.
pub trait Servers {
    fn kit(&self) -> &Kit;
    /// The installed versions of `engine`.
    fn installed_lines(&self, engine: DbEngine) -> Vec<String>;
    /// `engine` version `line` running; for MySQL aside, on a port of its
    /// own, when it does not run yet. True when it was running already.
    fn start(&self, engine: DbEngine, line: &str) -> Result<bool>;
    fn stop(&self, engine: DbEngine, line: &str);
    fn running(&self, engine: DbEngine, line: &str) -> bool;
    /// SQL as the administrator; `query` returns the rows, tab separated.
    fn sql(&self, engine: DbEngine, line: &str, sql: &str) -> Result<()>;
    fn query(&self, engine: DbEngine, line: &str, sql: &str) -> Result<String>;
    /// MariaDB's `database` alone dumped into `to` (no `CREATE DATABASE`,
    /// no `USE`).
    fn dump(&self, line: &str, database: &str, to: &Path) -> Result<()>;
    /// The file `from` read by MySQL's client into `database`, logged in as
    /// `user` (who reaches that database only), its own commands off.
    fn restore(
        &self,
        line: &str,
        database: &str,
        user: &str,
        password: &str,
        from: &Path,
    ) -> Result<()>;
    /// After the switch: MySQL `line` on its own port, for the sites.
    fn serve(&self, line: &str) -> Result<()>;
}

/// Where the move is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    /// The versions are copied (or not yet).
    #[default]
    Lines,
    /// MariaDB stopped for good: the configuration is being rewritten.
    Switching,
    Done,
}

/// A version of MariaDB with data, and where it goes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LineState {
    /// The version of MySQL that takes its data.
    pub mysql: String,
    /// Its port, which MySQL takes.
    pub port: u16,
    /// Running (or started with the app) before the move: MySQL starts
    /// there after it.
    #[serde(default)]
    pub serve: bool,
    /// The databases this move restored on MySQL (saved before the
    /// restore): dropped before another try, never any other.
    #[serde(default)]
    pub restored: Vec<String>,
    /// Copied and checked in this run.
    #[serde(default)]
    pub done: bool,
}

/// `data\app\db\mysql-migration.json`: no password in it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct State {
    #[serde(default)]
    pub phase: Phase,
    /// MariaDB version → its move.
    #[serde(default)]
    pub lines: BTreeMap<String, LineState>,
    /// The version the MySQL server runs after the move.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub main: Option<String>,
    /// MySQL had data of its own before the move: its version and ports
    /// stay, MariaDB's databases join it.
    #[serde(default)]
    pub mysql_had_data: bool,
    /// The `.pre-mysql` copies are made (the switch began).
    #[serde(default)]
    pub copied: bool,
    /// The projects whose `db.json` is copied and rewritten.
    #[serde(default)]
    pub projects: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// The dump of the step that failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dump: Option<String>,
}

/// What a run did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// No MariaDB data: nothing to move.
    Nothing,
    Done,
}

/// The server's own databases: never dumped (the users are in `mysql`).
const SYSTEM: [&str; 4] = ["information_schema", "mysql", "performance_schema", "sys"];

/// The suffix of the copies made before the switch.
pub const COPY: &str = "pre-mysql";

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());
/// The kits (their data folders) whose move runs now in this process.
static MOVING: Mutex<BTreeSet<PathBuf>> = Mutex::new(BTreeSet::new());

fn moving_set() -> std::sync::MutexGuard<'static, BTreeSet<PathBuf>> {
    MOVING
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// The move of `kit` runs now (in this process).
pub fn moving(kit: &Kit) -> bool {
    moving_set().contains(&kit.data())
}

pub fn state_path(kit: &Kit) -> PathBuf {
    auth::dir(kit).join("mysql-migration.json")
}

pub fn load(kit: &Kit) -> State {
    crate::fsx::read_json(&state_path(kit)).unwrap_or_default()
}

fn save(kit: &Kit, state: &State) -> Result<()> {
    auth::write_private_text(&state_path(kit), &serde_json::to_string_pretty(state)?)
}

/// The switch began and has not ended: MariaDB must not start (its writes
/// would not reach MySQL).
pub fn switching(kit: &Kit) -> bool {
    load(kit).phase == Phase::Switching
}

fn mariadb_root(kit: &Kit) -> PathBuf {
    kit.data().join("db").join(DbEngine::MariaDb.id())
}

/// `data\db\mariadb.pre-mysql`: MariaDB's data after the move, until the
/// user deletes it.
pub fn kept_dir(kit: &Kit) -> PathBuf {
    kit.data()
        .join("db")
        .join(format!("{}.{COPY}", DbEngine::MariaDb.id()))
}

/// The dumps of a move, inside MariaDB's data (they go with it).
pub fn dumps_dir(kit: &Kit) -> PathBuf {
    mariadb_root(kit).join("dumps")
}

/// The versions of MariaDB whose data has not moved, newest first.
pub fn pending_lines(kit: &Kit) -> Vec<String> {
    super::services::lines_with_data(kit, DbEngine::MariaDb)
}

/// MySQL's versions the kit installs, newest first (`sha256ByLine`).
pub fn mysql_offered() -> Vec<String> {
    let mut lines: Vec<String> = crate::install::mysql_pins().keys().cloned().collect();
    lines.sort_by_key(|l| std::cmp::Reverse(crate::install::version_key(l)));
    lines
}

/// Where each MariaDB version goes (D-117): the version of the server
/// (`main`) to MySQL's (`mysql_main`), the others newest first to the
/// other MySQL versions of `offered`; with more MariaDB versions than
/// MySQL ones, the oldest share the last MySQL version.
pub fn plan_lines(
    mariadb: &[String],
    main: Option<&str>,
    mysql_main: &str,
    offered: &[String],
) -> Vec<(String, String)> {
    let mut order: Vec<&String> = mariadb
        .iter()
        .filter(|l| Some(l.as_str()) == main)
        .collect();
    order.extend(mariadb.iter().filter(|l| Some(l.as_str()) != main));
    let mut targets: Vec<&str> = vec![mysql_main];
    targets.extend(
        offered
            .iter()
            .map(String::as_str)
            .filter(|l| *l != mysql_main),
    );
    order
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            let m = targets[i.min(targets.len() - 1)];
            (l.clone(), m.to_owned())
        })
        .collect()
}

/// The names of a `--batch` result's first column, unescaped.
fn names_of(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| unescape(l.split('\t').next().unwrap_or_default()))
        .filter(|l| !l.is_empty())
        .collect()
}

/// A value of `--batch` output (`\\`, `\t`, `\n`, `\0` escaped).
fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('t') => out.push('\t'),
            Some('n') => out.push('\n'),
            Some('0') => out.push('\0'),
            Some(o) => out.push(o),
            None => out.push('\\'),
        }
    }
    out
}

/// A SQL string literal.
pub fn sql_str(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "''"))
}

/// A SQL name.
pub fn sql_name(value: &str) -> String {
    format!("`{}`", value.replace('`', "``"))
}

/// ` CHARACTER SET x COLLATE y` of a database from MariaDB's
/// `default_character_set_name`, `default_collation_name` (a `--batch`
/// row), in MySQL's names; nothing for what is not a plain name.
pub fn charset_of(row: &str) -> String {
    let plain = |s: &str| {
        !s.is_empty()
            && s.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    };
    let mut cols = row.trim().split('\t').map(|c| {
        let c = c.trim().to_ascii_lowercase();
        filter::mysql_name(&c).unwrap_or(c)
    });
    match (cols.next(), cols.next()) {
        (Some(cs), Some(co)) if plain(&cs) && plain(&co) => {
            format!(" CHARACTER SET {cs} COLLATE {co}")
        }
        (Some(cs), _) if plain(&cs) => format!(" CHARACTER SET {cs}"),
        _ => String::new(),
    }
}

/// SQL for the account that restores the database `database` and nothing
/// else (a database no project owns), dropped after: `_` and `%` of the
/// name are not wildcards in the GRANT.
pub fn restore_account_sql(user: &str, password: &str, database: &str) -> String {
    let pattern = sql_name(&database.replace('_', "\\_").replace('%', "\\%"));
    let mut sql = String::new();
    for host in ["localhost", "127.0.0.1"] {
        sql.push_str(&format!(
            "CREATE USER '{user}'@'{host}' IDENTIFIED WITH caching_sha2_password BY '{password}';\n\
             GRANT ALL PRIVILEGES ON {pattern}.* TO '{user}'@'{host}';\n"
        ));
    }
    sql
}

/// The projects' databases on MariaDB, by name: (project, password).
fn project_users(kit: &Kit) -> BTreeMap<String, (String, String)> {
    let engine = DbEngine::MariaDb.id();
    let mut out = BTreeMap::new();
    for p in crate::projects::state::all(kit) {
        let Some(pw) = auth::project_passwords(kit, &p).remove(engine) else {
            continue;
        };
        match auth::project_db_name(kit, &p, engine) {
            Ok(name) => {
                out.insert(name.to_ascii_lowercase(), (name, pw));
            }
            Err(e) => crate::log::warn(&format!("{p}: mariadb: {e:#}")),
        }
    }
    out
}

/// What [`inventory`] reads: the objects, and the base tables by name.
type Inventory = (Vec<String>, Vec<(String, String)>);

/// The installation of the versions of MySQL a move needs (component,
/// line).
pub type Install<'a> = &'a dyn Fn(&[(String, String)]) -> Result<()>;

/// The move of version `line` in the state.
fn line_mut<'a>(state: &'a mut State, line: &str) -> Result<&'a mut LineState> {
    state
        .lines
        .get_mut(line)
        .ok_or_else(|| anyhow!("MariaDB {line}: not in the move"))
}

/// What a server has of `databases` (lower case, sorted): its tables and
/// views, and how many routines, triggers and events each has; the base
/// tables with their names as the server gives them, for the rows.
fn inventory(
    s: &dyn Servers,
    engine: DbEngine,
    line: &str,
    databases: &[String],
) -> Result<Inventory> {
    let list = databases
        .iter()
        .map(|d| sql_str(d))
        .collect::<Vec<_>>()
        .join(",");
    let sql = format!(
        "SELECT 'table', LOWER(table_schema), LOWER(table_name), IF(table_type = 'VIEW', 'view', 'table') \
         FROM information_schema.tables WHERE table_schema IN ({list});\n\
         SELECT 'routines', LOWER(routine_schema), COUNT(*) FROM information_schema.routines \
         WHERE routine_schema IN ({list}) GROUP BY routine_schema;\n\
         SELECT 'triggers', LOWER(trigger_schema), COUNT(*) FROM information_schema.triggers \
         WHERE trigger_schema IN ({list}) GROUP BY trigger_schema;\n\
         SELECT 'events', LOWER(event_schema), COUNT(*) FROM information_schema.events \
         WHERE event_schema IN ({list}) GROUP BY event_schema;\n\
         SELECT table_schema, table_name FROM information_schema.tables \
         WHERE table_schema IN ({list}) AND table_type <> 'VIEW';\n"
    );
    let text = s.query(engine, line, &sql)?;
    let mut items = Vec::new();
    let mut tables = Vec::new();
    for row in text.lines() {
        let cols: Vec<String> = row.split('\t').map(unescape).collect();
        match cols.first().map(String::as_str) {
            Some("table" | "routines" | "triggers" | "events") => items.push(cols.join(" ")),
            _ if cols.len() == 2 => tables.push((cols[0].clone(), cols[1].clone())),
            _ => {}
        }
    }
    items.sort();
    tables.sort_by_key(|(d, t)| (d.to_lowercase(), t.to_lowercase()));
    Ok((items, tables))
}

/// MySQL `mysql` has what MariaDB `line` has: the same tables, views,
/// routines, triggers and events, and the same rows in every table.
fn verify(s: &dyn Servers, line: &str, mysql: &str, databases: &[String]) -> Result<()> {
    if databases.is_empty() {
        return Ok(());
    }
    let (want, tables) = inventory(s, DbEngine::MariaDb, line, databases)?;
    let (got, _) = inventory(s, DbEngine::MySql, mysql, databases)?;
    if let Some(missing) = want.iter().find(|w| !got.contains(w)) {
        bail!("check: MySQL {mysql} has not what MariaDB {line} has ({missing})");
    }
    if want != got {
        bail!(
            "check: MySQL {mysql} has {} objects, MariaDB {line} {}",
            got.len(),
            want.len()
        );
    }
    let counts: String = tables
        .iter()
        .map(|(d, t)| format!("SELECT COUNT(*) FROM {}.{};\n", sql_name(d), sql_name(t)))
        .collect();
    if counts.is_empty() {
        return Ok(());
    }
    let want = s.query(DbEngine::MariaDb, line, &counts)?;
    let got = s.query(DbEngine::MySql, mysql, &counts)?;
    let (want, got): (Vec<&str>, Vec<&str>) = (want.lines().collect(), got.lines().collect());
    for (i, (d, t)) in tables.iter().enumerate() {
        if want.get(i) != got.get(i) {
            bail!(
                "check: {d}.{t} has {} rows on MariaDB {line}, {} on MySQL {mysql}",
                want.get(i).unwrap_or(&"?"),
                got.get(i).unwrap_or(&"?")
            );
        }
    }
    Ok(())
}

/// `from` into `to` through a `.partial` file: an interrupted write never
/// leaves a file that looks whole.
fn write_through(to: &Path, write: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let partial = to.with_extension("partial");
    let _ = fs::remove_file(&partial);
    write(&partial)?;
    fs::rename(&partial, to).with_context(|| to.display().to_string())?;
    Ok(())
}

/// The run's own record of what to undo when a step fails.
#[derive(Default)]
struct Undo {
    read_only: Vec<String>,
    mariadb_started: Vec<String>,
    mysql_started: BTreeSet<String>,
}

impl Undo {
    fn run(&self, s: &dyn Servers) {
        for l in &self.read_only {
            if let Err(e) = s.sql(DbEngine::MariaDb, l, "SET GLOBAL read_only = OFF;\n") {
                crate::log::warn(&format!("mariadb {l}: read_only not turned off: {e:#}"));
            }
        }
        for l in &self.mariadb_started {
            s.stop(DbEngine::MariaDb, l);
        }
        for l in &self.mysql_started {
            s.stop(DbEngine::MySql, l);
        }
    }
}

/// One version of MariaDB into MySQL (see the module's text).
fn move_line(
    s: &dyn Servers,
    state: &mut State,
    line: &str,
    undo: &mut Undo,
    users: &BTreeMap<String, (String, String)>,
) -> Result<()> {
    let kit = s.kit();
    let mysql = state.lines[line].mysql.clone();
    if !s.start(DbEngine::MariaDb, line)? {
        undo.mariadb_started.push(line.to_owned());
    }
    // The projects' users write no more: what they write now would not
    // reach MySQL. The administrator (the dump) still reads.
    s.sql(DbEngine::MariaDb, line, "SET GLOBAL read_only = ON;\n")?;
    undo.read_only.push(line.to_owned());
    let databases: Vec<String> =
        names_of(&s.query(DbEngine::MariaDb, line, "SHOW DATABASES;\n")?)
            .into_iter()
            .filter(|d| !SYSTEM.iter().any(|x| x.eq_ignore_ascii_case(d)))
            .collect();
    if let Some(bad) = databases
        .iter()
        .find(|d| d.starts_with('-') || d.contains('\\'))
    {
        bail!("MariaDB {line}: the database {bad} has a name the move cannot take");
    }
    // A dump made with NO_BACKSLASH_ESCAPES ends strings where the filter
    // would not (`filter`): refused, never guessed.
    let modes = s.query(
        DbEngine::MariaDb,
        line,
        "SELECT @@GLOBAL.sql_mode, @@SESSION.sql_mode;\n",
    )?;
    if modes.to_ascii_uppercase().contains("NO_BACKSLASH_ESCAPES") {
        bail!(
            "MariaDB {line} runs with NO_BACKSLASH_ESCAPES in its sql_mode: its dump cannot be \
             read safely; remove that mode from MariaDB's my.ini, then the move is tried again"
        );
    }
    let dumps = dumps_dir(kit);
    fs::create_dir_all(&dumps)?;
    // One dump per database, named by its place (a database's name may not
    // be a file's); `<line>.txt` says which is which.
    let index: String = databases
        .iter()
        .enumerate()
        .map(|(i, d)| format!("{i}\t{d}\n"))
        .collect();
    fs::write(dumps.join(format!("{line}.txt")), index)?;
    let mut files = Vec::new();
    for (i, d) in databases.iter().enumerate() {
        let raw = dumps.join(format!("{line}.{i}.sql"));
        let filtered = dumps.join(format!("{line}.{i}.mysql.sql"));
        state.dump = Some(raw.display().to_string());
        write_through(&raw, |to| s.dump(line, d, to))
            .with_context(|| format!("mariadb-dump {d}"))?;
        write_through(&filtered, |to| {
            let input = BufReader::new(fs::File::open(&raw)?);
            let output = std::io::BufWriter::new(fs::File::create(to)?);
            let report = filter::filter_dump(input, output, Some(d))?;
            crate::log::info(&format!(
                "mariadb {line}: dump of {d} filtered for MySQL: {report:?}"
            ));
            Ok(())
        })
        .with_context(|| format!("the dump of {d}"))?;
        state.dump = Some(filtered.display().to_string());
        files.push(filtered);
    }
    if !s.start(DbEngine::MySql, &mysql)? {
        undo.mysql_started.insert(mysql.clone());
    }
    // What an interrupted run left on MySQL: this move's own databases.
    for d in std::mem::take(&mut line_mut(state, line)?.restored) {
        s.sql(
            DbEngine::MySql,
            &mysql,
            &format!("DROP DATABASE IF EXISTS {};\n", sql_name(&d)),
        )?;
    }
    save(kit, state)?;
    let there: BTreeSet<String> =
        names_of(&s.query(DbEngine::MySql, &mysql, "SHOW DATABASES;\n")?)
            .into_iter()
            .map(|d| d.to_ascii_lowercase())
            .collect();
    if let Some(d) = databases
        .iter()
        .find(|d| there.contains(&d.to_ascii_lowercase()))
    {
        bail!(
            "MySQL {mysql} has a database {d} already, like MariaDB {line}: nothing overwritten. \
             Rename or drop one of them, then the move is tried again"
        );
    }
    line_mut(state, line)?.restored = databases.clone();
    save(kit, state)?;
    for (d, file) in databases.iter().zip(&files) {
        // The database first, with MariaDB's character set and collation.
        let charset = s.query(
            DbEngine::MariaDb,
            line,
            &format!(
                "SELECT default_character_set_name, default_collation_name \
                 FROM information_schema.schemata WHERE schema_name = {};\n",
                sql_str(d)
            ),
        )?;
        s.sql(
            DbEngine::MySql,
            &mysql,
            &format!("CREATE DATABASE {}{};\n", sql_name(d), charset_of(&charset)),
        )?;
        // Restored by an account that reaches this database only: the
        // project's user, or one made for the restore and dropped after.
        // Whatever the dump holds, it cannot touch another database, an
        // account or the server.
        let restored = match users.get(&d.to_ascii_lowercase()) {
            Some((name, pw)) => {
                s.sql(DbEngine::MySql, &mysql, &auth::mysql_project_sql(name, pw))
                    .with_context(|| format!("MySQL {mysql}: the user {name}"))?;
                s.restore(&mysql, d, name, pw, file)
            }
            None => {
                let user = format!("dkmove_{}", &auth::new_password()?[..8]);
                let pw = auth::new_password()?;
                s.sql(DbEngine::MySql, &mysql, &restore_account_sql(&user, &pw, d))?;
                let done = s.restore(&mysql, d, &user, &pw, file);
                s.sql(
                    DbEngine::MySql,
                    &mysql,
                    &format!("DROP USER IF EXISTS '{user}'@'localhost', '{user}'@'127.0.0.1';\n"),
                )?;
                done
            }
        };
        restored.with_context(|| format!("restore of {d} on MySQL"))?;
    }
    if databases.is_empty() {
        return Ok(());
    }
    verify(s, line, &mysql, &databases)?;
    crate::log::info(&format!(
        "mariadb {line}: {} databases copied to MySQL {mysql} and checked",
        databases.len()
    ));
    Ok(())
}

/// The copy of `path` made before the switch.
fn copy_of(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(format!(".{COPY}"));
    PathBuf::from(name)
}

fn services_path(kit: &Kit) -> PathBuf {
    kit.app_dir().join("services.json")
}

fn db_json(kit: &Kit, project: &str) -> PathBuf {
    kit.project_data()
        .join(project)
        .join("private")
        .join("db.json")
}

/// The files the switch rewrites (those that exist).
fn rewritten(kit: &Kit, projects: &[String]) -> Vec<PathBuf> {
    let mut files = vec![services_path(kit), auth::secrets_path(kit)];
    files.extend(projects.iter().map(|p| db_json(kit, p)));
    files.retain(|f| f.is_file());
    files
}

/// `services.json`: MySQL takes MariaDB's ports and its "start with the
/// app", MariaDB's entry goes. Nothing when it went already.
fn rewrite_services(kit: &Kit, state: &State, mysql_had_data: bool) -> Result<()> {
    let path = services_path(kit);
    let mut all: BTreeMap<String, crate::db::services::Setting> =
        crate::fsx::read_json(&path).unwrap_or_default();
    let Some(old) = all.remove(DbEngine::MariaDb.id()) else {
        return Ok(());
    };
    let mysql = all.entry(DbEngine::MySql.id().to_owned()).or_default();
    mysql.autostart |= old.autostart;
    if !mysql_had_data {
        mysql.line = state.main.clone();
        let main = state.main.as_deref().unwrap_or_default();
        for l in state.lines.values() {
            if l.mysql == main {
                mysql.port = (l.port != DbEngine::MySql.default_port()).then_some(l.port);
            } else {
                mysql.ports.entry(l.mysql.clone()).or_insert(l.port);
            }
        }
    }
    crate::fsx::write_json(&path, &all)
}

/// `secrets.json`: MySQL's `root` keeps its password (MariaDB's when MySQL
/// had none, given at its initialization), MariaDB's goes.
fn rewrite_secrets(kit: &Kit) -> Result<()> {
    let mut secrets = auth::load(kit);
    let Some(old) = secrets.mariadb_root.take() else {
        return Ok(());
    };
    secrets.mysql_root.get_or_insert(old);
    auth::save(kit, &secrets)
}

/// The switch (see the module's text), from wherever an interrupted one
/// stopped.
fn switch(s: &dyn Servers, state: &mut State, mysql_had_data: bool) -> Result<()> {
    let kit = s.kit();
    for (l, m) in state
        .lines
        .iter()
        .map(|(l, m)| (l.clone(), m.mysql.clone()))
    {
        s.stop(DbEngine::MySql, &m);
        s.stop(DbEngine::MariaDb, &l);
    }
    state.phase = Phase::Switching;
    save(kit, state)?;
    if !state.copied {
        state.projects = crate::projects::state::all(kit)
            .into_iter()
            .filter(|p| auth::project_passwords(kit, p).contains_key(DbEngine::MariaDb.id()))
            .collect();
        for f in rewritten(kit, &state.projects) {
            fs::copy(&f, copy_of(&f)).with_context(|| copy_of(&f).display().to_string())?;
        }
        state.copied = true;
        save(kit, state)?;
    }
    let main = state.main.clone().unwrap_or_default();
    let lines: BTreeMap<String, String> = state
        .lines
        .iter()
        .map(|(l, m)| (l.clone(), m.mysql.clone()))
        .collect();
    for p in &state.projects {
        auth::move_project_to_mysql(kit, p, &lines, &main)
            .with_context(|| format!("{p}: db.json"))?;
    }
    rewrite_services(kit, state, mysql_had_data).context("services.json")?;
    rewrite_secrets(kit).context("secrets.json")?;
    let (from, to) = (mariadb_root(kit), kept_dir(kit));
    if from.exists() {
        if to.exists() {
            bail!(
                "{} exists already: MariaDB's data not moved there",
                to.display()
            );
        }
        fs::rename(&from, &to).with_context(|| to.display().to_string())?;
    }
    state.phase = Phase::Done;
    state.error = None;
    state.dump = None;
    for l in state.lines.values_mut() {
        l.restored.clear();
    }
    save(kit, state)?;
    crate::log::info(
        "mariadb: data moved to MySQL; MariaDB's data kept in data/db/mariadb.pre-mysql",
    );
    Ok(())
}

/// The switch failed before its end: the copies back, MariaDB as before.
fn roll_back(s: &dyn Servers, state: &mut State) {
    let kit = s.kit();
    if state.copied {
        for f in rewritten(kit, &state.projects)
            .into_iter()
            .chain(state.projects.iter().map(|p| db_json(kit, p)))
            .collect::<BTreeSet<_>>()
        {
            let copy = copy_of(&f);
            if copy.is_file() {
                match fs::copy(&copy, &f) {
                    Ok(_) => {
                        let _ = fs::remove_file(&copy);
                    }
                    Err(e) => crate::log::warn(&format!(
                        "{}: not put back from its copy ({e})",
                        f.display()
                    )),
                }
            }
        }
        state.copied = false;
        state.projects.clear();
    }
    state.phase = Phase::Lines;
    let _ = save(kit, state);
    for (l, m) in state.lines.clone() {
        if m.serve {
            if let Err(e) = s.start(DbEngine::MariaDb, &l) {
                crate::log::warn(&format!("mariadb {l}: not started again: {e:#}"));
            }
        }
    }
}

/// Moves MariaDB's data to MySQL when there is some (see the module's
/// text). `install` installs the versions of MySQL the move needs
/// (component, line) and fails when it cannot. A failure is kept in the
/// state with the dump, and returned.
pub fn run(s: &dyn Servers, install: Install<'_>) -> Result<Outcome> {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    moving_set().insert(s.kit().data());
    let result = run_locked(s, install);
    moving_set().remove(&s.kit().data());
    if let Err(e) = &result {
        crate::log::warn(&format!("mariadb → mysql: {e:#}"));
    }
    result
}

fn run_locked(s: &dyn Servers, install: Install<'_>) -> Result<Outcome> {
    let kit = s.kit();
    let mut state = load(kit);
    if state.phase == Phase::Switching {
        // Interrupted while switching: MariaDB has not run since.
        let had = state.mysql_had_data;
        return finish(s, &mut state, had, &BTreeSet::new());
    }
    let pending = pending_lines(kit);
    if pending.is_empty() {
        return Ok(Outcome::Nothing);
    }
    crate::log::info(&format!("mariadb → mysql: versions with data {pending:?}"));
    // Seen at the first run: a MySQL that an earlier run initialized is
    // not "its own" data.
    let mysql_had_data = if state.lines.is_empty() {
        !super::services::lines_with_data(kit, DbEngine::MySql).is_empty()
    } else {
        state.mysql_had_data
    };
    let fail = |state: &mut State, e: anyhow::Error| -> Result<Outcome> {
        state.error = Some(format!("{e:#}"));
        let _ = save(kit, state);
        Err(e)
    };
    // MariaDB's own programs: the kit no longer downloads them (D-117).
    let installed = s.installed_lines(DbEngine::MariaDb);
    if let Some(l) = pending.iter().find(|l| !installed.contains(l)) {
        let e = anyhow!(
            "MariaDB {l}: its programs are not on this system ({}), so its data cannot move to \
             MySQL here: open the kit on the system where MariaDB ran (Windows or Linux x64) \
             and the move happens at its start",
            kit.apps().join(DbEngine::MariaDb.id()).display()
        );
        return fail(&mut state, e);
    }
    // Where each version goes, and on which port.
    let offered = mysql_offered();
    let mysql_main = if mysql_had_data {
        crate::db::services::server_line(kit, DbEngine::MySql.id())
    } else {
        None
    }
    .or_else(|| offered.first().cloned())
    .ok_or_else(|| anyhow!("no version of MySQL in the catalog"))?;
    let main = crate::db::services::server_line(kit, DbEngine::MariaDb.id());
    let settings = crate::db::services::load_settings(kit);
    let autostart = settings
        .get(DbEngine::MariaDb.id())
        .is_some_and(|s| s.autostart);
    let mut lines = BTreeMap::new();
    for (l, m) in plan_lines(&pending, main.as_deref(), &mysql_main, &offered) {
        let previous = state.lines.remove(&l).unwrap_or_default();
        lines.insert(
            l.clone(),
            LineState {
                mysql: m,
                port: crate::db::services::line_port(kit, DbEngine::MariaDb.id(), Some(&l)),
                serve: s.running(DbEngine::MariaDb, &l) || (autostart && Some(&l) == main.as_ref()),
                restored: previous.restored,
                done: false,
            },
        );
    }
    // The databases restored by a run for a version no longer pending
    // stay where they are (that version's data moved already).
    state.lines = lines;
    state.main = Some(mysql_main);
    state.mysql_had_data = mysql_had_data;
    state.error = None;
    state.dump = None;
    save(kit, &state)?;
    // The versions of MySQL it needs, installed first.
    let have = s.installed_lines(DbEngine::MySql);
    let wanted: Vec<(String, String)> = state
        .lines
        .values()
        .map(|l| l.mysql.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|m| !have.contains(m))
        .map(|m| (DbEngine::MySql.id().to_owned(), m))
        .collect();
    if !wanted.is_empty() {
        if let Err(e) = install(&wanted).context("installing MySQL") {
            return fail(&mut state, e);
        }
        let have = s.installed_lines(DbEngine::MySql);
        if let Some((_, m)) = wanted.iter().find(|(_, m)| !have.contains(m)) {
            return fail(&mut state, anyhow!("MySQL {m} is not installed"));
        }
    }
    // MySQL's root gets MariaDB's password when it has none yet: given at
    // its initialization (`services::data`), never later.
    let mut secrets = auth::load(kit);
    if secrets.mysql_root.is_none() {
        if let Some(pw) = secrets.mariadb_root.clone() {
            secrets.mysql_root = Some(pw);
            auth::save(kit, &secrets)?;
        }
    }
    let users = project_users(kit);
    let mut undo = Undo::default();
    // MySQL running for its own projects before the move: it runs after.
    let mysql_running: BTreeSet<String> = state
        .lines
        .values()
        .map(|l| l.mysql.clone())
        .filter(|m| s.running(DbEngine::MySql, m))
        .collect();
    // The version of the server first, then the others newest first.
    let order: Vec<String> = plan_lines(&pending, main.as_deref(), "", &[])
        .into_iter()
        .map(|(l, _)| l)
        .collect();
    for l in &order {
        if let Err(e) =
            move_line(s, &mut state, l, &mut undo, &users).with_context(|| format!("MariaDB {l}"))
        {
            undo.run(s);
            return fail(&mut state, e);
        }
        line_mut(&mut state, l)?.done = true;
        state.dump = None;
        save(kit, &state)?;
    }
    finish(s, &mut state, mysql_had_data, &mysql_running)
}

/// The switch, then MySQL where MariaDB ran.
fn finish(
    s: &dyn Servers,
    state: &mut State,
    mysql_had_data: bool,
    mysql_running: &BTreeSet<String>,
) -> Result<Outcome> {
    let kit = s.kit();
    if let Err(e) = switch(s, state, mysql_had_data) {
        roll_back(s, state);
        state.error = Some(format!("{e:#}"));
        let _ = save(kit, state);
        return Err(e);
    }
    let wanted: BTreeSet<String> = state
        .lines
        .values()
        .filter(|l| l.serve)
        .map(|l| l.mysql.clone())
        .chain(mysql_running.iter().cloned())
        .collect();
    for m in wanted {
        if let Err(e) = s.serve(&m) {
            crate::log::warn(&format!("mysql {m}: not started after the move: {e:#}"));
        }
    }
    Ok(Outcome::Done)
}

/// What the Database page and the Activity panel show of the move.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// `none` (nothing of MariaDB), `pending`, `running`, `failed`, `done`.
    pub state: &'static str,
    /// MariaDB version → MySQL version, port, copied in this run.
    pub lines: Vec<StatusLine>,
    pub error: Option<String>,
    pub dump: Option<String>,
    /// `data\db\mariadb.pre-mysql` and its bytes, while it is there.
    pub kept: Option<String>,
    pub kept_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusLine {
    pub mariadb: String,
    pub mysql: String,
    pub port: u16,
    pub done: bool,
}

fn dir_bytes(dir: &Path) -> u64 {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => dir_bytes(&e.path()),
            Ok(t) if t.is_file() => e.metadata().map(|m| m.len()).unwrap_or(0),
            _ => 0,
        })
        .sum()
}

pub fn status(kit: &Kit) -> Status {
    let state = load(kit);
    let pending = !pending_lines(kit).is_empty() || state.phase == Phase::Switching;
    let kept = kept_dir(kit);
    let kept_there = kept.is_dir();
    let word = if moving(kit) {
        "running"
    } else if pending && state.error.is_some() {
        "failed"
    } else if pending {
        "pending"
    } else if kept_there {
        "done"
    } else {
        "none"
    };
    Status {
        state: word,
        lines: state
            .lines
            .iter()
            .map(|(l, m)| StatusLine {
                mariadb: l.clone(),
                mysql: m.mysql.clone(),
                port: m.port,
                done: m.done || state.phase == Phase::Done,
            })
            .collect(),
        error: state.error.clone().filter(|_| pending),
        dump: state.dump.clone().filter(|_| pending),
        kept: kept_there.then(|| kept.display().to_string()),
        kept_bytes: if kept_there { dir_bytes(&kept) } else { 0 },
    }
}

/// Deletes `data\db\mariadb.pre-mysql` and the `.pre-mysql` copies, asked
/// by the user from the Database page once the move is done.
pub fn drop_kept(kit: &Kit) -> Result<()> {
    if !pending_lines(kit).is_empty() || switching(kit) {
        bail!("MariaDB's data has not moved to MySQL yet: nothing deleted");
    }
    let state = load(kit);
    let kept = kept_dir(kit);
    if crate::os::safe_path::is_link(&kept) {
        bail!("{}: a link, left alone", kept.display());
    }
    if kept.exists() {
        crate::net::archive::remove_dir(&kept)?;
    }
    for f in [services_path(kit), auth::secrets_path(kit)]
        .into_iter()
        .chain(state.projects.iter().map(|p| db_json(kit, p)))
    {
        let _ = fs::remove_file(copy_of(&f));
    }
    crate::log::info("mariadb: the data kept after the move to MySQL deleted by the user");
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/db/migrate_engine.rs"]
mod tests;
