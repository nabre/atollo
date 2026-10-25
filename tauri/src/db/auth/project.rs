//! A project's `private\db.json`: its users' passwords, the versions its databases are and were on.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use super::lock::db_json_lock;
use super::names::{check_name_free, name_in, names_mut, stored_name};
use super::{new_password, write_in_private};
use crate::kit::Kit;
use crate::projects::state::ProjectState;

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
pub(super) fn project_file_to_read(kit: &Kit, project: &str) -> Result<PathBuf> {
    Ok(project_state(kit, project)?
        .dir()
        .join("private")
        .join("db.json"))
}

/// `None` for a missing file; an error for one that cannot be read, so it
/// is never overwritten and its passwords lost.
pub(super) fn read_project(path: &Path) -> Result<Option<ProjectSecrets>> {
    if crate::platform::safe_path::is_link(path) {
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
        crate::common::log::warn(&format!("{project}: passwords: {e:#}"));
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
