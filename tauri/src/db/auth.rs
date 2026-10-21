//! Passwords of the kit's databases (DECISIONS.md D-74): the `root` of
//! MySQL, MongoDB's `admin` and one user per project and database, each
//! with a random password.
//!
//! - the administrators' in `data\app\db\secrets.json`, a project's in
//!   `data\projects\<project>\private\db.json` (D-89), readable by the owner
//!   only where the disk has permissions (NTFS; FAT32/exFAT sticks have none);
//! - `data\app\db\mysql.cnf` holds the `[client]` login of MySQL's `root`
//!   (D-110): the kit's `mysql`, `mysqldump` and `mysqladmin` launchers
//!   (`data\kit\bin`) pass it with `--defaults-extra-file` (read after the
//!   other files, it wins) and the app's tools with `--defaults-file`, so
//!   they log in without a password on the command line;
//! - passwords never go on a command line (other users can read those):
//!   SQL goes through stdin, mongosh reads them from its environment;
//! - a project's `.env` gets its connection only after the user saw the
//!   lines that change (`env_changes`, then `env_apply`).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::DbEngine;
use crate::fsx::{read_json, write_atomic};
use crate::kit::Kit;
use crate::projects::state::ProjectState;

/// The administrators' passwords. The projects' are in their own `db.json`;
/// keys this version does not know stay as they are, the `projects` of an
/// older kit included (moved to the projects' files by the migration).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Secrets {
    /// MySQL's `root` (D-110).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mysql_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mongodb_admin: Option<String>,
    #[serde(flatten)]
    pub other: BTreeMap<String, serde_json::Value>,
}

pub fn dir(kit: &Kit) -> PathBuf {
    kit.app_dir().join("db")
}

pub fn secrets_path(kit: &Kit) -> PathBuf {
    dir(kit).join("secrets.json")
}

pub fn load(kit: &Kit) -> Secrets {
    read_json(&secrets_path(kit)).unwrap_or_default()
}

/// Written in a folder made private first, so neither the temporary file of
/// `write_atomic` nor the file is ever readable by others, not even for a
/// moment (security review of D-74); the file's own DACL is set again after.
fn write_private(path: &Path, text: &str) -> Result<()> {
    crate::router::fs::write_private(path, text)
}

/// The atomic write and the file's own DACL, in a folder already private
/// (`ProjectState::private_dir` sets its DACL before returning it).
fn write_in_private(path: &Path, text: &str) -> Result<()> {
    crate::router::fs::write_in_private(path, text)
}

/// A private file of `data\app\db` (a tool's config with a password).
pub fn write_private_text(path: &Path, text: &str) -> Result<()> {
    write_private(path, text)
}

pub fn save(kit: &Kit, secrets: &Secrets) -> Result<()> {
    write_private(&secrets_path(kit), &serde_json::to_string_pretty(secrets)?)
}

/// A project's `private\db.json`: engine (`mysql`, `mongodb`) → password
/// of its user, and `names`: engine → name of its database and user (an
/// object, so the readers of passwords skip it); `lines`: engine → version of
/// the server its database is on, when not the one the server runs
/// (D-102); `used`: engine → every version where it was created; other keys
/// kept as they are.
///
/// ```json
/// { "mysql": "<password>", "names": { "mysql": "shop" }, "lines": { "mysql": "8.4" },
///   "used": { "mysql": ["9.7", "8.4"] } }
/// ```
pub(crate) type ProjectSecrets = serde_json::Map<String, serde_json::Value>;

/// The key of the names in `db.json`.
const NAMES: &str = "names";

/// Held by every read-change-write of a project's `db.json`: two commands
/// on blocking threads (a version chosen while a database is created), or
/// the command line beside the app, would otherwise each write back what
/// they read, and one change would be lost. In this process a mutex; across
/// the processes of the kit the lock of the Antigravity keyring
/// (`agents/keyring.rs`): `Local\Atollo-db-json-<kit id>` on Windows, an
/// exclusive `flock` on `data/app/db/db-json.lock` elsewhere.
fn db_json_lock(kit: &Kit) -> Result<DbJsonLock> {
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let process = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(DbJsonLock {
        _kit: KitLock::take(kit)?,
        _process: process,
    })
}

/// How long another process may hold `db.json`: then the change is refused
/// (nothing written), never made without the lock.
const DB_JSON_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

struct DbJsonLock {
    // Released first: the other processes, then this one's threads.
    _kit: KitLock,
    _process: std::sync::MutexGuard<'static, ()>,
}

#[cfg(windows)]
struct KitLock(windows::Win32::Foundation::HANDLE);

#[cfg(windows)]
impl KitLock {
    fn take(kit: &Kit) -> Result<KitLock> {
        use windows::Win32::Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::{CreateMutexW, WaitForSingleObject};
        let name = windows::core::HSTRING::from(format!(r"Local\Atollo-db-json-{}", kit.id()));
        // SAFETY: valid name, default security; the handle is closed here
        // or by the guard's Drop.
        unsafe {
            let h = CreateMutexW(None, false, &name)?;
            let r = WaitForSingleObject(h, DB_JSON_WAIT.as_millis() as u32);
            // Abandoned: a process that held it ended; it is ours now.
            if r != WAIT_OBJECT_0 && r != WAIT_ABANDONED {
                let _ = CloseHandle(h);
                bail!("db.json: another Atollo process holds it, try again");
            }
            Ok(KitLock(h))
        }
    }
}

#[cfg(windows)]
impl Drop for KitLock {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::ReleaseMutex;
        // SAFETY: handle from CreateMutexW, owned since the wait; released
        // and closed once.
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

/// The lock file stays (it is empty); the system drops the `flock` when the
/// file closes, also when the process dies.
#[cfg(unix)]
struct KitLock(#[allow(dead_code)] fs::File);

#[cfg(unix)]
impl KitLock {
    fn take(kit: &Kit) -> Result<KitLock> {
        use std::os::fd::AsRawFd;
        let path = dir(kit).join("db-json.lock");
        fs::create_dir_all(dir(kit))?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        let start = std::time::Instant::now();
        // SAFETY: a valid descriptor owned by `file`.
        while unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            if start.elapsed() > DB_JSON_WAIT {
                bail!("db.json: another Atollo process holds it, try again");
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        Ok(KitLock(file))
    }
}

/// The key of the versions in `db.json`.
const LINES: &str = "lines";

/// The version of `engine` chosen for the project's database (None: the
/// one the server runs); an error when `db.json` cannot be read. A value
/// that is not a version is ignored.
pub fn project_line(kit: &Kit, project: &str, engine: &str) -> Result<Option<String>> {
    let secrets = read_project(&project_file_to_read(kit, project)?)?.unwrap_or_default();
    Ok(secrets
        .get(LINES)
        .and_then(|l| l.get(engine))
        .and_then(serde_json::Value::as_str)
        .filter(|l| crate::versions::valid_line(l))
        .map(str::to_owned))
}

/// Stores (None: removes) the version of `engine` for the project's
/// database; nothing is created to remove one.
pub fn set_project_line(kit: &Kit, project: &str, engine: &str, line: Option<&str>) -> Result<()> {
    if line.is_some_and(|l| !crate::versions::valid_line(l)) {
        bail!("not a version line: {}", line.unwrap_or_default());
    }
    let _one = db_json_lock(kit)?;
    let mut secrets = read_project(&project_file_to_read(kit, project)?)?;
    if line.is_none() && secrets.is_none() {
        return Ok(());
    }
    let secrets = secrets.get_or_insert_with(Default::default);
    let lines = match secrets
        .entry(LINES)
        .or_insert_with(|| serde_json::Value::Object(Default::default()))
    {
        serde_json::Value::Object(lines) => lines,
        _ => bail!("db.json: \"{LINES}\" is not an object"),
    };
    match line {
        Some(l) => lines.insert(engine.to_owned(), l.into()),
        None => lines.remove(engine),
    };
    if lines.is_empty() {
        secrets.remove(LINES);
    }
    let path = project_file(kit, project)?;
    write_in_private(&path, &serde_json::to_string_pretty(&secrets)?)
}

/// The key of the versions each database was created on, in `db.json`.
const USED: &str = "used";

/// The versions of `engine` where the project's database and user were
/// created (`used` of `db.json`: engine → versions), so that deleting the
/// project drops them from each. Empty for a `db.json` of before (only the
/// version it is on now is known then); values that are not versions are
/// ignored.
pub fn project_used_lines(kit: &Kit, project: &str, engine: &str) -> Result<Vec<String>> {
    let secrets = read_project(&project_file_to_read(kit, project)?)?.unwrap_or_default();
    Ok(secrets
        .get(USED)
        .and_then(|u| u.get(engine))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .filter(|l| crate::versions::valid_line(l))
        .map(str::to_owned)
        .collect())
}

/// Adds `line` to the versions of `engine` where the project's database
/// exists ([`project_used_lines`]); nothing is created for a project
/// without `db.json`.
pub fn add_project_used_line(kit: &Kit, project: &str, engine: &str, line: &str) -> Result<()> {
    if !crate::versions::valid_line(line) {
        bail!("not a version line: {line}");
    }
    let _one = db_json_lock(kit)?;
    let Some(mut secrets) = read_project(&project_file_to_read(kit, project)?)? else {
        return Ok(());
    };
    let used = match secrets
        .entry(USED)
        .or_insert_with(|| serde_json::Value::Object(Default::default()))
    {
        serde_json::Value::Object(used) => used,
        _ => bail!("db.json: \"{USED}\" is not an object"),
    };
    let lines = match used
        .entry(engine)
        .or_insert_with(|| serde_json::Value::Array(Vec::new()))
    {
        serde_json::Value::Array(lines) => lines,
        _ => bail!("db.json: the versions used by {engine} are not a list"),
    };
    if lines.iter().any(|l| l.as_str() == Some(line)) {
        return Ok(());
    }
    lines.push(line.into());
    let path = project_file(kit, project)?;
    write_in_private(&path, &serde_json::to_string_pretty(&secrets)?)
}

fn project_state(kit: &Kit, project: &str) -> Result<ProjectState> {
    match ProjectState::of(kit, project) {
        Some(state) => Ok(state),
        None => bail!("not a project name: {project}"),
    }
}

/// The project's `db.json`, its folder made private first (to write it).
fn project_file(kit: &Kit, project: &str) -> Result<PathBuf> {
    Ok(project_state(kit, project)?.db_file()?)
}

/// The project's `db.json` to read: nothing is created (a project without
/// databases gets no `private\`, D-89).
fn project_file_to_read(kit: &Kit, project: &str) -> Result<PathBuf> {
    Ok(project_state(kit, project)?
        .dir()
        .join("private")
        .join("db.json"))
}

/// `None` for a missing file; an error for one that cannot be read, so it
/// is never overwritten and its passwords lost.
fn read_project(path: &Path) -> Result<Option<ProjectSecrets>> {
    if crate::os::safe_path::is_link(path) {
        bail!("{}: a link, left alone", path.display());
    }
    match fs::read_to_string(path) {
        // serde_json's errors name a position, never the text read.
        Ok(text) => Ok(Some(serde_json::from_str(&text).map_err(|e| {
            anyhow::anyhow!("{}: not valid JSON ({e})", path.display())
        })?)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn passwords(secrets: &ProjectSecrets) -> BTreeMap<String, String> {
    secrets
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_owned())))
        .collect()
}

/// The passwords of the project's users, by engine (empty when none); an
/// error when `db.json` is there but cannot be read (a link, not JSON, the
/// DACL of another PC's user): the caller must not take it for "none".
pub fn try_project_passwords(kit: &Kit, project: &str) -> Result<BTreeMap<String, String>> {
    let path = project_file_to_read(kit, project)?;
    Ok(passwords(&read_project(&path)?.unwrap_or_default()))
}

/// The passwords of the project's users, by engine (empty when none or
/// unreadable: only a warning, for the pages that show them).
pub fn project_passwords(kit: &Kit, project: &str) -> BTreeMap<String, String> {
    try_project_passwords(kit, project).unwrap_or_else(|e| {
        crate::log::warn(&format!("{project}: passwords: {e:#}"));
        BTreeMap::new()
    })
}

/// The password of the project's user on `engine`, once created (lenient,
/// for showing it; anything that writes uses [`try_project_password`]).
#[allow(dead_code)] // the tests and callers on other branches
pub fn project_password(kit: &Kit, project: &str, engine: &str) -> Option<String> {
    project_passwords(kit, project).remove(engine)
}

/// [`project_password`] that fails when `db.json` cannot be read.
pub fn try_project_password(kit: &Kit, project: &str, engine: &str) -> Result<Option<String>> {
    Ok(try_project_passwords(kit, project)?.remove(engine))
}

/// Name and password of the project's database and user on `engine`, as
/// the server must get them: those stored, or new ones stored (both in one
/// write) before any server hears of them. Refused, before anything is
/// written, when `db.json` cannot be read (a server never gets a password
/// stored nowhere), when the name is one of the server's own, or when
/// another project of the kit already uses it (its user would get this
/// project's password, its database would be shared). A server call that
/// fails after can be repeated with what is stored.
pub fn ensure_project_db(kit: &Kit, project: &str, engine: &str) -> Result<(String, String)> {
    // The check of the other projects' names and the write are one step:
    // two creations at once (`my-app`, `my_app`) cannot both pass.
    let _one = db_json_lock(kit)?;
    let mut secrets = read_project(&project_file_to_read(kit, project)?)?.unwrap_or_default();
    let name = name_in(&secrets, project, engine)?;
    check_name_free(kit, project, engine, &name)?;
    let stored_name = stored_name(&secrets, engine)?.is_some();
    let password = match secrets.get(engine).and_then(|v| v.as_str()) {
        Some(password) if stored_name => return Ok((name, password.to_owned())),
        Some(password) => password.to_owned(),
        None => new_password()?,
    };
    secrets.insert(engine.to_owned(), password.clone().into());
    names_mut(&mut secrets)?.insert(engine.to_owned(), name.clone().into());
    let path = project_file(kit, project)?;
    write_in_private(&path, &serde_json::to_string_pretty(&secrets)?)?;
    Ok((name, password))
}

/// The name of the project's database and user on `engine`: the one stored
/// in its `db.json`; for data created before names were stored (a password,
/// no name) the rule of that time ([`legacy_db_name`]); with nothing stored,
/// the name a creation gives ([`db_name`]). An error when `db.json` cannot
/// be read (never taken for "nothing stored") and when the name is one of
/// the server's own: the kit never creates, alters or drops those, the user
/// renames them by hand.
pub fn project_db_name(kit: &Kit, project: &str, engine: &str) -> Result<String> {
    let secrets = read_project(&project_file_to_read(kit, project)?)?.unwrap_or_default();
    name_in(&secrets, project, engine)
}

/// [`project_db_name`] on a `db.json` already read.
fn name_in(secrets: &ProjectSecrets, project: &str, engine: &str) -> Result<String> {
    let Some(name) = used_name(secrets, project, engine)? else {
        return Ok(db_name(project, engine));
    };
    if is_reserved(engine, &name) {
        bail!(
            "{project}: its database and user on {engine} are named {name}, like one of the \
             server's own: the kit leaves them alone. Rename database and user by hand and \
             write the new name in data\\projects\\{project}\\private\\db.json \
             (\"names\": {{ \"{engine}\": \"<name>\" }})"
        );
    }
    if !is_valid_name(&name) {
        bail!("{project}: the name of its database on {engine} in db.json is not valid");
    }
    Ok(name)
}

/// A name the kit gives: lower case letters, digits and `_`, at most 64
/// characters; nothing to escape in SQL or mongosh.
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The name stored for `engine`; an error when `names` is not what this
/// version writes (never guessed over).
fn stored_name(secrets: &ProjectSecrets, engine: &str) -> Result<Option<String>> {
    let Some(names) = secrets.get(NAMES) else {
        return Ok(None);
    };
    let Some(names) = names.as_object() else {
        bail!("db.json: \"{NAMES}\" is not an object");
    };
    match names.get(engine) {
        None => Ok(None),
        Some(serde_json::Value::String(name)) => Ok(Some(name.clone())),
        Some(_) => bail!("db.json: the name for {engine} is not text"),
    }
}

/// The name the project already uses on `engine`: stored, or the rule of
/// the time for data with a password and no name; `None` when it has
/// nothing there (no server knows a name of it).
fn used_name(secrets: &ProjectSecrets, project: &str, engine: &str) -> Result<Option<String>> {
    if let Some(name) = stored_name(secrets, engine)? {
        return Ok(Some(name));
    }
    Ok(secrets
        .get(engine)
        .is_some_and(serde_json::Value::is_string)
        .then(|| legacy_db_name(project)))
}

/// `names` of `db.json`, added when missing.
fn names_mut(secrets: &mut ProjectSecrets) -> Result<&mut ProjectSecrets> {
    match secrets
        .entry(NAMES)
        .or_insert_with(|| serde_json::Value::Object(Default::default()))
    {
        serde_json::Value::Object(names) => Ok(names),
        _ => bail!("db.json: \"{NAMES}\" is not an object"),
    }
}

/// The other project of the kit (a folder of `data\projects\`) whose
/// database and user on `engine` are named `name` (stored, or by the rule
/// of the time), if any. A `db.json` that cannot be read is an error: what
/// it names is unknown.
pub fn name_used_by(kit: &Kit, project: &str, engine: &str, name: &str) -> Result<Option<String>> {
    // Passwords left in secrets.json by the migration (a conflict, a name
    // that is not a project's): their databases exist, by the rule of before.
    if let Some(serde_json::Value::Object(left)) = load(kit).other.get("projects") {
        for (other, engines) in left {
            if !other.eq_ignore_ascii_case(project)
                && engines
                    .get(engine)
                    .is_some_and(serde_json::Value::is_string)
                && legacy_db_name(other).eq_ignore_ascii_case(name)
            {
                return Ok(Some(other.clone()));
            }
        }
    }
    for other in crate::projects::state::all(kit) {
        // One folder on Windows, whatever the case.
        if other.eq_ignore_ascii_case(project) {
            continue;
        }
        let used = read_project(&project_file_to_read(kit, &other)?)
            .and_then(|s| used_name(&s.unwrap_or_default(), &other, engine))
            .map_err(|e| anyhow::anyhow!("the databases of the project {other}: {e:#}"))?;
        if used.is_some_and(|n| n.eq_ignore_ascii_case(name)) {
            return Ok(Some(other));
        }
    }
    Ok(None)
}

/// Refused when another project of the kit uses `name` on `engine`
/// ([`name_used_by`]): creating would give its user this project's password,
/// dropping would take its database away. Refused as well while the data of
/// layout 1 is not all moved: some names would be unseen.
pub fn check_name_free(kit: &Kit, project: &str, engine: &str, name: &str) -> Result<()> {
    if crate::projects::migrate::pending(kit) {
        bail!(
            "the projects' data of an earlier version is not all moved yet (see the log): \
             nothing done on the databases until the next start moves it"
        );
    }
    if let Some(other) = name_used_by(kit, project, engine, name)? {
        bail!(
            "the database and user {name} on {engine} belong to the project {other} as well: \
             nothing done on the server. Rename one of the projects' databases by hand"
        );
    }
    Ok(())
}

/// Data created before names were stored (passwords, no names) gets the
/// names its databases were created with, from `project`: for one given to
/// another project (`projects::state::adopt`) and for the migration of a
/// kit of layout 1. A name already there is never replaced; one of the
/// server's own is left out ([`project_db_name`] reports it). True when
/// something was added.
pub(crate) fn add_legacy_names(secrets: &mut ProjectSecrets, project: &str) -> Result<bool> {
    let engines: Vec<String> = secrets
        .iter()
        .filter(|(k, v)| v.is_string() && DbEngine::from_id(k).is_some())
        .map(|(k, _)| k.clone())
        .collect();
    let mut added = false;
    for engine in engines {
        let name = legacy_db_name(project);
        if stored_name(secrets, &engine)?.is_none() && !is_reserved(&engine, &name) {
            names_mut(secrets)?.insert(engine, name.into());
            added = true;
        }
    }
    Ok(added)
}

/// [`add_legacy_names`] on the `db.json` of `project`, written back (in
/// its private folder) only when something was added.
pub(crate) fn keep_legacy_names(kit: &Kit, project: &str) -> Result<()> {
    let _one = db_json_lock(kit)?;
    let path = project_file_to_read(kit, project)?;
    let Some(mut secrets) = read_project(&path)? else {
        return Ok(());
    };
    if add_legacy_names(&mut secrets, project)? {
        write_private(&path, &serde_json::to_string_pretty(&secrets)?)?;
    }
    Ok(())
}

/// Stores the password of the project's user on `engine` (no name: as a
/// `db.json` of before the names; creations use [`ensure_project_db`]).
#[allow(dead_code)] // the tests
pub fn set_project_password(kit: &Kit, project: &str, engine: &str, password: &str) -> Result<()> {
    let _one = db_json_lock(kit)?;
    let path = project_file(kit, project)?;
    let mut secrets = read_project(&path)?.unwrap_or_default();
    secrets.insert(engine.to_owned(), password.into());
    write_in_private(&path, &serde_json::to_string_pretty(&secrets)?)
}

/// Removes the project's `db.json` and returns its passwords by engine, so
/// the users can be dropped from the servers (D-89). Nothing is
/// created: after the project's folder of the kit is gone it stays gone.
pub fn drop_project(kit: &Kit, project: &str) -> Result<BTreeMap<String, String>> {
    let _one = db_json_lock(kit)?;
    let path = project_file_to_read(kit, project)?;
    let Some(secrets) = read_project(&path)? else {
        return Ok(BTreeMap::new());
    };
    fs::remove_file(&path)?;
    Ok(passwords(&secrets))
}

/// 32 hex characters: nothing to escape in SQL, ini files or URLs.
pub fn new_password() -> Result<String> {
    Ok(crate::editor::random_token()?[..32].to_owned())
}

/// The `[client]` login of `root` for the kit's MySQL, on the port of the
/// version the server runs (D-110, [`write_mysql_cnf`]).
pub fn mysql_cnf(kit: &Kit) -> PathBuf {
    dir(kit).join("mysql.cnf")
}

/// The login of another version of MySQL, on its own port
/// (D-102): the `ATOLLO_MYSQL_CNF` of the programs of a project whose
/// database is on it (`services::project_client_env`).
pub fn mysql_line_cnf(kit: &Kit, line: &str) -> PathBuf {
    dir(kit).join("mysql-lines").join(line).join("mysql.cnf")
}

/// MySQL's `[client]` login: TCP on 127.0.0.1 (never a socket or a named
/// pipe of another MySQL of the PC), no paths, so nothing to escape (a
/// `\` is an escape in MySQL's option files). LF ends: read the same on
/// every system.
pub fn mysql_cnf_text(port: u16, password: &str) -> String {
    format!(
        "# Atollo: login of the kit's MySQL (written by the app).\n\
         [client]\nuser=root\npassword={password}\nhost=127.0.0.1\n\
         port={port}\nprotocol=TCP\n"
    )
}

/// [`mysql_cnf`] for `port`.
pub fn write_mysql_cnf(kit: &Kit, port: u16, password: &str) -> Result<()> {
    write_private(&mysql_cnf(kit), &mysql_cnf_text(port, password))
}

/// [`mysql_line_cnf`] for version `line` on its `port`.
pub fn write_mysql_line_cnf(kit: &Kit, line: &str, port: u16, password: &str) -> Result<()> {
    if !crate::versions::valid_line(line) {
        bail!("not a version line: {line}");
    }
    write_private(&mysql_line_cnf(kit, line), &mysql_cnf_text(port, password))
}

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

/// `name` is a database or user of the server of `engine` itself
/// ([`DbEngine::reserved_names`]; of every server for an engine this
/// version does not know).
pub fn is_reserved(engine: &str, name: &str) -> bool {
    let reserved = |e: DbEngine| {
        e.reserved_names()
            .iter()
            .any(|r| r.eq_ignore_ascii_case(name))
    };
    match DbEngine::from_id(engine) {
        Some(e) => reserved(e),
        None => DbEngine::ALL.into_iter().any(reserved),
    }
}

/// The rule of the names before they were stored: lower case letters,
/// digits and `_`, at most 64 characters (MySQL's limit for databases).
/// Many projects give the same name (`my-app`, `my_app`), so it is only
/// read for data created with it; [`project_db_name`] is the name in use.
fn legacy_db_name(project: &str) -> String {
    let mut name: String = project
        .to_ascii_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    name.truncate(64);
    if name.trim_matches('_').is_empty() {
        name = "progetto".into();
    }
    name
}

/// The name a new database and user of the project on `engine` gets (then
/// stored in `db.json`): [`legacy_db_name`], never one of that server's own
/// names, which get `_db` after them. Only for what is not created yet:
/// anything that reaches a server uses [`project_db_name`], so a name
/// stored before keeps winning.
pub fn db_name(project: &str, engine: &str) -> String {
    let mut name = legacy_db_name(project);
    if is_reserved(engine, &name) {
        name.push_str("_db");
    }
    // The user has the database's name, and MySQL takes user names of 32
    // characters at most (ERROR 1470, BUG-V6-07): a longer one is cut,
    // with 6 digits of the project's hash after it, so two long names that
    // start alike stay two.
    if engine == DbEngine::MySql.id() && name.len() > MYSQL_USER_MAX {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(project.to_ascii_lowercase().as_bytes());
        name.truncate(MYSQL_USER_MAX - 7);
        name.push_str(&format!("_{:02x}{:02x}{:02x}", hash[0], hash[1], hash[2]));
    }
    name
}

/// The longest user name of MySQL 8 and later.
const MYSQL_USER_MAX: usize = 32;

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

/// A project's connection to one of the kit's databases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDb {
    pub engine: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub url: String,
}

/// The connection to the database and user `name` ([`project_db_name`]).
pub fn project_db(engine: &str, name: &str, password: &str, port: u16) -> ProjectDb {
    let name = name.to_owned();
    // An id this version does not know: a SQL server, as before the engines
    // were an enum.
    let url = DbEngine::from_id(engine)
        .unwrap_or(DbEngine::MySql)
        .project_url(&name, password, port);
    ProjectDb {
        engine: engine.to_owned(),
        database: name.clone(),
        user: name,
        password: password.to_owned(),
        host: "127.0.0.1".into(),
        port,
        url,
    }
}

/// A line of `.env` that changes: `old` None = added.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvChange {
    pub key: String,
    pub old: Option<String>,
    pub new: String,
}

fn env_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let (k, v) = l.trim().split_once('=')?;
        (k.trim() == key).then(|| v.trim().to_owned())
    })
}

/// The keys `db` sets in a `.env` like `text`: Laravel's `DB_*` when the
/// file uses them (MySQL), otherwise `DATABASE_URL` / `MONGODB_URI`.
pub fn env_wanted(text: &str, db: &ProjectDb) -> Vec<(String, String)> {
    let laravel = text
        .lines()
        .any(|l| l.trim_start().starts_with("DB_CONNECTION="));
    // An id this version does not know: a SQL server, as before the engines
    // were an enum.
    let engine = DbEngine::from_id(&db.engine).unwrap_or(DbEngine::MySql);
    let pairs: Vec<(&str, String)> = match engine {
        DbEngine::MongoDb if laravel => vec![
            ("DB_CONNECTION", "mongodb".into()),
            ("DB_DATABASE", db.database.clone()),
            ("MONGODB_URI", db.url.clone()),
        ],
        DbEngine::MongoDb => vec![("MONGODB_URI", db.url.clone())],
        DbEngine::MySql if laravel => vec![
            ("DB_CONNECTION", "mysql".into()),
            ("DB_HOST", db.host.clone()),
            ("DB_PORT", db.port.to_string()),
            ("DB_DATABASE", db.database.clone()),
            ("DB_USERNAME", db.user.clone()),
            ("DB_PASSWORD", db.password.clone()),
        ],
        DbEngine::MySql => vec![("DATABASE_URL", db.url.clone())],
    };
    pairs.into_iter().map(|(k, v)| (k.to_owned(), v)).collect()
}

pub fn env_changes(text: &str, db: &ProjectDb) -> Vec<EnvChange> {
    env_wanted(text, db)
        .into_iter()
        .filter_map(|(key, new)| {
            let old = env_value(text, &key);
            (old.as_deref() != Some(new.as_str())).then_some(EnvChange { key, old, new })
        })
        .collect()
}

/// `text` with the keys of `db` set: lines replaced where they are,
/// missing keys added at the end; comments and other lines untouched.
pub fn env_apply_text(text: &str, db: &ProjectDb) -> String {
    env_set_text(text, &env_wanted(text, db))
}

/// `text` with the keys of `wanted` set: a line with the key is replaced
/// (comments stay), a missing key is added at the end.
pub fn env_set_text(text: &str, wanted: &[(String, String)]) -> String {
    let mut done: Vec<bool> = vec![false; wanted.len()];
    let newline = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut out: Vec<String> = text
        .lines()
        .map(|line| {
            let key = line
                .trim()
                .split_once('=')
                .map(|(k, _)| k.trim().to_owned());
            match key.and_then(|k| wanted.iter().position(|(w, _)| *w == k)) {
                Some(i) if !line.trim_start().starts_with('#') => {
                    done[i] = true;
                    format!("{}={}", wanted[i].0, wanted[i].1)
                }
                _ => line.to_owned(),
            }
        })
        .collect();
    for (i, (k, v)) in wanted.iter().enumerate() {
        if !done[i] {
            out.push(format!("{k}={v}"));
        }
    }
    let mut s = out.join(newline);
    s.push_str(newline);
    s
}

/// The project's `.env`, created from `.env.example` when missing.
pub fn env_source(project_dir: &Path) -> String {
    crate::os::safe_path::read_text(project_dir, &project_dir.join(".env"))
        .or_else(|| crate::os::safe_path::read_text(project_dir, &project_dir.join(".env.example")))
        .unwrap_or_default()
}

/// `.env` is listed in the project's `.gitignore` (it will hold a password).
pub fn env_ignored(project_dir: &Path) -> bool {
    crate::os::safe_path::read_text(project_dir, &project_dir.join(".gitignore"))
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .any(|l| matches!(l, ".env" | "/.env" | ".env*" | "*.env" | ".env.*"))
}

pub fn env_apply(project_dir: &Path, db: &ProjectDb) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("no project folder");
    }
    if crate::os::safe_path::is_link(&project_dir.join(".env")) {
        bail!(".env is a link: left alone");
    }
    let text = env_apply_text(&env_source(project_dir), db);
    write_atomic(&project_dir.join(".env"), text.as_bytes())?;
    Ok(())
}

/// Sets `wanted` in the project's `.env` (made from `.env.example`, or new).
pub fn env_set(project_dir: &Path, wanted: &[(String, String)]) -> Result<()> {
    if !project_dir.is_dir() {
        bail!("no project folder");
    }
    if crate::os::safe_path::is_link(&project_dir.join(".env")) {
        bail!(".env is a link: left alone");
    }
    let text = env_set_text(&env_source(project_dir), wanted);
    write_atomic(&project_dir.join(".env"), text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../test/rust/db/auth.rs"]
pub(crate) mod tests;
