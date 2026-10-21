//! The names of the projects' databases and users: stored, given by the kit, checked against others.

use anyhow::{bail, Result};

use super::lock::db_json_lock;
use super::project::{project_file_to_read, read_project, ProjectSecrets};
use super::{try_load, write_private};
use crate::db::DbEngine;
use crate::kit::Kit;

/// The key of the names in `db.json`.
const NAMES: &str = "names";

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
pub(super) fn name_in(secrets: &ProjectSecrets, project: &str, engine: &str) -> Result<String> {
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
pub(super) fn stored_name(secrets: &ProjectSecrets, engine: &str) -> Result<Option<String>> {
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
pub(super) fn names_mut(secrets: &mut ProjectSecrets) -> Result<&mut ProjectSecrets> {
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
    if let Some(serde_json::Value::Object(left)) = try_load(kit)?.other.get("projects") {
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
pub(super) fn legacy_db_name(project: &str) -> String {
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
