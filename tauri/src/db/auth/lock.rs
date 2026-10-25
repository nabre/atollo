//! The lock of `db.json` and `secrets.json`: a mutex in this process, the kit's lock across processes.

use anyhow::{bail, Result};

use super::dir;
use crate::kit::Kit;
use crate::platform::lock::{self, KitLock};

/// Held by every read-change-write of a project's `db.json` and of
/// `secrets.json` ([`update_secrets`]): two commands
/// on blocking threads (a version chosen while a database is created), or
/// the command line beside the app, would otherwise each write back what
/// they read, and one change would be lost. In this process a mutex; across
/// the processes of the kit the lock of the Antigravity keyring
/// (`agents/keyring/`): `Local\Atollo-db-json-<kit id>` on Windows, an
/// exclusive `flock` on `data/app/db/db-json.lock` elsewhere.
pub(crate) fn db_json_lock(kit: &Kit) -> Result<DbJsonLock> {
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let process = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(DbJsonLock {
        _kit: kit_lock(kit)?,
        _process: process,
    })
}

/// The kit's lock of `db.json` alone, without this process's mutex.
pub(super) fn kit_lock(kit: &Kit) -> Result<KitLock> {
    let name = format!("Atollo-db-json-{}", kit.id());
    match lock::wait(&name, &dir(kit).join("db-json.lock"), DB_JSON_WAIT)? {
        Some(held) => Ok(held),
        None => bail!("db.json: another Atollo process holds it, try again"),
    }
}

/// How long another process may hold `db.json`: then the change is refused
/// (nothing written), never made without the lock.
const DB_JSON_WAIT: std::time::Duration = std::time::Duration::from_secs(30);

pub(crate) struct DbJsonLock {
    // Released first: the other processes, then this one's threads.
    _kit: KitLock,
    _process: std::sync::MutexGuard<'static, ()>,
}
